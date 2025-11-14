/**
 * Custom hook for orchestration using Server-Sent Events (SSE)
 * 
 * Handles SSE parsing and state management for orchestration status updates.
 * Reuses the same SSE parsing pattern as useStreamingQuery.
 */

import { useState, useCallback } from 'react'
import { api, OrchestrationStatus } from '../api'

interface UseOrchestratorReturn {
  status: OrchestrationStatus | null
  running: boolean
  error: string | null
  runOrchestration: (goal?: string) => Promise<void>
  clearStatus: () => void
}

/**
 * Custom hook for executing orchestration workflows
 * 
 * @returns Object with status state and runOrchestration function
 */
export function useOrchestrator(): UseOrchestratorReturn {
  const [status, setStatus] = useState<OrchestrationStatus | null>(null)
  const [running, setRunning] = useState<boolean>(false)
  const [error, setError] = useState<string | null>(null)

  const clearStatus = useCallback(() => {
    setStatus(null)
    setError(null)
  }, [])

  const runOrchestration = useCallback(async (goal: string = '') => {
    setRunning(true)
    setError(null)
    setStatus(null)

    try {
      const fetchResponse = await api.orchestratePoem(goal)

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

        // SSE format: "data: <content>\n\n"
        const parts = buffer.split('\n\n')
        buffer = parts.pop() || ''

        for (const part of parts) {
          const lines = part.split('\n')
          for (const line of lines) {
            if (line.startsWith('data: ')) {
              const data = line.slice(6)
              if (data === '[DONE]') {
                setRunning(false)
                return
              } else if (data.startsWith('[ERROR]')) {
                const errorMessage = data.slice(8)
                setError(errorMessage)
                setRunning(false)
                return
                  } else {
                    try {
                      const statusUpdate: OrchestrationStatus = JSON.parse(data)
                      setStatus(statusUpdate)
                      if (statusUpdate.status === 'completed' || statusUpdate.status === 'error') {
                        setRunning(false)
                        if (statusUpdate.status === 'error') {
                          setError(statusUpdate.message)
                        }
                        // Don't return early - continue reading to get [DONE] signal
                        // This ensures the stream is properly closed
                        continue
                      }
                    } catch {
                      // Invalid JSON, skip
                    }
                  }
            }
          }
        }
      }

      setRunning(false)
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to run orchestration'
      setError(errorMessage)
      setRunning(false)
    }
  }, [])

  return {
    status,
    running,
    error,
    runOrchestration,
    clearStatus,
  }
}

