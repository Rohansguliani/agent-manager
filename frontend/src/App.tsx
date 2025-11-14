import { useState } from 'react'
import { ErrorBoundary } from './ErrorBoundary'
import { FileManager } from './components/FileManager'
import { useStreamingQuery } from './hooks/useStreamingQuery'
import { useOrchestrator } from './hooks/useOrchestrator'
import { styles } from './styles/components'

function App() {
  const [query, setQuery] = useState<string>('')
  const [workingDirectory, setWorkingDirectory] = useState<string | null>(null)
  const { response, loading, error, executeQuery } = useStreamingQuery()
  const {
    status: orchestrationStatus,
    running: orchestrating,
    error: orchestrationError,
    runOrchestration,
  } = useOrchestrator()

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!query.trim()) return
    await executeQuery(query)
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

        {/* V1 Orchestrator Section */}
        <div style={{ ...styles.fileManager, marginTop: '2rem' }}>
          <h2 style={styles.fileManagerTitle}>V1 Orchestrator</h2>
          <p style={{ marginBottom: '1rem', color: '#666' }}>
            Click the button below to run a hard-coded orchestration:
            Gemini generates a poem, then saves it to poem.txt
          </p>
          <button
            onClick={() =>
              runOrchestration('Write a 4-line poem about the Rust programming language.')
            }
            disabled={orchestrating}
            style={{
              ...styles.button,
              ...(orchestrating ? styles.buttonDisabled : styles.buttonPrimary),
            }}
          >
            {orchestrating ? 'Running Orchestration...' : 'Run Poem Orchestration'}
          </button>

          {orchestrationStatus && (
            <div
              style={{
                marginTop: '1rem',
                padding: '1rem',
                backgroundColor:
                  orchestrationStatus.status === 'error'
                    ? '#fee'
                    : orchestrationStatus.status === 'completed'
                      ? '#efe'
                      : '#f9f9f9',
                border: `1px solid ${
                  orchestrationStatus.status === 'error'
                    ? '#fcc'
                    : orchestrationStatus.status === 'completed'
                      ? '#cfc'
                      : '#ddd'
                }`,
                borderRadius: '4px',
              }}
            >
              <strong>Step {orchestrationStatus.step}:</strong>{' '}
              {orchestrationStatus.message}
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
