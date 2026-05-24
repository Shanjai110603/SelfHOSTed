import { Home, Server, HardDrive, Settings, Activity, Database, Share2, Globe, LayoutDashboard, FolderSync, Shield, Package } from 'lucide-react';
import './Sidebar.css';

interface SidebarProps {
  currentView: string;
  onNavigate: (view: string) => void;
}

export function Sidebar({ currentView, onNavigate }: SidebarProps) {
  return (
    <nav className="glass-panel sidebar">
      <div className="sidebar-header">
        <div className="logo-container">
          <div className="logo-icon"></div>
          <h2>SelfHOSTed</h2>
        </div>
      </div>
      <div className="sidebar-menu">
        <button className={`menu-item ${currentView === 'dashboard' ? 'active' : ''}`} onClick={() => onNavigate('dashboard')} style={{background: 'transparent', border: 'none', textAlign: 'left', cursor: 'pointer', fontFamily: 'inherit', fontSize: '1rem', width: '100%'}}>
          <Home size={20} />
          <span>Dashboard</span>
        </button>
        <button className={`menu-item ${currentView === 'website' ? 'active' : ''}`} onClick={() => onNavigate('website')} style={{background: 'transparent', border: 'none', textAlign: 'left', cursor: 'pointer', fontFamily: 'inherit', fontSize: '1rem', width: '100%'}}>
          <Server size={20} />
          <span>Website Hosting</span>
        </button>
        <button className={`menu-item ${currentView === 'database' ? 'active' : ''}`} onClick={() => onNavigate('database')} style={{background: 'transparent', border: 'none', textAlign: 'left', cursor: 'pointer', fontFamily: 'inherit', fontSize: '1rem', width: '100%'}}>
          <Database size={20} />
          <span>Database Hosting</span>
        </button>
        <button className={`menu-item ${currentView === 'marketplace' ? 'active' : ''}`} onClick={() => onNavigate('marketplace')} style={{background: 'transparent', border: 'none', textAlign: 'left', cursor: 'pointer', fontFamily: 'inherit', fontSize: '1rem', width: '100%'}}>
          <Package size={20} />
          <span>App Marketplace</span>
        </button>
        <button className={`menu-item ${currentView === 'networking' ? 'active' : ''}`} onClick={() => onNavigate('networking')} style={{background: 'transparent', border: 'none', textAlign: 'left', cursor: 'pointer', fontFamily: 'inherit', fontSize: '1rem', width: '100%'}}>
          <Activity size={20} />
          <span>Networking</span>
        </button>
        <button className={`menu-item ${currentView === 'fileshare' ? 'active' : ''}`} onClick={() => onNavigate('fileshare')} style={{background: 'transparent', border: 'none', textAlign: 'left', cursor: 'pointer', fontFamily: 'inherit', fontSize: '1rem', width: '100%'}}>
          <Share2 size={20} />
          <span>File Sharing</span>
        </button>
        <button className="menu-item" style={{background: 'transparent', border: 'none', textAlign: 'left', cursor: 'pointer', fontFamily: 'inherit', fontSize: '1rem', width: '100%'}}>
          <Settings size={20} />
          <span>Settings</span>
        </button>
        <div style={{ margin: '20px 0', borderTop: '1px solid rgba(255,255,255,0.1)' }}></div>
        <button className={`menu-item ${currentView === 'dev-dashboard' ? 'active' : ''}`} onClick={() => onNavigate('dev-dashboard')} style={{background: 'transparent', border: 'none', textAlign: 'left', cursor: 'pointer', fontFamily: 'inherit', fontSize: '0.9rem', width: '100%', color: 'var(--accent-color)'}}>
          <Activity size={18} />
          <span>Dev Dashboard</span>
        </button>
      </div>
    </nav>
  );
}
