import React, { useState, useEffect } from 'react';
import { Lock, Unlock, ArrowRight } from 'lucide-react';
import { EducationalTooltip } from '../components/EducationalTooltip';

interface LockScreenProps {
  onUnlock: () => void;
}

export const LockScreen: React.FC<LockScreenProps> = ({ onUnlock }) => {
  const [pin, setPin] = useState('');
  const [isSetup, setIsSetup] = useState(false);
  const [error, setError] = useState(false);

  useEffect(() => {
    const existingPin = localStorage.getItem('master_pin');
    if (!existingPin) {
      setIsSetup(true);
    }
  }, []);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (pin.length < 4) {
      setError(true);
      return;
    }

    if (isSetup) {
      localStorage.setItem('master_pin', btoa(pin)); // basic encoding for MVP
      onUnlock();
    } else {
      const stored = localStorage.getItem('master_pin');
      if (btoa(pin) === stored) {
        onUnlock();
      } else {
        setError(true);
        setPin('');
      }
    }
  };

  return (
    <div className="page-container animate-fade-in" style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: '100vh', flexDirection: 'column' }}>
      
      <div className="glass-panel" style={{ padding: '48px', maxWidth: '400px', width: '100%', textAlign: 'center' }}>
        <div style={{ background: 'rgba(59,130,246,0.1)', padding: '16px', borderRadius: '50%', display: 'inline-flex', marginBottom: '24px' }}>
          <Lock size={48} color="var(--primary-color)" />
        </div>
        
        <h2 style={{ marginBottom: '8px' }}>
          {isSetup ? 'Secure Your Environment' : 'App Locked'}
        </h2>
        
        <p className="text-secondary" style={{ marginBottom: '32px' }}>
          {isSetup ? (
            <span>
              Create a Master PIN to protect your orchestration engine from unauthorized physical access.
              <EducationalTooltip 
                title="Physical Security"
                content="Because SelfHOSTed has root-level control over workloads and exposure, a Master PIN prevents someone from walking up to your laptop and exposing a private database to the internet."
              />
            </span>
          ) : (
            'Enter your Master PIN to access the dashboard.'
          )}
        </p>

        <form onSubmit={handleSubmit}>
          <input
            type="password"
            placeholder={isSetup ? "Create 4+ digit PIN" : "Enter PIN"}
            value={pin}
            onChange={(e) => { setPin(e.target.value); setError(false); }}
            style={{ 
              width: '100%', 
              padding: '16px', 
              fontSize: '1.2rem',
              textAlign: 'center',
              letterSpacing: '0.2em',
              background: 'rgba(0,0,0,0.2)', 
              border: `1px solid ${error ? 'var(--warning-color)' : 'rgba(255,255,255,0.1)'}`, 
              color: 'white', 
              borderRadius: '8px', 
              marginBottom: '24px' 
            }}
            autoFocus
          />

          <button className="btn-primary" type="submit" style={{ width: '100%', justifyContent: 'center' }}>
            {isSetup ? 'Set PIN & Continue' : 'Unlock'} <ArrowRight size={18} />
          </button>
        </form>

        {error && (
          <p style={{ color: 'var(--warning-color)', fontSize: '0.9rem', marginTop: '16px' }}>
            {isSetup ? 'PIN must be at least 4 digits.' : 'Incorrect PIN. Try again.'}
          </p>
        )}
      </div>

    </div>
  );
};
