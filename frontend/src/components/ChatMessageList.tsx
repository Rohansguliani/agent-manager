import { useEffect, useRef, useMemo, memo } from 'react';
import { MessageContent } from './MessageContent';
import { MessageActions } from './MessageActions';
import { ChatIcon, LoaderIcon } from './icons';
import { theme } from '../styles/theme';

export interface Message {
  id: string;
  conversation_id: string;
  role: 'user' | 'assistant';
  content: string;
  created_at: number;
}

interface ChatMessageListProps {
  messages: Message[];
  streamingContent?: string;
  loading?: boolean;
  onRegenerate?: (messageId: string) => void;
}

export const ChatMessageList = memo<ChatMessageListProps>(({
  messages,
  streamingContent,
  loading = false,
  onRegenerate,
}) => {
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages, streamingContent]);

  const allMessages = useMemo(() => {
    return messages;
  }, [messages]);

  return (
    <div
      ref={containerRef}
      style={styles.container}
      role="log"
      aria-label="Chat messages"
      aria-live="polite"
      aria-atomic="false"
    >
      {allMessages.length === 0 && !loading && !streamingContent ? (
        <div style={styles.empty}>
          <ChatIcon size={64} style={styles.emptyIcon} />
          <p style={styles.emptyText}>
            Start a conversation by sending a message
          </p>
          <p style={styles.emptySubtext}>
            Your AI assistant is ready to help
          </p>
        </div>
      ) : (
        <>
          {allMessages.map((message, index) => (
            <MessageItem
              key={message.id}
              message={message}
              onRegenerate={onRegenerate}
              isFirst={index === 0}
            />
          ))}
          {streamingContent && (
            <div
              style={{
                ...styles.message,
                ...styles.messageAssistant,
              }}
              className="message-streaming"
            >
              <div
                style={{
                  ...styles.messageContent,
                  ...messageRoleStyles.assistant,
                }}
              >
                <MessageContent content={streamingContent} role="assistant" />
                <span style={styles.cursor}>▊</span>
              </div>
            </div>
          )}
          {loading && !streamingContent && (
            <div
              style={{
                ...styles.message,
                ...styles.messageAssistant,
              }}
              className="message-loading"
            >
              <div
                style={{
                  ...styles.messageContent,
                  ...messageRoleStyles.assistant,
                }}
              >
                <div style={styles.loadingContainer}>
                  <LoaderIcon size={16} style={styles.loaderIcon} />
                  <span style={styles.loadingText}>Thinking...</span>
                </div>
              </div>
            </div>
          )}
          <div ref={messagesEndRef} />
        </>
      )}
    </div>
  );
});

ChatMessageList.displayName = 'ChatMessageList';

const MessageItem = memo<{
  message: Message;
  onRegenerate?: (messageId: string) => void;
  isFirst: boolean;
}>(({ message, onRegenerate, isFirst }) => {
  return (
    <div
      className="message-item"
      style={{
        ...styles.message,
        ...(message.role === 'user'
          ? styles.messageUser
          : styles.messageAssistant),
        animationDelay: isFirst ? '0ms' : '100ms',
      }}
    >
      <div style={styles.messageHeader}>
        <div
          style={{
            ...styles.messageContent,
            ...(message.role === 'user'
              ? messageRoleStyles.user
              : messageRoleStyles.assistant),
          }}
        >
          <MessageContent content={message.content} role={message.role} />
        </div>
        <div className="message-actions" style={styles.messageActions}>
          <MessageActions
            content={message.content}
            role={message.role}
            onRegenerate={onRegenerate ? () => onRegenerate(message.id) : undefined}
          />
        </div>
      </div>
      <div style={styles.messageTime}>
        {formatMessageTime(message.created_at)}
      </div>
    </div>
  );
});

MessageItem.displayName = 'MessageItem';

function formatMessageTime(timestamp: number): string {
  const date = new Date(timestamp * 1000);
  const now = new Date();
  const isToday = date.toDateString() === now.toDateString();

  if (isToday) {
    return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  }
  return date.toLocaleString([], {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  });
}

const messageRoleStyles = {
  user: {
    backgroundColor: theme.colors.message.user.bg,
    color: theme.colors.message.user.text,
    boxShadow: theme.shadows.md,
  },
  assistant: {
    backgroundColor: theme.colors.message.assistant.bg,
    color: theme.colors.message.assistant.text,
    border: `1px solid ${theme.colors.message.assistant.border}`,
    boxShadow: theme.shadows.sm,
  },
};

