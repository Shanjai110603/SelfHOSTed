use serde::Serialize;
use tokio::process::Command;
use std::process::Stdio;
use std::sync::Mutex;
use std::collections::HashMap;
use tokio::io::{AsyncBufReadExt, BufReader};
use crate::db::DatabaseManager;
use tauri::Manager;

pub struct TunnelManager {
    pub tunnels: Mutex<HashMap<String, tokio::process::Child>>,
}

impl TunnelManager {
    pub fn new() -> Self {
        Self {
            tunnels: Mutex::new(HashMap::new()),
        }
    }
}

#[derive(Serialize)]
pub struct TunnelSession {
    pub id: String,
    pub local_port: u16,
    pub public_url: String,
}

#[tauri::command]
pub async fn start_tunnel(
    manager: tauri::State<'_, TunnelManager>,
    db: tauri::State<'_, DatabaseManager>,
    session_id: String,
    local_port: u16,
) -> Result<TunnelSession, String> {
    let check = std::process::Command::new("cloudflared").arg("--version").output();
    if check.is_err() {
        return Err("cloudflared is not installed or not in PATH. Please install it to enable Secure Public Access.".into());
    }

    let mut child = Command::new("cloudflared")
        .arg("tunnel")
        .arg("--url")
        .arg(&format!("http://localhost:{}", local_port))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn cloudflared: {}", e))?;

    let stderr = child.stderr.take().ok_or("Failed to capture stderr")?;
    let mut reader = BufReader::new(stderr).lines();

    let mut public_url = String::new();
    let id = format!("tunnel-{}", local_port);

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
        Ok(Some(url)) => public_url = url,
        _ => {
            let _ = child.kill().await;
            return Err("Failed to extract Cloudflare URL within timeout.".into());
        }
    }

    tokio::spawn(async move {
        while let Ok(Some(_)) = reader.next_line().await {}
    });

    db.log_public_link(&id, &session_id, &public_url, "cloudflare_quick", "public")?;
    manager.tunnels.lock().unwrap().insert(id.clone(), child);

    Ok(TunnelSession {
        id,
        local_port,
        public_url,
    })
}

#[tauri::command]
pub async fn stop_tunnel(
    manager: tauri::State<'_, TunnelManager>,
    db: tauri::State<'_, DatabaseManager>,
    id: String,
) -> Result<(), String> {
    let child_opt = {
        let mut tunnels = manager.tunnels.lock().unwrap();
        tunnels.remove(&id)
    };
    if let Some(mut child) = child_opt {
        let _ = child.kill().await;
    }
    let _ = db.remove_public_link(&id);
    Ok(())
}

pub fn auto_restore_tunnels(app: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        let db = app.state::<DatabaseManager>();
        let manager = app.state::<TunnelManager>();
        
        if let Ok(links) = db.get_active_links() {
            for link in links {
                if link.provider == "cloudflare_quick" {
                    let port_str = link.id.replace("tunnel-", "");
                    if let Ok(local_port) = port_str.parse::<u16>() {
                        
                        let check_container = std::process::Command::new("docker")
                            .args(["ps", "-q", "-f", &format!("name={}", link.session_id)])
                            .output();
                            
                        // Validate container existence before restoring tunnel
                        let is_running = check_container.map(|o| !o.stdout.is_empty()).unwrap_or(false);
                        
                        if !is_running {
                            println!("Skipping tunnel restore for {}, session dead.", link.session_id);
                            let _ = db.remove_public_link(&link.id);
                            continue;
                        }

                        let child_res = Command::new("cloudflared")
                            .arg("tunnel")
                            .arg("--url")
                            .arg(&format!("http://localhost:{}", local_port))
                            .stdout(Stdio::piped())
                            .stderr(Stdio::piped())
                            .spawn();
                            
                        if let Ok(mut child) = child_res {
                            if let Some(stderr) = child.stderr.take() {
                                let mut reader = BufReader::new(stderr).lines();
                                let mut new_url = String::new();
                                
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

                                if let Ok(Some(url)) = tokio::time::timeout(timeout_duration, fetch_url).await {
                                    new_url = url;
                                    let _ = db.log_public_link(&link.id, &link.session_id, &new_url, "cloudflare_quick", "public");
                                    manager.tunnels.lock().unwrap().insert(link.id.clone(), child);
                                    
                                    tokio::spawn(async move {
                                        while let Ok(Some(_)) = reader.next_line().await {}
                                    });
                                } else {
                                    let _ = child.kill().await;
                                    let _ = db.remove_public_link(&link.id);
                                }
                            }
                        }
                    }
                }
            }
        }
    });
}
