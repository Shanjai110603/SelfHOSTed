import React, { useState } from 'react';
import { Shield, Globe, Lock, Activity, Link2, CheckCircle } from 'lucide-react';
import { GlassCard } from '../components/GlassCard';

export const Networking: React.FC = () => {
  const [cloudflareToken, setCloudflareToken] = useState('');
  const [tailscaleKey, setTailscaleKey] = useState('');
  
  const [isCloudflareLinked, setIsCloudflareLinked] = useState(false);
  const [isTailscaleLinked, setIsTailscaleLinked] = useState(false);

  return (
    <div className="page-container animate-fade-in">
      <header style={{ marginBottom: '40px' }}>
        <h1 className="title-large">Global Networking</h1>
        <p className="text-secondary">
          Manage your exposure providers and routing policies for the SelfHOSTed platform.
        </p>
      </header>

      <div style={{ display: 'grid', gridTemplateColumns: '1fr', gap: '24px', maxWidth: '800px' }}>
        
        {/* EDUCATIONAL UX */}
        <div style={{ padding: '24px', background: 'rgba(59,130,246,0.1)', border: '1px solid rgba(59,130,246,0.3)', borderRadius: '12px' }}>
          <h3 style={{ display: 'flex', alignItems: 'center', gap: '8px', color: '#3b82f6', marginBottom: '12px', fontSize: '1.2rem' }}>
            <Shield size={20} /> Adaptive Secure Exposure
          </h3>
          <p className="text-secondary" style={{ lineHeight: '1.6' }}>
            SelfHOSTed treats exposure as <strong>intent-driven orchestration</strong>. By default, all workloads are Local Only (accessible only from this device). 
            Link your global network providers below to unlock advanced routing capabilities across your workloads.
          </p>
        </div>

        {/* CLOUDFLARE ZERO TRUST */}
        <GlassCard title="Cloudflare Zero Trust" icon={<Globe size={24} />}>
          <div style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
            <p className="text-secondary" style={{ fontSize: '0.9rem' }}>
              Cloudflare enables you to securely expose workloads to the public internet using custom domains, HTTPS, and robust DDoS protection.
            </p>
            
            <div style={{ display: 'flex', alignItems: 'center', gap: '12px', padding: '16px', background: 'rgba(0,0,0,0.2)', borderRadius: '8px' }}>
              <div style={{ flex: 1 }}>
                <span style={{ display: 'block', fontWeight: 'bold' }}>Quick Tunnels (Free)</span>
                <span className="text-secondary" style={{ fontSize: '0.8rem' }}>Temporary <code>trycloudflare.com</code> URLs. No account required.</span>
              </div>
              <div style={{ display: 'flex', alignItems: 'center', gap: '6px', color: 'var(--success-color)', fontSize: '0.9rem' }}>
                <CheckCircle size={16} /> Active
              </div>
            </div>

            <div style={{ display: 'flex', alignItems: 'center', gap: '12px', padding: '16px', background: 'rgba(0,0,0,0.2)', borderRadius: '8px' }}>
              <div style={{ flex: 1 }}>
                <span style={{ display: 'block', fontWeight: 'bold', color: 'rgba(255,255,255,0.5)' }}>Authenticated Tunnels</span>
                <span className="text-secondary" style={{ fontSize: '0.8rem' }}>Stable custom domains. Coming in EPIC-002 Phase 2.</span>
              </div>
              <Lock size={16} style={{ color: 'rgba(255,255,255,0.3)' }} />
            </div>
          </div>
        </GlassCard>

        {/* TAILSCALE MESH */}
        <GlassCard title="Tailscale Private Mesh" icon={<Activity size={24} />}>
          <div style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
            <p className="text-secondary" style={{ fontSize: '0.9rem' }}>
              Tailscale weaves your workloads into a secure, private wireguard mesh network. Accessible only by devices you explicitly authorize on your Tailnet.
            </p>
            
            {!isTailscaleLinked ? (
              <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
                <label className="text-secondary" style={{ fontSize: '0.9rem' }}>Provide a Reusable Auth Key to link this platform:</label>
                <div style={{ display: 'flex', gap: '12px' }}>
                  <input 
                    type="password" 
                    value={tailscaleKey} 
                    onChange={(e) => setTailscaleKey(e.target.value)}
                    placeholder="tskey-auth-xxxxxx-xxxxxx"
                    style={{ flex: 1, padding: '12px', background: 'rgba(0,0,0,0.2)', border: '1px solid rgba(255,255,255,0.1)', color: 'white', borderRadius: '8px' }}
                  />
                  <button 
                    className="btn-primary" 
                    onClick={() => {
                      if (tailscaleKey.trim()) {
                        setIsTailscaleLinked(true);
                      }
                    }}
                  >
                    <Link2 size={18} /> Link Account
                  </button>
                </div>
              </div>
            ) : (
              <div style={{ display: 'flex', alignItems: 'center', gap: '12px', padding: '16px', background: 'rgba(16,185,129,0.1)', border: '1px solid rgba(16,185,129,0.3)', borderRadius: '8px' }}>
                <CheckCircle size={24} color="#10b981" />
                <div style={{ flex: 1 }}>
                  <strong style={{ color: '#10b981', display: 'block' }}>Tailscale Mesh Active</strong>
                  <span className="text-secondary" style={{ fontSize: '0.85rem' }}>You can now orchestrate private mesh exposure directly from the Launch Workload screen.</span>
                </div>
                <button 
                  className="btn-secondary" 
                  onClick={() => {
                    setIsTailscaleLinked(false);
                    setTailscaleKey('');
                  }}
                  style={{ padding: '8px 16px', fontSize: '0.9rem' }}
                >
                  Unlink
                </button>
              </div>
            )}
          </div>
        </GlassCard>

      </div>
    </div>
  );
};
