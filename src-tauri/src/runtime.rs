use serde::{Deserialize, Serialize};
use std::process::Command;
use std::sync::Arc;
use async_trait::async_trait;

#[derive(Serialize, Deserialize, Clone)]
pub struct CapabilityEngine {
    pub supports_docker: bool,
    pub supports_databases: bool,
    pub supports_public_tunnels: bool,
    pub platform: String,
}

#[tauri::command]
pub fn get_capabilities() -> CapabilityEngine {
    if cfg!(target_os = "android") {
        CapabilityEngine {
            supports_docker: false,
            supports_databases: false,
            supports_public_tunnels: false,
            platform: "android".to_string(),
        }
    } else {
        CapabilityEngine {
            supports_docker: true,
            supports_databases: true,
            supports_public_tunnels: true,
            platform: std::env::consts::OS.to_string(),
        }
    }
}

#[derive(Serialize)]
pub struct RuntimeStatus {
    pub installed: bool,
    pub engine: String,
    pub active_containers: usize,
}

#[derive(Serialize)]
pub struct WebsiteSession {
    pub id: String,
    pub path: String,
    pub port: u16,
    pub status: String,
}

#[derive(Serialize)]
pub struct DatabaseSession {
    pub id: String,
    pub port: u16,
    pub status: String,
}

#[async_trait]
pub trait RuntimeProvider: Send + Sync {
    async fn check_runtime(&self) -> Result<RuntimeStatus, String>;
    async fn start_website(&self, path: &str, port: u16) -> Result<WebsiteSession, String>;
    async fn stop_website(&self, id: &str) -> Result<(), String>;
    async fn start_fileshare(&self, path: &str, port: u16) -> Result<WebsiteSession, String>;
    async fn start_database(&self, password: &str, port: u16) -> Result<DatabaseSession, String>;
    async fn stop_database(&self, id: &str) -> Result<(), String>;
    async fn pause_session(&self, id: &str) -> Result<(), String>;
    async fn resume_session(&self, id: &str) -> Result<(), String>;
}

pub struct ActiveRuntime {
    pub provider: Arc<dyn RuntimeProvider>,
}

// ==========================================
// DOCKER PROVIDER (Desktop)
// ==========================================
pub struct DockerProvider;

impl DockerProvider {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl RuntimeProvider for DockerProvider {
    async fn check_runtime(&self) -> Result<RuntimeStatus, String> {
        let output = Command::new("docker").arg("--version").output();
        Ok(RuntimeStatus {
            installed: output.is_ok(),
            engine: "Docker".to_string(),
            active_containers: 0,
        })
    }

    async fn start_website(&self, path: &str, port: u16) -> Result<WebsiteSession, String> {
        let output = Command::new("docker")
            .args(["run", "-d", "-p", &format!("{}:80", port), "-v", &format!("{}:/usr/share/nginx/html:ro", path), "nginx:alpine"])
            .output().map_err(|e| e.to_string())?;

        if !output.status.success() { return Err(String::from_utf8_lossy(&output.stderr).to_string()); }
        let id = String::from_utf8_lossy(&output.stdout).trim().to_string();

        Ok(WebsiteSession { id, path: path.to_string(), port, status: "Running".to_string() })
    }

    async fn stop_website(&self, id: &str) -> Result<(), String> {
        let output = Command::new("docker").args(["rm", "-f", id]).output().map_err(|e| e.to_string())?;
        if !output.status.success() { return Err(String::from_utf8_lossy(&output.stderr).to_string()); }
        Ok(())
    }

    async fn start_fileshare(&self, path: &str, port: u16) -> Result<WebsiteSession, String> {
        let output = Command::new("docker")
            .args(["run", "-d", "-p", &format!("{}:{}", port, port), "-v", &format!("{}:/shared:ro", path), "python:3.9-slim", "python", "-m", "http.server", &port.to_string(), "--directory", "/shared"])
            .output().map_err(|e| e.to_string())?;

        if !output.status.success() { return Err(String::from_utf8_lossy(&output.stderr).to_string()); }
        let id = String::from_utf8_lossy(&output.stdout).trim().to_string();

        Ok(WebsiteSession { id, path: path.to_string(), port, status: "Running".to_string() })
    }

    async fn start_database(&self, password: &str, port: u16) -> Result<DatabaseSession, String> {
        let output = Command::new("docker")
            .args(["run", "-d", "-p", &format!("{}:5432", port), "-e", &format!("POSTGRES_PASSWORD={}", password), "postgres:alpine"])
            .output().map_err(|e| e.to_string())?;

        if !output.status.success() { return Err(String::from_utf8_lossy(&output.stderr).to_string()); }
        let id = String::from_utf8_lossy(&output.stdout).trim().to_string();

        Ok(DatabaseSession { id, port, status: "Running".to_string() })
    }

