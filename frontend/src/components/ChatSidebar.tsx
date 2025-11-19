import { memo } from 'react';
import { PlusIcon, XIcon, ChatIcon } from './icons';
import { theme } from '../styles/theme';

export interface Conversation {
  id: string;
  title: string;
  created_at: number;
  updated_at: number;
}

interface ChatSidebarProps {
  conversations: Conversation[];
  selectedConversationId: string | null;
  onSelectConversation: (id: string) => void;
  onNewConversation: () => void;
  onDeleteConversation: (id: string) => void;
  loading?: boolean;
}

export const ChatSidebar = memo<ChatSidebarProps>(({
  conversations,
  selectedConversationId,
  onSelectConversation,
  onNewConversation,
  onDeleteConversation,
  loading = false,
}) => {
  return (
    <aside style={styles.container} aria-label="Conversations sidebar">
      <div style={styles.header}>
        <h2 style={styles.title}>Conversations</h2>
        <button
          onClick={onNewConversation}
          className="chat-sidebar-new-button"
          style={styles.newButton}
          title="New Chat"
          aria-label="Create new conversation"
        >
          <PlusIcon size={18} />
        </button>
      </div>

      {loading ? (
        <div style={styles.loading}>
          <div style={styles.loadingSpinner} />
          <span style={styles.loadingText}>Loading...</span>
        </div>
      ) : conversations.length === 0 ? (
        <div style={styles.empty}>
          <ChatIcon size={48} style={styles.emptyIcon} />
          <p style={styles.emptyText}>No conversations yet</p>
          <p style={styles.emptySubtext}>Start a new chat to begin</p>
        </div>
      ) : (
        <div style={styles.conversationList}>
          {conversations.map((conv) => (
            <div
              key={conv.id}
              className="chat-sidebar-conversation-item"
              style={{
                ...styles.conversationItem,
                ...(selectedConversationId === conv.id
                  ? styles.conversationItemSelected
                  : {}),
              }}
              onClick={() => onSelectConversation(conv.id)}
              onKeyDown={(e) => {
                if (e.key === 'Enter' || e.key === ' ') {
                  e.preventDefault();
                  onSelectConversation(conv.id);
                }
              }}
              role="button"
              tabIndex={0}
              aria-label={`Select conversation: ${conv.title}`}
              aria-selected={selectedConversationId === conv.id}
            >
              <div style={styles.conversationContent}>
                <div style={styles.conversationTitle}>{conv.title}</div>
                <div style={styles.conversationTime}>
                  {formatTime(conv.updated_at)}
                </div>
              </div>
              <button
                onClick={(e) => {
                  e.stopPropagation();
                  onDeleteConversation(conv.id);
                }}
                className="chat-sidebar-delete-button"
                style={styles.deleteButton}
                title="Delete conversation"
                aria-label={`Delete conversation: ${conv.title}`}
              >
                <XIcon size={16} />
              </button>
            </div>
          ))}
        </div>
      )}
    </aside>
  );
});

ChatSidebar.displayName = 'ChatSidebar';

function formatTime(timestamp: number): string {
  const date = new Date(timestamp * 1000);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffMins = Math.floor(diffMs / 60000);
  const diffHours = Math.floor(diffMs / 3600000);
  const diffDays = Math.floor(diffMs / 86400000);

  if (diffMins < 1) return 'Just now';
  if (diffMins < 60) return `${diffMins}m ago`;
  if (diffHours < 24) return `${diffHours}h ago`;
  if (diffDays < 7) return `${diffDays}d ago`;

  return date.toLocaleDateString();
}

