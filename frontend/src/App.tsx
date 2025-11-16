import { useState } from 'react'
import { ErrorBoundary } from './ErrorBoundary'
import { FileManager } from './components/FileManager'
import { useStreamingQuery } from './hooks/useStreamingQuery'
import { useOrchestrator } from './hooks/useOrchestrator'
import { styles } from './styles/components'

function App() {
  const [query, setQuery] = useState<string>('')
  const [orchestratorGoal, setOrchestratorGoal] = useState<string>('')
  const [workingDirectory, setWorkingDirectory] = useState<string | null>(null)
  const { response, loading, error, executeQuery } = useStreamingQuery()
  const {
    stepStatuses: orchestrationStepStatuses,
    running: orchestrating,
    error: orchestrationError,
    runOrchestration,
  } = useOrchestrator()

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!query.trim()) return
    await executeQuery(query)
  }

  const handleOrchestratorSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!orchestratorGoal.trim()) return
    await runOrchestration(orchestratorGoal.trim(), true)
  }

  return (
    <ErrorBoundary>
      <div style={styles.container}>
        <h1 style={styles.heading}>Agent Manager</h1>
        
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
          
          <form onSubmit={handleOrchestratorSubmit} style={styles.form}>
            <textarea
              value={orchestratorGoal}
              onChange={(e) => setOrchestratorGoal(e.target.value)}
              placeholder="Enter your goal here... (e.g., Write a poem about Rust and save it to poem.txt)"
              disabled={orchestrating}
              onKeyDown={(e) => {
                if (e.key === 'Enter' && !e.shiftKey) {
                  e.preventDefault()
                  if (!orchestrating && orchestratorGoal.trim()) {
                    handleOrchestratorSubmit(e)
                  }
                }
              }}
              style={styles.textarea}
            />
            <button
              type="submit"
              disabled={orchestrating || !orchestratorGoal.trim()}
              style={{
                ...styles.button,
                ...(orchestrating ? styles.buttonDisabled : styles.buttonPrimary),
              }}
            >
              {orchestrating ? 'Running Orchestration...' : 'Run Orchestration'}
            </button>
          </form>

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
