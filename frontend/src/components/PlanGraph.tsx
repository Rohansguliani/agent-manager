// Phase 6.2: Plan Graph Visualization Component

import React, { useMemo } from 'react'
import ReactFlow, {
  Node,
  Edge,
  Background,
  Controls,
  MiniMap,
  Position,
} from 'reactflow'
import 'reactflow/dist/style.css'
import { GraphStructure, OrchestrationEvent } from '../api'

interface PlanGraphProps {
  graph: GraphStructure | null
  goal?: string
  // Phase 6.3: Live status updates
  stepStatuses?: Record<string, { status: 'running' | 'completed' | 'error' | 'pending' }>
  events?: OrchestrationEvent[]
}

export const PlanGraph: React.FC<PlanGraphProps> = ({ graph, goal, stepStatuses, events }) => {
  // Build step status map once for O(1) lookups (optimization: avoid O(n) filtering per node)
  const stepStatusMap = useMemo(() => {
    const map = new Map<string, 'pending' | 'running' | 'completed' | 'error'>()
    
    // First, populate from stepStatuses if available
    if (stepStatuses) {
      Object.entries(stepStatuses).forEach(([stepId, status]) => {
        map.set(stepId, status.status as 'pending' | 'running' | 'completed' | 'error')
      })
    }
    
    // Then, process events to update statuses (events override stepStatuses)
    if (events) {
      events.forEach((event) => {
        // Only process events that have step_id
        if (event.type === 'step_complete' || event.type === 'step_error' || event.type === 'step_start') {
          const stepId = event.step_id
          if (event.type === 'step_complete') {
            map.set(stepId, 'completed')
          } else if (event.type === 'step_error') {
            map.set(stepId, 'error')
          } else if (event.type === 'step_start') {
            map.set(stepId, 'running')
          }
        }
      })
    }
    
    return map
  }, [stepStatuses, events])

  const { nodes, edges } = useMemo(() => {
    if (!graph) {
      return { nodes: [], edges: [] }
    }

    // Phase 6.3: Determine node status from status map (O(1) lookup)
    const getNodeStatus = (stepId: string): 'pending' | 'running' | 'completed' | 'error' => {
      return stepStatusMap.get(stepId) ?? 'pending'
    }

    // Phase 6.3: Get node style based on status
    const getNodeStyle = (status: string) => {
      switch (status) {
        case 'running':
          return { background: '#6366f1', color: '#fff', border: '2px solid #4f46e5' }
        case 'completed':
          return { background: '#10b981', color: '#fff', border: '2px solid #059669' }
        case 'error':
          return { background: '#ef4444', color: '#fff', border: '2px solid #dc2626' }
        default:
          return { background: '#f3f4f6', color: '#374151', border: '1px solid #d1d5db' }
      }
    }

    // Create nodes from task_ids
    const nodes: Node[] = graph.task_ids.map((taskId, index) => {
      const status = getNodeStatus(taskId)
      const style = getNodeStyle(status)
      
      return {
        id: taskId,
        type: 'default',
        data: {
          label: taskId,
        },
        position: {
          // Simple layout: arrange in a row for now
          // TODO: Better layout algorithm (e.g., hierarchical)
          x: index * 200,
          y: 0,
        },
        sourcePosition: Position.Right,
        targetPosition: Position.Left,
        style: {
          ...style,
          padding: '10px',
          borderRadius: '8px',
        },
      }
    })

    // Create edges from dependencies
    const edges: Edge[] = graph.edges.map((edge, index) => ({
      id: `edge-${index}`,
      source: edge.from,
      target: edge.to,
      type: 'smoothstep',
      animated: false,
      style: { stroke: '#6366f1', strokeWidth: 2 },
    }))

    return { nodes, edges }
  }, [graph, stepStatusMap])

  if (!graph) {
    return (
      <div
        style={{
          padding: '2rem',
          textAlign: 'center',
          color: '#666',
          border: '1px solid #ddd',
          borderRadius: '4px',
          backgroundColor: '#f9f9f9',
        }}
      >
        {goal ? 'Generate a plan to see the graph visualization' : 'No plan available'}
      </div>
    )
  }

  return (
    <div style={{ width: '100%', height: '500px', border: '1px solid #ddd', borderRadius: '4px' }}>
      <ReactFlow
        nodes={nodes}
        edges={edges}
        fitView
        attributionPosition="bottom-left"
      >
        <Background />
        <Controls />
        <MiniMap />
      </ReactFlow>
    </div>
  )
}

