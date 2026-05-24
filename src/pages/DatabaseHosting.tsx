import React, { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Database, Play, Square, Loader2, Key } from 'lucide-react';
import { GlassCard } from '../components/GlassCard';

interface DatabaseSession {
  id: string;
  port: number;
  status: string;
}

export function DatabaseHosting() {
  const [password, setPassword] = useState<string>('postgres');
  const [port, setPort] = useState<number>(5432);
  const [loading, setLoading] = useState(false);
  const [activeDBs, setActiveDBs] = useState<DatabaseSession[]>([]);

  const handleStartDB = async () => {
    if (!password) return;
    setLoading(true);
    try {
      const session = await invoke<DatabaseSession>('start_database', { password, port });
      setActiveDBs([...activeDBs, session]);
      setPort(port + 1);
    } catch (err) {
      console.error('Failed to start database:', err);
      alert(`Failed to start database: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  const handleStopDB = async (id: string) => {
    try {
      await invoke('stop_database', { id });
      setActiveDBs(activeDBs.filter(db => db.id !== id));
    } catch (err) {
      console.error('Failed to stop database:', err);
      alert(`Failed to stop database: ${err}`);
    }
  };

  return (
    <div className="animate-fade-in">
      <header style={{ marginBottom: '40px' }}>
        <h1 className="title-large">PostgreSQL Hosting</h1>
        <p className="text-secondary">Spin up local database instances instantly. Perfect for development and testing.</p>
      </header>

      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(300px, 1fr))', gap: '24px' }}>
        <GlassCard title="New Database" icon={<Database size={24} />}>
          <div style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
            
            <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
              <span className="text-secondary">Postgres Password:</span>
              <div style={{ display: 'flex', alignItems: 'center', gap: '12px', background: 'rgba(0,0,0,0.2)', border: '1px solid rgba(255,255,255,0.1)', padding: '8px 12px', borderRadius: '8px' }}>
                <Key size={16} className="text-secondary" />
                <input 
                  type="password" 
                  value={password} 
                  onChange={(e) => setPassword(e.target.value)}
                  style={{ background: 'transparent', border: 'none', color: 'white', outline: 'none', width: '100%', fontFamily: 'inherit' }}
                />
              </div>
            </div>

            <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
              <span className="text-secondary">Port:</span>
              <input 
                type="number" 
                value={port} 
                onChange={(e) => setPort(parseInt(e.target.value))}
                style={{ background: 'rgba(0,0,0,0.2)', border: '1px solid rgba(255,255,255,0.1)', color: 'white', padding: '8px 12px', borderRadius: '8px', width: '100px', fontFamily: 'inherit' }}
              />
            </div>

            <button className="btn-primary" style={{ justifyContent: 'center' }} onClick={handleStartDB} disabled={!password || loading}>
              {loading ? <Loader2 className="spinner" size={18} /> : <Play size={18} />}
              Start Database
            </button>
          </div>
        </GlassCard>

        <GlassCard title="Active Databases" description={activeDBs.length === 0 ? "No databases currently running." : undefined}>
          <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
            {activeDBs.map(db => (
              <div key={db.id} style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', background: 'rgba(0,0,0,0.2)', padding: '12px', borderRadius: '8px' }}>
                <div style={{ display: 'flex', flexDirection: 'column', gap: '4px' }}>
                  <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                     <div style={{ width: '8px', height: '8px', borderRadius: '50%', background: 'var(--success-color)', boxShadow: '0 0 10px var(--success-glow)' }}></div>
                     <span style={{ fontWeight: 500 }}>localhost:{db.port}</span>
                  </div>
                  <span className="text-secondary" style={{ fontSize: '0.8rem' }}>
                    postgres://postgres:****@localhost:{db.port}
                  </span>
                </div>
                <button className="btn-primary" style={{ background: 'rgba(239, 68, 68, 0.2)', color: '#f87171', boxShadow: 'none', padding: '8px' }} onClick={() => handleStopDB(db.id)}>
                  <Square size={16} />
                </button>
              </div>
            ))}
          </div>
        </GlassCard>
      </div>
    </div>
  );
}
