use tokio::sync::mpsc;
use tauri::{Manager, Emitter};
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::Mutex;

pub enum WorkerJob {
    ProcessExpirations,
    TickTelemetry,
    // Future extensions:
    // RecoverTunnels,
    // VerifyRuntimeHealth,
}

pub struct OrchestrationWorker {
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
        }
    });
}

async fn handle_expirations(app: &tauri::AppHandle) {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let mut expired = Vec::new();

    {
        let manager = app.state::<crate::session::SessionManager>();
        let mut map = manager.sessions.lock().unwrap();
        for (id, session) in map.iter() {
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
            use crate::network::TunnelManager;
            let tunnel_manager = app.state::<TunnelManager>();
            let child_opt = {
                let mut tunnels = tunnel_manager.tunnels.lock().unwrap();
                tunnels.remove(&session.id)
            };
            if let Some(mut child) = child_opt {
                let _ = child.kill().await;
            }
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
