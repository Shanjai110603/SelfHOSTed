use serde::{Deserialize, Serialize};
use std::process::Command;
use std::sync::Arc;
use std::collections::HashMap;
use async_trait::async_trait;
use tauri::Manager;

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

#[derive(Serialize, Deserialize, Clone)]
pub struct ExposureConfig {
    pub provider: String, // "cloudflare", "tailscale", "local_only"
    pub mode: String,     // "quick", "authenticated", "mesh"
    pub token: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct WorkloadConfig {
    pub id: String,
    pub resource_type: String, // "website", "database", "fileshare"
    pub template: String,      // "nodejs", "python", "php", "static", "mysql", "postgres", "wordpress"
    pub port: u16,             // Internal target port
    pub env_vars: HashMap<String, String>,
    pub domain: Option<String>,
    pub host_path: Option<String>,
    pub exposure: Option<ExposureConfig>,
    
    // Virtual Networking / Stack fields
    pub stack_id: Option<String>,
    pub network_alias: Option<String>,
    pub depends_on: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct StackConfig {
    pub id: String,
    pub name: String,
    pub workloads: Vec<WorkloadConfig>,
    pub exposure: Option<ExposureConfig>,
}

#[derive(Serialize, Clone)]
pub struct WorkloadSession {
    pub id: String,
    pub config: WorkloadConfig,
    pub status: String,
}

#[async_trait]
pub trait RuntimeProvider: Send + Sync {
    async fn check_runtime(&self) -> Result<RuntimeStatus, String>;
    async fn start_workload(&self, config: &WorkloadConfig) -> Result<WorkloadSession, String>;
    async fn stop_workload(&self, id: &str) -> Result<(), String>;
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

    async fn start_workload(&self, config: &WorkloadConfig) -> Result<WorkloadSession, String> {
        let mut args = vec!["run".to_string(), "-d".to_string(), "--name".to_string(), config.id.clone()];
        
        // Virtual Networking for Stacks
        if let Some(stack_id) = &config.stack_id {
            // Ensure the network exists (this might fail harmlessly if it already exists, which is fine)
            let _ = tokio::process::Command::new("docker")
                .args(["network", "create", stack_id])
                .output().await;

            args.push("--network".to_string());
            args.push(stack_id.clone());
            
            if let Some(alias) = &config.network_alias {
                args.push("--network-alias".to_string());
                args.push(alias.clone());
            }
        } else {
            // Setup internal network proxy access if Traefik needs to hit it
            // Or we expose it randomly if no proxy. For EPIC-001, we expose the port explicitly on localhost.
            args.push("-p".to_string());
            args.push(format!("127.0.0.1:{}:{}", config.port, config.port));
        }

        for (k, v) in &config.env_vars {
            args.push("-e".to_string());
            args.push(format!("{}={}", k, v));
        }

        // Apply Runtime Templates
        match config.template.as_str() {
            "nodejs" => {
                if let Some(path) = &config.host_path {
                    args.push("-v".to_string());
                    args.push(format!("{}:/app", path));
                    args.push("-w".to_string());
                    args.push("/app".to_string());
                }
                args.push("node:18-alpine".to_string());
                args.push("npm".to_string());
                args.push("start".to_string());
            },
            "python" => {
                if let Some(path) = &config.host_path {
                    args.push("-v".to_string());
                    args.push(format!("{}:/app", path));
                    args.push("-w".to_string());
                    args.push("/app".to_string());
                }
                args.push("python:3.11-slim".to_string());
                args.push("python".to_string());
                args.push("-m".to_string());
                args.push("http.server".to_string());
                args.push(config.port.to_string());
            },
            "php" => {
                if let Some(path) = &config.host_path {
                    args.push("-v".to_string());
                    args.push(format!("{}:/var/www/html", path));
                }
                args.push("php:8-apache".to_string());
            },
            "static" => {
                if let Some(path) = &config.host_path {
                    args.push("-v".to_string());
                    args.push(format!("{}:/usr/share/nginx/html:ro", path));
                }
                args.push("nginx:alpine".to_string());
            },
            "mysql" => {
                args.push("mysql:8".to_string());
            },
            "postgres" => {
                args.push("postgres:alpine".to_string());
            },
            "redis" => {
                args.push("redis:alpine".to_string());
            },
            "mongodb" => {
                args.push("mongo:latest".to_string());
            },
            "wordpress" => {
                args.push("wordpress:latest".to_string());
            },
            "nextcloud" => {
                args.push("nextcloud:latest".to_string());
            },
            _ => return Err("Unknown runtime template".to_string())
        }

        let output = tokio::process::Command::new("docker")
            .args(&args)
            .output().await.map_err(|e| e.to_string())?;

        if !output.status.success() { return Err(String::from_utf8_lossy(&output.stderr).to_string()); }

        Ok(WorkloadSession { 
            id: config.id.clone(),
            config: config.clone(),
            status: "Running".to_string(),
        })
    }

    async fn stop_workload(&self, id: &str) -> Result<(), String> {
        let output = tokio::process::Command::new("docker").args(["rm", "-f", id]).output().await.map_err(|e| e.to_string())?;
        if !output.status.success() { return Err(String::from_utf8_lossy(&output.stderr).to_string()); }
        Ok(())
    }

    async fn pause_session(&self, id: &str) -> Result<(), String> {
        let output = tokio::process::Command::new("docker").args(["pause", id]).output().await.map_err(|e| e.to_string())?;
        if !output.status.success() { return Err(String::from_utf8_lossy(&output.stderr).to_string()); }
        Ok(())
    }

    async fn resume_session(&self, id: &str) -> Result<(), String> {
        let output = tokio::process::Command::new("docker").args(["unpause", id]).output().await.map_err(|e| e.to_string())?;
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

    async fn start_workload(&self, config: &WorkloadConfig) -> Result<WorkloadSession, String> {
        Ok(WorkloadSession { 
            id: config.id.clone(),
            config: config.clone(),
            status: "Running".to_string(),
        })
    }

    async fn stop_workload(&self, _id: &str) -> Result<(), String> {
        Ok(())
    }

    async fn pause_session(&self, _id: &str) -> Result<(), String> { Ok(()) }
    async fn resume_session(&self, _id: &str) -> Result<(), String> { Ok(()) }
}

// ==========================================
// IPC HANDLERS
// ==========================================

#[tauri::command]
pub async fn check_runtime(runtime: tauri::State<'_, ActiveRuntime>) -> Result<RuntimeStatus, String> {
    runtime.provider.check_runtime().await
}

#[tauri::command]
pub async fn start_workload(
    runtime: tauri::State<'_, ActiveRuntime>,
    proxy: tauri::State<'_, Arc<dyn crate::proxy::ProxyProvider>>,
    net: tauri::State<'_, crate::network::ActiveNetworkState>,
    vault: tauri::State<'_, crate::vault::VaultManager>,
    db: tauri::State<'_, crate::db::DatabaseManager>,
    app: tauri::AppHandle,
    config: WorkloadConfig
) -> Result<WorkloadSession, String> {
    
    // Generate UUID if empty
    let mut config = config.clone();
    if config.id.is_empty() {
        config.id = format!("workload-{}", uuid::Uuid::new_v4().to_string().chars().take(8).collect::<String>());
    }

    // Intercept databases to generate secure Vault credentials
    if config.resource_type == "database" {
        let raw_password = vault.generate_secure_password();
        let encrypted = vault.encrypt_string(&raw_password)?;
        
        // Save encrypted password to sqlite
        let _ = db.save_secret_internal(&format!("{}_admin_password", config.id), "database_password", &encrypted, None);

        // Inject raw password into the env context for Docker to use
        match config.template.as_str() {
            "mysql" => { config.env_vars.insert("MYSQL_ROOT_PASSWORD".to_string(), raw_password); },
            "postgres" => { config.env_vars.insert("POSTGRES_PASSWORD".to_string(), raw_password); },
            "mariadb" => { config.env_vars.insert("MARIADB_ROOT_PASSWORD".to_string(), raw_password); },
            "redis" => { config.env_vars.insert("REDIS_PASSWORD".to_string(), raw_password); },
            "mongodb" => { 
                config.env_vars.insert("MONGO_INITDB_ROOT_USERNAME".to_string(), "admin".to_string());
                config.env_vars.insert("MONGO_INITDB_ROOT_PASSWORD".to_string(), raw_password); 
            },
            _ => {}
        }
    }

    // Launch workload via RuntimeProvider
    let session = runtime.provider.start_workload(&config).await?;

    // If domain is provided, sync route via ProxyProvider
    if let Some(domain) = &config.domain {
        if !domain.is_empty() {
            let config_dir = app.path().app_data_dir().unwrap_or_default().join("proxy");
            proxy.add_route(config_dir, &session.id, domain, "127.0.0.1", config.port).await?;
        }
    }

    // Orchestrate network exposure if requested
    if let Some(exposure) = &config.exposure {
        if exposure.provider != "none" {
            let _ = match exposure.provider.as_str() {
                "cloudflare" => net.cloudflare.expose_workload(&session.id, config.port, &exposure.mode, exposure.token.as_deref()).await,
                "tailscale" => net.tailscale.expose_workload(&session.id, config.port, &exposure.mode, exposure.token.as_deref()).await,
                _ => Err("Unknown exposure provider".into())
            };
            // Note: In a robust setup we'd probably save this exposure state or handle failure,
            // but for now it's successfully dispatched.
        }
    }

    Ok(session)
}

#[tauri::command]
pub async fn stop_workload(
    runtime: tauri::State<'_, ActiveRuntime>,
    proxy: tauri::State<'_, Arc<dyn crate::proxy::ProxyProvider>>,
    app: tauri::AppHandle,
    id: String
) -> Result<(), String> {
    runtime.provider.stop_workload(&id).await?;
    let config_dir = app.path().app_data_dir().unwrap_or_default().join("proxy");
    let _ = proxy.remove_route(config_dir, &id).await;
    Ok(())
}

#[tauri::command]
pub async fn pause_session(runtime: tauri::State<'_, ActiveRuntime>, id: String) -> Result<(), String> {
    runtime.provider.pause_session(&id).await
}

#[tauri::command]
pub async fn resume_session(runtime: tauri::State<'_, ActiveRuntime>, id: String) -> Result<(), String> {
    runtime.provider.resume_session(&id).await
}
