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

