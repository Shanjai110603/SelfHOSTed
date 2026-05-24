import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { GlassCard } from '../components/GlassCard';
import { CheckCircle, Clock, AlertTriangle, FastForward, Database } from 'lucide-react';

interface SessionHistoryEntry {
  id: string;
  resource_type: string;
  path?: string;
  port?: number;
  started_at: string;
}

export function DevDashboard() {
  const [history, setHistory] = useState<SessionHistoryEntry[]>([]);

  useEffect(() => {
    const fetchHistory = async () => {
      try {
        const data = await invoke<SessionHistoryEntry[]>('get_recent_sessions');
        setHistory(data);
      } catch (e) {
        console.error(e);
      }
    };
    fetchHistory();
  }, []);
  return (
    <div className="animate-fade-in" style={{ paddingBottom: '40px' }}>
      <header style={{ marginBottom: '40px' }}>
        <h1 className="title-large" style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
          <span style={{ color: 'var(--accent-color)' }}>[DEV]</span> Visual Development Dashboard
        </h1>
        <p className="text-secondary">Project roadmap, task tracking, and milestone overview.</p>
      </header>

      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(300px, 1fr))', gap: '24px', marginBottom: '40px' }}>
        <GlassCard title="Project Progress" icon={<CheckCircle size={24} />}>
          <div style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
            <div>
              <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '8px' }}>
                <span className="text-secondary">MVP Core Architecture</span>
                <span>100%</span>
              </div>
              <div style={{ width: '100%', height: '8px', background: 'rgba(255,255,255,0.1)', borderRadius: '4px' }}>
                <div style={{ width: '100%', height: '100%', background: 'var(--success-color)', borderRadius: '4px', boxShadow: '0 0 10px var(--success-glow)' }}></div>
              </div>
            </div>

            <div>
              <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '8px' }}>
                <span className="text-secondary">Phase 1 Polish</span>
                <span>80%</span>
              </div>
              <div style={{ width: '100%', height: '8px', background: 'rgba(255,255,255,0.1)', borderRadius: '4px' }}>
                <div style={{ width: '80%', height: '100%', background: '#f59e0b', borderRadius: '4px', boxShadow: '0 0 10px rgba(245, 158, 11, 0.5)' }}></div>
              </div>
            </div>

            <div>
              <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '8px' }}>
                <span className="text-secondary">Phase 2 (Post-MVP)</span>
                <span>0%</span>
              </div>
              <div style={{ width: '100%', height: '8px', background: 'rgba(255,255,255,0.1)', borderRadius: '4px' }}>
                <div style={{ width: '0%', height: '100%', background: 'var(--accent-color)', borderRadius: '4px' }}></div>
              </div>
            </div>
          </div>
        </GlassCard>

        <GlassCard title="Current Active Tasks" icon={<Clock size={24} />}>
           <ul style={{ display: 'flex', flexDirection: 'column', gap: '16px', padding: 0, margin: 0, listStyle: 'none' }}>
              <li style={{ background: 'rgba(245, 158, 11, 0.1)', borderLeft: '4px solid #f59e0b', padding: '12px', borderRadius: '4px' }}>
                <strong style={{ display: 'block', color: '#fcd34d' }}>TASK-012: Make Cloudflare Fully Functional</strong>
                <span className="text-secondary" style={{ fontSize: '0.9rem' }}>Execute tokio::process::Child to manage cloudflared daemon.</span>
              </li>
              <li style={{ background: 'rgba(245, 158, 11, 0.1)', borderLeft: '4px solid #f59e0b', padding: '12px', borderRadius: '4px' }}>
                <strong style={{ display: 'block', color: '#fcd34d' }}>TASK-013: Build Dev Dashboard</strong>
                <span className="text-secondary" style={{ fontSize: '0.9rem' }}>Implement visual UI for internal tracking.</span>
              </li>
           </ul>
        </GlassCard>
      </div>

      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(300px, 1fr))', gap: '24px' }}>
        <GlassCard title="Known Issues & Debt" icon={<AlertTriangle size={24} />}>
          <ul style={{ display: 'flex', flexDirection: 'column', gap: '16px', padding: 0, margin: 0, listStyle: 'none' }}>
            <li style={{ background: 'rgba(239, 68, 68, 0.1)', borderLeft: '4px solid #ef4444', padding: '12px', borderRadius: '4px' }}>
              <strong style={{ display: 'block', color: '#fca5a5' }}>DEBT-001: Docker Dependency Assumption</strong>
              <span className="text-secondary" style={{ fontSize: '0.9rem' }}>Needs robust Rust detection if Docker daemon is frozen or missing.</span>
            </li>
            <li style={{ background: 'rgba(239, 68, 68, 0.1)', borderLeft: '4px solid #ef4444', padding: '12px', borderRadius: '4px' }}>
              <strong style={{ display: 'block', color: '#fca5a5' }}>DEBT-002: Cloudflare Binary Auto-fetch</strong>
              <span className="text-secondary" style={{ fontSize: '0.9rem' }}>Implement an auto-downloader for the `cloudflared` binary.</span>
            </li>
          </ul>
        </GlassCard>

        <GlassCard title="Future Roadmap" icon={<FastForward size={24} />}>
           <ul style={{ display: 'flex', flexDirection: 'column', gap: '12px', margin: 0, paddingLeft: '20px', color: 'var(--text-secondary)' }}>
             <li style={{ color: 'var(--success-color)' }}><strong>[x] TASK-014:</strong> Persistent Database Layer (SQLite)</li>
             <li><strong>TASK-015:</strong> Session Lifecycle Management</li>
             <li><strong>TASK-016:</strong> Public Exposure Persistence</li>
             <li><strong>TASK-017:</strong> Runtime State Recovery</li>
           </ul>
        </GlassCard>

        <GlassCard title="DB Diagnostics" icon={<Database size={24} />}>
          {history.length === 0 ? (
            <p className="text-secondary" style={{ fontSize: '0.9rem' }}>No recent sessions found in SQLite. Try starting a website!</p>
          ) : (
            <ul style={{ display: 'flex', flexDirection: 'column', gap: '8px', padding: 0, margin: 0, listStyle: 'none' }}>
              {history.map((h, i) => (
                <li key={i} style={{ background: 'rgba(0,0,0,0.2)', padding: '8px', borderRadius: '4px', fontSize: '0.9rem' }}>
                  <strong style={{ color: 'var(--accent-color)' }}>{h.resource_type.toUpperCase()}</strong>: {h.path || 'N/A'} (Port {h.port})
                  <span style={{ display: 'block', fontSize: '0.8rem', color: 'var(--text-secondary)', marginTop: '4px' }}>Logged: {h.started_at}</span>
                </li>
              ))}
            </ul>
          )}
        </GlassCard>
      </div>
    </div>
  );
}