const styles = {
  container: {
    width: '280px',
    height: '100vh',
    backgroundColor: theme.colors.background.secondary,
    borderRight: `1px solid ${theme.colors.border.primary}`,
    display: 'flex',
    flexDirection: 'column' as const,
    color: theme.colors.text.secondary,
    position: 'relative' as const,
  },
  header: {
    padding: theme.spacing.lg,
    borderBottom: `1px solid ${theme.colors.border.primary}`,
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    backgroundColor: theme.colors.background.secondary,
  },
  title: {
    margin: 0,
    fontSize: theme.typography.fontSize.lg,
    fontWeight: theme.typography.fontWeight.semibold,
    color: theme.colors.text.primary,
    letterSpacing: '-0.01em',
  },
  newButton: {
    width: '36px',
    height: '36px',
    borderRadius: theme.borderRadius.md,
    border: `1px solid ${theme.colors.border.primary}`,
    backgroundColor: theme.colors.background.elevated,
    color: theme.colors.text.secondary,
    cursor: 'pointer',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    transition: `all ${theme.transitions.normal}`,
    boxShadow: theme.shadows.sm,
  },
  conversationList: {
    flex: 1,
    overflowY: 'auto' as const,
    padding: theme.spacing.sm,
    scrollbarWidth: 'thin' as const,
  },
  conversationItem: {
    padding: theme.spacing.md,
    marginBottom: theme.spacing.xs,
    borderRadius: theme.borderRadius.md,
    cursor: 'pointer',
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    transition: `all ${theme.transitions.normal}`,
    backgroundColor: 'transparent',
    position: 'relative' as const,
  },
  conversationItemSelected: {
    backgroundColor: theme.colors.background.elevated,
    boxShadow: theme.shadows.sm,
  },
  conversationContent: {
    flex: 1,
    minWidth: 0,
  },
  conversationTitle: {
    fontSize: theme.typography.fontSize.base,
    fontWeight: theme.typography.fontWeight.medium,
    color: theme.colors.text.primary,
    whiteSpace: 'nowrap' as const,
    overflow: 'hidden',
    textOverflow: 'ellipsis',
    marginBottom: theme.spacing.xs,
    lineHeight: theme.typography.lineHeight.tight,
  },
  conversationTime: {
    fontSize: theme.typography.fontSize.xs,
    color: theme.colors.text.tertiary,
    lineHeight: theme.typography.lineHeight.normal,
  },
  deleteButton: {
    width: '28px',
    height: '28px',
    borderRadius: theme.borderRadius.sm,
    border: 'none',
    backgroundColor: 'transparent',
    color: theme.colors.text.tertiary,
    cursor: 'pointer',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    opacity: 0,
    transition: `all ${theme.transitions.normal}`,
    padding: 0,
    flexShrink: 0,
  },
  loading: {
    padding: theme.spacing['2xl'],
    display: 'flex',
    flexDirection: 'column' as const,
    alignItems: 'center',
    gap: theme.spacing.md,
    color: theme.colors.text.tertiary,
  },
  loadingSpinner: {
    width: '24px',
    height: '24px',
    border: `3px solid ${theme.colors.border.primary}`,
    borderTopColor: theme.colors.accent.primary,
    borderRadius: '50%',
    animation: 'spin 1s linear infinite',
  },
  loadingText: {
    fontSize: theme.typography.fontSize.sm,
    color: theme.colors.text.tertiary,
  },
  empty: {
    padding: theme.spacing['2xl'],
    display: 'flex',
    flexDirection: 'column' as const,
    alignItems: 'center',
    textAlign: 'center' as const,
    color: theme.colors.text.tertiary,
    gap: theme.spacing.md,
  },
  emptyIcon: {
    color: theme.colors.text.muted,
    opacity: 0.5,
  },
  emptyText: {
    fontSize: theme.typography.fontSize.base,
    fontWeight: theme.typography.fontWeight.medium,
    color: theme.colors.text.secondary,
    margin: 0,
  },
  emptySubtext: {
    fontSize: theme.typography.fontSize.sm,
    color: theme.colors.text.tertiary,
    margin: 0,
  },
} as const;

// Add CSS animations and hover effects
if (!document.getElementById('chat-sidebar-styles')) {
  const styleSheet = document.createElement('style');
  styleSheet.id = 'chat-sidebar-styles';
  styleSheet.textContent = `
    @keyframes spin {
      to { transform: rotate(360deg); }
    }
    
    .chat-sidebar-conversation-item:hover {
      background-color: ${theme.colors.interactive.hover} !important;
    }
    
    .chat-sidebar-conversation-item:hover .chat-sidebar-delete-button {
      opacity: 1 !important;
    }
    
    .chat-sidebar-conversation-item:active {
      background-color: ${theme.colors.interactive.active} !important;
    }
    
    .chat-sidebar-new-button:hover {
      background-color: ${theme.colors.interactive.hover} !important;
      border-color: ${theme.colors.border.accent} !important;
      color: ${theme.colors.accent.primary} !important;
      transform: scale(1.05);
    }
    
    .chat-sidebar-new-button:active {
      transform: scale(0.95);
    }
    
    .chat-sidebar-delete-button:hover {
      background-color: ${theme.colors.status.error}20 !important;
      color: ${theme.colors.status.error} !important;
    }
    
    .chat-sidebar-conversation-item[aria-selected="true"]::before {
      content: '';
      position: absolute;
      left: 0;
      top: 50%;
      transform: translateY(-50%);
      width: 3px;
      height: 60%;
      background-color: ${theme.colors.accent.primary};
      border-radius: 0 2px 2px 0;
    }
    
    /* Smooth scrollbar */
    .chat-sidebar-conversation-list::-webkit-scrollbar {
      width: 6px;
    }
    
    .chat-sidebar-conversation-list::-webkit-scrollbar-track {
      background: transparent;
    }
    
    .chat-sidebar-conversation-list::-webkit-scrollbar-thumb {
      background: ${theme.colors.border.primary};
      border-radius: 3px;
    }
    
    .chat-sidebar-conversation-list::-webkit-scrollbar-thumb:hover {
      background: ${theme.colors.border.secondary};
    }
  `;
  document.head.appendChild(styleSheet);
}
