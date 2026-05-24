import React, { useState, useRef, useEffect } from 'react';
import { Info } from 'lucide-react';

interface EducationalTooltipProps {
  title: string;
  content: React.ReactNode;
  position?: 'top' | 'bottom' | 'left' | 'right';
  iconSize?: number;
}

export const EducationalTooltip: React.FC<EducationalTooltipProps> = ({ 
  title, 
  content, 
  position = 'top',
  iconSize = 16 
}) => {
  const [isOpen, setIsOpen] = useState(false);
  const containerRef = useRef<HTMLDivElement>(null);

  // Close when clicking outside
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (containerRef.current && !containerRef.current.contains(event.target as Node)) {
        setIsOpen(false);
      }
    };
    if (isOpen) document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, [isOpen]);

  const getPositionStyles = () => {
    switch (position) {
      case 'top': return { bottom: '100%', left: '50%', transform: 'translateX(-50%)', marginBottom: '8px' };
      case 'bottom': return { top: '100%', left: '50%', transform: 'translateX(-50%)', marginTop: '8px' };
      case 'left': return { right: '100%', top: '50%', transform: 'translateY(-50%)', marginRight: '8px' };
      case 'right': return { left: '100%', top: '50%', transform: 'translateY(-50%)', marginLeft: '8px' };
      default: return {};
    }
  };

  return (
    <div ref={containerRef} style={{ position: 'relative', display: 'inline-flex', alignItems: 'center', marginLeft: '6px' }}>
      <button
        onClick={() => setIsOpen(!isOpen)}
        onMouseEnter={() => setIsOpen(true)}
        onMouseLeave={() => setIsOpen(false)}
        style={{
          background: 'transparent',
          border: 'none',
          padding: '2px',
          cursor: 'pointer',
          color: isOpen ? 'var(--primary-color)' : 'rgba(255,255,255,0.4)',
          transition: 'all 0.2s ease',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center'
        }}
      >
        <Info size={iconSize} />
      </button>

      {isOpen && (
        <div 
          className="glass-panel animate-fade-in"
          style={{
            position: 'absolute',
            zIndex: 9999,
            width: 'max-content',
            maxWidth: '300px',
            padding: '16px',
            boxShadow: '0 10px 30px -10px rgba(0,0,0,0.5)',
            border: '1px solid rgba(59,130,246,0.3)',
            ...getPositionStyles()
          }}
        >
          <h4 style={{ color: 'var(--primary-color)', marginBottom: '8px', fontSize: '0.95rem' }}>{title}</h4>
          <div style={{ color: 'rgba(255,255,255,0.8)', fontSize: '0.85rem', lineHeight: '1.5' }}>
            {content}
          </div>
        </div>
      )}
    </div>
  );
};
