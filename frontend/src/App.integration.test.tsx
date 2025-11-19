import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import App from './App';
import { api } from './api';
import '@testing-library/jest-dom';

// Mock the API
vi.mock('./api', () => ({
  api: {
    listConversations: vi.fn(),
    createConversation: vi.fn(),
    getConversation: vi.fn(),
    deleteConversation: vi.fn(),
    updateConversationTitle: vi.fn(),
  },
}));

// Mock fetch for streaming
const mockFetch = vi.fn();
(globalThis as any).fetch = mockFetch;

describe('App Integration', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockFetch.mockClear();
  });

  it('renders the chat layout', async () => {
    (api.listConversations as any).mockResolvedValue([]);

    render(<App />);

    await waitFor(() => {
      expect(screen.getByText('Conversations')).toBeInTheDocument();
    });
  });

  it('displays empty state when no conversations', async () => {
    (api.listConversations as any).mockResolvedValue([]);

    render(<App />);

    await waitFor(() => {
      expect(screen.getByText('No conversations yet')).toBeInTheDocument();
    });
  });

  it('loads and displays conversations', async () => {
    const mockConversations = [
      {
        id: '1',
        title: 'Test Conversation',
        created_at: Math.floor(Date.now() / 1000),
        updated_at: Math.floor(Date.now() / 1000),
      },
    ];

    (api.listConversations as any).mockResolvedValue(mockConversations);

    render(<App />);

    await waitFor(() => {
      expect(screen.getByText('Test Conversation')).toBeInTheDocument();
    }, { timeout: 3000 });

    expect(api.listConversations).toHaveBeenCalled();
  });
});

