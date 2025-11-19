# Sidecar Implementation Plan

## Overview

Replace the one-shot `gemini -p` CLI calls with a persistent Node.js sidecar bridge that uses `@google/gemini-cli-core` SDK directly. This eliminates terminal parsing complexity and provides a clean JSON protocol for communication.

## Architecture

**Current**: Rust → `gemini -p` (one-shot) → Parse terminal output  
**New**: Rust → Node.js Bridge (persistent) → `@google/gemini-cli-core` SDK → JSON over stdin/stdout

### Benefits

- **Structured I/O**: JSON protocol instead of terminal parsing
- **Persistent sessions**: One process per conversation maintains state
- **No PTY complexity**: No EOF detection, prompt parsing, or buffering hacks
- **SDK-native**: Leverages Gemini CLI's internal state management

---

## Phase 1: Node.js Bridge Script

**Goal**: Create a standalone Node.js bridge that uses `@google/gemini-cli-core` SDK

### Subphase 1.1: Create Node.js Bridge Script
**File**: `agent-manager-gui/backend/bridge/gemini-bridge.js`

**Tasks**:
- Install `@google/gemini-cli-core` as dependency
- Create bridge script that:
  - Initializes `GeminiChat` instance once (maintains conversation state)
  - Listens for JSON lines on stdin
  - Processes requests and sends JSON responses to stdout
  - Handles errors gracefully
- Protocol: `{ "type": "message", "content": "..." }` → `{ "status": "success", "data": "..." }`

**Deliverable**: Working `gemini-bridge.js` script

**CRCCF Required**: ✅ After completion

---

### Subphase 1.2: Test Bridge Script Standalone
**Tests**:
- Test with simple messages
- Test with conversation context (multiple messages)
- Test error handling
- Test process lifecycle (startup, shutdown)

**Deliverable**: Verified bridge script that works standalone

**CRCCF Required**: ✅ After completion

---

## Phase 2: Rust Bridge Session

**Goal**: Create Rust struct to manage persistent Node.js bridge processes

### Subphase 2.1: Create Rust BridgeSession Struct
**File**: `agent-manager-gui/backend/src/chat/bridge_session.rs`

**Tasks**:
- Create `BridgeSession` struct:
  - Spawns Node.js bridge process (`node bridge/gemini-bridge.js`)
  - Manages stdin/stdout communication
  - Maintains process lifecycle
- Implement JSON request/response protocol
- Handle process spawning and cleanup

**Deliverable**: `BridgeSession` struct with basic process management

**CRCCF Required**: ✅ After completion

---

### Subphase 2.2: Implement JSON Protocol
**Tasks**:
- Implement `send_message()` method that:
  - Writes JSON request to stdin
  - Reads JSON response from stdout
  - Handles timeouts and errors
- Implement response parsing
- Add error handling for malformed JSON, process crashes

**Deliverable**: Full JSON protocol implementation in Rust

**CRCCF Required**: ✅ After completion

---

## Phase 3: Bridge Manager Integration

**Goal**: Integrate bridge sessions with conversation management

### Subphase 3.1: Create BridgeManager to Handle One Process Per Conversation
**File**: `agent-manager-gui/backend/src/chat/bridge_manager.rs` (update existing)

**Tasks**:
- Update `BridgeManager` to:
  - Store `HashMap<conversation_id, BridgeSession>`
  - Implement `get_or_create_session()` method
  - Handle session lifecycle (create on demand, cleanup on conversation delete)
- Implement idle timeout (kill sessions after 10 minutes of inactivity)

**Deliverable**: `BridgeManager` with per-conversation session management

**CRCCF Required**: ✅ After completion

---

### Subphase 3.2: Integrate BridgeManager with simple_chat.rs Endpoint
**File**: `agent-manager-gui/backend/src/api/simple_chat.rs`

**Tasks**:
- Replace one-shot `gemini -p` calls with `BridgeManager::send_message()`
- Remove conversation history formatting (GeminiChat handles this internally)
- Update to use persistent session per conversation
- Maintain backward compatibility with existing API

**Deliverable**: `simple_chat` endpoint using bridge approach

**CRCCF Required**: ✅ After completion

---

## Phase 4: Error Handling & Testing

**Goal**: Robust error handling and end-to-end testing

### Subphase 4.1: Add Error Handling and Process Lifecycle Management
**Tasks**:
- Implement process crash recovery (auto-restart bridge if it dies)
- Add timeout handling (request timeout, process startup timeout)
- Implement graceful shutdown (kill all bridge processes on server shutdown)
- Add logging for debugging bridge communication

**Deliverable**: Production-ready error handling

**CRCCF Required**: ✅ After completion

---

### Subphase 4.2: Test End-to-End with Frontend SimpleChat Component
**Tests**:
- Test new conversation flow
- Test multi-turn conversations (context preservation)
- Test concurrent conversations (multiple bridge processes)
- Test error scenarios (bridge crash, timeout, invalid JSON)
- Test idle timeout cleanup

**Deliverable**: Fully tested end-to-end implementation

**CRCCF Required**: ✅ After completion

---

## Success Criteria

✅ **MVP Complete When**:
- User can create a new conversation
- Messages maintain context across multiple turns
- Only one bridge process per conversation (persistent)
- Bridge processes are cleaned up on conversation delete
- Error handling works (process crashes, timeouts)
- Frontend SimpleChat component works with bridge approach

---

## Technical Details

### Bridge Script Protocol

**Request Format**:
```json
{
  "type": "message",
  "content": "User's message here",
  "model": "gemini-2.5-flash" // optional
}
```

**Response Format**:
```json
{
  "status": "success",
  "data": "Assistant's response here"
}
```

**Error Format**:
```json
{
  "status": "error",
  "message": "Error description"
}
```

### Process Management

- **One process per conversation**: `HashMap<conversation_id, BridgeSession>`
- **Idle timeout**: Kill processes after 10 minutes of inactivity
- **Cleanup**: Kill processes on conversation delete or server shutdown
- **Recovery**: Auto-restart bridge if process crashes

### Dependencies

- **Node.js**: Required for bridge script (must be installed on system)
- **@google/gemini-cli-core**: NPM package for Gemini CLI SDK
- **Rust**: Existing backend (no new dependencies needed)

---

## Migration Notes

- Old `simple_chat.rs` approach (one-shot `gemini -p`) will be replaced
- Existing conversation history in SQLite remains compatible
- Frontend SimpleChat component requires no changes
- Backward compatibility maintained at API level

