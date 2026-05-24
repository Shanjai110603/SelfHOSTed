import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

export interface CapabilityEngine {
  supports_docker: boolean;
  supports_databases: boolean;
  supports_public_tunnels: boolean;
  platform: string;
}

export function useCapabilities() {
  const [capabilities, setCapabilities] = useState<CapabilityEngine | null>(null);

  useEffect(() => {
    invoke<CapabilityEngine>('get_capabilities')
      .then(setCapabilities)
      .catch(console.error);
  }, []);

  return capabilities;
}
