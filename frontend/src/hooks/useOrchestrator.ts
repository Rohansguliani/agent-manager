/**
 * Custom hook for orchestration using Server-Sent Events (SSE)
 * 
 * Handles SSE parsing and state management for orchestration status updates.
 * Reuses the same SSE parsing pattern as useStreamingQuery.
 */

import { useState, useCallback } from 'react'
import { api, OrchestrationStatus, OrchestrationEvent } from '../api'

interface UseOrchestratorReturn {
  stepStatuses: Record<string, OrchestrationStatus> // Map of step_id -> status for parallel tracking
  events: OrchestrationEvent[] // Phase 6.3: Structured events for live graph updates
  running: boolean
  error: string | null
  runOrchestration: (goal: string, useDynamic?: boolean) => Promise<void>
  clearStatus: () => void
}

/**
 * Custom hook for executing orchestration workflows
 * 
 * @returns Object with status state and runOrchestration function
 */
export function useOrchestrator(): UseOrchestratorReturn {
  // Use Record (object) instead of array to support parallel execution tracking
  const [stepStatuses, setStepStatuses] = useState<Record<string, OrchestrationStatus>>({})
  // Phase 6.3: Structured events for live graph updates
  const [events, setEvents] = useState<OrchestrationEvent[]>([])
  const [running, setRunning] = useState<boolean>(false)
  const [error, setError] = useState<string | null>(null)

  const clearStatus = useCallback(() => {
    setStepStatuses({})
    setEvents([])
    setError(null)
  }, [])

  const runOrchestration = useCallback(async (goal: string, useDynamic: boolean = false) => {
    setRunning(true)
    setError(null)
    setStepStatuses({})
    setEvents([])

    try {
      const fetchResponse = useDynamic
        ? await api.orchestrate(goal)
        : await api.orchestratePoem(goal)

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
                      // Phase 6.3: Try parsing as structured event first
                      const parsed = JSON.parse(data)
                      
                      // Check if it's a structured OrchestrationEvent
                      if (parsed.type) {
                        const event = parsed as OrchestrationEvent
                        setEvents((prev) => [...prev, event])
                        
                        // Convert structured event to OrchestrationStatus for backward compatibility
                        let statusUpdate: OrchestrationStatus | null = null
                        
                        if (event.type === 'step_start') {
                          statusUpdate = {
                            step: event.step_number,
                            step_id: event.step_id,
                            message: `Step ${event.step_number} (${event.task}) starting`,
                            status: 'running',
                          }
                        } else if (event.type === 'step_complete') {
                          statusUpdate = {
                            step: event.step_number,
                            step_id: event.step_id,
                            message: `Step ${event.step_number} completed`,
                            status: 'completed',
                          }
                        } else if (event.type === 'step_error') {
                          statusUpdate = {
                            step: event.step_number,
                            step_id: event.step_id,
                            message: `Step ${event.step_number} failed: ${event.error}`,
                            status: 'error',
                          }
                        } else if (event.type === 'execution_complete') {
                          setRunning(false)
                          statusUpdate = {
                            step: event.total_steps,
                            step_id: 'completion',
                            message: `All ${event.total_steps} steps completed successfully!`,
                            status: 'completed',
                          }
                        } else if (event.type === 'execution_error') {
                          setRunning(false)
                          setError(event.error)
                          statusUpdate = {
                            step: 0,
                            step_id: 'execution_error',
                            message: event.error,
                            status: 'error',
                          }
                        }
                        
                        if (statusUpdate) {
                          const stepId = statusUpdate.step_id
                          setStepStatuses((prev) => ({
                            ...prev,
                            [stepId]: statusUpdate!,
                          }))
                        }
                      } else {
                        // Backward compatibility: parse as old OrchestrationStatus format
                        const statusUpdate = parsed as OrchestrationStatus
                        const stepId = statusUpdate.step_id || `step_${statusUpdate.step || 'unknown'}`
                        const statusWithId: OrchestrationStatus = {
                          ...statusUpdate,
                          step_id: stepId,
                        }
                        
                        setStepStatuses((prev) => ({
                          ...prev,
                          [stepId]: statusWithId,
                        }))
                        
                        if (statusUpdate.status === 'completed' || statusUpdate.status === 'error') {
                          setRunning(false)
                          if (statusUpdate.status === 'error') {
                            setError(statusUpdate.message)
                          }
                        }
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
    stepStatuses,
    events, // Phase 6.3: Structured events
    running,
    error,
    runOrchestration,
    clearStatus,
  }
}

