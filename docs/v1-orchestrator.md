# V1 Orchestrator Implementation

## Overview

This document describes the implementation of the "V1 Orchestrator" pattern - a hard-coded orchestration workflow that chains multiple operations (Gemini query + file creation) to complete a high-level goal.

This is a **first-pass implementation** designed to validate the orchestration pattern before building a generic orchestrator. The architecture is intentionally modular to make future refactoring straightforward.

## Architecture

### Design Principles

1. **Modularity**: Primitives are separated from orchestration logic
2. **Reusability**: Primitives can be used by multiple workflows
3. **Testability**: Each component can be tested independently
4. **Refactorability**: Easy to extract orchestration logic into a generic system

### Component Structure

```
backend/src/
├── orchestrator/
│   ├── mod.rs              # Module declaration
│   └── primitives.rs       # Reusable primitives (internal_run_gemini, internal_create_file)
├── api/
│   └── orchestrator.rs     # Hard-coded orchestration endpoint (orchestrate_poem)
└── services/
    └── files.rs            # FileService::write_file (new primitive)

frontend/src/
├── hooks/
│   └── useOrchestrator.ts  # React hook for SSE parsing
├── api.ts                  # API client (orchestratePoem method)
└── App.tsx                 # UI for orchestration
```

## Backend Implementation

### 1. Primitives Module (`orchestrator/primitives.rs`)

**Purpose**: Reusable building blocks that wrap existing services.

#### `internal_run_gemini(state, prompt) -> Result<String, AppError>`

- **What it does**: Runs Gemini CLI with a prompt and returns the full result (non-streaming)
- **Wraps**: `CliExecutor.execute()`
- **Features**:
  - Automatically finds or creates Gemini agent with proper context
  - Applies working directory from app state
  - Waits for complete result before returning

#### `internal_create_file(file_path, content, working_dir) -> Result<String, AppError>`

- **What it does**: Creates or writes a file with content
- **Wraps**: `FileService::write_file()`
- **Features**:
  - Handles relative/absolute paths
  - Creates parent directories if needed
  - Returns canonicalized absolute path

### 2. Orchestration Endpoint (`api/orchestrator.rs`)

**Purpose**: Hard-coded orchestration workflow that composes primitives.

#### `orchestrate_poem(State, Json<OrchestrationRequest>) -> Result<Response, AppError>`

**Flow**:
1. Get working directory from app state
2. Step 1: Run `internal_run_gemini()` to generate poem
3. Step 2: Run `internal_create_file()` to save poem to `poem.txt`
4. Stream status updates via SSE

**Status Updates** (sent via SSE):
- Step 1: "Task 1: Asking Gemini for a poem..."
- Step 2: "Task 2: Saving poem to 'poem.txt'... (Generated X characters)"
- Step 3: "Done! Poem saved to: /path/to/poem.txt"

**Error Handling**:
- If Gemini fails: Streams error at Step 1
- If file save fails: Streams error at Step 2
- Both errors set `status: "error"` in OrchestrationStatus

### 3. FileService Enhancement (`services/files.rs`)

**Added**: `write_file(file_path, content, working_dir) -> Result<PathBuf, AppError>`

- Creates parent directories if needed
- Handles relative paths (resolved against working_dir)
- Returns canonicalized absolute path
- Includes comprehensive error handling

## Frontend Implementation

### 1. API Client (`api.ts`)

**Added**:
- `OrchestrationRequest` interface: `{ goal: string }`
- `OrchestrationStatus` interface: `{ step: number, message: string, status: string }`
- `api.orchestratePoem(goal: string) -> Promise<Response>`

Returns raw `Response` object (not JSON) because it uses SSE streaming.

### 2. useOrchestrator Hook (`hooks/useOrchestrator.ts`)

**Purpose**: Encapsulates SSE parsing and state management for orchestration.

**Returns**:
- `status: OrchestrationStatus | null` - Current step status
- `running: boolean` - Whether orchestration is in progress
- `error: string | null` - Error message if failed
- `runOrchestration(goal?: string)` - Function to start orchestration
- `clearStatus()` - Function to reset state

**SSE Parsing**:
- Parses `data: {...}` lines as JSON
- Updates `status` state on each step
- Handles `[DONE]` and `[ERROR]` special messages
- Sets `running: false` when completed or errored

### 3. UI (`App.tsx`)

**Added**: V1 Orchestrator section with:
- Button to trigger orchestration
- Real-time status display (color-coded by status)
- Error display if orchestration fails
- Disabled state during execution

## Data Flow

