import { useState } from 'react';
import toast from 'react-hot-toast';
import { CopyIcon, CheckIcon, RefreshIcon } from './icons';
import { theme } from '../styles/theme';

interface MessageActionsProps {
  content: string;
  role: 'user' | 'assistant';
  onRegenerate?: () => void;
}

export const MessageActions: React.FC<MessageActionsProps> = ({
  content,
  role,
  onRegenerate,
}) => {
  const [copied, setCopied] = useState(false);

  const copyToClipboard = async () => {
    try {
      await navigator.clipboard.writeText(content);
      setCopied(true);
      toast.success('Message copied to clipboard');
      setTimeout(() => setCopied(false), 2000);
    } catch {
      toast.error('Failed to copy message');
    }
  };

  return (
    <div style={styles.container} role="group" aria-label="Message actions">
      <button
        onClick={copyToClipboard}
        style={styles.button}
        aria-label="Copy message"
        title="Copy message"
      >
        {copied ? (
          <CheckIcon size={14} style={styles.checkIcon} />
        ) : (
          <CopyIcon size={14} />
        )}
      </button>
      {role === 'assistant' && onRegenerate && (
        <button
          onClick={onRegenerate}
          style={styles.button}
          aria-label="Regenerate response"
          title="Regenerate response"
        >
          <RefreshIcon size={14} />
        </button>
      )}
    </div>
  );
};

const styles = {
  container: {
    display: 'flex',
    gap: theme.spacing.xs,
    opacity: 0,
    transition: `opacity ${theme.transitions.normal}`,
  },
  button: {
    width: '28px',
    height: '28px',
    padding: 0,
    backgroundColor: theme.colors.background.elevated,
    border: `1px solid ${theme.colors.border.primary}`,
    borderRadius: theme.borderRadius.sm,
    color: theme.colors.text.secondary,
    cursor: 'pointer',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    transition: `all ${theme.transitions.normal}`,
    boxShadow: theme.shadows.sm,
  },
  checkIcon: {
    color: theme.colors.status.success,
  },
} as const;

// Add hover styles
if (!document.getElementById('message-actions-styles')) {
  const styleSheet = document.createElement('style');
  styleSheet.id = 'message-actions-styles';
  styleSheet.textContent = `
    .message-item:hover .message-actions {
      opacity: 1 !important;
    }
    
    .message-actions button:hover {
      background-color: ${theme.colors.interactive.hover} !important;
      border-color: ${theme.colors.border.accent} !important;
      color: ${theme.colors.accent.primary} !important;
      transform: translateY(-1px);
      box-shadow: ${theme.shadows.md} !important;
    }
    
    .message-actions button:active {
      transform: translateY(0);
    }
  `;
  document.head.appendChild(styleSheet);
}
