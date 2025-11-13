import { useState, useEffect, useCallback } from 'react'
import { api, FileInfo, ApiError } from '../api'
import { getParentPath } from '../utils/fileUtils'
import { FileManagerHeader } from './FileManagerHeader'
import { PathBar } from './PathBar'
import { FileList } from './FileList'
import { styles } from '../styles/components'

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
      // Toggle: if already set, unset it; otherwise set it
      if (workingDirectory === path) {
        await api.setWorkingDirectory(null)
        setWorkingDirectory(null)
        onWorkingDirectoryChange?.(null)
      } else {
        await api.setWorkingDirectory(path)
        setWorkingDirectory(path)
        onWorkingDirectoryChange?.(path)
      }
    } catch (err) {
      setError(err instanceof ApiError ? err.message : 'Failed to toggle working directory')
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

  const handleBack = useCallback(() => {
    const parent = getParentPath(currentPath)
    if (parent !== null) {
      setCurrentPath(parent)
    }
  }, [currentPath])

  const handleToggleContext = async () => {
    try {
      if (workingDirectory === currentPath) {
        // Unset context
        await api.setWorkingDirectory(null)
        setWorkingDirectory(null)
        onWorkingDirectoryChange?.(null)
      } else {
        // Set context
        await api.setWorkingDirectory(currentPath)
        setWorkingDirectory(currentPath)
        onWorkingDirectoryChange?.(currentPath)
      }
    } catch (err) {
      setError(err instanceof ApiError ? err.message : 'Failed to toggle working directory')
    }
  }

  return (
    <div style={{ ...styles.fileManager, marginTop: '2rem' }}>
      <FileManagerHeader
        workingDirectory={workingDirectory}
        onClearContext={handleClearContext}
      />

      {error && (
        <div
          style={{
            ...styles.errorBox,
            padding: '0.5rem',
            marginBottom: '0.5rem',
            fontSize: '0.9rem',
          }}
        >
          {error}
        </div>
      )}

      <PathBar
        currentPath={currentPath}
        workingDirectory={workingDirectory}
        onBack={handleBack}
        onToggleContext={handleToggleContext}
      />

      {loading ? (
        <div style={styles.loading}>Loading...</div>
      ) : (
        <FileList
          files={files}
          workingDirectory={workingDirectory}
          onFolderClick={handleFolderClick}
          onSetContext={handleJumpHere}
        />
      )}
    </div>
  )
}

