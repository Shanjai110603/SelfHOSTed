use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::Manager;

#[derive(Clone, Serialize, Deserialize)]
pub struct ManagedSession {
    pub id: String,
    pub resource_type: String, 
    pub expires_at: u64, 
    pub status: String, // "starting", "running", "paused", "failed", "recovering"
}

pub struct SessionManager {
    pub sessions: Arc<Mutex<HashMap<String, ManagedSession>>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[tauri::command]
pub fn register_session(
    manager: tauri::State<'_, SessionManager>,
    id: String,
    resource_type: String,
    duration_minutes: u64,
) -> Result<(), String> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let expires_at = now + (duration_minutes * 60);

    let mut sessions = manager.sessions.lock().unwrap();
    sessions.insert(id.clone(), ManagedSession {
        id,
        resource_type,
        expires_at,
        status: "running".to_string(),
    });

    Ok(())
}

#[tauri::command]
pub fn update_session_status(manager: tauri::State<'_, SessionManager>, id: String, status: String) -> Result<(), String> {
    let mut sessions = manager.sessions.lock().unwrap();
    if let Some(session) = sessions.get_mut(&id) {
        session.status = status;
    }
    Ok(())
}

#[tauri::command]
pub fn get_active_sessions(manager: tauri::State<'_, SessionManager>) -> Vec<ManagedSession> {
    let sessions = manager.sessions.lock().unwrap();
    sessions.values().cloned().collect()
}

pub fn auto_recover_sessions(app: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        let db = app.state::<crate::db::DatabaseManager>();
        let session_manager = app.state::<SessionManager>();

        if let Ok(recent) = db.get_recent_sessions_internal() {
            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
            
            for s in recent {
                if s.resource_type == "website" || s.resource_type == "database" || s.resource_type == "fileshare" {
                    let check = tokio::process::Command::new("docker")
                        .args(["inspect", "-f", "{{.State.Status}}", &s.id])
                        .output()
                        .await;

                    if let Ok(output) = check {
                        let status_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                        if status_str == "running" || status_str == "paused" {
                            let mut map = session_manager.sessions.lock().unwrap();
                            // Only insert if it's not already tracked
                            if !map.contains_key(&s.id) {
                                map.insert(s.id.clone(), ManagedSession {
                                    id: s.id.clone(),
                                    resource_type: s.resource_type.clone(),
                                    expires_at: now + (60 * 60), // Recovered sessions get a fresh 1hr TTL
                                    status: status_str,
                                });
                            }
                        }
                    }
                }
            }
        }
    });
}
