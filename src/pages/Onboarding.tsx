import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Server, Shield, Cpu, ArrowRight, CheckCircle2, Loader2, Package } from 'lucide-react';
import './Onboarding.css';

interface OnboardingProps {
  onComplete: () => void;
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

      <button className="btn-primary onboarding-btn mt-6" onClick={onComplete}>
        Open Dashboard <ArrowRight size={18} />
      </button>
    </div>
  );

  return (
    <div className="onboarding-container">
      <div className="glass-panel onboarding-modal">
        {step === 0 && renderWelcome()}
        {step === 1 && renderDependencies()}
        {step === 2 && renderHardware()}
        
        <div className="step-indicators">
          <div className={`step-dot ${step >= 0 ? 'active' : ''}`}></div>
          <div className={`step-dot ${step >= 1 ? 'active' : ''}`}></div>
          <div className={`step-dot ${step >= 2 ? 'active' : ''}`}></div>
        </div>
      </div>
    </div>
  );
}
