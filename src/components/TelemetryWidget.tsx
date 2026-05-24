import React from 'react';
import { useTelemetry } from '../hooks/useTelemetry';

export const TelemetryWidget: React.FC = () => {
  const telemetry = useTelemetry();

  if (!telemetry) {
    return (
      <div className="telemetry-widget loading">
        <p>Initializing Adaptive Resource Intelligence...</p>
      </div>
    );
  }

  const memoryPercent = (telemetry.memory_used_mb / telemetry.memory_total_mb) * 100;
  
  return (
    <div className="telemetry-widget glass-panel">
      <div className="telemetry-header">
        <h3>System Telemetry</h3>
        <span className="live-indicator">● LIVE</span>
      </div>
      
      <div className="telemetry-grid">
        <div className="telemetry-item">
          <label>CPU Usage</label>
          <div className="progress-bar">
            <div 
              className="progress-fill" 
              style={{ width: `${Math.min(telemetry.cpu_usage, 100)}%`, backgroundColor: telemetry.cpu_usage > 85 ? '#ff4d4f' : '#52c41a' }}
            ></div>
          </div>
          <span>{telemetry.cpu_usage.toFixed(1)}%</span>
        </div>

        <div className="telemetry-item">
          <label>Memory Pressure</label>
          <div className="progress-bar">
            <div 
              className="progress-fill" 
              style={{ width: `${Math.min(memoryPercent, 100)}%`, backgroundColor: memoryPercent > 90 ? '#ff4d4f' : '#1890ff' }}
            ></div>
          </div>
          <span>{(telemetry.memory_used_mb / 1024).toFixed(1)}GB / {(telemetry.memory_total_mb / 1024).toFixed(1)}GB</span>
        </div>

        <div className="telemetry-item mini-stats">
          <div className="stat-pill">
            <span className="icon">🌡️</span>
            <span>{telemetry.temperature_c !== null ? `${telemetry.temperature_c.toFixed(1)}°C` : 'N/A'}</span>
          </div>
          <div className="stat-pill">
            <span className="icon">{telemetry.is_on_ac_power ? '🔌' : '🔋'}</span>
            <span>{telemetry.battery_percent !== null ? `${telemetry.battery_percent.toFixed(0)}%` : 'Desktop'}</span>
          </div>
        </div>
      </div>
    </div>
  );
};
