// Phase 6.4: Settings Panel Component

import React, { useState, useEffect } from 'react'
import { api, OrchestratorConfig } from '../api'
import { styles } from '../styles/components'

interface SettingsProps {
  onClose?: () => void
}

export const Settings: React.FC<SettingsProps> = ({ onClose }) => {
  const [, setConfig] = useState<OrchestratorConfig | null>(null)
  const [loading, setLoading] = useState<boolean>(true)
  const [saving, setSaving] = useState<boolean>(false)
  const [error, setError] = useState<string | null>(null)
  const [success, setSuccess] = useState<string | null>(null)

  // Form state
  const [maxParallelTasks, setMaxParallelTasks] = useState<string>('')
  const [geminiModel, setGeminiModel] = useState<string>('')
  const [maxGoalLength, setMaxGoalLength] = useState<string>('')
  const [planTimeoutSecs, setPlanTimeoutSecs] = useState<string>('')
  const [apiKey, setApiKey] = useState<string>('') // LocalStorage only for MVP

  useEffect(() => {
    loadConfig()
  }, [])

  const loadConfig = async () => {
    setLoading(true)
    setError(null)
    try {
      const currentConfig = await api.getConfig()
      setConfig(currentConfig)
      setMaxParallelTasks(currentConfig.max_parallel_tasks.toString())
      setGeminiModel(currentConfig.gemini_model)
      setMaxGoalLength(currentConfig.max_goal_length.toString())
      setPlanTimeoutSecs(currentConfig.plan_timeout_secs.toString())
      
      // Load API key from localStorage (MVP - should use secure storage later)
      const storedKey = localStorage.getItem('GEMINI_API_KEY')
      if (storedKey) {
        setApiKey(storedKey)
      }
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to load config'
      setError(errorMessage)
    } finally {
      setLoading(false)
    }
  }

  const handleSave = async (e: React.FormEvent) => {
    e.preventDefault()
    setSaving(true)
    setError(null)
    setSuccess(null)

    try {
      // Update config
      const updatedConfig = await api.updateConfig({
        max_parallel_tasks: parseInt(maxParallelTasks, 10) || undefined,
        gemini_model: geminiModel || undefined,
        max_goal_length: parseInt(maxGoalLength, 10) || undefined,
        plan_timeout_secs: parseInt(planTimeoutSecs, 10) || undefined,
      })

      // Save API key to localStorage (MVP - should use secure storage later)
      if (apiKey) {
        localStorage.setItem('GEMINI_API_KEY', apiKey)
      } else {
        localStorage.removeItem('GEMINI_API_KEY')
      }

      setConfig(updatedConfig)
      setSuccess('Settings saved successfully!')
      
      // Clear success message after 3 seconds
      setTimeout(() => setSuccess(null), 3000)
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to save config'
      setError(errorMessage)
    } finally {
      setSaving(false)
    }
  }

  if (loading) {
    return (
      <div style={{ padding: '2rem', textAlign: 'center', color: '#666' }}>
        Loading settings...
      </div>
    )
  }

  return (
    <div style={{ padding: '2rem', maxWidth: '600px', margin: '0 auto' }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '1.5rem' }}>
        <h2 style={{ margin: 0, fontSize: '1.5rem', color: '#2c3e50' }}>Settings</h2>
        {onClose && (
          <button
            onClick={onClose}
            style={{
              ...styles.button,
              backgroundColor: '#6c757d',
              color: '#fff',
            }}
          >
            Close
          </button>
        )}
      </div>

      {error && (
        <div style={{ ...styles.errorBox, marginBottom: '1rem' }}>
          <strong>Error:</strong> {error}
        </div>
      )}

      {success && (
        <div style={{ 
          padding: '0.75rem', 
          backgroundColor: '#d4edda', 
          border: '1px solid #c3e6cb', 
          borderRadius: '4px',
          color: '#155724',
          marginBottom: '1rem',
        }}>
          {success}
        </div>
      )}

      <form onSubmit={handleSave}>
        <div style={{ marginBottom: '1.5rem' }}>
          <label style={{ display: 'block', marginBottom: '0.5rem', fontWeight: 'bold', color: '#2c3e50' }}>
            Max Parallel Tasks:
          </label>
          <input
            type="number"
            value={maxParallelTasks}
            onChange={(e) => setMaxParallelTasks(e.target.value)}
            min="1"
            max="100"
            style={{
              width: '100%',
              padding: '0.5rem',
              border: '1px solid #ddd',
              borderRadius: '4px',
              fontSize: '1rem',
            }}
            placeholder="10"
          />
          <small style={{ color: '#666', fontSize: '0.875rem' }}>
            Maximum number of tasks that can run in parallel (default: 10)
          </small>
        </div>

        <div style={{ marginBottom: '1.5rem' }}>
          <label style={{ display: 'block', marginBottom: '0.5rem', fontWeight: 'bold', color: '#2c3e50' }}>
            Gemini Model:
          </label>
          <input
            type="text"
            value={geminiModel}
            onChange={(e) => setGeminiModel(e.target.value)}
            style={{
              width: '100%',
              padding: '0.5rem',
              border: '1px solid #ddd',
              borderRadius: '4px',
              fontSize: '1rem',
            }}
            placeholder="gemini-2.5-flash"
          />
          <small style={{ color: '#666', fontSize: '0.875rem' }}>
            Gemini model to use for API calls (default: gemini-2.5-flash)
          </small>
        </div>

        <div style={{ marginBottom: '1.5rem' }}>
          <label style={{ display: 'block', marginBottom: '0.5rem', fontWeight: 'bold', color: '#2c3e50' }}>
            Max Goal Length:
          </label>
          <input
            type="number"
            value={maxGoalLength}
            onChange={(e) => setMaxGoalLength(e.target.value)}
            min="1"
            max="100000"
            style={{
              width: '100%',
              padding: '0.5rem',
              border: '1px solid #ddd',
              borderRadius: '4px',
              fontSize: '1rem',
            }}
            placeholder="10000"
          />
          <small style={{ color: '#666', fontSize: '0.875rem' }}>
            Maximum length of user goals in characters (default: 10000)
          </small>
        </div>

        <div style={{ marginBottom: '1.5rem' }}>
          <label style={{ display: 'block', marginBottom: '0.5rem', fontWeight: 'bold', color: '#2c3e50' }}>
            Plan Timeout (seconds):
          </label>
          <input
            type="number"
            value={planTimeoutSecs}
            onChange={(e) => setPlanTimeoutSecs(e.target.value)}
            min="1"
            max="3600"
            style={{
              width: '100%',
              padding: '0.5rem',
              border: '1px solid #ddd',
              borderRadius: '4px',
              fontSize: '1rem',
            }}
            placeholder="300"
          />
          <small style={{ color: '#666', fontSize: '0.875rem' }}>
            Maximum execution time for plans in seconds (default: 300)
          </small>
        </div>

        <div style={{ marginBottom: '1.5rem' }}>
          <label style={{ display: 'block', marginBottom: '0.5rem', fontWeight: 'bold', color: '#2c3e50' }}>
            Gemini API Key:
          </label>
          <input
            type="password"
            value={apiKey}
            onChange={(e) => setApiKey(e.target.value)}
            style={{
              width: '100%',
              padding: '0.5rem',
              border: '1px solid #ddd',
              borderRadius: '4px',
              fontSize: '1rem',
            }}
            placeholder="Enter your Gemini API key"
          />
          <small style={{ color: '#666', fontSize: '0.875rem' }}>
            API key stored locally in browser (MVP - should use secure storage in production)
          </small>
        </div>

        <div style={{ display: 'flex', gap: '0.5rem', justifyContent: 'flex-end' }}>
          <button
            type="button"
            onClick={loadConfig}
            disabled={saving}
            style={{
              ...styles.button,
              backgroundColor: '#6c757d',
              color: '#fff',
            }}
          >
            Reset
          </button>
          <button
            type="submit"
            disabled={saving}
            style={{
              ...styles.button,
              ...styles.buttonPrimary,
            }}
          >
            {saving ? 'Saving...' : 'Save Settings'}
          </button>
        </div>
      </form>
    </div>
  )
}

