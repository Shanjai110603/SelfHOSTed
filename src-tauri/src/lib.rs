mod system;
mod runtime;
mod network;
mod session;
mod db;
mod worker;
mod vault;
pub mod telemetry;
pub mod policy;
pub mod proxy;

use tauri::Manager;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(system::SystemMonitor::new())
        .manage(session::SessionManager::new())
        .setup(|app| {
            // 1. Initialize VaultManager (OS Keyring + Cryptography)
            // If this fails (e.g., headless linux without ENV), we fail initialization cleanly.
            let vault_manager = match vault::VaultManager::new() {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("CRITICAL SECURITY FAILURE: {}", e);
                    return Err(e.into());
                }
            };
            app.manage(vault_manager);

            // 2. Initialize Database
            let app_data_dir = app.path().app_data_dir().expect("failed to get app data dir");
            let db_manager = db::DatabaseManager::new(app_data_dir).expect("Failed to initialize database");
            app.manage(db_manager);

            // 3. Initialize Runtime Provider
            let provider: std::sync::Arc<dyn runtime::RuntimeProvider> = if cfg!(target_os = "android") {
                std::sync::Arc::new(runtime::NativeProvider::new())
            } else {
                std::sync::Arc::new(runtime::DockerProvider::new())
            };
            app.manage(runtime::ActiveRuntime { provider });

            // 3.5 Initialize Proxy Provider
            let proxy_provider: std::sync::Arc<dyn proxy::ProxyProvider> = std::sync::Arc::new(proxy::TraefikProxyProvider::new());
            app.manage(proxy_provider.clone());
            
            let config_dir = app.path().app_data_dir().unwrap_or_default().join("proxy");
            tauri::async_runtime::spawn(async move {
                if let Err(e) = proxy_provider.start_proxy(config_dir).await {
                    eprintln!("Failed to start proxy provider: {}", e);
                }
            });

            // 4. Initialize Telemetry and Policy Engine
            let telemetry_provider: std::sync::Arc<dyn telemetry::TelemetryProvider> = std::sync::Arc::new(telemetry::SysinfoTelemetryProvider::new());
            app.manage(telemetry_provider);

            let policies: Vec<std::sync::Arc<dyn policy::OrchestrationPolicy>> = vec![
                std::sync::Arc::new(policy::ThermalProtectionPolicy::new(85.0)),
                std::sync::Arc::new(policy::BatteryProtectionPolicy::new(20.0)),
            ];
            app.manage(policies);

            // 5. Initialize Network Providers
            let network_state = network::ActiveNetworkState {
                cloudflare: std::sync::Arc::new(network::CloudflareProvider::new()),
                tailscale: std::sync::Arc::new(network::TailscaleProvider::new()),
            };
            app.manage(network_state);
            
            // Recover any orphaned but running containers
            session::auto_recover_sessions(app.handle().clone());

            // Start the background orchestration worker queue
            worker::start_orchestration_worker(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            system::get_system_stats,
            runtime::get_capabilities,
            runtime::check_runtime,
            runtime::start_workload,
            runtime::stop_workload,
            runtime::pause_session,
            runtime::resume_session,
            network::expose_workload,
            network::revoke_exposure,
            session::register_session,
            session::update_session_status,
            session::get_active_sessions,
            db::save_preference,
            db::get_preference,
            db::log_session,
            db::get_recent_sessions,
            db::save_secret,
            db::get_secret
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
