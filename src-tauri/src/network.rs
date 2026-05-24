use serde::{Deserialize, Serialize};
use tokio::process::Command;
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tokio::io::{AsyncBufReadExt, BufReader};
use crate::db::DatabaseManager;
use async_trait::async_trait;

#[derive(Serialize, Deserialize, Clone)]
pub struct ExposureSession {
    pub id: String,
    pub workload_id: String,
    pub provider: String,
    pub mode: String,
    pub public_url: Option<String>,
    pub status: String,
}

#[async_trait]
pub trait NetworkProvider: Send + Sync {
    async fn expose_workload(&self, workload_id: &str, target_port: u16, mode: &str, token: Option<&str>) -> Result<ExposureSession, String>;
    async fn revoke_exposure(&self, id: &str) -> Result<(), String>;
}

// -----------------------------------------------------------------------------
// Cloudflare Provider
// -----------------------------------------------------------------------------
pub struct CloudflareProvider {
    pub tunnels: Mutex<HashMap<String, tokio::process::Child>>,
}

impl CloudflareProvider {
    pub fn new() -> Self {
        Self {
            tunnels: Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl NetworkProvider for CloudflareProvider {
    async fn expose_workload(&self, workload_id: &str, target_port: u16, mode: &str, _token: Option<&str>) -> Result<ExposureSession, String> {
        if mode == "quick" {
            let check = std::process::Command::new("cloudflared").arg("--version").output();
            if check.is_err() {
                return Err("cloudflared is not installed or not in PATH.".into());
            }

            let mut child = Command::new("cloudflared")
                .arg("tunnel")
                .arg("--url")
                .arg(&format!("http://localhost:{}", target_port))
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .map_err(|e| format!("Failed to spawn cloudflared: {}", e))?;

            let stderr = child.stderr.take().ok_or("Failed to capture stderr")?;
            let mut reader = BufReader::new(stderr).lines();

            let mut public_url_res = String::new();
            let id = format!("cf-{}", workload_id);

            let timeout_duration = std::time::Duration::from_secs(10);
            let fetch_url = async {
                while let Ok(Some(line)) = reader.next_line().await {
                    if line.contains(".trycloudflare.com") {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        for part in parts {
                            if part.starts_with("https://") && part.ends_with(".trycloudflare.com") {
                                return Some(part.to_string());
                            }
                        }
                    }
                }
                None
            };

            match tokio::time::timeout(timeout_duration, fetch_url).await {
                Ok(Some(url)) => public_url_res = url,
                _ => {
                    let _ = child.kill().await;
                    return Err("Failed to extract Cloudflare URL within timeout.".into());
                }
            }

            tokio::spawn(async move {
                while let Ok(Some(_)) = reader.next_line().await {}
            });

            self.tunnels.lock().unwrap().insert(id.clone(), child);

            Ok(ExposureSession {
                id,
                workload_id: workload_id.to_string(),
                provider: "cloudflare".to_string(),
                mode: "quick".to_string(),
                public_url: Some(public_url_res),
                status: "active".to_string(),
            })
        } else {
            Err("Authenticated mode coming soon in EPIC-002 Phase 2".into())
        }
    }

    async fn revoke_exposure(&self, id: &str) -> Result<(), String> {
        let child_opt = {
            let mut tunnels = self.tunnels.lock().unwrap();
            tunnels.remove(id)
        };
        if let Some(mut child) = child_opt {
            let _ = child.kill().await;
        }
        Ok(())
    }
}

// -----------------------------------------------------------------------------
// Tailscale Provider (Sidecar Architecture)
// -----------------------------------------------------------------------------
pub struct TailscaleProvider;

impl TailscaleProvider {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl NetworkProvider for TailscaleProvider {
    async fn expose_workload(&self, workload_id: &str, _target_port: u16, mode: &str, token: Option<&str>) -> Result<ExposureSession, String> {
        if mode != "mesh" {
            return Err("Tailscale only supports mesh mode".into());
        }

        let auth_key = token.ok_or("Tailscale Auth Key required for mesh networking.")?;
        
        let container_name = format!("ts-sidecar-{}", workload_id);

        let output = tokio::process::Command::new("docker")
            .args([
                "run", "-d",
                "--name", &container_name,
                "--network", &format!("container:{}", workload_id), // Sidecar network attach
                "-e", &format!("TS_AUTHKEY={}", auth_key),
                "-e", "TS_STATE_DIR=/var/lib/tailscale",
                "--cap-add=NET_ADMIN",
                "--cap-add=NET_RAW",
                "tailscale/tailscale:latest"
            ])
            .output().await.map_err(|e| e.to_string())?;

        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).to_string());
        }

        Ok(ExposureSession {
            id: container_name,
            workload_id: workload_id.to_string(),
            provider: "tailscale".to_string(),
            mode: "mesh".to_string(),
            public_url: None, // Tailnet IPs are private
            status: "active".to_string(),
        })
    }

    async fn revoke_exposure(&self, id: &str) -> Result<(), String> {
        let _ = tokio::process::Command::new("docker").args(["rm", "-f", id]).output().await;
        Ok(())
    }
}

// -----------------------------------------------------------------------------
// Active Network State
// -----------------------------------------------------------------------------
pub struct ActiveNetworkState {
    pub cloudflare: Arc<dyn NetworkProvider>,
    pub tailscale: Arc<dyn NetworkProvider>,
}

#[tauri::command]
pub async fn expose_workload(
    net: tauri::State<'_, ActiveNetworkState>,
    db: tauri::State<'_, DatabaseManager>,
    workload_id: String,
    target_port: u16,
    provider: String,
    mode: String,
    token: Option<String>
) -> Result<ExposureSession, String> {
    
    let session = match provider.as_str() {
        "cloudflare" => net.cloudflare.expose_workload(&workload_id, target_port, &mode, token.as_deref()).await?,
        "tailscale" => net.tailscale.expose_workload(&workload_id, target_port, &mode, token.as_deref()).await?,
        _ => return Err("Unknown network provider".into())
    };

    if let Some(url) = &session.public_url {
        let _ = db.log_public_link(&session.id, &workload_id, url, &provider, &mode);
    }

    Ok(session)
}

#[tauri::command]
pub async fn revoke_exposure(
    net: tauri::State<'_, ActiveNetworkState>,
    db: tauri::State<'_, DatabaseManager>,
    id: String,
    provider: String
) -> Result<(), String> {
    match provider.as_str() {
        "cloudflare" => net.cloudflare.revoke_exposure(&id).await?,
        "tailscale" => net.tailscale.revoke_exposure(&id).await?,
        _ => return Err("Unknown network provider".into())
    };

    let _ = db.remove_public_link(&id);
    Ok(())
}
