import { useState, useEffect, useCallback } from 'react'
import { api, FileInfo, ApiError } from '../api'

interface FileManagerProps {
  onWorkingDirectoryChange?: (path: string | null) => void
}

export function FileManager({ onWorkingDirectoryChange }: FileManagerProps) {
  const [files, setFiles] = useState<FileInfo[]>([])
  // Default to empty string, which will trigger the backend to use its default (home directory)
  const [currentPath, setCurrentPath] = useState<string>('')
  const [workingDirectory, setWorkingDirectory] = useState<string | null>(null)
  const [loading, setLoading] = useState<boolean>(false)
  const [error, setError] = useState<string | null>(null)

  const loadWorkingDirectory = useCallback(async () => {
    try {
      const response = await api.getWorkingDirectory()
      setWorkingDirectory(response.path)
    } catch (err) {
      // Silently fail - working directory is optional
      setError(err instanceof ApiError ? err.message : 'Failed to load working directory')
    }
  }, [])

  const loadFiles = useCallback(async (path: string) => {
    setLoading(true)
    setError(null)
    try {
      const response = await api.listFiles(path)
      setFiles(response.files)
      setCurrentPath(response.path)
    } catch (err) {
      setError(err instanceof ApiError ? err.message : 'Failed to load files')
    } finally {
      setLoading(false)
    }
  }, [])

  // Load current working directory on mount
  useEffect(() => {
    loadWorkingDirectory()
  }, [loadWorkingDirectory])

  // Load files when path changes
  // If currentPath is empty, the backend will use its default (home directory)
  useEffect(() => {
    if (currentPath === '') {
      // Load with no path parameter, backend will default to home
      loadFiles('')
    } else {
      loadFiles(currentPath)
    }
  }, [currentPath, loadFiles])

  const handleFolderClick = (file: FileInfo) => {
    if (file.is_directory) {
      setCurrentPath(file.path)
    }
  }

  const handleJumpHere = async (path: string) => {
    try {
      await api.setWorkingDirectory(path)
      setWorkingDirectory(path)
      onWorkingDirectoryChange?.(path)
    } catch (err) {
      setError(err instanceof ApiError ? err.message : 'Failed to set working directory')
    }
  }

  const handleClearContext = async () => {
    try {
      await api.setWorkingDirectory(null)
      setWorkingDirectory(null)
      onWorkingDirectoryChange?.(null)
    } catch (err) {
      setError(err instanceof ApiError ? err.message : 'Failed to clear working directory')
    }
  }

  const formatSize = (bytes?: number): string => {
    if (!bytes) return ''
    if (bytes < 1024) return `${bytes} B`
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
  }

  const formatPath = (path: string): string => {
    // Show relative path or last few segments
    if (path === '.' || path === './') return 'Current Directory'
    const parts = path.split('/')
    if (parts.length <= 3) return path
    return '.../' + parts.slice(-3).join('/')
  }

  return (
    <div style={{
      marginTop: '2rem',
      padding: '1rem',
      backgroundColor: '#f9f9f9',
      border: '1px solid #ddd',
      borderRadius: '4px'
    }}>
      <div style={{
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        marginBottom: '1rem'
      }}>
        <h2 style={{ margin: 0, fontSize: '1.2rem' }}>File Manager</h2>
        {workingDirectory && (
          <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
            <span style={{ fontSize: '0.9rem', color: '#666' }}>
              Context: {formatPath(workingDirectory)}
            </span>
            <button
              onClick={handleClearContext}
              style={{
                padding: '0.25rem 0.5rem',
                fontSize: '0.8rem',
                backgroundColor: '#dc3545',
                color: 'white',
                border: 'none',
                borderRadius: '3px',
                cursor: 'pointer'
              }}
            >
              Clear
            </button>
          </div>
        )}
      </div>

      {error && (
        <div style={{
          padding: '0.5rem',
          backgroundColor: '#fee',
          border: '1px solid #fcc',
          borderRadius: '4px',
          color: '#c00',
          marginBottom: '0.5rem',
          fontSize: '0.9rem'
        }}>
          {error}
        </div>
      )}

      <div style={{
        marginBottom: '0.5rem',
        fontSize: '0.9rem',
        color: '#666',
        padding: '0.25rem'
      }}>
        {formatPath(currentPath)}
      </div>

      {loading ? (
        <div style={{ padding: '1rem', textAlign: 'center', color: '#666' }}>
          Loading...
        </div>
      ) : (
        <div style={{
          maxHeight: '300px',
          overflowY: 'auto',
          border: '1px solid #ddd',
          borderRadius: '4px',
          backgroundColor: 'white'
        }}>
          {files.length === 0 ? (
            <div style={{ padding: '1rem', textAlign: 'center', color: '#666' }}>
              No files found
            </div>
          ) : (
            <table style={{ width: '100%', borderCollapse: 'collapse' }}>
              <thead>
                <tr style={{ backgroundColor: '#f0f0f0', borderBottom: '1px solid #ddd' }}>
                  <th style={{ padding: '0.5rem', textAlign: 'left', fontSize: '0.9rem' }}>Name</th>
                  <th style={{ padding: '0.5rem', textAlign: 'right', fontSize: '0.9rem' }}>Size</th>
                  <th style={{ padding: '0.5rem', textAlign: 'center', fontSize: '0.9rem' }}>Action</th>
                </tr>
              </thead>
              <tbody>
                {files.map((file) => (
                  <tr
                    key={file.path}
                    style={{
                      borderBottom: '1px solid #eee',
                      cursor: file.is_directory ? 'pointer' : 'default'
                    }}
                    onClick={() => handleFolderClick(file)}
                    onMouseEnter={(e) => {
                      if (file.is_directory) {
                        e.currentTarget.style.backgroundColor = '#f5f5f5'
                      }
                    }}
                    onMouseLeave={(e) => {
                      e.currentTarget.style.backgroundColor = 'transparent'
                    }}
                  >
                    <td style={{ padding: '0.5rem' }}>
                      <span style={{ marginRight: '0.5rem' }}>
                        {file.is_directory ? '📁' : '📄'}
                      </span>
                      {file.name}
                    </td>
                    <td style={{ padding: '0.5rem', textAlign: 'right', fontSize: '0.9rem', color: '#666' }}>
                      {formatSize(file.size)}
                    </td>
                    <td style={{ padding: '0.5rem', textAlign: 'center' }}>
                      {file.is_directory && (
                        <button
                          onClick={(e) => {
                            e.stopPropagation()
                            handleJumpHere(file.path)
                          }}
                          style={{
                            padding: '0.25rem 0.5rem',
                            fontSize: '0.8rem',
                            backgroundColor: workingDirectory === file.path ? '#28a745' : '#007bff',
                            color: 'white',
                            border: 'none',
                            borderRadius: '3px',
                            cursor: 'pointer'
                          }}
                        >
                          {workingDirectory === file.path ? '✓ Active' : 'Jump Here'}
                        </button>
                      )}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </div>
      )}
    </div>
  )
}

