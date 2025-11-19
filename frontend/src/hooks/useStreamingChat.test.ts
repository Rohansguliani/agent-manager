import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, waitFor } from '@testing-library/react';
import { useStreamingChat } from './useStreamingChat';

// Mock fetch globally
const mockFetch = vi.fn();
(globalThis as any).fetch = mockFetch;

describe('useStreamingChat', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockFetch.mockClear();
  });

  it('initializes with empty state', () => {
    const { result } = renderHook(() => useStreamingChat());

    expect(result.current.streamingContent).toBe('');
    expect(result.current.sending).toBe(false);
    expect(result.current.error).toBe(null);
  });

  it('sends a message and streams response', async () => {
    const mockReader = {
      read: vi.fn()
        .mockResolvedValueOnce({
          done: false,
          value: new TextEncoder().encode('data: Hello\n\n'),
        })
        .mockResolvedValueOnce({
          done: false,
          value: new TextEncoder().encode('data: World\n\n'),
        })
        .mockResolvedValueOnce({
          done: true,
          value: undefined,
        }),
    };

    mockFetch.mockResolvedValueOnce({
      ok: true,
      body: {
        getReader: () => mockReader,
      },
    });

    const { result } = renderHook(() => useStreamingChat());

    result.current.sendMessage('test', 'conv1');

    await waitFor(() => {
      expect(result.current.sending).toBe(false);
    });

    expect(mockFetch).toHaveBeenCalledWith(
      expect.stringContaining('/api/query/stream'),
      expect.objectContaining({
        method: 'POST',
        body: JSON.stringify({
          query: 'test',
          conversation_id: 'conv1',
        }),
      })
    );
  });

  it('handles errors', async () => {
    mockFetch.mockResolvedValueOnce({
      ok: false,
      status: 500,
    });

    const { result } = renderHook(() => useStreamingChat());

    await result.current.sendMessage('test', null);

    await waitFor(() => {
      expect(result.current.sending).toBe(false);
    });

    expect(result.current.error).toBeTruthy();
  });

  it('clears stream', () => {
    const { result } = renderHook(() => useStreamingChat());

    // Set some state first
    result.current.clearStream();

    expect(result.current.streamingContent).toBe('');
    expect(result.current.error).toBe(null);
  });

  it('handles [DONE] signal', async () => {
    const mockReader = {
      read: vi.fn().mockResolvedValueOnce({
        done: false,
        value: new TextEncoder().encode('data: [DONE]\n\n'),
      }),
    };

    mockFetch.mockResolvedValueOnce({
      ok: true,
      body: {
        getReader: () => mockReader,
      },
    });

    const { result } = renderHook(() => useStreamingChat());

    result.current.sendMessage('test', null);

    await waitFor(() => {
      expect(result.current.sending).toBe(false);
    });
  });

  it('handles [ERROR] signal', async () => {
    const mockReader = {
      read: vi.fn()
        .mockResolvedValueOnce({
          done: false,
          value: new TextEncoder().encode('data: [ERROR]Test error\n\n'),
        })
        .mockResolvedValueOnce({
          done: true,
          value: undefined,
        }),
    };

    mockFetch.mockResolvedValueOnce({
      ok: true,
      body: {
        getReader: () => mockReader,
      },
    });

    const { result } = renderHook(() => useStreamingChat());

    result.current.sendMessage('test', null);

    await waitFor(() => {
      expect(result.current.sending).toBe(false);
    }, { timeout: 3000 });

    expect(result.current.error).toContain('error');
  });

  it('does not send empty message', async () => {
    const { result } = renderHook(() => useStreamingChat());

    await result.current.sendMessage('', null);

    expect(mockFetch).not.toHaveBeenCalled();
  });

  it('does not send when message is only whitespace', async () => {
    const { result } = renderHook(() => useStreamingChat());

    await result.current.sendMessage('   ', null);

    expect(mockFetch).not.toHaveBeenCalled();
  });
});

