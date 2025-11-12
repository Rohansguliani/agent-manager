# Agent Manager

A local-first agent management system with remote access capabilities. Manage CLI-based AI agents (Ollama, Gemini CLI, Claude Code, etc.) from anywhere via a beautiful web interface.

## Architecture

- **Backend**: Rust + Axum (REST API server)
- **Frontend**: React + TypeScript + Vite
- **Orchestration**: Docker Compose

## Quick Start

### Prerequisites

- Docker and Docker Compose installed (for Docker setup)
- OR Rust and Node.js installed (for local development)

### Option 1: Running with Docker (Recommended for First Time)

**Yes, it's just one command!** Docker Compose handles everything for you.

1. **Navigate to the project directory:**
   ```bash
   cd agent-manager-gui
   ```

2. **Start everything with one command:**
   ```bash
   docker-compose up
   ```
   
   **What this does:**
   - Builds Docker images for both backend and frontend (first time only)
   - Starts the backend server on port 8080
   - Starts the frontend dev server on port 3000
   - Sets up hot reloading (code changes auto-reload)
   - Connects them on the same network
   - Waits for backend to be healthy before starting frontend

3. **Access the application:**
   - **Frontend UI**: Open your browser to **http://localhost:3000**
   - **Backend API**: http://localhost:8080
   - **Health Check**: http://localhost:8080/api/health (returns version info)

4. **Stop services:**
   - Press `Ctrl+C` in the terminal where `docker-compose up` is running
   - Or run `docker-compose down` in another terminal

**That's it!** One command (`docker-compose up`) starts everything. The frontend will automatically connect to the backend.

### Option 2: Local Development (Without Docker)

**Use this if you want to run services separately** (useful for debugging or if you don't have Docker).

**Note:** You need Rust and Node.js installed for this option.

#### Backend Setup

1. **Navigate to backend directory:**
   ```bash
   cd agent-manager-gui/backend
   ```

2. **Run the backend:**
   ```bash
   cargo run
   ```
   
   The backend will start on http://localhost:8080
   
   You should see: `🚀 Server running on http://0.0.0.0:8080`

#### Frontend Setup

1. **Open a new terminal and navigate to frontend directory:**
   ```bash
   cd agent-manager-gui/frontend
   ```

2. **Install dependencies (first time only):**
   ```bash
   npm install
   ```

3. **Start the development server:**
   ```bash
   npm run dev
   ```
   
   The frontend will start on http://localhost:3000
   
   Vite will show you the local and network URLs.

### What to Do Next

After starting the application:

1. **Open http://localhost:3000** in your browser
2. You should see the Agent Manager UI with:
   - Backend status indicator
   - Empty agents list (since no agents are created yet)
3. **Test the API** by creating an agent:
   ```bash
   curl -X POST http://localhost:8080/api/agents \
     -H "Content-Type: application/json" \
     -d '{"name": "Test Agent", "agent_type": "Generic"}'
   ```
4. **Refresh the frontend** - you should see the new agent appear!

## Project Structure

```
agent-manager-gui/
├── backend/              # Rust API server
│   ├── src/
│   │   ├── main.rs      # Axum server entry point
│   │   ├── api/         # API handlers
│   │   ├── error.rs     # Error types
│   │   ├── websocket.rs # WebSocket handlers
│   │   ├── config.rs    # Configuration
│   │   └── state/       # Agent state management
│   ├── Cargo.toml
│   └── Dockerfile.dev
├── frontend/            # React application
│   ├── src/
│   │   ├── App.tsx      # Main React component
│   │   ├── main.tsx     # React entry point
│   │   ├── api.ts       # API client
│   │   └── ErrorBoundary.tsx # Error boundary
│   ├── package.json
│   ├── vite.config.ts
│   └── Dockerfile.dev
├── docs/                # Documentation
│   ├── ROADMAP.txt      # Development roadmap
│   ├── NEXT_STEPS.md    # Immediate next steps
│   ├── QUICKSTART.md    # Quick start guide
│   └── ...
├── docker-compose.yml   # Docker orchestration
└── README.md
```

## Development

### Hot Reloading

Both backend and frontend support hot reloading when running in Docker:

- **Backend**: Uses `cargo watch` to automatically rebuild on file changes
- **Frontend**: Uses Vite's HMR (Hot Module Replacement) for instant updates

### Environment Variables

Copy `.env.example` to `.env` and customize as needed:

```bash
cp .env.example .env
```

## API Endpoints

The backend provides the following REST API endpoints:

- `GET /` - Hello world endpoint
- `GET /api/health` - Health check
- `GET /api/agents` - List all agents
- `GET /api/agents/:id` - Get a specific agent
- `POST /api/agents` - Create a new agent
- `PUT /api/agents/:id` - Update an agent
- `DELETE /api/agents/:id` - Delete an agent
- `POST /api/agents/:id/start` - Start an agent
- `POST /api/agents/:id/stop` - Stop an agent
- `GET /ws` - WebSocket endpoint for real-time updates

## Features

- ✅ Agent management API (CRUD operations)
- ✅ WebSocket support for real-time updates
- ✅ Error handling and validation
- ✅ State persistence (JSON file)
- ✅ Environment variable configuration
- ✅ Structured logging
- ✅ React frontend with error boundaries
- ✅ TypeScript API client

## Testing

### Backend Tests

Run Rust tests:
```bash
cd backend
cargo test
```

Run tests with output:
```bash
cargo test -- --nocapture
```

### Frontend Tests

Frontend tests are set up but require Jest/Vitest configuration. For now, manual testing is recommended.

## Code Quality

### Backend

- **Format code**: `cargo fmt`
- **Lint code**: `cargo clippy`
- **Check compilation**: `cargo check`

### Frontend

- **Lint code**: `npm run lint` (if configured)
- **Type check**: `npm run type-check` (if configured)

## Next Steps

**🎯 Immediate Priority:** Make agents executable - users should be able to run CLI commands and see output.

See `docs/NEXT_STEPS.md` for the detailed development plan and immediate priorities.

Future development will include:

- ✅ Agent management API (CRUD operations) - **DONE**
- ⏳ CLI process execution - **NEXT**
- ⏳ AutoAgents integration
- ⏳ Real-time agent output streaming
- ⏳ Multi-agent orchestration

For detailed plans, see:
- `docs/NEXT_STEPS.md` - Immediate next steps and development plan
- `docs/ROADMAP.txt` - Full development roadmap
