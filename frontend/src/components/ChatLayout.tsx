import { memo } from 'react';
import { ChatSidebar, Conversation } from './ChatSidebar';
import { ChatMessageList, Message } from './ChatMessageList';
import { ChatInput } from './ChatInput';
import { theme } from '../styles/theme';

interface ChatLayoutProps {
  conversations: Conversation[];
  selectedConversationId: string | null;
  messages: Message[];
  streamingContent?: string;
  loading?: boolean;
  onSelectConversation: (id: string) => void;
  onNewConversation: () => void;
  onDeleteConversation: (id: string) => void;
  onSendMessage: (message: string) => void;
  onRegenerate?: (messageId: string) => void;
  sending?: boolean;
}

export const ChatLayout = memo<ChatLayoutProps>(({
  conversations,
  selectedConversationId,
  messages,
  streamingContent,
  loading = false,
  onSelectConversation,
  onNewConversation,
  onDeleteConversation,
  onSendMessage,
  onRegenerate,
  sending = false,
}) => {
  return (
    <div style={styles.container} role="main">
      <ChatSidebar
        conversations={conversations}
        selectedConversationId={selectedConversationId}
        onSelectConversation={onSelectConversation}
        onNewConversation={onNewConversation}
        onDeleteConversation={onDeleteConversation}
        loading={loading}
      />
      <div style={styles.chatArea} role="region" aria-label="Chat area">
        <ChatMessageList
          messages={messages}
          streamingContent={streamingContent}
          loading={sending && !streamingContent}
          onRegenerate={onRegenerate}
        />
        <ChatInput
          onSend={onSendMessage}
          disabled={sending || !selectedConversationId}
          placeholder={
            !selectedConversationId
              ? 'Select a conversation or create a new one'
              : 'Type your message...'
          }
        />
      </div>
    </div>
  );
});

ChatLayout.displayName = 'ChatLayout';

const styles = {
  container: {
    display: 'flex',
    height: '100vh',
    width: '100vw',
    backgroundColor: theme.colors.background.primary,
    color: theme.colors.text.primary,
    fontFamily: theme.typography.fontFamily.primary,
    overflow: 'hidden',
  },
  chatArea: {
    flex: 1,
    display: 'flex',
    flexDirection: 'column' as const,
    overflow: 'hidden',
    backgroundColor: theme.colors.background.primary,
  },
} as const;
