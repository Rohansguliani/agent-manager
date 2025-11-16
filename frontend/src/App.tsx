import { useState } from 'react'
import { ErrorBoundary } from './ErrorBoundary'
import { FileManager } from './components/FileManager'
import { PlanGraph } from './components/PlanGraph'
import { Settings } from './components/Settings'
import { useStreamingQuery } from './hooks/useStreamingQuery'
import { useOrchestrator } from './hooks/useOrchestrator'
import { api, PlanAnalysisResponse, GraphStructure } from './api'
import { styles } from './styles/components'

function App() {
  const [query, setQuery] = useState<string>('')
  const [orchestratorGoal, setOrchestratorGoal] = useState<string>('')
  const [workingDirectory, setWorkingDirectory] = useState<string | null>(null)
  const [planAnalysis, setPlanAnalysis] = useState<PlanAnalysisResponse | null>(null)
  const [planning, setPlanning] = useState<boolean>(false)
  const [planError, setPlanError] = useState<string | null>(null)
  const [graph, setGraph] = useState<GraphStructure | null>(null)
  const [graphLoading, setGraphLoading] = useState<boolean>(false)
  const [graphError, setGraphError] = useState<string | null>(null)
  const [activeTab, setActiveTab] = useState<'summary' | 'graph'>('summary')
  const [showSettings, setShowSettings] = useState<boolean>(false)
  const { response, loading, error, executeQuery } = useStreamingQuery()
  const {
    stepStatuses: orchestrationStepStatuses,
    events: orchestrationEvents, // Phase 6.3: Structured events
    running: orchestrating,
    error: orchestrationError,
    runOrchestration,
  } = useOrchestrator()

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!query.trim()) return
    await executeQuery(query)
  }

  // Phase 6.1: Pre-flight check - Plan first
  const handlePlanSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!orchestratorGoal.trim()) return

    setPlanning(true)
    setPlanError(null)
    setPlanAnalysis(null)

    try {
      const analysis = await api.plan(orchestratorGoal.trim())
      setPlanAnalysis(analysis)
      setPlanError(null)
      // Phase 6.2: Fetch graph after planning
      await fetchGraph(orchestratorGoal.trim())
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to generate plan'
      setPlanError(errorMessage)
      setGraph(null)
    } finally {
      setPlanning(false)
    }
  }

  // Phase 6.2: Fetch graph structure
  const fetchGraph = async (goal: string) => {
    setGraphLoading(true)
    setGraphError(null)

    try {
      const graphData = await api.getGraph(goal)
      setGraph(graphData)
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to load graph'
      setGraphError(errorMessage)
      setGraph(null)
    } finally {
      setGraphLoading(false)
    }
  }

  // Confirm & Run - Execute the planned orchestration
  const handleConfirmAndRun = async () => {
    if (!orchestratorGoal.trim()) return
    setPlanAnalysis(null) // Clear plan analysis before execution
    await runOrchestration(orchestratorGoal.trim(), true)
  }

  return (
    <ErrorBoundary>
      <div style={styles.container}>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '1rem' }}>
          <h1 style={styles.heading}>Agent Manager</h1>
          <button
            onClick={() => setShowSettings(true)}
            style={{
              ...styles.button,
              backgroundColor: '#6c757d',
              color: '#fff',
            }}
          >
            Settings
          </button>
        </div>

        {/* Phase 6.4: Settings Panel */}
        {showSettings && (
          <div
            style={{
              position: 'fixed',
              top: 0,
              left: 0,
              right: 0,
              bottom: 0,
              backgroundColor: 'rgba(0, 0, 0, 0.5)',
              zIndex: 1000,
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              padding: '2rem',
            }}
            onClick={() => setShowSettings(false)}
          >
            <div
              onClick={(e) => e.stopPropagation()}
              style={{
                backgroundColor: '#fff',
                borderRadius: '8px',
        maxWidth: '800px', 
                width: '100%',
                maxHeight: '90vh',
                overflowY: 'auto',
              }}
            >
              <Settings onClose={() => setShowSettings(false)} />
            </div>
          </div>
        )}
        
        <form onSubmit={handleSubmit} style={styles.form}>
          <textarea
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="Type your query and press Enter..."
            disabled={loading}
            onKeyDown={(e) => {
              if (e.key === 'Enter' && !e.shiftKey) {
                e.preventDefault()
                if (!loading && query.trim()) {
                  handleSubmit(e)
                }
              }
            }}
            style={styles.textarea}
          />
          <button
            type="submit"
            disabled={loading || !query.trim()}
            style={{
              ...styles.button,
              ...(loading ? styles.buttonDisabled : styles.buttonPrimary),
            }}
          >
            {loading ? 'Sending...' : 'Send Query'}
          </button>
        </form>

        {error && (
          <div style={styles.errorBox}>
            <strong>Error:</strong> {error}
          </div>
        )}

        {response && (
          <div style={styles.responseBox}>
            {response}
          </div>
        )}

        {/* Dynamic Orchestrator Section */}
        <div style={{ ...styles.fileManager, marginTop: '2rem' }}>
          <h2 style={styles.fileManagerTitle}>Dynamic Orchestrator</h2>
          <p style={{ marginBottom: '1rem', color: '#666' }}>
            Enter a high-level goal and the orchestrator will plan and execute it automatically.
            Example: &quot;Write a poem about Rust and save it to rust_poem.txt&quot;
          </p>
          
          <form onSubmit={handlePlanSubmit} style={styles.form}>
            <textarea
              value={orchestratorGoal}
              onChange={(e) => {
                setOrchestratorGoal(e.target.value)
                setPlanAnalysis(null) // Clear plan when goal changes
                setPlanError(null)
              }}
              placeholder="Enter your goal here... (e.g., Write a poem about Rust and save it to poem.txt)"
              disabled={planning || orchestrating}
              onKeyDown={(e) => {
                if (e.key === 'Enter' && !e.shiftKey) {
                  e.preventDefault()
                  if (!planning && !orchestrating && orchestratorGoal.trim()) {
                    handlePlanSubmit(e)
                  }
                }
              }}
              style={styles.textarea}
            />
            <button
              type="submit"
              disabled={planning || orchestrating || !orchestratorGoal.trim()}
              style={{
                ...styles.button,
                ...(planning || orchestrating ? styles.buttonDisabled : styles.buttonPrimary),
              }}
            >
              {planning ? 'Planning...' : orchestrating ? 'Running...' : 'Plan'}
            </button>
          </form>

          {/* Phase 6.1: Pre-flight Summary + Phase 6.2: Graph Visualization */}
          {planAnalysis && (
            <div
              style={{
                marginTop: '1rem',
                padding: '1rem',
                backgroundColor: '#f0f8ff',
                border: '1px solid #4a90e2',
                borderRadius: '4px',
              }}
            >
              {/* Tabs */}
              <div style={{ display: 'flex', gap: '0.5rem', marginBottom: '1rem', borderBottom: '1px solid #ddd' }}>
                <button
                  onClick={() => setActiveTab('summary')}
                  style={{
                    padding: '0.5rem 1rem',
                    border: 'none',
                    backgroundColor: activeTab === 'summary' ? '#4a90e2' : 'transparent',
                    color: activeTab === 'summary' ? '#fff' : '#666',
                    cursor: 'pointer',
                    borderRadius: '4px 4px 0 0',
                    fontWeight: activeTab === 'summary' ? 'bold' : 'normal',
                  }}
                >
                  Pre-flight Summary
                </button>
                <button
                  onClick={() => setActiveTab('graph')}
                  style={{
                    padding: '0.5rem 1rem',
                    border: 'none',
                    backgroundColor: activeTab === 'graph' ? '#4a90e2' : 'transparent',
                    color: activeTab === 'graph' ? '#fff' : '#666',
                    cursor: 'pointer',
                    borderRadius: '4px 4px 0 0',
                    fontWeight: activeTab === 'graph' ? 'bold' : 'normal',
                  }}
                >
                  Plan Graph
                </button>
              </div>

              {/* Tab Content */}
              {activeTab === 'summary' && (
                <div>
                  <h3 style={{ marginTop: 0, marginBottom: '1rem', fontSize: '1rem', color: '#2c3e50' }}>
                    Pre-flight Summary
                  </h3>
                  <div style={{ marginBottom: '0.75rem' }}>
                    <strong>Estimated Cost:</strong> {planAnalysis.estimated_tokens.toLocaleString()} tokens
                  </div>
                  <div style={{ marginBottom: '0.75rem' }}>
                    <strong>Estimated Time:</strong> ~{planAnalysis.estimated_time_secs} seconds
                  </div>
                  <div style={{ marginBottom: '0.75rem' }}>
                    <strong>Steps:</strong> {planAnalysis.plan.steps.length} step{planAnalysis.plan.steps.length !== 1 ? 's' : ''}
                    {' | '}
                    <strong>Independent:</strong> {planAnalysis.bottlenecks.independent_steps} (can run in parallel)
                  </div>
                  {planAnalysis.bottlenecks.high_dependency_steps.length > 0 && (
                    <div style={{ marginBottom: '0.75rem', color: '#d68910' }}>
                      <strong>⚠ Bottleneck:</strong> Step{planAnalysis.bottlenecks.high_dependency_steps.length !== 1 ? 's' : ''}{' '}
                      {planAnalysis.bottlenecks.high_dependency_steps.join(', ')} have many dependencies
                    </div>
                  )}
                  {planAnalysis.bottlenecks.longest_chain_length > 1 && (
                    <div style={{ marginBottom: '0.75rem', color: '#8e44ad' }}>
                      <strong>📊 Longest Chain:</strong> {planAnalysis.bottlenecks.longest_chain_length} sequential steps
                    </div>
                  )}
                  <button
                    onClick={handleConfirmAndRun}
                    disabled={orchestrating}
                    style={{
                      ...styles.button,
                      ...styles.buttonPrimary,
                      marginTop: '0.5rem',
                    }}
                  >
                    {orchestrating ? 'Running Orchestration...' : 'Confirm & Run'}
                  </button>
                </div>
              )}

              {activeTab === 'graph' && (
                <div>
                  <h3 style={{ marginTop: 0, marginBottom: '1rem', fontSize: '1rem', color: '#2c3e50' }}>
                    Execution Plan Graph
                  </h3>
                  {graphLoading && (
                    <div style={{ padding: '2rem', textAlign: 'center', color: '#666' }}>
                      Loading graph...
                    </div>
                  )}
                  {graphError && (
                    <div style={{ ...styles.errorBox, marginTop: '0.5rem' }}>
                      <strong>Graph Error:</strong> {graphError}
                    </div>
                  )}
                  {!graphLoading && !graphError && (
                    <PlanGraph
                      graph={graph}
                      goal={orchestratorGoal}
                      stepStatuses={orchestrationStepStatuses}
                      events={orchestrationEvents}
                    />
                  )}
                </div>
              )}
            </div>
          )}

          {planError && (
            <div style={{ ...styles.errorBox, marginTop: '1rem' }}>
              <strong>Planning Error:</strong> {planError}
            </div>
          )}

          {/* Status History - Live Log (Parallel Execution Support) */}
          {Object.keys(orchestrationStepStatuses).length > 0 && (
            <div
              style={{
                marginTop: '1rem',
            padding: '1rem',
            backgroundColor: '#f9f9f9',
            border: '1px solid #ddd',
            borderRadius: '4px',
                maxHeight: '400px',
                overflowY: 'auto',
              }}
            >
              <h3 style={{ marginTop: 0, marginBottom: '0.5rem', fontSize: '1rem' }}>
                Execution Log:
              </h3>
              {Object.values(orchestrationStepStatuses)
                .sort((a, b) => (a.step || 0) - (b.step || 0)) // Sort by step number
                .map((status) => {
                  const statusColor =
                    status.status === 'error'
                      ? '#fee'
                      : status.status === 'completed'
                        ? '#efe'
                        : status.status === 'running'
                          ? '#eef'
                          : status.status === 'pending'
                            ? '#fefefe'
                            : '#fff'
                  const borderColor =
                    status.status === 'error'
                      ? '#fcc'
                      : status.status === 'completed'
                        ? '#cfc'
                        : status.status === 'running'
                          ? '#ccf'
                          : status.status === 'pending'
                            ? '#eee'
                            : '#ddd'
                  
                  return (
                    <div
                      key={status.step_id}
                      style={{
                        padding: '0.5rem',
                        marginBottom: '0.5rem',
                        backgroundColor: statusColor,
                        border: `1px solid ${borderColor}`,
                        borderRadius: '4px',
                        fontSize: '0.9rem',
                      }}
                    >
                      <strong>
                        {status.step !== undefined ? `Step ${status.step}` : status.step_id}
                        :
                      </strong>{' '}
                      {status.message}
                      <span
                        style={{
                          float: 'right',
                          fontSize: '0.8rem',
                          color: '#666',
                          textTransform: 'uppercase',
                        }}
                      >
                        [{status.status}]
                      </span>
                    </div>
                  )
                })}
            </div>
          )}

          {orchestrationError && (
            <div style={styles.errorBox}>
              <strong>Error:</strong> {orchestrationError}
            </div>
          )}
        </div>

        {/* File Manager */}
        <FileManager onWorkingDirectoryChange={setWorkingDirectory} />

        {/* Show current context */}
        {workingDirectory && (
          <div style={styles.contextBox}>
            <strong>Context:</strong> Queries will run in: {workingDirectory}
          </div>
        )}
      </div>
    </ErrorBoundary>
  )
}

export default App
