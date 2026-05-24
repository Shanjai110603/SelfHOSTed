import React, { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { Share2, Folder, Play, Square, Loader2, Globe, Clock } from 'lucide-react';
import { GlassCard } from '../components/GlassCard';

interface FileShareSession {
  id: string;
  path: string;
  port: number;
  status: string;
  public_url?: string;
}

interface TunnelSession {
  id: string;
  local_port: number;
  public_url: string;
}

export function FileSharing() {
  const [selectedPath, setSelectedPath] = useState<string | null>(null);
  const [port, setPort] = useState<number>(9000);
  const [duration, setDuration] = useState<number>(60); // minutes
  const [loading, setLoading] = useState(false);
  const [activeShares, setActiveShares] = useState<FileShareSession[]>([]);

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

  const handleStartShare = async () => {
    if (!selectedPath) return;
    setLoading(true);
    try {
      // 1. Start File Share Container
      const session = await invoke<FileShareSession>('start_fileshare', { path: selectedPath, port });
      
      // 2. Register Expiration Session
      await invoke('register_session', { id: session.id, resourceType: 'fileshare', durationMinutes: duration });

      setActiveShares([...activeShares, session]);
      setSelectedPath(null);
      setPort(port + 1);
    } catch (err) {
      console.error('Failed to start fileshare:', err);
      alert(`Failed to start fileshare: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  const handleEnablePublicLink = async (share: FileShareSession) => {
    const confirm = window.confirm("Are you sure you want to expose this folder to the public internet? Anyone with the link will be able to access it.");
    if (!confirm) return;

    try {
      // Auto-start tunnel for public file sharing
      const tunnel = await invoke<TunnelSession>('start_tunnel', { sessionId: share.id, localPort: share.port });
      
      setActiveShares(activeShares.map(s => {
        if (s.id === share.id) {
          return { ...s, public_url: tunnel.public_url };
        }
        return s;
      }));
    } catch (err) {
      console.error('Failed to start tunnel:', err);
      alert(err);
    }
  };

  const handleStopShare = async (id: string) => {
    try {
      await invoke('stop_website', { id }); // Using same docker stop command for simplicity
      setActiveShares(activeShares.filter(s => s.id !== id));
    } catch (err) {
      console.error('Failed to stop fileshare:', err);
      alert(`Failed to stop fileshare: ${err}`);
    }
  };

  return (
    <div className="animate-fade-in">
      <header style={{ marginBottom: '40px' }}>
        <h1 className="title-large">Secure File Sharing</h1>
        <p className="text-secondary">Temporarily share local folders over the network with automatic expiration.</p>
      </header>

      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(300px, 1fr))', gap: '24px' }}>
        <GlassCard title="Create New Share" icon={<Share2 size={24} />}>
          <div style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
            
            <button className="btn-primary" style={{ background: 'rgba(255,255,255,0.05)', boxShadow: 'none', display: 'flex', justifyContent: 'flex-start' }} onClick={handleSelectFolder}>
              <Folder size={18} />
              <span style={{ whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' }}>
                 {selectedPath ? selectedPath : 'Select Folder to Share'}
              </span>
            </button>

            <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
              <Clock size={18} className="text-secondary" />
              <span className="text-secondary">Expire in:</span>
              <select 
                value={duration} 
                onChange={(e) => setDuration(parseInt(e.target.value))}
                style={{ background: 'rgba(0,0,0,0.2)', border: '1px solid rgba(255,255,255,0.1)', color: 'white', padding: '8px 12px', borderRadius: '8px', flex: 1, fontFamily: 'inherit' }}
              >
                <option value={15}>15 Minutes</option>
                <option value={60}>1 Hour</option>
                <option value={1440}>1 Day</option>
              </select>
            </div>

            <button className="btn-primary" style={{ justifyContent: 'center' }} onClick={handleStartShare} disabled={!selectedPath || loading}>
              {loading ? <Loader2 className="spinner" size={18} /> : <Play size={18} />}
              Start Sharing
            </button>
          </div>
        </GlassCard>

        <GlassCard title="Active Shares" description={activeShares.length === 0 ? "No active file shares." : undefined}>
          <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
            {activeShares.map(share => (
              <div key={share.id} style={{ display: 'flex', flexDirection: 'column', gap: '12px', background: 'rgba(0,0,0,0.2)', padding: '16px', borderRadius: '12px' }}>
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start' }}>
                  <div style={{ display: 'flex', flexDirection: 'column', gap: '4px' }}>
                    {share.public_url ? (
                      <>
                        <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                          <div style={{ width: '8px', height: '8px', borderRadius: '50%', background: '#8b5cf6', boxShadow: '0 0 10px rgba(139, 92, 246, 0.5)' }}></div>
                          <a href={share.public_url} target="_blank" rel="noreferrer" style={{ color: 'white', textDecoration: 'none', fontWeight: 500 }}>
                            {share.public_url}
                          </a>
                        </div>
                        <span className="text-secondary" style={{ fontSize: '0.8rem', marginTop: '4px', display: 'flex', alignItems: 'center', gap: '4px' }}>
                          <AlertTriangle size={12} color="#f59e0b" />
                          Temporary Access Endpoint - URL may change after restart
                        </span>
                      </>
                    ) : (
                      <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                        <div style={{ width: '8px', height: '8px', borderRadius: '50%', background: 'var(--success-color)', boxShadow: '0 0 10px var(--success-glow)' }}></div>
                        <span style={{ fontWeight: 500 }}>Local: http://localhost:{share.port}</span>
                      </div>
                    )}
                    <span className="text-secondary" style={{ fontSize: '0.8rem', whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis', maxWidth: '200px' }}>
                      {share.path}
                    </span>
                  </div>
                  <button className="btn-primary" style={{ background: 'rgba(239, 68, 68, 0.2)', color: '#f87171', boxShadow: 'none', padding: '8px' }} onClick={() => handleStopShare(share.id)}>
                    <Square size={16} />
                  </button>
                </div>
                
                {share.public_url ? (
                  <div style={{ background: 'rgba(99, 102, 241, 0.1)', padding: '8px 12px', borderRadius: '8px', border: '1px solid rgba(99, 102, 241, 0.2)' }}>
                    <span style={{ fontSize: '0.85rem', color: '#a5b4fc', display: 'block', marginBottom: '4px' }}>Public Link (Expires with session):</span>
                    <a href={share.public_url} target="_blank" rel="noreferrer" style={{ color: 'white', wordBreak: 'break-all', fontSize: '0.9rem' }}>
                      {share.public_url}
                    </a>
                  </div>
                ) : (
                  <button className="btn-primary" style={{ background: 'rgba(255,255,255,0.05)', boxShadow: 'none', width: '100%', fontSize: '0.9rem', justifyContent: 'center' }} onClick={() => handleEnablePublicLink(share)}>
                    <Globe size={16} />
                    Enable Public Access
                  </button>
                )}
              </div>
            ))}
          </div>
        </GlassCard>
      </div>
    </div>
  );
}
