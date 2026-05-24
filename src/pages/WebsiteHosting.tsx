import React, { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Globe, Settings, Terminal, Play, Folder, Box } from 'lucide-react';
import { GlassCard } from '../components/GlassCard';
import { EducationalTooltip } from '../components/EducationalTooltip';

export const WebsiteHosting: React.FC = () => {
  const [template, setTemplate] = useState('static');
  const [hostPath, setHostPath] = useState('');
  const [port, setPort] = useState(8080);
  
  // Advanced State
  const [showAdvanced, setShowAdvanced] = useState(false);
  const [domain, setDomain] = useState('');
  const [envVars, setEnvVars] = useState<Record<string, string>>({});
  const [exposureProvider, setExposureProvider] = useState('none');
  
  const [status, setStatus] = useState<string | null>(null);
  const [isLaunching, setIsLaunching] = useState(false);

  const handleLaunch = async () => {
    if (!hostPath && template !== 'php') {
      setStatus('Error: Please select a project folder.');
      return;
    }
    
    setIsLaunching(true);
    setStatus('Provisioning workload...');
    
    try {
      await invoke('start_workload', {
        config: {
          id: '',
          resource_type: 'website',
          template,
          port,
          env_vars: envVars,
          domain: domain.length > 0 ? domain : null,
          host_path: hostPath,
          exposure: exposureProvider !== 'none' ? {
            provider: exposureProvider === 'cloudflare' ? 'cloudflare' : 'tailscale',
            mode: exposureProvider === 'cloudflare' ? 'quick' : 'mesh',
            token: null
          } : null
        }
      });
      
      setStatus(`Success! Workload deployed.`);
    } catch (e) {
      setStatus(`Failed: ${e}`);
    } finally {
      setIsLaunching(false);
    }
  };

  return (
    <div className="page-container animate-fade-in">
      <header style={{ marginBottom: '40px' }}>
        <h1 className="title-large">Deploy Website</h1>
        <p className="text-secondary">
          Launch a generic infrastructure workload with intelligent routing.
        </p>
      </header>

      <div style={{ display: 'grid', gridTemplateColumns: '1fr', gap: '24px', maxWidth: '800px' }}>
        
        {/* BASIC WORKLOAD SECTION */}
        <GlassCard title="1. Basic Configuration" icon={<Box size={24} />}>
          <div style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
            <div>
              <label className="text-secondary" style={{ display: 'block', marginBottom: '8px' }}>Runtime Template</label>
              <select 
                value={template} 
                onChange={(e) => setTemplate(e.target.value)}
                style={{ width: '100%', padding: '12px', background: 'rgba(0,0,0,0.2)', border: '1px solid rgba(255,255,255,0.1)', color: 'white', borderRadius: '8px' }}
              >
                <option value="static">Static HTML (NGINX)</option>
                <option value="nodejs">Node.js (18 Alpine)</option>
                <option value="python">Python (3.11 Slim)</option>
                <option value="php">PHP (8 Apache)</option>
              </select>
            </div>

            <div>
              <label className="text-secondary" style={{ display: 'block', marginBottom: '8px' }}>Project Directory (Host Path)</label>
              <div style={{ display: 'flex', gap: '12px' }}>
                <input 
                  type="text" 
                  value={hostPath} 
                  onChange={(e) => setHostPath(e.target.value)}
                  placeholder="C:\Projects\MyWebsite"
                  style={{ flex: 1, padding: '12px', background: 'rgba(0,0,0,0.2)', border: '1px solid rgba(255,255,255,0.1)', color: 'white', borderRadius: '8px' }}
                />
                <button className="btn-primary" style={{ background: 'rgba(255,255,255,0.1)' }}>
                  <Folder size={18} /> Browse
                </button>
              </div>
            </div>

            <div>
              <label className="text-secondary" style={{ display: 'block', marginBottom: '8px' }}>Internal Port</label>
              <input 
                type="number" 
                value={port} 
                onChange={(e) => setPort(parseInt(e.target.value))}
                style={{ width: '100%', padding: '12px', background: 'rgba(0,0,0,0.2)', border: '1px solid rgba(255,255,255,0.1)', color: 'white', borderRadius: '8px' }}
              />
              <p className="text-secondary" style={{ fontSize: '0.8rem', marginTop: '6px' }}>
                The internal port your app listens on (e.g., 3000 for Node). The proxy will route to this.
              </p>
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
            <GlassCard title="Advanced: Custom Domain" icon={<Globe size={24} />}>
              <p className="text-secondary" style={{ marginBottom: '16px' }}>
                Bind a domain (like <code>myapp.local</code>) directly to this workload. Traefik will automatically handle the routing.
              </p>
              <label style={{ display: 'flex', alignItems: 'center', marginBottom: '8px', color: 'var(--text-secondary)' }}>
                Custom Domain (Optional)
                <EducationalTooltip 
                  title="Proxy Routing" 
                  content="If you provide a custom domain (e.g., app.selfhosted.local), the internal Traefik Proxy will automatically intercept traffic for that domain and route it securely to your container, without exposing ports directly to your host OS."
                />
              </label>
              <input 
                type="text" 
                value={domain} 
                onChange={(e) => setDomain(e.target.value)}
                placeholder="myapp.local"
                style={{ width: '100%', padding: '12px', background: 'rgba(0,0,0,0.2)', border: '1px solid rgba(255,255,255,0.1)', color: 'white', borderRadius: '8px', marginBottom: '24px' }}
              />

              <label style={{ display: 'flex', alignItems: 'center', marginBottom: '8px', color: 'var(--text-primary)', fontWeight: 'bold' }}>
                Global Network Exposure
                <EducationalTooltip 
                  title="Adaptive Networking" 
                  content={
                    <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
                      <p><strong>Local Only:</strong> Visible only on your home network.</p>
                      <p><strong>Cloudflare:</strong> Publicly exposed to the entire internet.</p>
                      <p><strong>Tailscale:</strong> Privately exposed over a secure global mesh network.</p>
                    </div>
                  }
                />
              </label>
              <p className="text-secondary" style={{ marginBottom: '16px', fontSize: '0.9rem' }}>
                Orchestrate public or mesh routing directly to this workload. 
                <strong style={{ color: '#f59e0b', marginLeft: '4px' }}>Secure by Default: Local Only.</strong>
              </p>
              
              <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '12px' }}>
                <button 
                  onClick={() => setExposureProvider('none')}
                  className="btn-secondary" 
                  style={{ display: 'flex', flexDirection: 'column', padding: '16px', border: exposureProvider === 'none' ? '1px solid rgba(59,130,246,0.3)' : '1px solid rgba(255,255,255,0.1)', background: exposureProvider === 'none' ? 'rgba(59,130,246,0.1)' : 'transparent' }}>
                  <span style={{ color: exposureProvider === 'none' ? '#3b82f6' : 'white', fontWeight: 'bold' }}>Local Only</span>
                  <span className="text-secondary" style={{ fontSize: '0.8rem', marginTop: '4px' }}>Accessible only on this machine.</span>
                </button>
                <button 
                  onClick={() => setExposureProvider('cloudflare')}
                  className="btn-secondary" 
                  style={{ display: 'flex', flexDirection: 'column', padding: '16px', border: exposureProvider === 'cloudflare' ? '1px solid rgba(59,130,246,0.3)' : '1px solid rgba(255,255,255,0.1)', background: exposureProvider === 'cloudflare' ? 'rgba(59,130,246,0.1)' : 'transparent' }}>
                  <span style={{ color: exposureProvider === 'cloudflare' ? '#3b82f6' : 'white', fontWeight: 'bold' }}>Cloudflare Quick Tunnel</span>
                  <span className="text-secondary" style={{ fontSize: '0.8rem', marginTop: '4px' }}>Temporary public trycloudflare URL.</span>
                </button>
                <button 
                  onClick={() => setExposureProvider('tailscale')}
                  className="btn-secondary" 
                  style={{ display: 'flex', flexDirection: 'column', padding: '16px', border: exposureProvider === 'tailscale' ? '1px solid rgba(59,130,246,0.3)' : '1px solid rgba(255,255,255,0.1)', background: exposureProvider === 'tailscale' ? 'rgba(59,130,246,0.1)' : 'transparent' }}>
                  <span style={{ color: exposureProvider === 'tailscale' ? '#3b82f6' : 'white', fontWeight: 'bold' }}>Tailscale Mesh</span>
                  <span className="text-secondary" style={{ fontSize: '0.8rem', marginTop: '4px' }}>Private access across your devices.</span>
                </button>
              </div>
            </GlassCard>

            <GlassCard title="Advanced: Environment Variables" icon={<Terminal size={24} />}>
              <p className="text-secondary" style={{ marginBottom: '16px' }}>
                Inject secure environment variables into the runtime container.
              </p>
              <div style={{ padding: '24px', background: 'rgba(0,0,0,0.2)', borderRadius: '8px', border: '1px dashed rgba(255,255,255,0.1)', textAlign: 'center' }}>
                <span className="text-secondary">Key/Value pair editor coming soon...</span>
              </div>
            </GlassCard>
          </>
        )}

        {/* LAUNCH BAR */}
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginTop: '12px' }}>
          <span style={{ color: status?.includes('Error') ? '#ff4d4f' : 'var(--text-secondary)' }}>
            {status}
          </span>
          <button 
            className="btn-primary" 
            onClick={handleLaunch} 
            disabled={isLaunching}
            style={{ opacity: isLaunching ? 0.7 : 1 }}
          >
            <Play size={18} />
            {isLaunching ? 'Provisioning...' : 'Deploy Workload'}
          </button>
        </div>

      </div>
    </div>
  );
};