    async fn stop_database(&self, id: &str) -> Result<(), String> {
        let output = Command::new("docker").args(["rm", "-f", id]).output().map_err(|e| e.to_string())?;
        if !output.status.success() { return Err(String::from_utf8_lossy(&output.stderr).to_string()); }
        Ok(())
    }

    async fn pause_session(&self, id: &str) -> Result<(), String> {
        let output = Command::new("docker").args(["pause", id]).output().map_err(|e| e.to_string())?;
        if !output.status.success() { return Err(String::from_utf8_lossy(&output.stderr).to_string()); }
        Ok(())
    }

    async fn resume_session(&self, id: &str) -> Result<(), String> {
        let output = Command::new("docker").args(["unpause", id]).output().map_err(|e| e.to_string())?;
        if !output.status.success() { return Err(String::from_utf8_lossy(&output.stderr).to_string()); }
        Ok(())
    }
}

// ==========================================
// NATIVE PROVIDER (Android/Mobile)
// ==========================================
pub struct NativeProvider;

impl NativeProvider {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl RuntimeProvider for NativeProvider {
    async fn check_runtime(&self) -> Result<RuntimeStatus, String> {
        Ok(RuntimeStatus {
            installed: true,
            engine: "Native HTTP".to_string(),
            active_containers: 0,
        })
    }

    async fn start_website(&self, path: &str, port: u16) -> Result<WebsiteSession, String> {
        // TODO: Spawn a Warp / Tokio background thread to serve the path natively
        // For MVP, we simulate success
        let id = format!("native-web-{}", port);
        Ok(WebsiteSession { id, path: path.to_string(), port, status: "Running".to_string() })
    }

    async fn stop_website(&self, _id: &str) -> Result<(), String> {
        // TODO: Stop the background thread
        Ok(())
    }

    async fn start_fileshare(&self, path: &str, port: u16) -> Result<WebsiteSession, String> {
        // TODO: Spawn Warp with directory listing
        let id = format!("native-file-{}", port);
        Ok(WebsiteSession { id, path: path.to_string(), port, status: "Running".to_string() })
    }

    async fn start_database(&self, _password: &str, _port: u16) -> Result<DatabaseSession, String> {
        Err("Databases are not supported on this platform capabilities tier.".into())
    }

    async fn stop_database(&self, _id: &str) -> Result<(), String> {
        Err("Databases are not supported on this platform capabilities tier.".into())
    }

    async fn pause_session(&self, _id: &str) -> Result<(), String> {
        Ok(()) // Native webservers pause inherently or don't consume CPU when idle
    }

    async fn resume_session(&self, _id: &str) -> Result<(), String> {
        Ok(())
    }
}

// ==========================================
// IPC HANDLERS
// ==========================================

#[tauri::command]
pub async fn check_runtime(runtime: tauri::State<'_, ActiveRuntime>) -> Result<RuntimeStatus, String> {
    runtime.provider.check_runtime().await
}

#[tauri::command]
pub async fn start_website(runtime: tauri::State<'_, ActiveRuntime>, path: String, port: u16) -> Result<WebsiteSession, String> {
    runtime.provider.start_website(&path, port).await
}

#[tauri::command]
pub async fn stop_website(runtime: tauri::State<'_, ActiveRuntime>, id: String) -> Result<(), String> {
    runtime.provider.stop_website(&id).await
}

#[tauri::command]
pub async fn start_fileshare(runtime: tauri::State<'_, ActiveRuntime>, path: String, port: u16) -> Result<WebsiteSession, String> {
    runtime.provider.start_fileshare(&path, port).await
}

#[tauri::command]
pub async fn start_database(runtime: tauri::State<'_, ActiveRuntime>, password: String, port: u16) -> Result<DatabaseSession, String> {
    runtime.provider.start_database(&password, port).await
}

#[tauri::command]
pub async fn stop_database(runtime: tauri::State<'_, ActiveRuntime>, id: String) -> Result<(), String> {
    runtime.provider.stop_database(&id).await
}

#[tauri::command]
pub async fn pause_session(runtime: tauri::State<'_, ActiveRuntime>, id: String) -> Result<(), String> {
    runtime.provider.pause_session(&id).await
}

#[tauri::command]
pub async fn resume_session(runtime: tauri::State<'_, ActiveRuntime>, id: String) -> Result<(), String> {
    runtime.provider.resume_session(&id).await
}
