import { useState, useRef, useEffect, memo, useCallback } from 'react';
import { SendIcon } from './icons';
import { theme } from '../styles/theme';

interface ChatInputProps {
  onSend: (message: string) => void;
  disabled?: boolean;
  placeholder?: string;
}

export const ChatInput = memo<ChatInputProps>(({
  onSend,
  disabled = false,
  placeholder = 'Type your message...',
}) => {
  const [message, setMessage] = useState('');
  const [isFocused, setIsFocused] = useState(false);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  useEffect(() => {
    // Auto-resize textarea
    if (textareaRef.current) {
      textareaRef.current.style.height = 'auto';
      textareaRef.current.style.height = `${Math.min(
        textareaRef.current.scrollHeight,
        200
      )}px`;
    }
  }, [message]);

  const handleSubmit = useCallback((e: React.FormEvent) => {
    e.preventDefault();
    if (message.trim() && !disabled) {
      onSend(message.trim());
      setMessage('');
      if (textareaRef.current) {
        textareaRef.current.style.height = 'auto';
      }
    }
  }, [message, disabled, onSend]);

  const handleKeyDown = useCallback((e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSubmit(e);
    }
  }, [handleSubmit]);

  const canSend = message.trim().length > 0 && !disabled;

  return (
    <div style={styles.container}>
      <form onSubmit={handleSubmit} style={styles.form}>
        <div
          style={{
            ...styles.inputWrapper,
            ...(isFocused ? styles.inputWrapperFocused : {}),
          }}
        >
          <textarea
            ref={textareaRef}
            className="chat-input-textarea"
            value={message}
            onChange={(e) => setMessage(e.target.value)}
            onKeyDown={handleKeyDown}
            onFocus={() => setIsFocused(true)}
            onBlur={() => setIsFocused(false)}
            placeholder={placeholder}
            disabled={disabled}
            style={styles.textarea}
            rows={1}
            aria-label="Message input"
            aria-describedby="chat-input-help"
          />
          <span id="chat-input-help" style={styles.srOnly}>
            Press Enter to send, Shift+Enter for new line
          </span>
        </div>
        <button
          type="submit"
          className="chat-input-send-button"
          disabled={!canSend}
          style={{
            ...styles.sendButton,
            ...(!canSend ? styles.sendButtonDisabled : {}),
          }}
          title="Send message (Enter)"
          aria-label="Send message"
        >
          <SendIcon size={18} />
        </button>
      </form>
    </div>
  );
});

ChatInput.displayName = 'ChatInput';

const styles = {
  container: {
    borderTop: `1px solid ${theme.colors.border.primary}`,
    backgroundColor: theme.colors.background.secondary,
    padding: theme.spacing.lg,
    boxShadow: `0 -4px 6px ${theme.colors.background.primary}40`,
  },
  form: {
    display: 'flex',
    gap: theme.spacing.md,
    alignItems: 'flex-end',
    maxWidth: '100%',
  },
  inputWrapper: {
    flex: 1,
    position: 'relative' as const,
    borderRadius: theme.borderRadius.lg,
    backgroundColor: theme.colors.background.elevated,
    border: `1px solid ${theme.colors.border.primary}`,
    transition: `all ${theme.transitions.normal}`,
    boxShadow: theme.shadows.sm,
  },
  inputWrapperFocused: {
    borderColor: theme.colors.accent.primary,
    boxShadow: `0 0 0 3px ${theme.colors.accent.primary}20`,
  },
  textarea: {
    width: '100%',
    minHeight: '44px',
    maxHeight: '200px',
    padding: `${theme.spacing.md} ${theme.spacing.lg}`,
    fontSize: theme.typography.fontSize.base,
    fontFamily: theme.typography.fontFamily.primary,
    backgroundColor: 'transparent',
    border: 'none',
    borderRadius: theme.borderRadius.lg,
    color: theme.colors.text.primary,
    resize: 'none' as const,
    overflowY: 'auto' as const,
    lineHeight: theme.typography.lineHeight.relaxed,
    outline: 'none',
    transition: `all ${theme.transitions.normal}`,
  },
  sendButton: {
    width: '44px',
    height: '44px',
    borderRadius: theme.borderRadius.lg,
    border: 'none',
    backgroundColor: theme.colors.accent.primary,
    color: theme.colors.text.primary,
    cursor: 'pointer',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    transition: `all ${theme.transitions.spring}`,
    flexShrink: 0,
    boxShadow: theme.shadows.md,
  },
  sendButtonDisabled: {
    backgroundColor: theme.colors.interactive.disabled,
    color: theme.colors.text.muted,
    cursor: 'not-allowed',
    boxShadow: 'none',
  },
  srOnly: {
    position: 'absolute' as const,
    width: '1px',
    height: '1px',
    padding: 0,
    margin: '-1px',
    overflow: 'hidden',
    clip: 'rect(0, 0, 0, 0)',
    whiteSpace: 'nowrap' as const,
    borderWidth: 0,
  },
} as const;

// Add hover and focus effects
if (!document.getElementById('chat-input-styles')) {
  const styleSheet = document.createElement('style');
  styleSheet.id = 'chat-input-styles';
  styleSheet.textContent = `
    .chat-input-textarea::placeholder {
      color: ${theme.colors.text.tertiary};
      opacity: 0.6;
    }
    
    .chat-input-textarea:disabled {
      opacity: 0.5;
      cursor: not-allowed;
    }
    
    .chat-input-send-button:hover:not(:disabled) {
      background-color: ${theme.colors.accent.hover} !important;
      transform: scale(1.05);
      box-shadow: ${theme.shadows.lg} !important;
    }
    
    .chat-input-send-button:active:not(:disabled) {
      transform: scale(0.95);
    }
    
    .chat-input-send-button:disabled {
      opacity: 0.5;
    }
    
    /* Smooth scrollbar for textarea */
    .chat-input-textarea::-webkit-scrollbar {
      width: 6px;
    }
    
    .chat-input-textarea::-webkit-scrollbar-track {
      background: transparent;
    }
    
    .chat-input-textarea::-webkit-scrollbar-thumb {
      background: ${theme.colors.border.primary};
      border-radius: 3px;
    }
    
    .chat-input-textarea::-webkit-scrollbar-thumb:hover {
      background: ${theme.colors.border.secondary};
    }
  `;
  document.head.appendChild(styleSheet);
}
