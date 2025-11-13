/**
 * Component style definitions
 * 
 * Centralized style objects for reusable components
 */

export const styles = {
  container: {
    maxWidth: '800px',
    margin: '0 auto',
    padding: '2rem',
    fontFamily: 'system-ui, -apple-system, sans-serif',
  } as const,

  heading: {
    marginBottom: '2rem',
  } as const,

  form: {
    marginBottom: '2rem',
  } as const,

  textarea: {
    width: '100%',
    minHeight: '100px',
    padding: '0.5rem',
    fontSize: '1rem',
    border: '1px solid #ddd',
    borderRadius: '4px',
    fontFamily: 'inherit',
    resize: 'vertical' as const,
  } as const,

  button: {
    marginTop: '0.5rem',
    padding: '0.5rem 1rem',
    fontSize: '1rem',
    border: 'none',
    borderRadius: '4px',
    cursor: 'pointer',
  } as const,

  buttonPrimary: {
    backgroundColor: '#007bff',
    color: 'white',
  } as const,

  buttonDisabled: {
    backgroundColor: '#ccc',
    color: 'white',
    cursor: 'not-allowed' as const,
  } as const,

  buttonSecondary: {
    backgroundColor: '#6c757d',
    color: 'white',
  } as const,

  buttonSuccess: {
    backgroundColor: '#28a745',
    color: 'white',
  } as const,

  buttonDanger: {
    backgroundColor: '#dc3545',
    color: 'white',
  } as const,

  buttonSmall: {
    padding: '0.25rem 0.5rem',
    fontSize: '0.8rem',
  } as const,

  errorBox: {
    padding: '1rem',
    backgroundColor: '#fee',
    border: '1px solid #fcc',
    borderRadius: '4px',
    color: '#c00',
    marginBottom: '1rem',
  } as const,

  responseBox: {
    padding: '1rem',
    backgroundColor: '#f9f9f9',
    border: '1px solid #ddd',
    borderRadius: '4px',
    whiteSpace: 'pre-wrap' as const,
    fontFamily: 'monospace',
    fontSize: '0.9rem',
  } as const,

  contextBox: {
    marginTop: '1rem',
    padding: '0.5rem 1rem',
    backgroundColor: '#e7f3ff',
    border: '1px solid #b3d9ff',
    borderRadius: '4px',
    fontSize: '0.9rem',
    color: '#0066cc',
  } as const,

  fileManager: {
    marginTop: '2rem',
    padding: '1rem',
    backgroundColor: '#f9f9f9',
    border: '1px solid #ddd',
    borderRadius: '4px',
  } as const,

  fileManagerHeader: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: '1rem',
  } as const,

  fileManagerTitle: {
    margin: 0,
    fontSize: '1.2rem',
  } as const,

  fileManagerContext: {
    display: 'flex',
    alignItems: 'center',
    gap: '0.5rem',
  } as const,

  fileManagerContextText: {
    fontSize: '0.9rem',
    color: '#666',
  } as const,

  pathBar: {
    marginBottom: '0.5rem',
    fontSize: '0.9rem',
    color: '#666',
    padding: '0.25rem',
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    gap: '0.5rem',
  } as const,

  pathBarLeft: {
    display: 'flex',
    alignItems: 'center',
    gap: '0.5rem',
    flex: 1,
  } as const,

  fileList: {
    maxHeight: '300px',
    overflowY: 'auto' as const,
    border: '1px solid #ddd',
    borderRadius: '4px',
    backgroundColor: 'white',
  } as const,

  fileTable: {
    width: '100%',
    borderCollapse: 'collapse' as const,
  } as const,

  fileTableHeader: {
    backgroundColor: '#f0f0f0',
    borderBottom: '1px solid #ddd',
  } as const,

  fileTableHeaderCell: {
    padding: '0.5rem',
    textAlign: 'left' as const,
    fontSize: '0.9rem',
  } as const,

  fileTableRow: {
    borderBottom: '1px solid #eee',
  } as const,

  fileTableCell: {
    padding: '0.5rem',
  } as const,

  fileTableCellRight: {
    padding: '0.5rem',
    textAlign: 'right' as const,
    fontSize: '0.9rem',
    color: '#666',
  } as const,

  fileTableCellCenter: {
    padding: '0.5rem',
    textAlign: 'center' as const,
  } as const,

  loading: {
    padding: '1rem',
    textAlign: 'center' as const,
    color: '#666',
  } as const,

  empty: {
    padding: '1rem',
    textAlign: 'center' as const,
    color: '#666',
  } as const,
} as const

