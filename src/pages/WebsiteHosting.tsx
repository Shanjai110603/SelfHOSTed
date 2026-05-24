import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { Globe, Folder, Play, Square, Loader2, Pause } from 'lucide-react';
import { GlassCard } from '../components/GlassCard';

interface WebsiteSession {
  id: string;
  path: string;
  port: number;
  status: string;
}

export function WebsiteHosting() {
  const [selectedPath, setSelectedPath] = useState<string | null>(null);
  const [port, setPort] = useState<number>(8080);
  const [loading, setLoading] = useState(false);
  const [activeSites, setActiveSites] = useState<WebsiteSession[]>([]);

  useEffect(() => {
    const fetchActiveSessions = async () => {
      try {
        const sessions = await invoke<any[]>('get_active_sessions');
        // Currently ManagedSession returns { id, resource_type, expires_at, status }
        // We need to merge this with the paths/ports from the DB history, but for MVP
        // we can just extract port from id (e.g. "website-8080") or do a dedicated IPC.
        // As a quick sync, let's parse port/path from history or just rely on backend recovery.
        
        // Actually, let's just fetch recent sessions from SQLite and filter by what's active
        const history = await invoke<any[]>('get_recent_sessions');
        const activeIds = sessions.filter(s => s.resource_type === 'website').map(s => s.id);
        
        const recoveredSites = history
          .filter(h => activeIds.includes(h.id))
          .map(h => {
            const activeStatus = sessions.find(s => s.id === h.id)?.status || 'running';
            return {
              id: h.id,
              path: h.path || 'Unknown Path',
              port: h.port || 8080,
              status: activeStatus,
            };
          });
          
        setActiveSites(recoveredSites);
      } catch (err) {
        console.error('Failed to fetch active sessions:', err);
      }
    };
    fetchActiveSessions();
  }, []);

  const handleSelectFolder = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
      });
      if (selected) {
        setSelectedPath(selected as string);
      }
    } catch (err) {
      console.error('Failed to open dialog:', err);
    }
  };

  const handleStartHosting = async () => {
    if (!selectedPath) return;
    setLoading(true);
    try {
      const session = await invoke<WebsiteSession>('start_website', { path: selectedPath, port });
      
      // Log to SQLite database
      await invoke('log_session', { 
        id: session.id, 
        resourceType: 'website', 
        path: selectedPath, 
        port: port 
      });

      setActiveSites([...activeSites, { ...session, status: 'running' }]);
      setSelectedPath(null);
      setPort(port + 1);
    } catch (err) {
      console.error('Failed to start website:', err);
      alert(`Failed to start website: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  const handleStopHosting = async (id: string) => {
    try {
      await invoke('stop_website', { id });
      setActiveSites(activeSites.filter(site => site.id !== id));
    } catch (err) {
      console.error('Failed to stop website:', err);
      alert(`Failed to stop website: ${err}`);
    }
  };

  const handlePauseSession = async (id: string) => {
    try {
      await invoke('pause_session', { id });
      await invoke('update_session_status', { id, status: 'paused' });
      setActiveSites(activeSites.map(site => site.id === id ? { ...site, status: 'paused' } : site));
    } catch (err) {
      console.error('Failed to pause session:', err);
      alert(`Failed to pause session: ${err}`);
    }
  };

  const handleResumeSession = async (id: string) => {
    try {
      await invoke('resume_session', { id });
      await invoke('update_session_status', { id, status: 'running' });
      setActiveSites(activeSites.map(site => site.id === id ? { ...site, status: 'running' } : site));
    } catch (err) {
      console.error('Failed to resume session:', err);
      alert(`Failed to resume session: ${err}`);
    }
  };

  return (
    <div className="animate-fade-in">
      <header style={{ marginBottom: '40px' }}>
        <h1 className="title-large">Website Hosting</h1>
        <p className="text-secondary">Host a static HTML/CSS website securely from your local folders.</p>
      </header>

      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(300px, 1fr))', gap: '24px' }}>
        <GlassCard title="New Website" icon={<Globe size={24} />}>
          <div style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
            
            <button className="btn-primary" style={{ background: 'rgba(255,255,255,0.05)', boxShadow: 'none', display: 'flex', justifyContent: 'flex-start' }} onClick={handleSelectFolder}>
              <Folder size={18} />
              <span style={{ whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' }}>
                 {selectedPath ? selectedPath : 'Select Folder containing HTML files'}
              </span>
            </button>

            <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
              <span className="text-secondary">Port:</span>
              <input 
                type="number" 
                value={port} 
                onChange={(e) => setPort(parseInt(e.target.value))}
                style={{ background: 'rgba(0,0,0,0.2)', border: '1px solid rgba(255,255,255,0.1)', color: 'white', padding: '8px 12px', borderRadius: '8px', width: '100px', fontFamily: 'inherit' }}
              />
            </div>

            <button className="btn-primary" style={{ justifyContent: 'center' }} onClick={handleStartHosting} disabled={!selectedPath || loading}>
              {loading ? <Loader2 className="spinner" size={18} /> : <Play size={18} />}
              Start Hosting
            </button>
          </div>
        </GlassCard>

        <GlassCard title="Active Websites" description={activeSites.length === 0 ? "No websites currently running." : undefined}>
          <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
            {activeSites.map(site => (
              <div key={site.id} style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', background: 'rgba(0,0,0,0.2)', padding: '12px', borderRadius: '8px' }}>
                <div style={{ display: 'flex', flexDirection: 'column', gap: '4px' }}>
                  <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                     <div style={{ width: '8px', height: '8px', borderRadius: '50%', background: site.status === 'paused' ? '#f59e0b' : 'var(--success-color)', boxShadow: site.status === 'paused' ? '0 0 10px rgba(245, 158, 11, 0.5)' : '0 0 10px var(--success-glow)' }}></div>
                     <a href={`http://localhost:${site.port}`} target="_blank" rel="noreferrer" style={{ color: 'white', textDecoration: site.status === 'paused' ? 'line-through' : 'none', fontWeight: 500, opacity: site.status === 'paused' ? 0.5 : 1 }}>
                       localhost:{site.port} {site.status === 'paused' && '(Paused)'}
                     </a>
                  </div>
                  <span className="text-secondary" style={{ fontSize: '0.8rem', whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis', maxWidth: '200px' }}>
                    {site.path}
                  </span>
                </div>
                <div style={{ display: 'flex', gap: '8px' }}>
                  {site.status === 'paused' ? (
                    <button className="btn-primary" style={{ background: 'rgba(34, 197, 94, 0.2)', color: '#4ade80', boxShadow: 'none', padding: '8px' }} onClick={() => handleResumeSession(site.id)} title="Resume">
                      <Play size={16} />
                    </button>
                  ) : (
                    <button className="btn-primary" style={{ background: 'rgba(245, 158, 11, 0.2)', color: '#fbbf24', boxShadow: 'none', padding: '8px' }} onClick={() => handlePauseSession(site.id)} title="Pause">
                      <Pause size={16} />
                    </button>
                  )}
                  <button className="btn-primary" style={{ background: 'rgba(239, 68, 68, 0.2)', color: '#f87171', boxShadow: 'none', padding: '8px' }} onClick={() => handleStopHosting(site.id)} title="Stop">
                    <Square size={16} />
                  </button>
                </div>
              </div>
            ))}
          </div>
        </GlassCard>
      </div>
    </div>
  );
}
