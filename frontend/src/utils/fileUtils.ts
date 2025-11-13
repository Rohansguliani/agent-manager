/**
 * File utility functions
 * 
 * Provides utility functions for file path manipulation and formatting
 */

/**
 * Format file size in bytes to human-readable string
 * 
 * @param bytes - File size in bytes
 * @returns Formatted size string (e.g., "1.5 KB", "2.3 MB")
 */
export function formatSize(bytes?: number): string {
  if (!bytes) return ''
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}

/**
 * Format file path for display (shows last 3 segments for long paths)
 * 
 * @param path - File path to format
 * @returns Formatted path string
 */
export function formatPath(path: string): string {
  // Show relative path or last few segments
  if (path === '.' || path === './') return 'Current Directory'
  const parts = path.split('/').filter(p => p !== '')
  if (parts.length <= 3) return '/' + parts.join('/')
  return '/.../' + parts.slice(-3).join('/')
}

/**
 * Get parent directory path from a given path
 * 
 * @param path - Current directory path
 * @returns Parent directory path, or null if at root
 */
export function getParentPath(path: string): string | null {
  if (!path || path === '' || path === '.' || path === './') return null
  // Normalize path: remove trailing slashes but keep leading slash
  const normalized = path.replace(/\/+$/, '')
  if (!normalized || normalized === '/') return null // Root directory
  const parts = normalized.split('/').filter(p => p !== '')
  if (parts.length <= 1) {
    // At mount root (e.g., /host), can't go back
    return null
  }
  parts.pop()
  const parent = '/' + parts.join('/')
  return parent || null
}

