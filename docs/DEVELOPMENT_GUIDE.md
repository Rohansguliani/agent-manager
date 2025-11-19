# Development Guide

## Table of Contents
1. [Development Workflow](#development-workflow)
2. [CRCCF Process](#crccf-process)
3. [Phases & Subphases](#phases--subphases)
4. [What We've Completed](#what-weve-completed)
5. [Next Steps](#next-steps)
6. [Debugging](#debugging)
7. [Code Quality Standards](#code-quality-standards)

---

## Development Workflow

### Getting Started

1. **Start Docker Compose** (Development Environment)
   ```bash
   cd agent-manager-gui
   docker-compose up
   ```
   - Backend: http://localhost:8080
   - Frontend: http://localhost:3001
   - Hot reloading enabled for both

2. **Local Development** (Without Docker)
   ```bash
   # Backend
   cd agent-manager-gui/backend
   cargo run
   
   # Frontend (new terminal)
   cd agent-manager-gui/frontend
   npm install
   npm run dev
   ```

3. **Stop Services**
   ```bash
   docker-compose down
   ```

---

## CRCCF Process

**CRCCF** is our 5-step quality assurance process that must be run after **every subphase**:

### 1. **C**reate Tests
- Write unit tests for new functionality
- Write integration tests for API endpoints
- Write component tests for UI components
- Ensure test coverage for critical paths

### 2. **R**un All Tests
```bash
# Backend tests
cd agent-manager-gui/backend
cargo test

# Frontend tests
cd agent-manager-gui/frontend
npm test -- --run
```

### 3. **C**heck Linter
```bash
# Backend linter (clippy)
cd agent-manager-gui/backend
cargo clippy -- -D warnings

# Frontend linter
cd agent-manager-gui/frontend
npm run lint
```

### 4. **C**heck Overall Style
- Review code formatting
- Ensure consistent naming conventions
- Check for code smells
- Verify component structure and organization

### 5. **F**unctionality and Modularity Review
- Test functionality manually
- Verify components work together
- Check for proper error handling
- Ensure code is modular and reusable
- Review for performance issues

**Important**: CRCCF must be completed after **every subphase** before moving to the next one.

---

## Phases & Subphases

### Phase 1: MVP Development ✅ COMPLETE
**Goal**: Basic chat functionality with persistent storage

**Subphases**:
1. ✅ Backend Chat API Setup
2. ✅ SQLite Database Integration
3. ✅ Frontend Chat UI Implementation
4. ✅ Message Streaming (SSE)
5. ✅ Conversation Management

**Status**: All subphases complete with CRCCF

---

### Phase 2: Core Features ✅ COMPLETE
**Goal**: Robust chat backend with full CRUD operations

**Subphases**:
1. ✅ Chat Database Schema & Migrations
2. ✅ Conversation CRUD API
3. ✅ Message Persistence
4. ✅ Context Management (previous messages)
5. ✅ Auto-generated Titles

**Status**: All subphases complete with CRCCF

---

### Phase 3: Frontend Chat UI Implementation ✅ COMPLETE
**Goal**: Complete chat interface with all features

**Subphases**:
1. ✅ ChatSidebar Component
2. ✅ ChatMessageList Component
3. ✅ ChatInput Component
4. ✅ ChatLayout Component
5. ✅ useChat Hook
6. ✅ useStreamingChat Hook

**Status**: All subphases complete with CRCCF

---

### Phase 4: Testing & Quality Assurance ✅ COMPLETE
**Goal**: Comprehensive test coverage and code quality

**Subphases**:
1. ✅ Component Unit Tests (28 tests)
2. ✅ Hook Unit Tests (16 tests)
3. ✅ API Integration Tests (8 tests)
4. ✅ Integration Tests (3 tests)
5. ✅ End-to-End Test Setup
6. ✅ Test Coverage & Quality

**Status**: All subphases complete with CRCCF
- **Total Tests**: 62 frontend tests, 138 backend tests
- **Coverage**: Critical paths covered
- **All tests passing**: ✅

---

### Phase 5: Production Readiness & Polish ✅ COMPLETE
**Goal**: Production-ready UI with polish and optimizations

**Subphases**:
1. ✅ Message Formatting & Display (Markdown, syntax highlighting)
2. ✅ Error Handling & User Feedback (Toast notifications, connection status)
3. ✅ Performance Optimizations (React.memo, useCallback, useMemo)
4. ✅ Accessibility & Keyboard Navigation (ARIA labels, keyboard shortcuts)
5. ✅ Advanced Chat Features (Copy, regenerate, message actions)
6. ✅ Production Configuration & Deployment (CI/CD, health checks)

**Status**: All subphases complete with CRCCF

---

### Phase 6: UI Polish & Design System ✅ COMPLETE
**Goal**: Sleek, beautiful, production-ready UI

**Subphases**:
1. ✅ Shared Theme/Design System
2. ✅ Refined Color Palette & Shadows
3. ✅ Improved Typography System
4. ✅ Enhanced Animations & Transitions
5. ✅ Polished Sidebar Design
6. ✅ Refined Message Bubbles
7. ✅ Improved Input Area
8. ✅ SVG Icons (replaced emojis)
9. ✅ Better Empty States
10. ✅ CRCCF - All Tests Passing

**Status**: All subphases complete with CRCCF

---

## What We've Completed

### Backend ✅
- ✅ Rust backend with Axum framework
- ✅ REST API endpoints for chat operations
- ✅ SQLite database with migrations
- ✅ Server-Sent Events (SSE) for streaming
- ✅ Error handling with custom AppError types
- ✅ Request ID middleware for tracing
- ✅ Health check endpoints
- ✅ CORS configuration
- ✅ Structured logging
- ✅ **PTY-based persistent subprocess architecture** - Each conversation maintains its own `gemini chat` process
- ✅ **Process lifecycle management** - Process reuse, cleanup on deletion, graceful shutdown
- ✅ **Process cleanup mechanisms** - Idle timeout support, graceful shutdown cleanup, client disconnect handling
- ✅ 142 passing tests (updated from 138)

### Frontend ✅
- ✅ React + TypeScript + Vite
- ✅ Complete chat UI with sidebar
- ✅ Message streaming with SSE
- ✅ Markdown rendering with syntax highlighting
- ✅ Toast notifications (react-hot-toast)
- ✅ Connection status indicator
- ✅ Message actions (copy, regenerate)
- ✅ Performance optimizations (memoization)
- ✅ Accessibility features (ARIA, keyboard nav)
- ✅ Beautiful dark theme design system
- ✅ SVG icon system
- ✅ 62 passing tests

### Infrastructure ✅
- ✅ Docker Compose setup
- ✅ Hot reloading for development
- ✅ Persistent storage (SQLite volume)
- ✅ GitHub Actions CI workflow
- ✅ Type checking
- ✅ Linting (ESLint, Clippy)

---

## Next Steps

### Immediate Priorities

#### Option A: Enhanced Chat Features
- **Agent Selection**: Allow users to select/switch agents within conversations
- **File Attachments**: Upload and display files in chat
- **Code Execution**: Execute code blocks directly
- **Conversation Export**: Export conversations as JSON/Markdown
- **Search**: Search within conversations

#### Option B: Agent Execution Integration
- **CLI Execution**: Make agents actually run CLI commands
- **Process Management**: Track running agent processes
- **Output Streaming**: Stream CLI output to chat
- **Agent Configuration UI**: Configure agents within chat
- **Execution History**: Show agent execution logs

#### Option C: Advanced Features
- **Multi-agent Conversations**: Use multiple agents in one conversation
- **Agent Orchestration**: Chain agents together
- **Plugin System**: Extensible agent system
- **Workflow Builder**: Visual workflow creation
- **Collaboration**: Share conversations

### Recommended Next Phase

**Phase 7: Agent Execution & Integration**
- Connect chat to agent execution system
- Display agent output in chat
- Add agent selection/switching
- Implement process management
- Add execution status indicators

---

## Debugging

### Docker Logs

#### View All Logs
```bash
cd agent-manager-gui
docker-compose logs
```

#### View Backend Logs Only
```bash
docker-compose logs backend
# Follow logs in real-time
docker-compose logs -f backend
```

#### View Frontend Logs Only
```bash
docker-compose logs frontend
# Follow logs in real-time
docker-compose logs -f frontend
```

#### View Last N Lines
```bash
docker-compose logs --tail=100 backend
```

#### View Logs Since Specific Time
```bash
docker-compose logs --since=10m backend
```

### Common Issues & Solutions

#### Issue: Backend Won't Start
```bash
# Check if port 8080 is already in use
lsof -i :8080

# Check backend logs
docker-compose logs backend

# Common causes:
# - Port already in use
# - Database migration failed
# - Missing environment variables
```

#### Issue: Frontend Can't Connect to Backend
```bash
# Check backend health
curl http://localhost:8080/api/health

# Check CORS configuration
# Verify VITE_API_URL in frontend/.env

# Check network connectivity
docker-compose ps
```

#### Issue: Database Errors
```bash
# Check database file permissions
ls -la agent-manager-gui/backend/data/

# Check database file exists
docker-compose exec backend ls -la /app/data/

# Reset database (WARNING: deletes all data)
docker-compose down -v
docker-compose up
```

#### Issue: Hot Reload Not Working
```bash
# Check volume mounts
docker-compose config

# Restart services
docker-compose restart

# Rebuild containers
docker-compose up --build
```

### Backend Debugging

#### Run Backend Locally (Without Docker)
```bash
cd agent-manager-gui/backend

# Set environment variables
export RUST_LOG=debug
export RUST_BACKTRACE=1
export PORT=8080
export HOST=0.0.0.0
export DB_PATH=./data/chat.db

# Run with debug logging
cargo run
```

#### Run Backend Tests
```bash
cd agent-manager-gui/backend
cargo test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture
```

#### Check Clippy Warnings
```bash
cd agent-manager-gui/backend
cargo clippy -- -D warnings
```

### Frontend Debugging

#### Run Frontend Locally (Without Docker)
```bash
cd agent-manager-gui/frontend

# Install dependencies
npm install

# Run dev server
npm run dev

# Set API URL
export VITE_API_URL=http://localhost:8080
```

#### Run Frontend Tests
```bash
cd agent-manager-gui/frontend
npm test

# Run in watch mode
npm test -- --watch

# Run with coverage
npm test -- --coverage
```

#### Check Type Errors
```bash
cd agent-manager-gui/frontend
npm run type-check
```

#### Check Linter Errors
```bash
cd agent-manager-gui/frontend
npm run lint

# Auto-fix where possible
npm run lint -- --fix
```

### Testing Chat from CLI

The chat system uses a **PTY-based persistent subprocess architecture** where each conversation maintains its own `gemini chat` process for the entire conversation lifecycle. This section shows how to test the chat functionality from the command line.

#### Prerequisites
- Docker containers running (`docker-compose up`)
- Backend accessible at http://localhost:8080
- Gemini CLI installed and authenticated in the Docker container

#### Create a Conversation
```bash
# Create a new conversation
curl -X POST http://localhost:8080/api/chat/conversations \
  -H 'Content-Type: application/json' \
  -d '{}' | jq

# Response includes conversation ID:
# {
#   "id": "9a5813f0-da91-4334-a4fb-262f1a7098d8",
#   "title": "New Chat",
#   "created_at": 1763490018,
#   "updated_at": 1763490018
# }
```

#### Send a Message to a Conversation
```bash
# Replace CONVERSATION_ID with the ID from above
CONVERSATION_ID="9a5813f0-da91-4334-a4fb-262f1a7098d8"

# Send a message (with timeout to prevent hanging)
curl --max-time 30 -N -X POST http://localhost:8080/api/query/stream \
  -H 'Content-Type: application/json' \
  -d "{\"conversation_id\":\"$CONVERSATION_ID\",\"query\":\"hi\"}"

# Expected response format (SSE):
# data: <response text>
# data: [DONE]
```

#### Test Multiple Messages (Process Reuse)
```bash
# Send first message
curl --max-time 30 -N -X POST http://localhost:8080/api/query/stream \
  -H 'Content-Type: application/json' \
  -d "{\"conversation_id\":\"$CONVERSATION_ID\",\"query\":\"What is 2+2?\"}"

# Send second message (should reuse same process)
curl --max-time 30 -N -X POST http://localhost:8080/api/query/stream \
  -H 'Content-Type: application/json' \
  -d "{\"conversation_id\":\"$CONVERSATION_ID\",\"query\":\"What about 3+3?\"}"

# The second message should have context from the first message
```

#### Verify Process Persistence
```bash
# Check Docker logs to see process spawn/reuse
docker-compose logs --tail=50 backend | grep -E "(Spawning|reusing|process_manager)"

# You should see:
# - "Spawning gemini chat process" on first message
# - "Process exists and is alive, reusing" on subsequent messages
```

#### Test Process Cleanup on Conversation Deletion
```bash
# Delete the conversation (this should kill the process)
curl -X DELETE http://localhost:8080/api/chat/conversations/$CONVERSATION_ID | jq

# Check logs to verify cleanup
docker-compose logs --tail=20 backend | grep -E "(Killing|Process killed|exited gracefully)"

# You should see:
# - "Killing process for conversation"
# - "Process exited gracefully" or "Process killed successfully"
```

#### Test Graceful Shutdown Cleanup
```bash
# 1. Create a few conversations and send messages
CONV1=$(curl -s -X POST http://localhost:8080/api/chat/conversations \
  -H 'Content-Type: application/json' -d '{}' | jq -r '.id')
CONV2=$(curl -s -X POST http://localhost:8080/api/chat/conversations \
  -H 'Content-Type: application/json' -d '{}' | jq -r '.id')

# 2. Send messages to create processes
curl --max-time 30 -N -X POST http://localhost:8080/api/query/stream \
  -H 'Content-Type: application/json' \
  -d "{\"conversation_id\":\"$CONV1\",\"query\":\"test\"}" > /dev/null

curl --max-time 30 -N -X POST http://localhost:8080/api/query/stream \
  -H 'Content-Type: application/json' \
  -d "{\"conversation_id\":\"$CONV2\",\"query\":\"test\"}" > /dev/null

# 3. Stop Docker Compose (should trigger graceful shutdown)
docker-compose down

# 4. Check logs to verify all processes were cleaned up
docker-compose logs backend | grep -E "(Cleaning up all processes|Killing process|All processes killed)"

# You should see:
# - "Cleaning up all processes..."
# - "Killing process for conversation" (for each conversation)
# - "All processes killed successfully"
```

#### Test Idle Timeout Cleanup (Manual)
```bash
# Note: This requires calling the cleanup method manually or via a background task
# Currently, idle cleanup is available but not automatically called

# To test manually, you would need to:
# 1. Create a conversation and send a message
# 2. Wait for the idle timeout period
# 3. Call cleanup_idle_processes() (requires code modification or API endpoint)

# For now, processes are only cleaned up on:
# - Conversation deletion
# - Server graceful shutdown
```

#### View Conversation Messages
```bash
# Get conversation with all messages
curl http://localhost:8080/api/chat/conversations/$CONVERSATION_ID | jq

# List all conversations
curl http://localhost:8080/api/chat/conversations | jq
```

#### Debug PTY Process Issues
```bash
# Check if processes are hanging
docker-compose logs --tail=100 backend | grep -E "(Reading response|Timeout|hanging)"

# Check process spawn errors
docker-compose logs backend | grep -E "(Failed to spawn|PTY|process_manager)"

# Verify Gemini CLI is available in container
docker-compose exec backend which gemini
docker-compose exec backend gemini --version

# Check if credentials are mounted
docker-compose exec backend ls -la /root/.gemini/
```

#### Common Issues

**Issue: Request Times Out**
- Check if Gemini CLI is authenticated: `docker-compose exec backend gemini auth status`
- Check Docker logs for errors: `docker-compose logs --tail=50 backend`
- Verify PTY is working: Look for "Successfully spawned gemini chat process" in logs

**Issue: Process Not Reusing**
- Check logs for "Process exists and is alive, reusing"
- Verify conversation_id is the same across requests
- Check if process crashed: Look for "Process has exited, will remove and spawn new one"

**Issue: Processes Not Cleaning Up**
- Verify `delete_conversation` is called: Check logs for "Killing process for conversation"
- Check graceful shutdown: Look for "Cleaning up all processes..." on `docker-compose down`
- Verify no orphaned processes: `docker-compose exec backend ps aux | grep gemini`

### Database Debugging

#### Connect to SQLite Database
```bash
# Inside Docker container
docker-compose exec backend sqlite3 /app/data/chat.db

# Or locally if sqlite3 is installed
sqlite3 agent-manager-gui/backend/data/chat.db
```

#### Common SQLite Commands
```sql
-- List all tables
.tables

-- View conversations
SELECT * FROM conversations;

-- View messages
SELECT * FROM messages LIMIT 10;

-- Check database schema
.schema

-- Exit
.quit
```

#### Reset Database
```bash
# Stop containers
docker-compose down

# Remove volume (WARNING: deletes all data)
docker volume rm agent-manager-gui_chat-db

# Or delete database file directly
rm agent-manager-gui/backend/data/chat.db

# Restart
docker-compose up
```

---

## Code Quality Standards

### TypeScript/React Standards

1. **Type Safety**
   - Use TypeScript strictly (no `any` unless necessary)
   - Define interfaces for all props
   - Use proper return types

2. **Component Structure**
   - Use functional components with hooks
   - Memoize expensive components (`React.memo`)
   - Use `useCallback` for event handlers
   - Use `useMemo` for computed values

3. **Styling**
   - Use centralized theme system (`src/styles/theme.ts`)
   - Consistent spacing and colors
   - Responsive design considerations

4. **Accessibility**
   - ARIA labels on interactive elements
   - Keyboard navigation support
   - Focus management
   - Screen reader compatibility

### Rust Standards

1. **Error Handling**
   - Use `Result<T, E>` for fallible operations
   - Custom error types (`AppError`)
   - Proper error propagation

2. **Code Organization**
   - Modular structure (modules in separate files)
   - Clear separation of concerns
   - Reusable utilities

3. **Documentation**
   - Doc comments for public APIs
   - Inline comments for complex logic
   - README files for modules

4. **Testing**
   - Unit tests for all functions
   - Integration tests for API endpoints
   - Test helpers for common scenarios

### Git Workflow

1. **Branch Naming**
   - `feature/description` for new features
   - `fix/description` for bug fixes
   - `refactor/description` for refactoring

2. **Commit Messages**
   - Clear, descriptive messages
   - Reference issue numbers if applicable
   - Use present tense ("Add feature" not "Added feature")

3. **Pull Requests**
   - All tests must pass
   - Code review required
   - CRCCF completed

---

## Project Structure

```
agent-manager-gui/
├── backend/
│   ├── src/
│   │   ├── api/          # API endpoints
│   │   ├── chat/         # Chat database & models
│   │   ├── config.rs     # Configuration
│   │   ├── error.rs      # Error types
│   │   ├── executor/     # Agent execution
│   │   ├── orchestrator/ # Task orchestration
│   │   ├── state/        # Application state
│   │   └── main.rs       # Entry point
│   ├── migrations/       # Database migrations
│   ├── tests/           # Integration tests
│   └── Cargo.toml       # Dependencies
│
├── frontend/
│   ├── src/
│   │   ├── components/   # React components
│   │   ├── hooks/        # Custom hooks
│   │   ├── api.ts        # API client
│   │   ├── styles/       # Theme & styles
│   │   └── App.tsx       # Main component
│   └── package.json      # Dependencies
│
├── docs/                # Documentation
├── docker-compose.yml   # Docker setup
└── README.md           # Project overview
```

---

## Environment Variables

### Backend
- `PORT`: Server port (default: 8080)
- `HOST`: Server host (default: 0.0.0.0)
- `RUST_LOG`: Log level (debug, info, warn, error)
- `RUST_BACKTRACE`: Backtrace on errors (1 = full)
- `DB_PATH`: SQLite database path (default: /app/data/chat.db)
- `DATA_DIR`: Data directory for agent files

### Frontend
- `VITE_API_URL`: Backend API URL (default: http://localhost:8080)

---

## Useful Commands

### Backend
```bash
# Run tests
cargo test

# Run with output
cargo test -- --nocapture

# Check formatting
cargo fmt --check

# Format code
cargo fmt

# Lint
cargo clippy -- -D warnings

# Build release
cargo build --release

# Run locally
cargo run
```

### Frontend
```bash
# Install dependencies
npm install

# Run dev server
npm run dev

# Build for production
npm run build

# Run tests
npm test

# Type check
npm run type-check

# Lint
npm run lint
```

### Docker
```bash
# Start services
docker-compose up

# Start in background
docker-compose up -d

# Stop services
docker-compose down

# Rebuild containers
docker-compose up --build

# View logs
docker-compose logs -f

# Execute command in container
docker-compose exec backend cargo test
docker-compose exec frontend npm test
```

---

## Troubleshooting Checklist

When something isn't working:

1. ✅ Check Docker logs: `docker-compose logs`
2. ✅ Verify services are running: `docker-compose ps`
3. ✅ Check port availability: `lsof -i :8080` or `lsof -i :3001`
4. ✅ Verify environment variables are set
5. ✅ Check database file exists and has correct permissions
6. ✅ Run tests to identify specific failures
7. ✅ Check browser console for frontend errors
8. ✅ Verify API connectivity: `curl http://localhost:8080/api/health`
9. ✅ Check CORS configuration if API calls fail
10. ✅ Review recent code changes for breaking changes

---

## Resources

- **Backend API**: http://localhost:8080/api/health
- **Frontend**: http://localhost:3001
- **GitHub Actions**: Check `.github/workflows/ci.yml`
- **Documentation**: See `docs/` folder
- **Roadmap**: See `docs/ROADMAP.txt`

---

## Quick Reference

### CRCCF Checklist
- [ ] Create Tests
- [ ] Run All Tests
- [ ] Check Linter
- [ ] Check Overall Style
- [ ] Functionality and Modularity Review

### Before Committing
- [ ] All tests pass
- [ ] Linter passes
- [ ] Type checking passes
- [ ] Build succeeds
- [ ] Manual testing completed

### Before Starting New Subphase
- [ ] Previous subphase CRCCF complete
- [ ] All tests passing
- [ ] Code reviewed
- [ ] Documentation updated

---

**Last Updated**: After PTY-based persistent subprocess architecture implementation
**Status**: All phases 1-6 complete with CRCCF ✅
**Recent Updates**: 
- PTY-based persistent subprocess architecture for Gemini CLI
- Process lifecycle management with cleanup mechanisms
- Graceful shutdown with process cleanup
- CLI testing documentation added

---

## Quick Debugging Commands

### Check if Services Are Running
```bash
docker-compose ps
```

### View Real-time Logs
```bash
# All services
docker-compose logs -f

# Backend only
docker-compose logs -f backend

# Frontend only
docker-compose logs -f frontend
```

### Restart Services
```bash
# Restart all
docker-compose restart

# Restart specific service
docker-compose restart backend
docker-compose restart frontend
```

### Check Backend Health
```bash
curl http://localhost:8080/api/health
```

### Check Database
```bash
# Enter backend container
docker-compose exec backend sh

# Check database file
ls -la /app/data/chat.db

# Connect to database
sqlite3 /app/data/chat.db
```

### Common Fixes

**Port Already in Use:**
```bash
# Find process using port 8080
lsof -i :8080

# Kill process (replace PID)
kill -9 <PID>
```

**Database Locked:**
```bash
# Stop containers
docker-compose down

# Remove volume (WARNING: deletes data)
docker volume rm agent-manager-gui_chat-db

# Restart
docker-compose up
```

**Frontend Can't Connect:**
```bash
# Check backend is running
curl http://localhost:8080/api/health

# Check environment variable
docker-compose exec frontend env | grep VITE_API_URL

# Restart frontend
docker-compose restart frontend
```

---

## Phase Summary

| Phase | Status | Description |
|-------|--------|-------------|
| Phase 1: MVP | ✅ Complete | Basic chat functionality |
| Phase 2: Core Features | ✅ Complete | Backend chat API |
| Phase 3: Frontend UI | ✅ Complete | Complete chat interface |
| Phase 4: Testing | ✅ Complete | Comprehensive test coverage |
| Phase 5: Production Ready | ✅ Complete | Polish & optimizations |
| Phase 6: UI Polish | ✅ Complete | Design system & refinements |
| Phase 7: Agent Execution | 🔜 Next | Connect chat to agent execution |

---

## Development Tips

1. **Always run CRCCF after each subphase** - Don't skip this!
2. **Check logs first** - Most issues are visible in logs
3. **Test locally before Docker** - Faster iteration
4. **Use TypeScript strictly** - Catch errors early
5. **Follow the theme system** - Don't hardcode colors/spacing
6. **Write tests as you go** - Easier than retrofitting
7. **Keep components small** - Easier to test and maintain
8. **Use meaningful names** - Code should be self-documenting

---

## Getting Help

1. Check this guide first
2. Review Docker logs
3. Check test failures
4. Review recent code changes
5. Check GitHub Issues (if applicable)

---

**Remember**: CRCCF after every subphase ensures quality and prevents technical debt!

