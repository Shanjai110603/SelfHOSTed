import React from 'react';

interface GlassCardProps {
  title: string;
  description?: string;
  children: React.ReactNode;
  icon?: React.ReactNode;
}

export function GlassCard({ title, description, children, icon }: GlassCardProps) {
  return (
    <div className="glass-card">
      <div style={{ display: 'flex', alignItems: 'center', gap: '12px', marginBottom: '16px' }}>
        {icon && <div style={{ color: 'var(--accent-color)' }}>{icon}</div>}
        <h3 style={{ margin: 0 }}>{title}</h3>
      </div>
      {description && <p className="text-secondary" style={{ marginTop: 0, marginBottom: '20px' }}>{description}</p>}
      <div>
        {children}
      </div>
    </div>
  );
}
