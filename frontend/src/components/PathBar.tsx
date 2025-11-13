/**
 * Path Bar Component
 * 
 * Displays the current path with back button and context toggle
 */

import { formatPath, getParentPath } from '../utils/fileUtils'
import { styles } from '../styles/components'

interface PathBarProps {
  currentPath: string
  workingDirectory: string | null
  onBack: () => void
  onToggleContext: () => void
}

export function PathBar({
  currentPath,
  workingDirectory,
  onBack,
  onToggleContext,
}: PathBarProps) {
  return (
    <div style={styles.pathBar}>
      <div style={styles.pathBarLeft}>
        {getParentPath(currentPath) !== null && (
          <button
            onClick={onBack}
            style={{
              ...styles.button,
              ...styles.buttonSmall,
              ...styles.buttonSecondary,
            }}
            title="Go to parent directory"
          >
            ← Back
          </button>
        )}
        <span style={{ flex: 1 }}>{formatPath(currentPath)}</span>
      </div>
      {currentPath && currentPath !== '' && (
        <button
          onClick={onToggleContext}
          style={{
            ...styles.button,
            ...styles.buttonSmall,
            ...(workingDirectory === currentPath
              ? styles.buttonDanger
              : styles.buttonSuccess),
          }}
          title={workingDirectory === currentPath ? 'Unset context' : 'Set as context'}
        >
          {workingDirectory === currentPath ? '✗ Unset Context' : '✓ Set Context'}
        </button>
      )}
    </div>
  )
}

