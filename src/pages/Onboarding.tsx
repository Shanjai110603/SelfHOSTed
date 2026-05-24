import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Server, Shield, Cpu, ArrowRight, CheckCircle2, Loader2, Package, Globe, Lock, Play } from 'lucide-react';
import './Onboarding.css';

interface OnboardingProps {
  onComplete: (view?: string) => void;
}

interface RuntimeStatus {
  installed: boolean;
  engine: string;
  active_containers: number;
}

interface SystemStats {
  cpu_usage: number;
  total_memory: number;
  used_memory: number;
}

export function Onboarding({ onComplete }: OnboardingProps) {
  const [step, setStep] = useState(0);
  const [checkingDeps, setCheckingDeps] = useState(true);
  const [depsInstalled, setDepsInstalled] = useState(false);
  const [sysStats, setSysStats] = useState<SystemStats | null>(null);
  const [isDeploying, setIsDeploying] = useState(false);

  useEffect(() => {
    if (step === 1) {
      setCheckingDeps(true);
      invoke<RuntimeStatus>('check_runtime')
        .then((status) => {
          setDepsInstalled(status.installed);
          setCheckingDeps(false);
        })
        .catch((err) => {
          console.error('Failed to check runtime:', err);
          setDepsInstalled(false);
          setCheckingDeps(false);
        });
    } else if (step === 2) {
      invoke<SystemStats>('get_system_stats')
        .then(stats => setSysStats(stats))
        .catch(err => console.error('Failed to get hardware stats:', err));
    }
  }, [step]);

  const renderWelcome = () => (
    <div className="onboarding-step animate-fade-in">
      <div className="onboarding-icon-large">
        <Server size={48} />
      </div>
      <h2>Turn your device into a personal cloud</h2>
      <p className="text-secondary onboarding-desc">
        SelfHOSTed allows you to securely host websites, databases, and files right from your own hardware. No cloud subscriptions required.
      </p>
      <button className="btn-primary onboarding-btn" onClick={() => setStep(1)}>
        Get Started <ArrowRight size={18} />
      </button>
    </div>
  );

  const renderDependencies = () => (
    <div className="onboarding-step animate-fade-in">
      <div className="onboarding-icon-large">
        <Package size={48} />
      </div>
      <h2>App Runtime Check</h2>
      
      {checkingDeps ? (
        <div className="status-box">
          <Loader2 className="spinner" size={24} />
          <span>Scanning system for required runtimes...</span>
        </div>
      ) : depsInstalled ? (
        <div className="status-box success flex-col items-start text-left">
          <div style={{display:'flex', gap:'12px'}}>
            <CheckCircle2 size={24} />
            <span>All required runtimes are installed!</span>
          </div>
          <button className="btn-primary mt-4" style={{alignSelf: 'center'}} onClick={() => setStep(2)}>Continue</button>
        </div>
      ) : (
        <div className="dependency-explanation text-left">
          <p className="text-secondary">We noticed your system is missing the required <strong>App Runtime</strong>.</p>
          
          <div className="glass-card dep-card">
            <h4>What is an App Runtime?</h4>
            <p className="text-secondary text-sm">
              It is a secure environment (like a sandbox) that allows applications and databases to run isolated from your main system.
            </p>
            
            <h4 className="mt-4">Why is it needed?</h4>
            <p className="text-secondary text-sm">
              It ensures that anything you host cannot interfere with your personal files or slow down your device permanently.
            </p>

            <button className="btn-primary mt-4 w-full justify-center" onClick={() => setDepsInstalled(true)}>
              Install Automatically
            </button>
          </div>
        </div>
      )}
    </div>
  );

  const formatMem = (bytes: number) => (bytes / 1e9).toFixed(1);

  const renderHardware = () => (
    <div className="onboarding-step animate-fade-in">
      <div className="onboarding-icon-large">
        <Cpu size={48} />
      </div>
      <h2>Hardware Profiling</h2>
      <p className="text-secondary onboarding-desc">
        We are analyzing your device to recommend safe hosting limits.
      </p>
      
      <div className="glass-card text-left mt-4" style={{width: '100%'}}>
        {sysStats ? (
          <ul className="capabilities-list">
            <li><CheckCircle2 size={18} className="text-success" /> {formatMem(sysStats.total_memory)} GB RAM Detected</li>
            <li><CheckCircle2 size={18} className="text-success" /> Multi-core CPU active</li>
            <li><CheckCircle2 size={18} className="text-success" /> Battery healthy (Plugged in)</li>
          </ul>
        ) : (
           <div className="status-box"><Loader2 className="spinner" size={20} /><span>Analyzing hardware...</span></div>
        )}
        <div className="mt-4 p-3 bg-accent-alpha rounded text-sm">
          Recommendation: Your device can comfortably host up to 5 websites and a local database simultaneously.
        </div>
      </div>

      <button className="btn-primary onboarding-btn mt-6" onClick={() => setStep(3)}>
        Next: Exposure & Networking <ArrowRight size={18} />
      </button>
    </div>
  );

  const renderExposureEdu = () => (
    <div className="onboarding-step animate-fade-in">
      <div className="onboarding-icon-large">
        <Globe size={48} />
      </div>
      <h2>Who can see your apps?</h2>
      <p className="text-secondary onboarding-desc">
        SelfHOSTed defaults to extreme privacy. By default, everything you host is <strong>Local Only</strong>.
      </p>
      
      <div className="glass-card text-left mt-4" style={{width: '100%'}}>
        <div style={{ marginBottom: '16px' }}>
          <strong style={{ color: 'var(--success-color)' }}>1. Local Only (Default)</strong>
          <p className="text-secondary text-sm">Visible only to devices connected to your home Wi-Fi network. Completely isolated from the internet.</p>
        </div>
        <div style={{ marginBottom: '16px' }}>
          <strong style={{ color: 'var(--primary-color)' }}>2. Tailscale Mesh</strong>
          <p className="text-secondary text-sm">Visible anywhere in the world, but <strong>only</strong> to devices you explicitly approve on your Tailnet. Think of it as a private, encrypted global Wi-Fi.</p>
        </div>
        <div>
          <strong style={{ color: 'var(--warning-color)' }}>3. Cloudflare Public</strong>
          <p className="text-secondary text-sm">Visible to anyone on the public internet, protected by Cloudflare's enterprise-grade DDoS mitigation.</p>
        </div>
      </div>

      <button className="btn-primary onboarding-btn mt-6" onClick={() => setStep(4)}>
        Next: Security <ArrowRight size={18} />
      </button>
    </div>
  );

  const renderVaultEdu = () => (
    <div className="onboarding-step animate-fade-in">
      <div className="onboarding-icon-large">
        <Lock size={48} />
      </div>
      <h2>The Secure Vault</h2>
      <p className="text-secondary onboarding-desc">
        No more manually copying and pasting database passwords into <code>.env</code> files.
      </p>
      
      <div className="glass-card text-left mt-4" style={{width: '100%', background: 'rgba(16,185,129,0.1)', border: '1px solid rgba(16,185,129,0.3)'}}>
        <p className="text-secondary" style={{ lineHeight: '1.6' }}>
          When you launch an application, SelfHOSTed automatically <strong>generates, encrypts, and seamlessly injects</strong> secure passwords directly into the workloads behind the scenes.
        </p>
        <p className="text-secondary mt-3" style={{ lineHeight: '1.6' }}>
          Your secrets are kept perfectly safe out of plaintext.
        </p>
      </div>

      <button className="btn-primary onboarding-btn mt-6" onClick={() => setStep(5)}>
        Final Step <ArrowRight size={18} />
      </button>
    </div>
  );

  const deployHelloWorld = async () => {
    setIsDeploying(true);
    try {
      await invoke('deploy_stack', {
        templateId: "hello-world",
        domain: null,
        exposure: null
      });
      // Redirect straight to WebsiteHosting to see the newly launched site
      onComplete('website');
    } catch (e) {
      console.error(e);
      alert("Failed to launch hello world template: " + e);
      setIsDeploying(false);
    }
  };

  const renderInstantDeploy = () => (
    <div className="onboarding-step animate-fade-in">
      <div className="onboarding-icon-large">
        <Play size={48} />
      </div>
      <h2>You're Ready.</h2>
      <p className="text-secondary onboarding-desc">
        Experience the orchestration engine instantly. Launch a lightning-fast demonstration website in 1 click.
      </p>
      
      <div style={{ display: 'flex', gap: '16px', marginTop: '32px', width: '100%' }}>
        <button 
          className="btn-secondary" 
          style={{ flex: 1, justifyContent: 'center' }} 
          onClick={() => onComplete('dashboard')}
          disabled={isDeploying}
        >
          Skip to Dashboard
        </button>
        <button 
          className="btn-primary" 
          style={{ flex: 2, justifyContent: 'center', background: 'var(--success-color)' }} 
          onClick={deployHelloWorld}
          disabled={isDeploying}
        >
          {isDeploying ? <><Loader2 size={18} className="spinner" /> Provisioning...</> : 'Deploy "Hello World"'}
        </button>
      </div>
    </div>
  );

  return (
    <div className="onboarding-container">
      <div className="glass-panel onboarding-modal">
        {step === 0 && renderWelcome()}
        {step === 1 && renderDependencies()}
        {step === 2 && renderHardware()}
        {step === 3 && renderExposureEdu()}
        {step === 4 && renderVaultEdu()}
        {step === 5 && renderInstantDeploy()}
        
        <div className="step-indicators">
          <div className={`step-dot ${step >= 0 ? 'active' : ''}`}></div>
          <div className={`step-dot ${step >= 1 ? 'active' : ''}`}></div>
          <div className={`step-dot ${step >= 2 ? 'active' : ''}`}></div>
          <div className={`step-dot ${step >= 3 ? 'active' : ''}`}></div>
          <div className={`step-dot ${step >= 4 ? 'active' : ''}`}></div>
          <div className={`step-dot ${step >= 5 ? 'active' : ''}`}></div>
        </div>
      </div>
    </div>
  );
}
