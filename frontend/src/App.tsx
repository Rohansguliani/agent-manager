import { useState } from 'react'
import { ErrorBoundary } from './ErrorBoundary'
import { FileManager } from './components/FileManager'
import { useStreamingQuery } from './hooks/useStreamingQuery'
import { styles } from './styles/components'

function App() {
  const [query, setQuery] = useState<string>('')
  const [workingDirectory, setWorkingDirectory] = useState<string | null>(null)
  const { response, loading, error, executeQuery } = useStreamingQuery()

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
