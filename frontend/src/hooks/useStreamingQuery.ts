/**
 * Custom hook for streaming queries using Server-Sent Events (SSE)
 * 
 * Handles SSE parsing and state management for streaming responses
 */

import { useState, useCallback } from 'react'

interface UseStreamingQueryOptions {
  apiUrl?: string
  onError?: (error: string) => void
}

interface UseStreamingQueryReturn {
  response: string
  loading: boolean
  error: string | null
  executeQuery: (query: string) => Promise<void>
  clearResponse: () => void
}

/**
 * Custom hook for executing streaming queries
 * 
 * @param options - Configuration options
 * @returns Object with response state and executeQuery function
 */
export function useStreamingQuery(
  options: UseStreamingQueryOptions = {}
): UseStreamingQueryReturn {
  const { apiUrl, onError } = options
  const [response, setResponse] = useState<string>('')
  const [loading, setLoading] = useState<boolean>(false)
  const [error, setError] = useState<string | null>(null)

  const clearResponse = useCallback(() => {
    setResponse('')
    setError(null)
  }, [])

  const executeQuery = useCallback(
    async (query: string) => {
      if (!query.trim()) return

      setLoading(true)
      setError(null)
      setResponse('')

      try {
        const baseUrl = apiUrl || import.meta.env.VITE_API_URL || 'http://localhost:8080'
        const fetchResponse = await fetch(`${baseUrl}/api/query/stream`, {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify({ query }),
        })

        if (!fetchResponse.ok) {
          throw new Error(`HTTP error! status: ${fetchResponse.status}`)
        }

        const reader = fetchResponse.body?.getReader()
        const decoder = new TextDecoder()

        if (!reader) {
          throw new Error('No response body')
        }

        let buffer = ''
        while (true) {
          const { done, value } = await reader.read()
          if (done) break

          buffer += decoder.decode(value, { stream: true })

          // SSE format: "data: <content>\n\n" or "data: <content>\n"
          const parts = buffer.split('\n\n')
          buffer = parts.pop() || ''

          for (const part of parts) {
            const lines = part.split('\n')
            for (const line of lines) {
              if (line.startsWith('data: ')) {
                const data = line.slice(6)
                if (data === '[DONE]') {
                  setLoading(false)
                  return
                } else if (data.startsWith('[ERROR]')) {
                  const errorMessage = data.slice(8)
                  setError(errorMessage)
                  if (onError) {
                    onError(errorMessage)
                  }
                  setLoading(false)
                  return
                } else {
                  setResponse(prev => prev + data)
                }
              }
            }
          }
        }

        setLoading(false)
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : 'Failed to stream response'
        setError(errorMessage)
        if (onError) {
          onError(errorMessage)
        }
        setLoading(false)
      }
    },
    [apiUrl, onError]
  )

  return {
    response,
    loading,
    error,
    executeQuery,
    clearResponse,
  }
}

