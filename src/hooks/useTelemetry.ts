import { useState, useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';

export interface TelemetryPayload {
  cpu_usage: number; // 0.0 to 100.0
  memory_used_mb: number;
  memory_total_mb: number;
  temperature_c: number | null;
  battery_percent: number | null;
  is_on_ac_power: boolean | null;
}

export function useTelemetry() {
  const [telemetry, setTelemetry] = useState<TelemetryPayload | null>(null);

  useEffect(() => {
    const unlisten = listen<TelemetryPayload>('telemetry-update', (event) => {
      setTelemetry(event.payload);
    });

    return () => {
      unlisten.then(f => f());
    };
  }, []);

  return telemetry;
}
