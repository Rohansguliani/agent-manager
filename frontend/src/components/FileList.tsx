/**
 * File List Component
 * 
 * Displays a table of files and directories
 */

import { FileInfo } from '../api'
import { formatSize } from '../utils/fileUtils'
import { styles } from '../styles/components'

interface FileListProps {
  files: FileInfo[]
  workingDirectory: string | null
  onFolderClick: (file: FileInfo) => void
  onSetContext: (path: string) => void
}

export function FileList({
  files,
  workingDirectory,
  onFolderClick,
  onSetContext,
}: FileListProps) {
  if (files.length === 0) {
    return <div style={styles.empty}>No files found</div>
  }

  return (
    <div style={styles.fileList}>
      <table style={styles.fileTable}>
        <thead>
          <tr style={styles.fileTableHeader}>
            <th style={styles.fileTableHeaderCell}>Name</th>
            <th style={{ ...styles.fileTableHeaderCell, textAlign: 'right' }}>Size</th>
            <th style={{ ...styles.fileTableHeaderCell, textAlign: 'center' }}>Action</th>
          </tr>
        </thead>
        <tbody>
          {files.map((file) => (
            <tr
              key={file.path}
              style={{
                ...styles.fileTableRow,
                cursor: file.is_directory ? 'pointer' : 'default',
              }}
              onClick={() => onFolderClick(file)}
              onMouseEnter={(e) => {
                if (file.is_directory) {
                  e.currentTarget.style.backgroundColor = '#f5f5f5'
                }
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.backgroundColor = 'transparent'
              }}
            >
              <td style={styles.fileTableCell}>
                <span style={{ marginRight: '0.5rem' }}>
                  {file.is_directory ? '📁' : '📄'}
                </span>
                {file.name}
              </td>
              <td style={styles.fileTableCellRight}>{formatSize(file.size)}</td>
              <td style={styles.fileTableCellCenter}>
                {file.is_directory && (
                  <button
                    onClick={(e) => {
                      e.stopPropagation()
                      onSetContext(file.path)
                    }}
                    style={{
                      ...styles.button,
                      ...styles.buttonSmall,
                      ...(workingDirectory === file.path
                        ? styles.buttonDanger
                        : styles.buttonPrimary),
                    }}
                    title={workingDirectory === file.path ? 'Unset context' : 'Set as context'}
                  >
                    {workingDirectory === file.path ? '✗ Unset' : '✓ Set'}
                  </button>
                )}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  )
}

