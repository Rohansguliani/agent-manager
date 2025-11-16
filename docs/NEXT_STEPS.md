# Next Steps - Development Plan

## Current Status ✅

**What's Working:**
- ✅ Rust backend API with Axum (REST + WebSocket)
- ✅ React frontend with TypeScript
- ✅ Agent management API (CRUD operations)
- ✅ Agent state persistence (JSON files)
- ✅ WebSocket infrastructure (basic setup)
- ✅ Docker Compose setup with hot reloading
- ✅ Error handling and validation
- ✅ Health checks and logging

**What's Missing:**
- ❌ CLI process execution (agents can't actually run yet)
- ❌ Terminal output streaming
- ❌ AutoAgents integration
- ❌ Agent execution UI in frontend

## Immediate Next Steps (Phase 4: CLI Integration)

### Priority 1: Make Agents Executable 🎯

**Goal:** Users should be able to click "Start" on an agent and see it actually run a CLI command.

#### Step 1: Implement Process Execution Backend
**File:** `backend/src/process/executor.rs` (new module)

**Tasks:**
- Create `ProcessExecutor` struct to manage CLI processes
- Use `tokio::process::Command` to spawn processes
- Store running processes in `AppState` (add `running_processes: HashMap<AgentId, ProcessHandle>`)
- Implement process lifecycle: start, stop, restart
- Handle process errors and exit codes

**API Endpoints to Add:**
- `POST /api/agents/:id/execute` - Execute agent with input/prompt
- `GET /api/agents/:id/output` - Get process output (streaming)
- `POST /api/agents/:id/kill` - Force kill a process

#### Step 2: Stream Output via WebSocket
**File:** `backend/src/websocket.rs` (enhance existing)

**Tasks:**
- Stream stdout/stderr from running processes to WebSocket clients
- Send real-time output chunks as they arrive
- Handle multiple concurrent agent executions
- Broadcast status updates when processes start/stop

**WebSocket Message Types:**
```rust
{
  "type": "agent_output",
  "agent_id": "...",
  "output": "...",
  "stream": "stdout" | "stderr"
}
```

#### Step 3: Frontend - Agent Execution UI
**Files:** `frontend/src/components/AgentCard.tsx`, `frontend/src/components/Terminal.tsx` (new)

**Tasks:**
- Add "Execute" button to agent cards
- Create terminal output component that displays streaming output
- Connect to WebSocket for real-time updates
- Show process status (Running, Idle, Error)
- Add input field to send prompts/commands to agents

**UI Components Needed:**
- `AgentCard` - Display agent with execute button
- `Terminal` - Show streaming output with auto-scroll
- `ExecuteDialog` - Modal to enter prompt/input before execution

### Priority 2: Test with Real CLI Tools 🧪

#### Step 4: Test with Simple Commands
- Test with `echo`, `ls`, `date` (simple commands)
- Verify output streaming works
- Test process cleanup on stop

#### Step 5: Test with Ollama
- Install Ollama locally (if not already)
- Create Ollama agent configuration
- Test `ollama run qwen2:7b "Hello"` execution
- Verify streaming output from Ollama

### Priority 3: AutoAgents Integration 🤖

#### Step 6: Add AutoAgents Dependency
**File:** `backend/Cargo.toml`

**Tasks:**
- Add `autoagents-core` and `autoagents-llm` dependencies
- Set up basic AutoAgents executor
- Configure Ollama provider

#### Step 7: Create CliProcessTool
**File:** `backend/src/orchestration/tools.rs` (new module)

**Tasks:**
- Implement `ToolRuntime` trait for CLI process execution
- Wrap our `ProcessExecutor` as an AutoAgents tool
- Register tool with AutoAgents executor
- Test tool execution through AutoAgents

#### Step 8: Integrate AutoAgents with API
**File:** `backend/src/api/handlers.rs`

**Tasks:**
- Add endpoint: `POST /api/agents/:id/orchestrate`
- Use AutoAgents to execute agent with ReAct pattern
- Return structured output from AutoAgents
- Handle AutoAgents errors gracefully

## Development Order (Recommended)

### Week 1: Basic Process Execution
1. ✅ **Day 1-2:** Implement `ProcessExecutor` in backend
2. ✅ **Day 3:** Add process execution API endpoints
3. ✅ **Day 4:** Test with simple CLI commands (`echo`, `ls`)
4. ✅ **Day 5:** Frontend - Add execute button and basic output display

### Week 2: Streaming & Real-time Updates
1. ✅ **Day 1-2:** Implement WebSocket streaming for process output
2. ✅ **Day 3:** Frontend - Connect WebSocket and display streaming output
3. ✅ **Day 4:** Test with longer-running commands
4. ✅ **Day 5:** Polish UI and error handling

### Week 3: AutoAgents Integration
1. ✅ **Day 1-2:** Add AutoAgents dependencies and basic setup
2. ✅ **Day 3:** Implement CliProcessTool
3. ✅ **Day 4:** Integrate AutoAgents with API
4. ✅ **Day 5:** Test with Ollama through AutoAgents

### Week 4: Testing & Polish
1. ✅ **Day 1-2:** End-to-end testing with multiple agents
2. ✅ **Day 3:** Error handling and edge cases
3. ✅ **Day 4:** UI/UX improvements
4. ✅ **Day 5:** Documentation and cleanup

## Technical Decisions Needed

### 1. Process Management Architecture
**Question:** How to store running processes?
- **Option A:** Store in `AppState` as `HashMap<AgentId, ProcessHandle>`
- **Option B:** Separate `ProcessManager` service
- **Recommendation:** Option A for MVP, migrate to B if needed

### 2. Output Streaming Strategy
**Question:** How to stream output efficiently?
- **Option A:** WebSocket per agent (one connection per agent)
- **Option B:** Single WebSocket with message routing by agent_id
- **Recommendation:** Option B (simpler, fewer connections)

### 3. AutoAgents Integration Level
**Question:** When to use AutoAgents vs direct process execution?
- **Option A:** Always use AutoAgents (even for simple commands)
- **Option B:** Direct execution for simple, AutoAgents for orchestration
- **Recommendation:** Option B (start simple, add AutoAgents for complex tasks)

## Success Criteria for Next Phase

✅ **MVP Complete When:**
- User can create an agent (Ollama, Generic, etc.)
- User can click "Execute" on an agent
- Agent spawns CLI process with configured command
- Output streams to frontend in real-time
- User can stop a running agent
- Process errors are handled gracefully

## Files to Create

### Backend
- `backend/src/process/mod.rs` - Process management module
- `backend/src/process/executor.rs` - Process execution logic
- `backend/src/process/stream.rs` - Output streaming utilities
- `backend/src/orchestration/mod.rs` - AutoAgents integration module
- `backend/src/orchestration/tools.rs` - CliProcessTool implementation

### Frontend
- `frontend/src/components/AgentCard.tsx` - Agent display with execute button
- `frontend/src/components/Terminal.tsx` - Terminal output display
- `frontend/src/components/ExecuteDialog.tsx` - Execution input dialog
- `frontend/src/hooks/useWebSocket.ts` - WebSocket hook for real-time updates

## Dependencies to Add

### Backend (`backend/Cargo.toml`)
```toml
# AutoAgents (when ready)
autoagents-core = { path = "../../AutoAgents/crates/autoagents-core" }
autoagents-llm = { path = "../../AutoAgents/crates/autoagents-llm" }

# Process management (if needed)
tokio-stream = "0.1"  # Already have this
```

### Frontend (`frontend/package.json`)
```json
// No new dependencies needed - WebSocket is built into browsers
```

## Testing Strategy

### Backend Tests
- Unit tests for `ProcessExecutor`
- Integration tests for process execution
- WebSocket streaming tests

### Frontend Tests
- Component tests for AgentCard
- WebSocket connection tests
- Terminal output rendering tests

### End-to-End Tests
- Create agent → Execute → See output
- Multiple concurrent executions
- Error scenarios (invalid command, process crash)

## Questions to Resolve

1. **Process isolation:** Should each agent run in its own working directory?
2. **Output limits:** Should we limit output buffer size? (prevent memory issues)
3. **Concurrent executions:** How many agents can run simultaneously?
4. **Input handling:** How to handle interactive CLI prompts? (e.g., password prompts)
5. **Ollama integration:** Should we auto-detect Ollama installation or require manual config?

## Resources

- [Tokio Process Documentation](https://docs.rs/tokio/latest/tokio/process/index.html)
- [AutoAgents Documentation](https://liquidos-ai.github.io/AutoAgents/)
- [Axum WebSocket Guide](https://docs.rs/axum/latest/axum/extract/ws/index.html)


