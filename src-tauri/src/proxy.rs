use async_trait::async_trait;
use std::path::PathBuf;
use tokio::process::Command;
use tokio::fs;

#[async_trait]
pub trait ProxyProvider: Send + Sync {
    async fn start_proxy(&self, config_dir: PathBuf) -> Result<(), String>;
    async fn add_route(&self, config_dir: PathBuf, workload_id: &str, domain: &str, target_ip: &str, target_port: u16) -> Result<(), String>;
    async fn remove_route(&self, config_dir: PathBuf, workload_id: &str) -> Result<(), String>;
}

pub struct TraefikProxyProvider;

impl TraefikProxyProvider {
    pub fn new() -> Self {
        Self {}
    }

    async fn ensure_traefik_config(&self, config_dir: &PathBuf) -> Result<(), String> {
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir).await.map_err(|e| e.to_string())?;
        }

        let static_config_path = config_dir.join("traefik.yml");
        if !static_config_path.exists() {
            let static_config = r#"
[entryPoints]
  [entryPoints.web]
    address = ":80"
  [entryPoints.websecure]
    address = ":443"

[providers.file]
  directory = "/etc/traefik/dynamic"
  watch = true

[api]
  insecure = true
  dashboard = true
"#;
            fs::write(static_config_path, static_config).await.map_err(|e| e.to_string())?;
        }

        let dynamic_dir = config_dir.join("dynamic");
        if !dynamic_dir.exists() {
            fs::create_dir_all(&dynamic_dir).await.map_err(|e| e.to_string())?;
        }

        Ok(())
    }
}

#[async_trait]
impl ProxyProvider for TraefikProxyProvider {
    async fn start_proxy(&self, config_dir: PathBuf) -> Result<(), String> {
        self.ensure_traefik_config(&config_dir).await?;

        // Stop any existing instance
        let _ = Command::new("docker").args(["rm", "-f", "selfhosted-proxy"]).output().await;

        let config_dir_str = config_dir.to_string_lossy().to_string();
        
        let status = Command::new("docker")
            .args([
                "run", "-d",
                "--name", "selfhosted-proxy",
                "--restart", "unless-stopped",
                "-p", "80:80",
                "-p", "443:443",
                "-p", "8080:8080", // Dashboard
                "-v", &format!("{}:/etc/traefik", config_dir_str),
                "traefik:v3.0"
            ])
            .status()
            .await
            .map_err(|e| e.to_string())?;

        if !status.success() {
            return Err("Failed to start Traefik proxy container.".to_string());
        }

        Ok(())
    }

    async fn add_route(&self, config_dir: PathBuf, workload_id: &str, domain: &str, target_ip: &str, target_port: u16) -> Result<(), String> {
        // We write a dynamic YAML file per workload
        let dynamic_dir = config_dir.join("dynamic");
        let route_file = dynamic_dir.join(format!("{}.yml", workload_id));

        // Note: For localhost testing, we use HostRegexp or strict Host matching
        let rule = format!("Host(`{}`)", domain);

        let config = format!(r#"
http:
  routers:
    router-{id}:
      rule: "{rule}"
      service: "service-{id}"
      entryPoints:
        - "web"

  services:
    service-{id}:
      loadBalancer:
        servers:
          - url: "http://{ip}:{port}"
"#,
            id = workload_id,
            rule = rule,
            ip = target_ip,
            port = target_port
        );

        fs::write(route_file, config).await.map_err(|e| e.to_string())?;

        Ok(())
    }

    async fn remove_route(&self, config_dir: PathBuf, workload_id: &str) -> Result<(), String> {
        let dynamic_dir = config_dir.join("dynamic");
        let route_file = dynamic_dir.join(format!("{}.yml", workload_id));

        if route_file.exists() {
            fs::remove_file(route_file).await.map_err(|e| e.to_string())?;
        }

        Ok(())
    }
}
