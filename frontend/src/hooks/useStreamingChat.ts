import { useState, useCallback, useRef } from 'react';
import { QueryRequest } from '../api';

export interface UseStreamingChatReturn {
  streamingContent: string;
  sending: boolean;
  error: string | null;
  sendMessage: (message: string, conversationId: string | null) => Promise<void>;
  clearStream: () => void;
}

/**
 * Custom hook for streaming chat messages using Server-Sent Events (SSE)
 * 
 * Handles SSE parsing and state management for streaming responses in chat conversations
 */
export function useStreamingChat(): UseStreamingChatReturn {
  const [streamingContent, setStreamingContent] = useState<string>('');
  const [sending, setSending] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);
  const abortControllerRef = useRef<AbortController | null>(null);

  const clearStream = useCallback(() => {
    setStreamingContent('');
    setError(null);
    if (abortControllerRef.current) {
      abortControllerRef.current.abort();
      abortControllerRef.current = null;
    }
  }, []);

  const sendMessage = useCallback(
    async (message: string, conversationId: string | null) => {
      if (!message.trim()) return;

      setSending(true);
      setError(null);
      setStreamingContent('');

      // Abort any existing request
      if (abortControllerRef.current) {
        abortControllerRef.current.abort();
      }

      const abortController = new AbortController();
      abortControllerRef.current = abortController;

      try {
        const request: QueryRequest = {
          query: message,
          conversation_id: conversationId || undefined,
        };

        const API_URL = import.meta.env.VITE_API_URL || 'http://localhost:8080';
        const fetchResponse = await fetch(`${API_URL}/api/query/stream`, {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify(request),
          signal: abortController.signal,
        });

        if (!fetchResponse.ok) {
          throw new Error(`HTTP error! status: ${fetchResponse.status}`);
        }

        const reader = fetchResponse.body?.getReader();
        const decoder = new TextDecoder();

        if (!reader) {
          throw new Error('No response body');
        }

        let buffer = '';
        while (true) {
          const { done, value } = await reader.read();
          if (done) break;

          buffer += decoder.decode(value, { stream: true });

          // SSE format: "data: <content>\n\n" or "data: <content>\n"
          const parts = buffer.split('\n\n');
          buffer = parts.pop() || '';

          for (const part of parts) {
            const lines = part.split('\n');
            for (const line of lines) {
              if (line.startsWith('data: ')) {
                const data = line.slice(6);
                if (data === '[DONE]') {
                  setSending(false);
                  return;
                } else if (data.startsWith('[ERROR]')) {
                  const errorMessage = data.slice(8);
                  setError(errorMessage);
                  setSending(false);
                  return;
                } else {
                  setStreamingContent((prev) => prev + data);
                }
              }
            }
          }
        }

        setSending(false);
      } catch (err) {
        if (err instanceof Error && err.name === 'AbortError') {
          // Request was aborted, don't set error
          return;
        }
        const errorMessage = err instanceof Error ? err.message : 'Failed to stream response';
        setError(errorMessage);
        setSending(false);
      }
    },
    []
  );

  return {
    streamingContent,
    sending,
    error,
    sendMessage,
    clearStream,
  };
}

