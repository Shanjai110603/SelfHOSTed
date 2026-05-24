use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tauri::Manager;
use crate::runtime::{StackConfig, WorkloadConfig, ExposureConfig, ActiveRuntime};
use crate::network::ActiveNetworkState;
use crate::proxy::ProxyProvider;
use crate::vault::VaultManager;
use crate::db::DatabaseManager;

#[derive(Serialize, Deserialize, Clone)]
pub struct AppTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub category: String,
    pub recommended_ram_gb: u32,
    pub workloads: Vec<WorkloadConfig>,
}

// In Option B (Local JSON Registry), this reads from the app_data_dir/templates folder.
// For the MVP, we will bundle a few hardcoded JSON strings that simulate the file read
// so we don't have to ship a separate zip file, but architecturally it parses JSON exactly like Option B.
pub struct TemplateEngine {
    pub templates: HashMap<String, AppTemplate>,
}

impl TemplateEngine {
    pub fn new() -> Self {
        let mut templates = HashMap::new();
        
        let wordpress_json = r#"{
            "id": "wordpress-stack",
            "name": "WordPress",
            "description": "The world's most popular CMS. Includes MariaDB backend.",
            "icon": "wordpress",
            "category": "CMS",
            "recommended_ram_gb": 2,
            "workloads": [
                {
                    "id": "db",
                    "resource_type": "database",
                    "template": "mariadb",
                    "port": 3306,
                    "env_vars": {
                        "MARIADB_USER": "wordpress",
                        "MARIADB_DATABASE": "wordpress"
                    },
                    "domain": null,
                    "host_path": null,
                    "exposure": null,
                    "stack_id": null,
                    "network_alias": "db",
                    "depends_on": []
                },
                {
                    "id": "web",
                    "resource_type": "website",
                    "template": "wordpress",
                    "port": 80,
                    "env_vars": {
                        "WORDPRESS_DB_HOST": "db",
                        "WORDPRESS_DB_USER": "wordpress",
                        "WORDPRESS_DB_NAME": "wordpress"
                    },
                    "domain": null,
                    "host_path": null,
                    "exposure": null,
                    "stack_id": null,
                    "network_alias": "web",
                    "depends_on": ["db"]
                }
            ]
        }"#;

        if let Ok(wp) = serde_json::from_str::<AppTemplate>(wordpress_json) {
            templates.insert(wp.id.clone(), wp);
        }

        let hello_world_json = r#"{
            "id": "hello-world",
            "name": "Hello World",
            "description": "A lightning fast, static demonstration website.",
            "icon": "globe",
            "category": "Demo",
            "recommended_ram_gb": 0,
            "workloads": [
                {
                    "id": "web",
                    "resource_type": "website",
                    "template": "static",
                    "port": 80,
                    "env_vars": {},
                    "domain": null,
                    "host_path": null,
                    "exposure": null,
                    "stack_id": null,
                    "network_alias": "web",
                    "depends_on": []
                }
            ]
        }"#;

        if let Ok(hw) = serde_json::from_str::<AppTemplate>(hello_world_json) {
            templates.insert(hw.id.clone(), hw);
        }

        Self { templates }
    }

    pub fn get_templates(&self) -> Vec<AppTemplate> {
        self.templates.values().cloned().collect()
    }
}

#[tauri::command]
pub async fn get_marketplace_templates(
    engine: tauri::State<'_, Arc<TemplateEngine>>
) -> Result<Vec<AppTemplate>, String> {
    Ok(engine.get_templates())
}

