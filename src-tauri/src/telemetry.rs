use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use sysinfo::{System, Components};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TelemetryPayload {
    pub cpu_usage: f32, // 0.0 to 100.0
    pub memory_used_mb: u64,
    pub memory_total_mb: u64,
    pub temperature_c: Option<f32>,
    pub battery_percent: Option<f32>,
    pub is_on_ac_power: Option<bool>,
}

#[async_trait::async_trait]
pub trait TelemetryProvider: Send + Sync {
    async fn get_telemetry(&self) -> TelemetryPayload;
}

pub struct SysinfoTelemetryProvider {
    system: Mutex<System>,
    components: Mutex<Components>,
}

impl SysinfoTelemetryProvider {
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        let mut components = Components::new_with_refreshed_list();
        components.refresh();

        Self {
            system: Mutex::new(system),
            components: Mutex::new(components),
        }
    }
}

#[async_trait::async_trait]
impl TelemetryProvider for SysinfoTelemetryProvider {
    async fn get_telemetry(&self) -> TelemetryPayload {
        let mut sys = self.system.lock().unwrap();
        sys.refresh_cpu();
        sys.refresh_memory();

        let cpu_usage = sys.global_cpu_info().cpu_usage();
        let memory_used_mb = sys.used_memory() / 1024 / 1024;
        let memory_total_mb = sys.total_memory() / 1024 / 1024;
        
        // Find maximum temperature across components
        let mut max_temp = None;
        let mut comps = self.components.lock().unwrap();
        comps.refresh();
        for comp in comps.list() {
            let temp = comp.temperature();
            if temp > max_temp.unwrap_or(0.0) {
                max_temp = Some(temp);
            }
        }

        // Battery telemetry
        let mut battery_percent = None;
        let mut is_on_ac_power = None;
        
        if let Ok(manager) = battery::Manager::new() {
            if let Ok(mut batteries) = manager.batteries() {
                if let Some(Ok(mut bat)) = batteries.next() {
                    let _ = manager.refresh(&mut bat);
                    battery_percent = Some(bat.state_of_charge().value * 100.0);
                    is_on_ac_power = match bat.state() {
                        battery::State::Charging | battery::State::Full => Some(true),
                        battery::State::Discharging | battery::State::Empty => Some(false),
                        _ => None,
                    };
                }
            }
        }

        TelemetryPayload {
            cpu_usage,
            memory_used_mb,
            memory_total_mb,
            temperature_c: max_temp,
            battery_percent,
            is_on_ac_power,
        }
    }
}
