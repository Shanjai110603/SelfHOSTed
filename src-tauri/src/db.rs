use rusqlite::{params, Connection, Result as SqlResult};
use std::sync::Mutex;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

pub struct DatabaseManager {
    // MVP uses Mutex<Connection>. Architected to allow future swap to r2d2/sqlx connection pool.
    pub conn: Mutex<Connection>,
}

impl DatabaseManager {
    pub fn new(app_data_dir: PathBuf) -> Result<Self, String> {
        // Ensure the data directory exists
        if !app_data_dir.exists() {
            std::fs::create_dir_all(&app_data_dir).map_err(|e| format!("Failed to create app data dir: {}", e))?;
        }

        let db_path = app_data_dir.join("selfhosted.sqlite");
        
        let mut conn = Connection::open(&db_path)
            .map_err(|e| format!("Failed to open DB at {:?}: {}", db_path, e))?;
            
        // Apply versioned migrations atomically
        Self::run_migrations(&mut conn).map_err(|e| format!("Database Migration failed: {}", e))?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    fn run_migrations(conn: &mut Connection) -> SqlResult<()> {
        let tx = conn.transaction()?;
        
        // 1. Create migration tracking table
        tx.execute(
            "CREATE TABLE IF NOT EXISTS migrations (
                version INTEGER PRIMARY KEY,
                applied_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        // 2. Read current version
        let current_version: i32 = tx.query_row(
            "SELECT COALESCE(MAX(version), 0) FROM migrations",
            [],
            |row| row.get(0),
        ).unwrap_or(0);

        // 3. Define schema migrations (Future-proofed schema)
        let migrations = vec![
            // V1: Core Infrastructure Schema
            "
            CREATE TABLE IF NOT EXISTS preferences (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS session_history (
                id TEXT PRIMARY KEY,
                resource_type TEXT NOT NULL,
                path TEXT,
                port INTEGER,
                started_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );
            CREATE TABLE IF NOT EXISTS public_links (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                public_url TEXT NOT NULL,
                expires_at DATETIME
            );
            CREATE TABLE IF NOT EXISTS runtimes (
                name TEXT PRIMARY KEY,
                version TEXT,
                status TEXT
            );
            CREATE TABLE IF NOT EXISTS device_profiles (
                id TEXT PRIMARY KEY,
                hardware_tier TEXT
            );
            ",
            // V2: Add provider-agnostic exposure metadata
            "
            ALTER TABLE public_links ADD COLUMN provider TEXT DEFAULT 'cloudflare_quick';
            ALTER TABLE public_links ADD COLUMN access_mode TEXT DEFAULT 'public';
            ",
            // V3: Generic secrets table for Vault integration
            "
            CREATE TABLE IF NOT EXISTS secrets (
                id TEXT PRIMARY KEY,
                secret_type TEXT,
                encrypted_payload TEXT,
                metadata TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );
            "
        ];

        // 4. Apply pending migrations sequentially
        for (i, sql) in migrations.iter().enumerate() {
            let version = (i + 1) as i32;
            if current_version < version {
                tx.execute_batch(sql)?;
                tx.execute("INSERT INTO migrations (version) VALUES (?1)", params![version])?;
            }
        }

        tx.commit()?;
        Ok(())
    }
}

// -----------------------------------------------------------------------------
// DTOs
// -----------------------------------------------------------------------------

#[derive(Serialize, Deserialize)]
pub struct SessionHistoryEntry {
    pub id: String,
    pub resource_type: String,
    pub path: Option<String>,
    pub port: Option<u16>,
    pub started_at: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PublicLinkEntry {
    pub id: String,
    pub session_id: String,
    pub public_url: String,
    pub provider: String,
    pub access_mode: String,
}

// -----------------------------------------------------------------------------
// IPC Commands
// -----------------------------------------------------------------------------

#[tauri::command]
pub fn save_preference(manager: tauri::State<'_, DatabaseManager>, key: String, value: String) -> Result<(), String> {
    let conn = manager.conn.lock().unwrap();
    conn.execute(
        "INSERT INTO preferences (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_preference(manager: tauri::State<'_, DatabaseManager>, key: String) -> Result<Option<String>, String> {
    let conn = manager.conn.lock().unwrap();
    let mut stmt = conn.prepare("SELECT value FROM preferences WHERE key = ?1").map_err(|e| e.to_string())?;
    let mut rows = stmt.query(params![key]).map_err(|e| e.to_string())?;
    if let Some(row) = rows.next().map_err(|e| e.to_string())? {
        Ok(Some(row.get(0).map_err(|e| e.to_string())?))
    } else {
        Ok(None)
    }
}

#[tauri::command]
pub fn save_secret(
    db: tauri::State<'_, DatabaseManager>,
    vault: tauri::State<'_, crate::vault::VaultManager>,
    id: String,
    secret_type: String,
    plaintext: String,
    metadata: Option<String>
) -> Result<(), String> {
    let encrypted_payload = vault.encrypt_string(&plaintext)?;
    let conn = db.conn.lock().unwrap();
    conn.execute(
        "INSERT INTO secrets (id, secret_type, encrypted_payload, metadata) VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(id) DO UPDATE SET encrypted_payload = excluded.encrypted_payload, metadata = excluded.metadata, updated_at = CURRENT_TIMESTAMP",
        params![id, secret_type, encrypted_payload, metadata]
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_secret(
    db: tauri::State<'_, DatabaseManager>,
    vault: tauri::State<'_, crate::vault::VaultManager>,
    id: String
) -> Result<Option<String>, String> {
    let conn = db.conn.lock().unwrap();
    let mut stmt = conn.prepare("SELECT encrypted_payload FROM secrets WHERE id = ?1").map_err(|e| e.to_string())?;
    let mut rows = stmt.query(params![id]).map_err(|e| e.to_string())?;
    
    if let Some(row) = rows.next().map_err(|e| e.to_string())? {
        let payload: String = row.get(0).map_err(|e| e.to_string())?;
        // Memory safety: the decrypted string exists only as long as this variable does
        let plaintext = vault.decrypt_string(&payload)?;
        Ok(Some(plaintext))
    } else {
        Ok(None)
    }
}

#[tauri::command]
pub fn log_session(manager: tauri::State<'_, DatabaseManager>, id: String, resource_type: String, path: Option<String>, port: Option<u16>) -> Result<(), String> {
    let conn = manager.conn.lock().unwrap();
    conn.execute(
        "INSERT INTO session_history (id, resource_type, path, port) VALUES (?1, ?2, ?3, ?4)",
        params![id, resource_type, path, port],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_recent_sessions(manager: tauri::State<'_, DatabaseManager>) -> Result<Vec<SessionHistoryEntry>, String> {
    let conn = manager.conn.lock().unwrap();
    let mut stmt = conn.prepare("SELECT id, resource_type, path, port, started_at FROM session_history ORDER BY started_at DESC LIMIT 10").map_err(|e| e.to_string())?;
    let session_iter = stmt.query_map([], |row| {
        Ok(SessionHistoryEntry {
            id: row.get(0)?,
            resource_type: row.get(1)?,
            path: row.get(2)?,
            port: row.get(3)?,
            started_at: row.get(4)?,
        })
    }).map_err(|e| e.to_string())?;

    let mut sessions = Vec::new();
    for session in session_iter {
        sessions.push(session.map_err(|e| e.to_string())?);
    }
    Ok(sessions)
}

impl DatabaseManager {
    pub fn get_recent_sessions_internal(&self) -> Result<Vec<SessionHistoryEntry>, String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, resource_type, path, port, started_at FROM session_history ORDER BY started_at DESC LIMIT 50").map_err(|e| e.to_string())?;
        let session_iter = stmt.query_map([], |row| {
            Ok(SessionHistoryEntry {
                id: row.get(0)?,
                resource_type: row.get(1)?,
                path: row.get(2)?,
                port: row.get(3)?,
                started_at: row.get(4)?,
            })
        }).map_err(|e| e.to_string())?;

        let mut sessions = Vec::new();
        for session in session_iter {
            sessions.push(session.map_err(|e| e.to_string())?);
        }
        Ok(sessions)
    }

    pub fn save_secret_internal(&self, id: &str, secret_type: &str, encrypted_payload: &str, metadata: Option<&str>) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO secrets (id, secret_type, encrypted_payload, metadata) VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(id) DO UPDATE SET encrypted_payload = excluded.encrypted_payload, metadata = excluded.metadata, updated_at = CURRENT_TIMESTAMP",
            params![id, secret_type, encrypted_payload, metadata]
        ).map_err(|e| e.to_string())?;
        Ok(())
    }
}

// Tunnel persistence internally used by network.rs
impl DatabaseManager {
    pub fn log_public_link(&self, id: &str, session_id: &str, url: &str, provider: &str, access_mode: &str) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO public_links (id, session_id, public_url, provider, access_mode) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![id, session_id, url, provider, access_mode],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn remove_public_link(&self, id: &str) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM public_links WHERE id = ?1", params![id]).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_active_links(&self) -> Result<Vec<PublicLinkEntry>, String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, session_id, public_url, provider, access_mode FROM public_links").map_err(|e| e.to_string())?;
        let link_iter = stmt.query_map([], |row| {
            Ok(PublicLinkEntry {
                id: row.get(0)?,
                session_id: row.get(1)?,
                public_url: row.get(2)?,
                provider: row.get(3)?,
                access_mode: row.get(4)?,
            })
        }).map_err(|e| e.to_string())?;

        let mut links = Vec::new();
        for l in link_iter {
            links.push(l.map_err(|e| e.to_string())?);
        }
        Ok(links)
    }
}
