# Quick Start Guide

## Hello World Setup

This is a minimal hello world implementation to verify the stack works:
- **Backend**: Rust + Axum (REST API)
- **Frontend**: React + TypeScript + Vite
- **Orchestration**: Docker Compose

## Running the Application

### Option 1: Docker Compose (Recommended)

```bash
# Start all services
docker-compose up

# In another terminal, view logs
docker-compose logs -f

# Stop services
docker-compose down
```

**Access points:**
- Frontend: http://localhost:3000
- Backend API: http://localhost:8080
- Health Check: http://localhost:8080/api/health

### Option 2: Local Development

#### Backend
```bash
cd backend
cargo run
```

#### Frontend
```bash
cd frontend
npm install
npm run dev
```

## Gemini Authentication (CLI-Only Workflow)

We do **not** call the hosted Gemini HTTP API. Instead, the backend shells out to the official Gemini CLI that is authenticated via your Google account. Make sure you complete these steps on the host machine **before** starting Docker:

1. Install the Gemini CLI globally (see Google’s instructions, usually `npm install -g @google/gemini-cli`).
2. Run `gemini auth login` locally and finish the browser-based Google sign-in.
3. Confirm the CLI created credentials under `~/.gemini/` (files such as `oauth_creds.json` and `google_accounts.json`).

The Docker setup mounts `~/.gemini` into the backend container (`/root/.gemini`), so the running backend automatically reuses your signed-in session. Because of this flow **you do not need** (and should not set) `GEMINI_API_KEY` unless you deliberately switch to API-key mode.

## What's Working

✅ Rust backend with Axum serving REST API
✅ React frontend with TypeScript
✅ CORS enabled for development
✅ Health check endpoint
✅ Docker Compose orchestration
✅ Hot reloading for both backend and frontend

## Next Steps

See `ROADMAP.txt` for the full development plan. Next features to add:
- Agent management API endpoints
- CLI process execution
- AutoAgents integration
- WebSocket support

