import React, { useState, useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';

interface OrchestrationAlert {
  severity: string;
  title: string;
  description: string;
}

export const OrchestrationToast: React.FC = () => {
  const [alerts, setAlerts] = useState<OrchestrationAlert[]>([]);

  useEffect(() => {
    const unlisten = listen<OrchestrationAlert>('orchestration-alert', (event) => {
      setAlerts((prev) => [...prev, event.payload]);
      
      // Auto-dismiss after 10 seconds
      setTimeout(() => {
        setAlerts((prev) => prev.slice(1));
      }, 10000);
    });

    return () => {
      unlisten.then(f => f());
    };
  }, []);

  if (alerts.length === 0) return null;

  return (
    <div className="orchestration-toast-container">
      {alerts.map((alert, idx) => (
        <div key={idx} className={`orchestration-toast ${alert.severity}`}>
          <div className="toast-icon">
            {alert.severity === 'critical' ? '⚡' : '⚠️'}
          </div>
          <div className="toast-content">
            <h4>{alert.title}</h4>
            <p>{alert.description}</p>
          </div>
        </div>
      ))}
    </div>
  );
};
