import { useState } from 'react'
import { ErrorBoundary } from './ErrorBoundary'
import { FileManager } from './components/FileManager'

function App() {
  const [query, setQuery] = useState<string>('')
  const [response, setResponse] = useState<string>('')
  const [loading, setLoading] = useState<boolean>(false)
  const [error, setError] = useState<string | null>(null)
  const [workingDirectory, setWorkingDirectory] = useState<string | null>(null)

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!query.trim()) return

    setLoading(true)
    setError(null)
    setResponse('')

    try {
      // Use fetch with ReadableStream for POST request with SSE
      const response = await fetch(
        `${import.meta.env.VITE_API_URL || 'http://localhost:8080'}/api/query/stream`,
        {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify({ query }),
        }
      )

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`)
      }

      const reader = response.body?.getReader()
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
                setError(data.slice(8))
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
      setError(err instanceof Error ? err.message : 'Failed to stream response')
      setLoading(false)
    }
  }

  return (
    <ErrorBoundary>
      <div style={{ 
        maxWidth: '800px', 
        margin: '0 auto', 
        padding: '2rem',
        fontFamily: 'system-ui, -apple-system, sans-serif'
      }}>
        <h1 style={{ marginBottom: '2rem' }}>Agent Manager</h1>
        
        <form onSubmit={handleSubmit} style={{ marginBottom: '2rem' }}>
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
            style={{
              width: '100%',
              minHeight: '100px',
              padding: '0.5rem',
              fontSize: '1rem',
              border: '1px solid #ddd',
              borderRadius: '4px',
              fontFamily: 'inherit',
              resize: 'vertical'
            }}
          />
          <button
            type="submit"
            disabled={loading || !query.trim()}
            style={{
              marginTop: '0.5rem',
              padding: '0.5rem 1rem',
              fontSize: '1rem',
              backgroundColor: loading ? '#ccc' : '#007bff',
              color: 'white',
              border: 'none',
              borderRadius: '4px',
              cursor: loading ? 'not-allowed' : 'pointer'
            }}
          >
            {loading ? 'Sending...' : 'Send Query'}
          </button>
        </form>

        {error && (
          <div style={{
            padding: '1rem',
            backgroundColor: '#fee',
            border: '1px solid #fcc',
            borderRadius: '4px',
            color: '#c00',
            marginBottom: '1rem'
          }}>
            <strong>Error:</strong> {error}
          </div>
        )}

        {response && (
          <div style={{
            padding: '1rem',
            backgroundColor: '#f9f9f9',
            border: '1px solid #ddd',
            borderRadius: '4px',
            whiteSpace: 'pre-wrap',
            fontFamily: 'monospace',
            fontSize: '0.9rem'
          }}>
            {response}
          </div>
        )}

        {/* File Manager */}
        <FileManager onWorkingDirectoryChange={setWorkingDirectory} />

        {/* Show current context */}
        {workingDirectory && (
          <div style={{
            marginTop: '1rem',
            padding: '0.5rem 1rem',
            backgroundColor: '#e7f3ff',
            border: '1px solid #b3d9ff',
            borderRadius: '4px',
            fontSize: '0.9rem',
            color: '#0066cc'
          }}>
            <strong>Context:</strong> Queries will run in: {workingDirectory}
          </div>
        )}
      </div>
    </ErrorBoundary>
  )
}

export default App
