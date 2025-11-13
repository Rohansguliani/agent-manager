/**
 * File Manager Header Component
 * 
 * Displays the file manager title and current context
 */

import { formatPath } from '../utils/fileUtils'
import { styles } from '../styles/components'

interface FileManagerHeaderProps {
  workingDirectory: string | null
  onClearContext: () => void
}

export function FileManagerHeader({
  workingDirectory,
  onClearContext,
}: FileManagerHeaderProps) {
  return (
    <div style={styles.fileManagerHeader}>
      <h2 style={styles.fileManagerTitle}>File Manager</h2>
      {workingDirectory && (
        <div style={styles.fileManagerContext}>
          <span style={styles.fileManagerContextText}>
            Context: {formatPath(workingDirectory)}
          </span>
          <button
            onClick={onClearContext}
            style={{
              ...styles.button,
              ...styles.buttonSmall,
              ...styles.buttonDanger,
            }}
          >
            Clear
          </button>
        </div>
      )}
    </div>
  )
}

