import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Sidebar } from './components/Sidebar';
import { GlassCard } from './components/GlassCard';
import { Rocket, Shield, Cpu, Activity, Share2 } from 'lucide-react';
import { Onboarding } from './pages/Onboarding';
import { WebsiteHosting } from './pages/WebsiteHosting';
import { DatabaseHosting } from './pages/DatabaseHosting';
import { Networking } from './pages/Networking';
import { FileSharing } from './pages/FileSharing';
import { Marketplace } from './pages/Marketplace';
import { DevDashboard } from './pages/DevDashboard';
import { LockScreen } from './pages/LockScreen';
import { TelemetryWidget } from './components/TelemetryWidget';
import { OrchestrationToast } from './components/OrchestrationToast';
import './App.css';

interface SystemStats {
  cpu_usage: number;
  total_memory: number;
  used_memory: number;
}

function App() {
  const [onboardingComplete, setOnboardingComplete] = useState(false);
  const [isUnlocked, setIsUnlocked] = useState(false);
  const [stats, setStats] = useState<SystemStats | null>(null);
  const [currentView, setCurrentView] = useState('dashboard');

  useEffect(() => {
    if (!onboardingComplete) return;
  }, [onboardingComplete]);

  if (!onboardingComplete) {
    return <Onboarding onComplete={(view) => {
      setOnboardingComplete(true);
      if (view) setCurrentView(view);
    }} />;
  }

  if (!isUnlocked) {
    return <LockScreen onUnlock={() => setIsUnlocked(true)} />;
  }

  const formatMem = (bytes: number) => (bytes / 1e9).toFixed(1);
  const cpuPercent = stats ? Math.round(stats.cpu_usage) : 0;
  const memPercent = stats ? Math.round((stats.used_memory / stats.total_memory) * 100) : 0;

  return (
    <div className="app-container">
      <OrchestrationToast />
      <Sidebar currentView={currentView} onNavigate={setCurrentView} />
      <main className="main-content animate-fade-in">
        {currentView === 'dashboard' ? (
          <>
            <header style={{ marginBottom: '40px' }}>
              <h1 className="title-large">Welcome to SelfHOSTed</h1>
              <p className="text-secondary">
                Your personal infrastructure platform. Securely host websites, databases, and files with one click.
              </p>
            </header>

            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(300px, 1fr))', gap: '24px' }}>
              <TelemetryWidget />

              <GlassCard 
                title="Quick Start" 
                description="Deploy your first application instantly."
                icon={<Rocket size={24} />}
              >
                <button className="btn-primary" style={{ width: '100%', justifyContent: 'center' }} onClick={() => setCurrentView('website')}>
                  <Rocket size={18} />
                  Host a Website
                </button>
                <div style={{ display: 'flex', gap: '12px', marginTop: '12px' }}>
                  <button className="btn-primary" style={{ flex: 1, justifyContent: 'center', background: 'rgba(255,255,255,0.1)', boxShadow: 'none' }} onClick={() => setCurrentView('database')}>
                    <Shield size={18} />
                    Database
                  </button>
                  <button className="btn-primary" style={{ flex: 1, justifyContent: 'center', background: 'rgba(255,255,255,0.1)', boxShadow: 'none' }} onClick={() => setCurrentView('fileshare')}>
                    <Share2 size={18} />
                    Share Files
                  </button>
                </div>
              </GlassCard>
              
              <GlassCard 
                title="Capabilities" 
                description="Hardware analysis complete."
                icon={<Cpu size={24} />}
              >
                 <ul style={{ margin: 0, paddingLeft: '20px', color: 'var(--text-secondary)', display: 'flex', flexDirection: 'column', gap: '8px' }}>
                   <li>Supports lightweight APIs</li>
                   <li>Supports local PostgreSQL</li>
                   <li>Up to 5 static websites</li>
                   <li>Battery is healthy (plugged in)</li>
                 </ul>
              </GlassCard>
            </div>
          </>
        ) : currentView === 'website' ? (
          <WebsiteHosting />
        ) : currentView === 'database' ? (
          <DatabaseHosting />
        ) : currentView === 'marketplace' ? (
          <Marketplace />
        ) : currentView === 'networking' ? (
          <Networking />
        ) : currentView === 'fileshare' ? (
          <FileSharing />
        ) : currentView === 'dev-dashboard' ? (
          <DevDashboard />
        ) : null}
      </main>
    </div>
  );
}

export default App;
