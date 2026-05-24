import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Package, Globe, Shield, Play, Lock, Network, AlertTriangle } from 'lucide-react';
import { GlassCard } from '../components/GlassCard';
import { useTelemetry } from '../hooks/useTelemetry';

interface WorkloadConfig {
  id: string;
  resource_type: string;
  template: string;
  port: number;
}

interface AppTemplate {
  id: string;
  name: string;
  description: string;
  icon: string;
  category: string;
  recommended_ram_gb: number;
  workloads: WorkloadConfig[];
}

export const Marketplace: React.FC = () => {
  const [templates, setTemplates] = useState<AppTemplate[]>([]);
  const [selectedTemplate, setSelectedTemplate] = useState<AppTemplate | null>(null);
  const [isDeploying, setIsDeploying] = useState(false);
  
  // Deployment options
  const [domain, setDomain] = useState('');
  const [exposureMode, setExposureMode] = useState('local');
  
  // Adaptive Orchestration integration
  const telemetry = useTelemetry();
  const availableRamGb = telemetry ? (telemetry.memory_total - telemetry.memory_used) / (1024 * 1024 * 1024) : 0;
  
  // Calculate if the selected template is safe to run
  const isResourceWarning = selectedTemplate && telemetry && (availableRamGb < selectedTemplate.recommended_ram_gb);

  useEffect(() => {
    loadTemplates();
  }, []);

  const loadTemplates = async () => {
    try {
      const data = await invoke<AppTemplate[]>('get_marketplace_templates');
      setTemplates(data);
    } catch (e) {
      console.error("Failed to load templates", e);
    }
  };

  const deployStack = async () => {
    if (!selectedTemplate) return;
    setIsDeploying(true);
    
    try {
      let exposure = null;
      if (exposureMode === 'cloudflare') {
        exposure = { provider: 'cloudflare', mode: 'quick', token: null };
      } else if (exposureMode === 'tailscale') {
        exposure = { provider: 'tailscale', mode: 'mesh', token: null };
      }

      await invoke('deploy_stack', {
        templateId: selectedTemplate.id,
        domain: domain || null,
        exposure
      });

      // Clear selection
      setSelectedTemplate(null);
      alert(`Successfully deployed ${selectedTemplate.name}!`);
    } catch (e) {
      alert(`Failed to deploy stack: ${e}`);
    } finally {
      setIsDeploying(false);
    }
  };

  return (
    <div className="page-container animate-fade-in">
      <header style={{ marginBottom: '40px' }}>
        <h1 className="title-large">App Marketplace</h1>
        <p className="text-secondary">
          Deploy modular infrastructure stacks with isolated networking and secure credentials.
        </p>
      </header>

      {/* EDUCATIONAL UX */}
      <div style={{ padding: '24px', background: 'rgba(59,130,246,0.1)', border: '1px solid rgba(59,130,246,0.3)', borderRadius: '12px', marginBottom: '32px' }}>
        <h3 style={{ display: 'flex', alignItems: 'center', gap: '8px', color: '#3b82f6', marginBottom: '12px', fontSize: '1.2rem' }}>
          <Network size={20} /> Stack Orchestration
        </h3>
        <p className="text-secondary" style={{ lineHeight: '1.6' }}>
          Marketplace apps are deployed as <strong>Stacks</strong>. SelfHOSTed automatically provisions a secure, isolated virtual network for each stack, ensuring databases remain completely invisible to the host OS while frontend workloads are exposed securely.
        </p>
      </div>

      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(300px, 1fr))', gap: '24px' }}>
        {templates.map(tmpl => (
          <GlassCard key={tmpl.id} title={tmpl.name} icon={<Package size={24} />}>
            <p className="text-secondary" style={{ fontSize: '0.9rem', marginBottom: '16px' }}>{tmpl.description}</p>
            <div style={{ display: 'flex', gap: '8px', marginBottom: '16px' }}>
              <span style={{ fontSize: '0.8rem', padding: '4px 8px', background: 'rgba(255,255,255,0.1)', borderRadius: '4px' }}>
                {tmpl.category}
              </span>
              <span style={{ fontSize: '0.8rem', padding: '4px 8px', background: 'rgba(255,255,255,0.1)', borderRadius: '4px' }}>
                {tmpl.workloads.length} workloads
              </span>
            </div>
            <button 
              className="btn-primary" 
              style={{ width: '100%', justifyContent: 'center' }}
              onClick={() => setSelectedTemplate(tmpl)}
            >
              Configure & Deploy
            </button>
          </GlassCard>
        ))}
      </div>

      {/* DEPLOYMENT MODAL */}
      {selectedTemplate && (
        <div style={{
          position: 'fixed', top: 0, left: 0, right: 0, bottom: 0,
          background: 'rgba(0,0,0,0.8)', backdropFilter: 'blur(10px)',
          display: 'flex', alignItems: 'center', justifyContent: 'center',
          zIndex: 1000
        }}>
          <div className="glass-panel" style={{ width: '100%', maxWidth: '600px', padding: '32px' }}>
            <h2 style={{ marginBottom: '8px' }}>Deploy {selectedTemplate.name}</h2>
            <p className="text-secondary" style={{ marginBottom: '24px' }}>Configure orchestration settings for this stack.</p>
            
            {isResourceWarning && (
              <div style={{ display: 'flex', gap: '12px', padding: '16px', background: 'rgba(239,68,68,0.1)', border: '1px solid rgba(239,68,68,0.3)', borderRadius: '8px', marginBottom: '24px' }}>
                <AlertTriangle size={24} color="#ef4444" />
                <div>
                  <h4 style={{ color: '#ef4444', marginBottom: '4px' }}>Adaptive Orchestration Warning</h4>
                  <p style={{ fontSize: '0.9rem', color: 'rgba(255,255,255,0.8)' }}>
                    This device may struggle to run {selectedTemplate.name}. It requires ~{selectedTemplate.recommended_ram_gb}GB of free RAM, but you currently only have {availableRamGb.toFixed(1)}GB available. Proceed with caution to avoid thermal throttling or system freezes.
                  </p>
                </div>
              </div>
            )}

            <div style={{ background: 'rgba(0,0,0,0.2)', padding: '16px', borderRadius: '8px', marginBottom: '24px' }}>
              <h4 style={{ marginBottom: '12px' }}>Workload Topology</h4>
              <ul style={{ listStyle: 'none', padding: 0, margin: 0, display: 'flex', flexDirection: 'column', gap: '8px' }}>
                {selectedTemplate.workloads.map(w => (
                  <li key={w.id} style={{ display: 'flex', alignItems: 'center', gap: '8px', fontSize: '0.9rem' }}>
                    {w.resource_type === 'database' ? <Lock size={14} color="var(--warning-color)" /> : <Globe size={14} color="var(--primary-color)" />}
                    {w.id} ({w.template})
                  </li>
                ))}
              </ul>
            </div>

            <div className="form-group" style={{ marginBottom: '16px' }}>
              <label>Custom Domain (Optional)</label>
              <input 
                type="text" 
                placeholder="e.g., app.selfhosted.local" 
                value={domain}
                onChange={e => setDomain(e.target.value)}
              />
              <span className="text-secondary" style={{ fontSize: '0.8rem', marginTop: '4px', display: 'block' }}>Only the frontend workload will be bound to this domain.</span>
            </div>

            <div className="form-group" style={{ marginBottom: '32px' }}>
              <label>Exposure Intent</label>
              <select value={exposureMode} onChange={e => setExposureMode(e.target.value)}>
                <option value="local">Local Only (Private Network)</option>
                <option value="cloudflare">Cloudflare Quick Tunnel (Public Internet)</option>
                <option value="tailscale">Tailscale Mesh (Private Mesh Network)</option>
              </select>
            </div>

            <div style={{ display: 'flex', gap: '16px', justifyContent: 'flex-end' }}>
              <button className="btn-secondary" onClick={() => setSelectedTemplate(null)} disabled={isDeploying}>
                Cancel
              </button>
              <button className="btn-primary" onClick={deployStack} disabled={isDeploying}>
                {isDeploying ? 'Orchestrating...' : <><Play size={18} /> Deploy Stack</>}
              </button>
            </div>
          </div>
        </div>
      )}

    </div>
  );
};
