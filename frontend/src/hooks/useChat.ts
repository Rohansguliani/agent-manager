import { useState, useEffect, useCallback } from 'react';
import { api, Conversation, Message } from '../api';

export interface UseChatReturn {
  conversations: Conversation[];
  selectedConversation: Conversation | null;
  messages: Message[];
  loading: boolean;
  error: string | null;
  selectConversation: (id: string) => Promise<void>;
  createConversation: (title?: string) => Promise<string>;
  deleteConversation: (id: string) => Promise<void>;
  updateTitle: (id: string, title: string) => Promise<void>;
  refreshConversations: () => Promise<void>;
  addMessage: (message: Message) => void;
  updateMessages: (messages: Message[]) => void;
}

export function useChat(): UseChatReturn {
  const [conversations, setConversations] = useState<Conversation[]>([]);
  const [selectedConversation, setSelectedConversation] = useState<Conversation | null>(null);
  const [messages, setMessages] = useState<Message[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const refreshConversations = useCallback(async () => {
    try {
      setError(null);
      const convs = await api.listConversations();
      setConversations(convs);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load conversations');
    }
  }, []);

  const selectConversation = useCallback(async (id: string) => {
    try {
      setError(null);
      setLoading(true);
      const convWithMessages = await api.getConversation(id);
      setSelectedConversation(convWithMessages.conversation);
      setMessages(convWithMessages.messages);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load conversation');
      setSelectedConversation(null);
      setMessages([]);
    } finally {
      setLoading(false);
    }
  }, []);

  const createConversation = useCallback(async (title?: string): Promise<string> => {
    try {
      setError(null);
      const conv = await api.createConversation({ title });
      await refreshConversations();
      return conv.id;
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : 'Failed to create conversation';
      setError(errorMsg);
      throw new Error(errorMsg);
    }
  }, [refreshConversations]);

  const deleteConversation = useCallback(async (id: string) => {
    try {
      setError(null);
      await api.deleteConversation(id);
      if (selectedConversation?.id === id) {
        setSelectedConversation(null);
        setMessages([]);
      }
      await refreshConversations();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to delete conversation');
    }
  }, [selectedConversation, refreshConversations]);

  const updateTitle = useCallback(async (id: string, title: string) => {
    try {
      setError(null);
      const updated = await api.updateConversationTitle(id, title);
      setConversations((prev) =>
        prev.map((c) => (c.id === id ? updated : c))
      );
      if (selectedConversation?.id === id) {
        setSelectedConversation(updated);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to update title');
    }
  }, [selectedConversation]);

  const addMessage = useCallback((message: Message) => {
    setMessages((prev) => [...prev, message]);
  }, []);

  const updateMessages = useCallback((newMessages: Message[]) => {
    setMessages(newMessages);
  }, []);

  // Load conversations on mount
  useEffect(() => {
    refreshConversations().finally(() => setLoading(false));
  }, [refreshConversations]);

  return {
    conversations,
    selectedConversation,
    messages,
    loading,
    error,
    selectConversation,
    createConversation,
    deleteConversation,
    updateTitle,
    refreshConversations,
    addMessage,
    updateMessages,
  };
}

