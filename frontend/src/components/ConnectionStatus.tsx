import React, { useEffect, useState } from 'react';
import { api } from '../api';

export const ConnectionStatus: React.FC = () => {
  const [isOnline, setIsOnline] = useState(true);
  const [isChecking, setIsChecking] = useState(false);

  useEffect(() => {
    const checkConnection = async () => {
      setIsChecking(true);
      try {
        await api.healthCheck();
        setIsOnline(true);
      } catch {
        setIsOnline(false);
      } finally {
        setIsChecking(false);
      }
    };

    // Check immediately
    checkConnection();

    // Check every 30 seconds
    const interval = setInterval(checkConnection, 30000);

    // Check on online/offline events
    const handleOnline = () => {
      setIsOnline(true);
      checkConnection();
    };
    const handleOffline = () => {
      setIsOnline(false);
    };

    window.addEventListener('online', handleOnline);
    window.addEventListener('offline', handleOffline);

    return () => {
      clearInterval(interval);
      window.removeEventListener('online', handleOnline);
      window.removeEventListener('offline', handleOffline);
    };
  }, []);

  if (isOnline && !isChecking) {
    return null; // Don't show when online
  }

  return (
    <div
      style={{
        position: 'fixed',
        top: '1rem',
        left: '50%',
        transform: 'translateX(-50%)',
        zIndex: 10001,
        padding: '0.5rem 1rem',
        backgroundColor: isOnline ? '#28a745' : '#dc3545',
        color: '#ffffff',
        borderRadius: '6px',
        fontSize: '0.85rem',
        display: 'flex',
        alignItems: 'center',
        gap: '0.5rem',
        boxShadow: '0 4px 6px rgba(0, 0, 0, 0.3)',
      }}
    >
      <span>{isChecking ? '🔄' : isOnline ? '✓' : '✗'}</span>
      <span>
        {isChecking
          ? 'Checking connection...'
          : isOnline
          ? 'Connected'
          : 'Connection lost - Retrying...'}
      </span>
    </div>
  );
};

