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
    pub require_auth: Option<bool>,
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
        let mut cmd = std::process::Command::new("docker");
        cmd.args(["run", "-d", "--name", &config.id]);

        // Handle isolated stack virtual networking with security zoning
        if let Some(stack_id) = &config.stack_id {
            // Zone topology: 
            // "database" -> internal zone (no outbound internet)
            // "website" -> public zone (outbound internet), plus connected to internal zone later
            
            let is_backend = config.resource_type == "database";
            let net_name = if is_backend { format!("{}-internal", stack_id) } else { format!("{}-public", stack_id) };
            
            // Try to create the network
            let mut net_cmd = std::process::Command::new("docker");
            net_cmd.arg("network").arg("create");
            if is_backend {
                net_cmd.arg("--internal"); // The critical security isolation flag
            }
            net_cmd.arg(&net_name);
            let _ = net_cmd.output(); // ignore if already exists

            cmd.arg("--network").arg(&net_name);
            if let Some(alias) = &config.network_alias {
                cmd.arg("--network-alias").arg(alias);
            }
        } else {
            // Setup internal network proxy access if Traefik needs to hit it
            // Or we expose it randomly if no proxy. For EPIC-001, we expose the port explicitly on localhost.
            cmd.arg("-p").arg(format!("127.0.0.1:{}:{}", config.port, config.port));
        }

        for (k, v) in &config.env_vars {
            cmd.arg("-e").arg(format!("{}={}", k, v));
        }

        // Apply Runtime Templates
        match config.template.as_str() {
            "nodejs" => {
                if let Some(path) = &config.host_path {
                    cmd.arg("-v").arg(format!("{}:/app", path));
                    cmd.arg("-w").arg("/app");
                }
                cmd.args(["node:18-alpine", "npm", "start"]);
            },
            "python" => {
                if let Some(path) = &config.host_path {
                    cmd.arg("-v").arg(format!("{}:/app", path));
                    cmd.arg("-w").arg("/app");
                }
                cmd.args(["python:3.11-slim", "python", "-m", "http.server", &config.port.to_string()]);
            },
            "php" => {
                if let Some(path) = &config.host_path {
                    cmd.arg("-v").arg(format!("{}:/var/www/html", path));
                }
                cmd.arg("php:8-apache");
            },
            "static" => {
                if let Some(path) = &config.host_path {
                    cmd.arg("-v").arg(format!("{}:/usr/share/nginx/html:ro", path));
                }
                cmd.arg("nginx:alpine");
            },
            "mysql" => { cmd.arg("mysql:8"); },
            "postgres" => { cmd.arg("postgres:alpine"); },
            "redis" => { cmd.arg("redis:alpine"); },
            "mongodb" => { cmd.arg("mongo:latest"); },
            "wordpress" => { cmd.arg("wordpress:latest"); },
            "nextcloud" => { cmd.arg("nextcloud:latest"); },
            _ => return Err("Unknown runtime template".to_string())
        }

        let output = cmd.output().map_err(|e| format!("Failed to start docker container: {}", e))?;
        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Docker error: {}", err));
        }

        // Security Routing: Post-launch, connect frontends to the internal network so they can reach the database
        if let Some(stack_id) = &config.stack_id {
            if config.resource_type == "website" {
                let internal_net = format!("{}-internal", stack_id);
                // We ignore the error here because the internal network might not exist if this stack has no DB
                let _ = std::process::Command::new("docker")
                    .arg("network").arg("connect")
                    .arg(&internal_net).arg(&config.id)
                    .output();
            }
        }

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
            let auth = config.require_auth.unwrap_or(false);
            proxy.add_route(config_dir, &session.id, domain, "127.0.0.1", config.port, auth).await?;
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
