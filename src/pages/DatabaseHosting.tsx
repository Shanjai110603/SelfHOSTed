import React, { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Database, Settings, Shield, HardDrive, Play } from 'lucide-react';
import { GlassCard } from '../components/GlassCard';

export const DatabaseHosting: React.FC = () => {
  const [template, setTemplate] = useState('mysql');
  const [port, setPort] = useState(3306);
  
  // Advanced State
  const [showAdvanced, setShowAdvanced] = useState(false);
  const [domain, setDomain] = useState('');
  
  const [status, setStatus] = useState<string | null>(null);
  const [isLaunching, setIsLaunching] = useState(false);

  const handleLaunch = async () => {
    setIsLaunching(true);
    setStatus('Provisioning secure database workload...');
    
    try {
      await invoke('start_workload', {
        config: {
          id: '',
          resource_type: 'database',
          template,
          port,
          env_vars: {},
          domain: domain.length > 0 ? domain : null,
          host_path: null, // MVP persistence handled internally via docker volumes in future PR
        }
      });
      
      setStatus(`Success! Database deployed. Password is encrypted in Vault.`);
    } catch (e) {
      setStatus(`Failed: ${e}`);
    } finally {
      setIsLaunching(false);
    }
  };

  return (
    <div className="page-container animate-fade-in">
      <header style={{ marginBottom: '40px' }}>
        <h1 className="title-large">Deploy Database</h1>
        <p className="text-secondary">
          Launch secure, managed database workloads.
        </p>
      </header>

      <div style={{ display: 'grid', gridTemplateColumns: '1fr', gap: '24px', maxWidth: '800px' }}>
        
        {/* BASIC WORKLOAD SECTION */}
        <GlassCard title="1. Engine Configuration" icon={<Database size={24} />}>
          <div style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
            <div>
              <label className="text-secondary" style={{ display: 'block', marginBottom: '8px' }}>Database Engine</label>
              <select 
                value={template} 
                onChange={(e) => {
                  setTemplate(e.target.value);
                  if (e.target.value === 'mysql') setPort(3306);
                  if (e.target.value === 'postgres') setPort(5432);
                  if (e.target.value === 'redis') setPort(6379);
                  if (e.target.value === 'mongodb') setPort(27017);
                }}
                style={{ width: '100%', padding: '12px', background: 'rgba(0,0,0,0.2)', border: '1px solid rgba(255,255,255,0.1)', color: 'white', borderRadius: '8px' }}
              >
                <option value="mysql">MySQL 8</option>
                <option value="postgres">PostgreSQL</option>
                <option value="mariadb">MariaDB</option>
                <option value="redis">Redis Cache</option>
                <option value="mongodb">MongoDB</option>
              </select>
            </div>

            <div style={{ padding: '16px', background: 'rgba(59,130,246,0.1)', border: '1px solid rgba(59,130,246,0.3)', borderRadius: '8px' }}>
              <div style={{ display: 'flex', gap: '12px', alignItems: 'center', marginBottom: '8px' }}>
                <Shield size={18} color="#3b82f6" />
                <strong style={{ color: '#3b82f6' }}>Automated Vault Security</strong>
              </div>
              <p className="text-secondary" style={{ fontSize: '0.9rem', margin: 0 }}>
                A strong cryptographic password will be generated automatically. It is never logged in plaintext and is encrypted directly into your OS Master Keyring via the Vault system.
              </p>
            </div>
            
            <div>
              <label className="text-secondary" style={{ display: 'block', marginBottom: '8px' }}>Internal Port</label>
              <input 
                type="number" 
                value={port} 
                onChange={(e) => setPort(parseInt(e.target.value))}
                style={{ width: '100%', padding: '12px', background: 'rgba(0,0,0,0.2)', border: '1px solid rgba(255,255,255,0.1)', color: 'white', borderRadius: '8px' }}
              />
            </div>
          </div>
        </GlassCard>

        {/* ADVANCED TOGGLE */}
        <div style={{ display: 'flex', justifyContent: 'center' }}>
          <button 
            onClick={() => setShowAdvanced(!showAdvanced)}
            style={{ background: 'transparent', border: 'none', color: 'var(--text-secondary)', display: 'flex', gap: '8px', alignItems: 'center', cursor: 'pointer' }}
          >
            <Settings size={16} />
            {showAdvanced ? 'Hide Advanced Settings' : 'Show Advanced Settings'}
          </button>
        </div>

        {/* ADVANCED SECTIONS */}
        {showAdvanced && (
          <>
            <GlassCard title="Advanced: Persistence Policies" icon={<HardDrive size={24} />}>
              <p className="text-secondary" style={{ marginBottom: '16px' }}>
                Configure how database storage is mounted and backed up.
              </p>
              <div style={{ padding: '24px', background: 'rgba(0,0,0,0.2)', borderRadius: '8px', border: '1px dashed rgba(255,255,255,0.1)', textAlign: 'center' }}>
                <span className="text-secondary">Snapshot and Host Volume settings coming in EPIC-004...</span>
              </div>
            </GlassCard>
          </>
        )}

        {/* LAUNCH BAR */}
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginTop: '12px' }}>
          <span style={{ color: status?.includes('Error') || status?.includes('Failed') ? '#ff4d4f' : '#10b981' }}>
            {status}
          </span>
          <button 
            className="btn-primary" 
            onClick={handleLaunch} 
            disabled={isLaunching}
            style={{ opacity: isLaunching ? 0.7 : 1 }}
          >
            <Play size={18} />
            {isLaunching ? 'Provisioning...' : 'Deploy Database Workload'}
          </button>
        </div>

      </div>
    </div>
  );
};