```
User clicks "Run Poem Orchestration"
  ↓
Frontend: useOrchestrator.runOrchestration()
  ↓
Frontend: api.orchestratePoem() → POST /api/orchestrate/poem
  ↓
Backend: orchestrate_poem() handler
  ↓
Backend: Stream Step 1 status via SSE
  ↓
Backend: internal_run_gemini() → CliExecutor → Gemini CLI
  ↓
Backend: Stream Step 2 status via SSE
  ↓
Backend: internal_create_file() → FileService.write_file()
  ↓
Backend: Stream Step 3 (completed) status via SSE
  ↓
Frontend: useOrchestrator parses SSE, updates state
  ↓
UI: Displays status updates in real-time
```

## API Endpoint

### POST `/api/orchestrate/poem`

**Request**:
```json
{
  "goal": "Write a 4-line poem about Rust"
}
```

**Response**: SSE stream with status updates:
```
data: {"step": 1, "message": "Task 1: Asking Gemini for a poem...", "status": "running"}

data: {"step": 2, "message": "Task 2: Saving poem to 'poem.txt'... (Generated 123 characters)", "status": "running"}

data: {"step": 3, "message": "Done! Poem saved to: /host/home/dev/poem.txt", "status": "completed"}
```

## Testing

### Backend Tests

1. **FileService::write_file**:
   - `test_write_file_simple`: Tests basic file creation
   - `test_write_file_with_working_dir`: Tests relative path resolution

2. **Orchestrator primitives** (future):
   - Test `internal_run_gemini` with mock executor
   - Test `internal_create_file` with mock FileService

3. **Orchestration endpoint** (future):
   - Integration test for full workflow
   - Error handling tests

### Frontend Tests

1. **useOrchestrator hook** (future):
   - Test SSE parsing
   - Test state updates
   - Test error handling

## Why This Architecture?

### Separation of Concerns

- **Primitives** (`orchestrator/primitives.rs`): Reusable building blocks
- **Orchestration** (`api/orchestrator.rs`): Hard-coded workflow composition
- **Services** (`services/files.rs`): Business logic for file operations

### Easy Refactoring Path

When building V2 (generic orchestrator):

1. **Keep primitives**: They're already reusable
2. **Extract orchestration logic**: Move `orchestrate_poem` logic to a generic `Orchestrator` struct
3. **Add workflow DSL**: Create a way to define workflows declaratively
4. **Keep SSE pattern**: Status streaming can be reused for any workflow

### Benefits

✅ **Fast to implement**: One endpoint, minimal new code  
✅ **Easy to test**: Each component is independently testable  
✅ **Validates pattern**: Proves orchestration works before building framework  
✅ **Clean separation**: Primitives vs. orchestration logic  
✅ **Reusable primitives**: Can be used by other workflows  

## Limitations (By Design)

This is a **V1 implementation** with intentional limitations:

1. **Hard-coded workflow**: Only supports "poem + save" pattern
2. **Single endpoint**: `/api/orchestrate/poem` is specific to this workflow
3. **No generic orchestrator**: Can't dynamically compose workflows yet
4. **No planning**: Doesn't use AI to decide steps (that's V2)

## Next Steps (V2)

1. **Generic Orchestrator**: Extract orchestration logic into reusable `Orchestrator` struct
2. **Workflow DSL**: Define workflows declaratively (JSON/YAML config)
3. **Dynamic Composition**: Allow workflows to be defined at runtime
4. **Planning Agent**: Use AI to break down goals into steps
5. **Conditional Logic**: Support if/then/else in workflows
6. **Parallel Execution**: Support multiple steps running simultaneously

## Files Changed/Added

### Backend
- ✅ `services/files.rs` - Added `write_file` method
- ✅ `orchestrator/mod.rs` - New module
- ✅ `orchestrator/primitives.rs` - New primitives module
- ✅ `api/orchestrator.rs` - New orchestration endpoint
- ✅ `api/mod.rs` - Added orchestrator module
- ✅ `main.rs` - Added orchestrator module and route

### Frontend
- ✅ `api.ts` - Added orchestration types and method
- ✅ `hooks/useOrchestrator.ts` - New hook for orchestration
- ✅ `App.tsx` - Added orchestration UI

## Usage Example

```typescript
// Frontend
const { status, running, error, runOrchestration } = useOrchestrator()

// Click button to run orchestration
<button onClick={() => runOrchestration('Write a poem about Rust')}>
  Run Orchestration
</button>

// Status updates appear automatically
{status && <div>Step {status.step}: {status.message}</div>}
```

```rust
// Backend: Primitives can be used independently
let poem = internal_run_gemini(&state, "create a poem").await?;
let file_path = internal_create_file("poem.txt", &poem, working_dir).await?;
```

## Conclusion

This V1 implementation successfully validates the orchestration pattern while maintaining clean, modular code that's easy to refactor. The primitives are reusable, the orchestration is simple to understand, and the architecture supports future evolution to a generic orchestrator.

