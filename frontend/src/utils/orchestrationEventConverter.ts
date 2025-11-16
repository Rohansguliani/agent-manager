//! Orchestration event conversion utilities
//!
//! Converts structured OrchestrationEvent types to OrchestrationStatus
//! for backward compatibility with the existing log display.

import { OrchestrationEvent, OrchestrationStatus } from '../api'

/**
 * Convert an OrchestrationEvent to an OrchestrationStatus
 *
 * This function converts the new structured event format to the legacy
 * status format for backward compatibility with the existing UI components.
 *
 * @param event - The orchestration event to convert
 * @returns OrchestrationStatus or null if event doesn't map to a status
 */
export function eventToStatus(event: OrchestrationEvent): OrchestrationStatus | null {
  if (event.type === 'step_start') {
    return {
      step: event.step_number,
      step_id: event.step_id,
      message: `Step ${event.step_number} (${event.task}) starting`,
      status: 'running',
    }
  } else if (event.type === 'step_complete') {
    return {
      step: event.step_number,
      step_id: event.step_id,
      message: `Step ${event.step_number} completed`,
      status: 'completed',
    }
  } else if (event.type === 'step_error') {
    return {
      step: event.step_number,
      step_id: event.step_id,
      message: `Step ${event.step_number} failed: ${event.error}`,
      status: 'error',
    }
  } else if (event.type === 'execution_complete') {
    return {
      step: event.total_steps,
      step_id: 'completion',
      message: `All ${event.total_steps} steps completed successfully!`,
      status: 'completed',
    }
  } else if (event.type === 'execution_error') {
    return {
      step: 0,
      step_id: 'error',
      message: `Execution failed: ${event.error}`,
      status: 'error',
    }
  }
  // plan_generated doesn't map to a status (it's informational only)
  return null
}

/**
 * Convert multiple OrchestrationEvents to OrchestrationStatus records
 *
 * @param events - Array of orchestration events
 * @returns Record mapping step_id to OrchestrationStatus
 */
export function eventsToStatusRecord(events: OrchestrationEvent[]): Record<string, OrchestrationStatus> {
  const statuses: Record<string, OrchestrationStatus> = {}
  for (const event of events) {
    const status = eventToStatus(event)
    if (status && status.step_id) {
      statuses[status.step_id] = status
    }
  }
  return statuses
}

