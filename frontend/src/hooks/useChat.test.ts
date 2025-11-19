import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, waitFor } from '@testing-library/react';
import { useChat } from './useChat';
import { api } from '../api';

// Mock the API
vi.mock('../api', () => ({
  api: {
    listConversations: vi.fn(),
    createConversation: vi.fn(),
    getConversation: vi.fn(),
    deleteConversation: vi.fn(),
    updateConversationTitle: vi.fn(),
  },
}));

describe('useChat', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('loads conversations on mount', async () => {
    const mockConversations = [
      {
        id: '1',
        title: 'Test Conversation',
        created_at: Math.floor(Date.now() / 1000),
        updated_at: Math.floor(Date.now() / 1000),
      },
    ];

    (api.listConversations as any).mockResolvedValue(mockConversations);

    const { result } = renderHook(() => useChat());

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(api.listConversations).toHaveBeenCalled();
    expect(result.current.conversations).toEqual(mockConversations);
  });

  it('selects a conversation', async () => {
    const mockConversation = {
      id: '1',
      title: 'Test Conversation',
      created_at: Math.floor(Date.now() / 1000),
      updated_at: Math.floor(Date.now() / 1000),
    };

    const mockMessages = [
      {
        id: '1',
        conversation_id: '1',
        role: 'user' as const,
        content: 'Hello',
        created_at: Math.floor(Date.now() / 1000),
      },
    ];

    (api.listConversations as any).mockResolvedValue([mockConversation]);
    (api.getConversation as any).mockResolvedValue({
      conversation: mockConversation,
      messages: mockMessages,
    });

    const { result } = renderHook(() => useChat());

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    await result.current.selectConversation('1');

    await waitFor(() => {
      expect(result.current.selectedConversation).not.toBeNull();
    });

    expect(api.getConversation).toHaveBeenCalledWith('1');
    expect(result.current.selectedConversation).toEqual(mockConversation);
    expect(result.current.messages).toEqual(mockMessages);
  });

  it('creates a conversation', async () => {
    const mockConversation = {
      id: 'new-id',
      title: 'New Conversation',
      created_at: Math.floor(Date.now() / 1000),
      updated_at: Math.floor(Date.now() / 1000),
    };

    (api.listConversations as any).mockResolvedValue([]);
    (api.createConversation as any).mockResolvedValue(mockConversation);
    (api.listConversations as any).mockResolvedValueOnce([mockConversation]);

    const { result } = renderHook(() => useChat());

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    const newId = await result.current.createConversation();

    expect(api.createConversation).toHaveBeenCalled();
    expect(newId).toBe('new-id');
  });

  it('deletes a conversation', async () => {
    const mockConversations = [
      {
        id: '1',
        title: 'Test Conversation',
        created_at: Math.floor(Date.now() / 1000),
        updated_at: Math.floor(Date.now() / 1000),
      },
    ];

    (api.listConversations as any).mockResolvedValue(mockConversations);
    (api.deleteConversation as any).mockResolvedValue(undefined);
    (api.listConversations as any).mockResolvedValueOnce([]);

    const { result } = renderHook(() => useChat());

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    await result.current.deleteConversation('1');

    expect(api.deleteConversation).toHaveBeenCalledWith('1');
  });

  it('updates conversation title', async () => {
    const mockConversation = {
      id: '1',
      title: 'Old Title',
      created_at: Math.floor(Date.now() / 1000),
      updated_at: Math.floor(Date.now() / 1000),
    };

    const updatedConversation = {
      ...mockConversation,
      title: 'New Title',
    };

    (api.listConversations as any).mockResolvedValue([mockConversation]);
    (api.updateConversationTitle as any).mockResolvedValue(updatedConversation);

    const { result } = renderHook(() => useChat());

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    await result.current.updateTitle('1', 'New Title');

    await waitFor(() => {
      expect(result.current.conversations[0]?.title).toBe('New Title');
    });

    expect(api.updateConversationTitle).toHaveBeenCalledWith('1', 'New Title');
  });

  it('handles errors', async () => {
    const error = new Error('API Error');
    (api.listConversations as any).mockRejectedValue(error);

    const { result } = renderHook(() => useChat());

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(result.current.error).toBe('API Error');
  });

  it('adds a message', async () => {
    (api.listConversations as any).mockResolvedValue([]);

    const { result } = renderHook(() => useChat());

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    const message = {
      id: '1',
      conversation_id: '1',
      role: 'user' as const,
      content: 'Test',
      created_at: Math.floor(Date.now() / 1000),
    };

    result.current.addMessage(message);

    await waitFor(() => {
      expect(result.current.messages.length).toBeGreaterThan(0);
    });

    expect(result.current.messages).toContainEqual(message);
  });

  it('updates messages', async () => {
    (api.listConversations as any).mockResolvedValue([]);

    const { result } = renderHook(() => useChat());

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    const messages = [
      {
        id: '1',
        conversation_id: '1',
        role: 'user' as const,
        content: 'Test',
        created_at: Math.floor(Date.now() / 1000),
      },
    ];

    result.current.updateMessages(messages);

    await waitFor(() => {
      expect(result.current.messages.length).toBe(1);
    });

    expect(result.current.messages).toEqual(messages);
  });
});