#[tauri::command]
pub async fn deploy_stack(
    engine: tauri::State<'_, Arc<TemplateEngine>>,
    runtime: tauri::State<'_, ActiveRuntime>,
    proxy: tauri::State<'_, Arc<dyn ProxyProvider>>,
    net: tauri::State<'_, ActiveNetworkState>,
    vault: tauri::State<'_, VaultManager>,
    db: tauri::State<'_, DatabaseManager>,
    app: tauri::AppHandle,
    template_id: String,
    domain: Option<String>,
    exposure: Option<ExposureConfig>,
) -> Result<StackConfig, String> {
    
    let template = engine.templates.get(&template_id).ok_or("Template not found in local registry")?;
    
    let stack_id = format!("stack-{}", uuid::Uuid::new_v4().to_string().chars().take(8).collect::<String>());
    
    // Sort workloads by dependencies (naive sort for MVP: workloads with empty depends_on first)
    let mut sorted_workloads = template.workloads.clone();
    sorted_workloads.sort_by(|a, b| {
        let a_deps = a.depends_on.as_ref().map(|d| d.len()).unwrap_or(0);
        let b_deps = b.depends_on.as_ref().map(|d| d.len()).unwrap_or(0);
        a_deps.cmp(&b_deps)
    });

    let mut generated_db_passwords: HashMap<String, String> = HashMap::new();
    let mut running_workloads = Vec::new();

    for mut w in sorted_workloads {
        // Inject Stack UUID mapping
        w.stack_id = Some(stack_id.clone());
        let original_alias = w.network_alias.clone().unwrap_or(w.id.clone());
        w.id = format!("{}-{}", stack_id, original_alias);

        // Vault Secret Generation & Injection (Propagating secrets to dependent workloads)
        if w.resource_type == "database" {
            let raw_password = vault.generate_secure_password();
            let encrypted = vault.encrypt_string(&raw_password)?;
            let _ = db.save_secret_internal(&format!("{}_admin_password", w.id), "database_password", &encrypted, None);
            
            // Standardize injection based on engine
            match w.template.as_str() {
                "mysql" | "mariadb" => { w.env_vars.insert("MARIADB_ROOT_PASSWORD".to_string(), raw_password.clone()); w.env_vars.insert("MARIADB_PASSWORD".to_string(), raw_password.clone()); },
                "postgres" => { w.env_vars.insert("POSTGRES_PASSWORD".to_string(), raw_password.clone()); },
                _ => {}
            }
            
            // Store it to inject into frontend web workloads
            generated_db_passwords.insert(original_alias.clone(), raw_password);
        } else if w.resource_type == "website" {
            // Check if this website depends on a DB that we just generated a password for
            if let Some(deps) = &w.depends_on {
                for dep in deps {
                    if let Some(pwd) = generated_db_passwords.get(dep) {
                        // Very naive injection for WordPress MVP
                        w.env_vars.insert("WORDPRESS_DB_PASSWORD".to_string(), pwd.clone());
                    }
                }
            }

            // Only expose the FRONTEND website to Traefik / Network Exposure, not the DB
            w.domain = domain.clone();
            w.exposure = exposure.clone();
        }

        // Call the underlying generic orchestrator!
        // We bypass the IPC command `crate::runtime::start_workload` slightly to avoid cloning states redundantly,
        // so we call the exact same logic directly.
        let session = runtime.provider.start_workload(&w).await?;
        
        // Wait 3 seconds to let databases initialize before starting the web server.
        // In a real environment, we would poll Docker healthchecks.
        if w.resource_type == "database" {
            sleep(Duration::from_secs(4)).await;
        }

        // Proxy Route (only if domain exists and we are a frontend app)
        if let Some(dom) = &w.domain {
            if !dom.is_empty() {
                let config_dir = app.path().app_data_dir().unwrap_or_default().join("proxy");
                let auth = w.require_auth.unwrap_or(false);
                proxy.add_route(config_dir, &session.id, dom, "127.0.0.1", w.port, auth).await?;
            }
        }

        // Network Exposure (only if exposure exists)
        if let Some(exp) = &w.exposure {
            if exp.provider != "none" {
                let _ = match exp.provider.as_str() {
                    "cloudflare" => net.cloudflare.expose_workload(&session.id, w.port, &exp.mode, exp.token.as_deref()).await,
                    "tailscale" => net.tailscale.expose_workload(&session.id, w.port, &exp.mode, exp.token.as_deref()).await,
                    _ => Err("Unknown exposure provider".into())
                };
            }
        }

        running_workloads.push(w);
    }

    Ok(StackConfig {
        id: stack_id,
        name: template.name.clone(),
        workloads: running_workloads,
        exposure,
    })
}
