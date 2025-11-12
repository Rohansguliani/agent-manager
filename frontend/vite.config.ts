import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

export default defineConfig({
  plugins: [react()],
  server: {
    host: '0.0.0.0',
    port: 3000,
    watch: {
      usePolling: true, // Needed for Docker volume mounts
    },
    // Allow requests from Caddy reverse proxy
    allowedHosts: [
      'localhost',
      'local.agentManager.dev',
      'local.agentmanager.dev', // Allow lowercase variant
    ],
  },
  test: {
    globals: true,
    environment: 'jsdom',
    setupFiles: './src/test/setup.ts',
  },
})

