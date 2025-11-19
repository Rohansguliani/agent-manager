import { Toaster } from 'react-hot-toast';

export const ToastProvider: React.FC = () => {
  return (
    <Toaster
      position="top-right"
      toastOptions={{
        duration: 4000,
        style: {
          background: '#2d2d2d',
          color: '#d4d4d4',
          border: '1px solid #3d3d3d',
          borderRadius: '8px',
          padding: '12px 16px',
          fontSize: '0.9rem',
          maxWidth: '400px',
        },
        success: {
          iconTheme: {
            primary: '#28a745',
            secondary: '#ffffff',
          },
          style: {
            borderColor: '#28a745',
          },
        },
        error: {
          iconTheme: {
            primary: '#dc3545',
            secondary: '#ffffff',
          },
          style: {
            borderColor: '#dc3545',
          },
        },
      }}
    />
  );
};