const styles = {
  container: {
    flex: 1,
    overflowY: 'auto' as const,
    padding: `${theme.spacing.xl} ${theme.spacing.lg}`,
    backgroundColor: theme.colors.background.primary,
    scrollBehavior: 'smooth' as const,
  },
  empty: {
    display: 'flex',
    flexDirection: 'column' as const,
    alignItems: 'center',
    justifyContent: 'center',
    height: '100%',
    color: theme.colors.text.tertiary,
    gap: theme.spacing.md,
  },
  emptyIcon: {
    color: theme.colors.text.muted,
    opacity: 0.4,
  },
  emptyText: {
    fontSize: theme.typography.fontSize.lg,
    fontWeight: theme.typography.fontWeight.medium,
    color: theme.colors.text.secondary,
    margin: 0,
  },
  emptySubtext: {
    fontSize: theme.typography.fontSize.sm,
    color: theme.colors.text.tertiary,
    margin: 0,
  },
  message: {
    marginBottom: theme.spacing.xl,
    display: 'flex',
    flexDirection: 'column' as const,
    maxWidth: '85%',
    animation: 'fadeInUp 0.3s ease-out',
  },
  messageUser: {
    alignSelf: 'flex-end',
    alignItems: 'flex-end',
  },
  messageAssistant: {
    alignSelf: 'flex-start',
    alignItems: 'flex-start',
  },
  messageHeader: {
    display: 'flex',
    alignItems: 'flex-start',
    gap: theme.spacing.sm,
    position: 'relative' as const,
  },
  messageContent: {
    padding: `${theme.spacing.md} ${theme.spacing.lg}`,
    borderRadius: theme.borderRadius.lg,
    fontSize: theme.typography.fontSize.base,
    lineHeight: theme.typography.lineHeight.relaxed,
    whiteSpace: 'pre-wrap' as const,
    wordBreak: 'break-word' as const,
    flex: 1,
    transition: `box-shadow ${theme.transitions.normal}`,
  },
  messageActions: {
    paddingTop: theme.spacing.sm,
    opacity: 0,
    transition: `opacity ${theme.transitions.normal}`,
  },
  messageTime: {
    fontSize: theme.typography.fontSize.xs,
    color: theme.colors.text.tertiary,
    marginTop: theme.spacing.xs,
    paddingLeft: theme.spacing.md,
    paddingRight: theme.spacing.md,
    fontWeight: theme.typography.fontWeight.normal,
  },
  cursor: {
    display: 'inline-block',
    animation: 'blink 1s infinite',
    marginLeft: '2px',
    color: theme.colors.accent.primary,
  },
  loadingContainer: {
    display: 'flex',
    alignItems: 'center',
    gap: theme.spacing.sm,
    color: theme.colors.text.tertiary,
  },
  loaderIcon: {
    animation: 'spin 1s linear infinite',
  },
  loadingText: {
    fontSize: theme.typography.fontSize.sm,
    fontStyle: 'italic' as const,
  },
} as const;

// Add CSS animations
if (!document.getElementById('chat-message-animations')) {
  const styleSheet = document.createElement('style');
  styleSheet.id = 'chat-message-animations';
  styleSheet.textContent = `
    @keyframes fadeInUp {
      from {
        opacity: 0;
        transform: translateY(10px);
      }
      to {
        opacity: 1;
        transform: translateY(0);
      }
    }
    
    @keyframes blink {
      0%, 50% { opacity: 1; }
      51%, 100% { opacity: 0; }
    }
    
    @keyframes spin {
      to { transform: rotate(360deg); }
    }
    
    .message-item:hover .message-content {
      box-shadow: ${theme.shadows.lg} !important;
    }
    
    .message-item:hover .message-actions {
      opacity: 1 !important;
    }
    
    .message-streaming .message-content {
      animation: pulse 2s ease-in-out infinite;
    }
    
    @keyframes pulse {
      0%, 100% { opacity: 1; }
      50% { opacity: 0.95; }
    }
    
    /* Smooth scrollbar */
    .chat-message-list::-webkit-scrollbar {
      width: 8px;
    }
    
    .chat-message-list::-webkit-scrollbar-track {
      background: transparent;
    }
    
    .chat-message-list::-webkit-scrollbar-thumb {
      background: ${theme.colors.border.primary};
      border-radius: 4px;
    }
    
    .chat-message-list::-webkit-scrollbar-thumb:hover {
      background: ${theme.colors.border.secondary};
    }
  `;
  document.head.appendChild(styleSheet);
}
