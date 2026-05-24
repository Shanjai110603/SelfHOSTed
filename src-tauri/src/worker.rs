use tokio::sync::mpsc;
use tauri::{Manager, Emitter};
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::Mutex;

pub enum WorkerJob {
    ProcessExpirations,
    TickTelemetry,
    ReconcileExposure,
}

pub struct OrchestrationWorker {
    #[allow(dead_code)]
    pub tx: mpsc::UnboundedSender<WorkerJob>,
    pub rx: Mutex<Option<mpsc::UnboundedReceiver<WorkerJob>>>,
}

pub fn start_orchestration_worker(app: tauri::AppHandle) {
    let (tx, rx) = mpsc::unbounded_channel::<WorkerJob>();
    
    // Manage the sender so other modules can push jobs
    app.manage(OrchestrationWorker { tx: tx.clone(), rx: Mutex::new(Some(rx)) });

    let worker = app.state::<OrchestrationWorker>();
    let mut rx = worker.rx.lock().unwrap().take().expect("Worker already started");

    // Spawn the primary worker consumer loop
    tauri::async_runtime::spawn(async move {
        while let Some(job) = rx.recv().await {
            match job {
                WorkerJob::ProcessExpirations => {
                    handle_expirations(&app).await;
                }
                WorkerJob::TickTelemetry => {
                    handle_telemetry(&app).await;
                }
                WorkerJob::ReconcileExposure => {
                    handle_exposure_reconciliation(&app).await;
                }
            }
        }
    });

    // Spawn a periodic ticker that pushes recurring jobs into the queue
    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(10));
        loop {
            interval.tick().await;
            let _ = tx.send(WorkerJob::ProcessExpirations);
            let _ = tx.send(WorkerJob::TickTelemetry);
            let _ = tx.send(WorkerJob::ReconcileExposure);
        }
    });
}

async fn handle_expirations(app: &tauri::AppHandle) {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let mut expired = Vec::new();

    {
        let manager = app.state::<crate::session::SessionManager>();
        let mut map = manager.sessions.lock().unwrap();
        for (_id, session) in map.iter() {
            if now >= session.expires_at {
                expired.push(session.clone());
            }
        }
        for s in &expired {
            map.remove(&s.id);
        }
    }

    for session in expired {
        println!("Worker: Session expired: {}", session.id);
        if session.resource_type == "website" || session.resource_type == "database" || session.resource_type == "fileshare" {
            let _ = tokio::process::Command::new("docker")
                .args(["rm", "-f", &session.id])
                .output().await;
        } else if session.resource_type == "tunnel" {
            let net = app.state::<crate::network::ActiveNetworkState>();
            let db = app.state::<crate::db::DatabaseManager>();
            // Attempt to revoke from both providers since we don't store provider in ManagedSession directly for now
            let _ = net.cloudflare.revoke_exposure(&session.id).await;
            let _ = net.tailscale.revoke_exposure(&session.id).await;
            let _ = db.remove_public_link(&session.id);
        }
    }
}

async fn handle_telemetry(app: &tauri::AppHandle) {
    let provider = app.state::<std::sync::Arc<dyn crate::telemetry::TelemetryProvider>>();
    let telemetry_payload = provider.get_telemetry().await;

    // Broadcast to frontend
    let _ = app.emit("telemetry-update", telemetry_payload.clone());

    // Evaluate orchestration policies
    let policies = app.state::<Vec<std::sync::Arc<dyn crate::policy::OrchestrationPolicy>>>();
    for policy in policies.iter() {
        if let Some(alert) = policy.evaluate(&telemetry_payload, app).await {
            let _ = app.emit("orchestration-alert", alert);
        }
    }
}

async fn handle_exposure_reconciliation(app: &tauri::AppHandle) {
    let db = app.state::<crate::db::DatabaseManager>();
    let net = app.state::<crate::network::ActiveNetworkState>();
    
    // Attempt to fetch active links; if none or error, exit early
    if let Ok(links) = db.get_active_links() {
        for link in links {
            // Check if the workload container itself is still alive
            let check_container = std::process::Command::new("docker")
                .args(["ps", "-q", "-f", &format!("name={}", link.session_id)])
                .output();
                
            let is_running = check_container.map(|o| !o.stdout.is_empty()).unwrap_or(false);
            
            if !is_running {
                println!("Orchestrator: Workload {} is dead, cleaning up orphaned exposure: {}", link.session_id, link.id);
                let _ = net.cloudflare.revoke_exposure(&link.id).await;
                let _ = net.tailscale.revoke_exposure(&link.id).await;
                let _ = db.remove_public_link(&link.id);
                continue;
            }

            // If workload is alive, ensure the provider tunnel is also alive.
            // Simplified reconciliation: We just verify provider intent.
            // A more advanced approach would query the `NetworkProvider::check_status()` trait method.
        }
    }
}
