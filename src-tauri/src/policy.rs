use crate::telemetry::TelemetryPayload;
use crate::runtime::ActiveRuntime;
use tauri::Manager;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OrchestrationAlert {
    pub severity: String,
    pub title: String,
    pub description: String,
}

#[async_trait::async_trait]
pub trait OrchestrationPolicy: Send + Sync {
    async fn evaluate(&self, telemetry: &TelemetryPayload, app: &tauri::AppHandle) -> Option<OrchestrationAlert>;
}

// ==========================================
// THERMAL PROTECTION POLICY
// ==========================================
pub struct ThermalProtectionPolicy {
    pub max_safe_temp: f32,
}

impl ThermalProtectionPolicy {
    pub fn new(max_safe_temp: f32) -> Self {
        Self { max_safe_temp }
    }
}

#[async_trait::async_trait]
impl OrchestrationPolicy for ThermalProtectionPolicy {
    async fn evaluate(&self, telemetry: &TelemetryPayload, app: &tauri::AppHandle) -> Option<OrchestrationAlert> {
        if let Some(temp) = telemetry.temperature_c {
            if temp > self.max_safe_temp {
                // Throttle active database workloads
                let session_manager = app.state::<crate::session::SessionManager>();
                let runtime = app.state::<ActiveRuntime>();
                
                let mut paused_any = false;
                
                // We use a clone of the sessions to avoid holding the lock across await points
                let active_sessions = {
                    let map = session_manager.sessions.lock().unwrap();
                    map.values().cloned().collect::<Vec<_>>()
                };

                for session in active_sessions {
                    if session.resource_type == "database" && session.status != "Paused" {
                        if let Ok(_) = runtime.provider.pause_session(&session.id).await {
                            // Update state
                            let mut map = session_manager.sessions.lock().unwrap();
                            if let Some(s) = map.get_mut(&session.id) {
                                s.status = "Paused".to_string();
                            }
                            paused_any = true;
                        }
                    }
                }

                if paused_any {
                    return Some(OrchestrationAlert {
                        severity: "warning".to_string(),
                        title: "Thermal Protection Triggered".to_string(),
                        description: format!("Your device temperature exceeded {}°C. Heavy workloads were paused to protect your device.", self.max_safe_temp),
                    });
                }
            }
        }
        None
    }
}

// ==========================================
// BATTERY PROTECTION POLICY
// ==========================================
pub struct BatteryProtectionPolicy {
    pub min_safe_percent: f32,
}

impl BatteryProtectionPolicy {
    pub fn new(min_safe_percent: f32) -> Self {
        Self { min_safe_percent }
    }
}

#[async_trait::async_trait]
impl OrchestrationPolicy for BatteryProtectionPolicy {
    async fn evaluate(&self, telemetry: &TelemetryPayload, app: &tauri::AppHandle) -> Option<OrchestrationAlert> {
        if let (Some(battery), Some(is_on_ac)) = (telemetry.battery_percent, telemetry.is_on_ac_power) {
            if battery < self.min_safe_percent && !is_on_ac {
                // Throttle workloads to save power
                let session_manager = app.state::<crate::session::SessionManager>();
                let runtime = app.state::<ActiveRuntime>();
                
                let mut paused_any = false;
                
                let active_sessions = {
                    let map = session_manager.sessions.lock().unwrap();
                    map.values().cloned().collect::<Vec<_>>()
                };

                for session in active_sessions {
                    // Pause everything except critical fileshares
                    if session.resource_type != "fileshare" && session.status != "Paused" {
                        if let Ok(_) = runtime.provider.pause_session(&session.id).await {
                            let mut map = session_manager.sessions.lock().unwrap();
                            if let Some(s) = map.get_mut(&session.id) {
                                s.status = "Paused".to_string();
                            }
                            paused_any = true;
                        }
                    }
                }

                if paused_any {
                    return Some(OrchestrationAlert {
                        severity: "critical".to_string(),
                        title: "Battery Protection Triggered".to_string(),
                        description: format!("Laptop battery dropped below {}%. Background services have been paused to conserve power.", self.min_safe_percent),
                    });
                }
            }
        }
        None
    }
}
