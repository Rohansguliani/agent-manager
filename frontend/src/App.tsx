import { useState, useEffect, useCallback } from 'react';
import { ErrorBoundary } from './ErrorBoundary';
import { ChatLayout } from './components/ChatLayout';
import { ToastProvider } from './components/ToastProvider';
import { ConnectionStatus } from './components/ConnectionStatus';
import { useChat } from './hooks/useChat';
import { useStreamingChat } from './hooks/useStreamingChat';
import { Message } from './api';
import toast from 'react-hot-toast';

function App() {
  const {
    conversations,
    messages,
    loading: chatLoading,
    error: chatError,
    selectConversation,
    createConversation,
    deleteConversation,
    addMessage,
  } = useChat();

  const {
    streamingContent,
    sending,
    error: streamingError,
    sendMessage: sendStreamingMessage,
    clearStream,
  } = useStreamingChat();

  const [selectedConversationId, setSelectedConversationId] = useState<string | null>(null);

  const handleNewConversation = async () => {
    try {
      const newId = await createConversation();
      setSelectedConversationId(newId);
      await selectConversation(newId);
      clearStream();
    } catch (err) {
      console.error('Failed to create conversation:', err);
    }
  };

  // Keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Cmd/Ctrl+K for new chat
      if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
        e.preventDefault();
        handleNewConversation();
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // Auto-select first conversation if none selected
  useEffect(() => {
    if (!selectedConversationId && conversations.length > 0) {
      const firstId = conversations[0].id;
      // Use setTimeout to avoid synchronous setState in effect
      setTimeout(() => {
        setSelectedConversationId(firstId);
        selectConversation(firstId);
      }, 0);
    }
  }, [conversations, selectedConversationId, selectConversation]);

  // Handle streaming completion - save assistant message
  useEffect(() => {
    if (!sending && streamingContent && selectedConversationId) {
      // The backend should have already saved the message, but we refresh to be sure
      if (selectedConversationId) {
        selectConversation(selectedConversationId);
      }
      clearStream();
    }
  }, [sending, streamingContent, selectedConversationId, selectConversation, clearStream]);

  const handleSelectConversation = useCallback(async (id: string) => {
    setSelectedConversationId(id);
    await selectConversation(id);
    clearStream();
  }, [selectConversation, clearStream]);

  const handleDeleteConversation = useCallback(async (id: string) => {
    try {
      await deleteConversation(id);
      if (selectedConversationId === id) {
        setSelectedConversationId(null);
        if (conversations.length > 1) {
          const remaining = conversations.filter((c) => c.id !== id);
          if (remaining.length > 0) {
            setSelectedConversationId(remaining[0].id);
            await selectConversation(remaining[0].id);
          }
        }
      }
      clearStream();
      toast.success('Conversation deleted');
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : 'Failed to delete conversation';
      toast.error(errorMsg);
    }
  }, [deleteConversation, selectedConversationId, conversations, selectConversation, clearStream]);

  const handleSendMessage = useCallback(async (message: string) => {
    if (!message.trim()) return;

    let conversationId = selectedConversationId;

    // Create conversation if none selected
    if (!conversationId) {
      try {
        conversationId = await createConversation();
        setSelectedConversationId(conversationId);
        await selectConversation(conversationId);
        toast.success('New conversation created');
      } catch (err) {
        const errorMsg = err instanceof Error ? err.message : 'Failed to create conversation';
        toast.error(errorMsg);
        console.error('Failed to create conversation:', err);
        return;
      }
    }

    // Add user message optimistically
    const userMessage: Message = {
      id: `temp-${Date.now()}`,
      conversation_id: conversationId,
      role: 'user',
      content: message,
      created_at: Math.floor(Date.now() / 1000),
    };
    addMessage(userMessage);

    try {
      // Send streaming message
      await sendStreamingMessage(message, conversationId);

      // Refresh messages after streaming completes
      if (conversationId) {
        setTimeout(async () => {
          await selectConversation(conversationId!);
        }, 500);
      }
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : 'Failed to send message';
      toast.error(errorMsg);
    }
  }, [selectedConversationId, createConversation, selectConversation, addMessage, sendStreamingMessage]);

  const handleRegenerate = useCallback(async (messageId: string) => {
    // Find the message and regenerate from the previous user message
    const messageIndex = messages.findIndex((m) => m.id === messageId);
    if (messageIndex > 0 && messages[messageIndex - 1].role === 'user') {
      const userMessage = messages[messageIndex - 1].content;
      try {
        await sendStreamingMessage(userMessage, selectedConversationId || null);
      } catch (err) {
        const errorMsg = err instanceof Error ? err.message : 'Failed to regenerate response';
        toast.error(errorMsg);
      }
    }
  }, [messages, sendStreamingMessage, selectedConversationId]);

  // Display error if any (with auto-dismiss)
  const displayError = chatError || streamingError;

  // Show toast notifications for errors
  useEffect(() => {
    if (displayError) {
      toast.error(displayError, {
        duration: 5000,
      });
    }
  }, [displayError]);

  return (
    <ErrorBoundary>
      <ToastProvider />
      <ConnectionStatus />
      <ChatLayout
        conversations={conversations}
        selectedConversationId={selectedConversationId}
        messages={messages}
        streamingContent={streamingContent}
        loading={chatLoading}
        onSelectConversation={handleSelectConversation}
        onNewConversation={handleNewConversation}
        onDeleteConversation={handleDeleteConversation}
        onSendMessage={handleSendMessage}
        onRegenerate={handleRegenerate}
        sending={sending}
      />
    </ErrorBoundary>
  );
}

export default App;
