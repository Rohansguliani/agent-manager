# One Process Per Chat Session Architecture

## Overview

This document explains how we implemented the sidecar architecture where **each chat conversation gets its own persistent Node.js bridge process**. Every new chat creates a new process, and messages within the same chat reuse the same process, maintaining conversation context automatically.

## The Problem We Solved

Previously, we were using a stateless approach where each message would:
1. Call the `gemini` CLI command with the full conversation history formatted as text
2. Parse terminal output to extract the response
3. Lose all context between messages (no persistent state)

This approach had several problems:
- **No persistent state**: Had to reload and format conversation history for every message
- **Terminal parsing complexity**: Had to parse ANSI codes, prompts, and handle EOF detection
- **Context loss**: If the backend restarted, all context was lost (history had to be reloaded from DB)

## The Solution: Sidecar Architecture

We implemented a **sidecar architecture** where:
- **One persistent Node.js process per conversation**
- **Each process maintains its own conversation state** via `GeminiChat` from `@google/gemini-cli-core`
- **Processes stay alive** for the lifetime of the conversation
- **New conversations create new processes** automatically

### Architecture Flow

```
┌─────────────┐
│   Frontend  │
│  (React)    │
└──────┬──────┘
       │ HTTP Request
       │ POST /api/simple-chat
       │ { message, conversation_id }
       ▼
┌─────────────────────────────────────┐
│   Rust Backend (Axum)               │
│                                     │
│  ┌──────────────────────────────┐  │
│  │   BridgeManager              │  │
│  │  ┌────────────────────────┐  │  │
│  │  │ HashMap<conversation_id│  │  │
│  │  │         ↓              │  │  │
│  │  │    BridgeSession       │  │  │
│  │  └────────────────────────┘  │  │
│  └──────────────────────────────┘  │
└──────┬─────────────────────────────┘
       │ JSON over stdin/stdout
       │ { type: "message", content: "..." }
       ▼
┌─────────────────────────────────────┐
│   Node.js Bridge Process            │
│   (One per conversation)            │
│                                     │
│  ┌──────────────────────────────┐  │
│  │   GeminiChat instance        │  │
│  │   (maintains conversation    │  │
│  │    history internally)       │  │
│  └──────────────────────────────┘  │
│                                     │
│  ┌──────────────────────────────┐  │
│  │   @google/gemini-cli-core    │  │
│  │   SDK                        │  │
│  └──────────────────────────────┘  │
└──────┬─────────────────────────────┘
       │ API Call
       ▼
┌─────────────────────────────────────┐
│   Gemini API                        │
└─────────────────────────────────────┘
```

## Key Components

### 1. BridgeManager (`backend/src/chat/bridge_manager.rs`)

The `BridgeManager` is responsible for managing all bridge sessions. It maintains a `HashMap<conversation_id, BridgeSession>` to track one session per conversation.

**Key Methods:**

- **`get_or_create_session(conversation_id)`**: 
  - Checks if a session already exists for the conversation
  - If it exists and is running, reuses the existing session
  - If it doesn't exist or has died, creates a new session
  - Returns an `Arc<BridgeSession>` for reuse

- **`send_message(conversation_id, content, model)`**:
  - Gets or creates the session for the conversation
  - Sends the message to that session's bridge process
  - Returns the response

- **`kill_process(conversation_id)`**:
  - Kills the bridge process for a specific conversation
  - Removes the session from the HashMap

**How One Process Per Chat Works:**

```rust
// When a message arrives for conversation_id "abc-123"
let session = bridge_manager.get_or_create_session("abc-123").await?;

// First message: Creates new Node.js process for "abc-123"
// Second message (same conversation): Reuses the same process
// Third message (same conversation): Reuses the same process
```

**How New Chat Creates New Process:**

```rust
// Message arrives for conversation_id "xyz-456"
let session = bridge_manager.get_or_create_session("xyz-456").await?;

// This creates a NEW Node.js process for "xyz-456"
// Different from "abc-123" process - completely separate
```

### 2. BridgeSession (`backend/src/chat/bridge_session.rs`)

Each `BridgeSession` manages a single Node.js bridge process. It handles:
- Spawning the Node.js process
- Communication via stdin/stdout (JSON protocol)
- Process lifecycle (checking if alive, killing)
- Timeout handling (120 seconds per request)

**Process Spawning:**

```rust
pub async fn new(conversation_id: String, bridge_script_path: PathBuf) -> Result<Self, String> {
    // Spawn the Node.js bridge process
    let mut child = Command::new("node")
        .arg(&bridge_script_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    
    // Extract stdin/stdout handles for communication
    // Process stays alive until killed
}
```

**Communication Protocol:**

- **Request**: JSON line written to stdin
  ```json
  { "type": "message", "content": "Hello", "model": "gemini-2.5-flash" }
  ```

- **Response**: JSON line read from stdout
  ```json
  { "status": "success", "data": "Hi there!" }
  ```

### 3. Node.js Bridge Script (`backend/bridge/gemini-bridge.js`)

The Node.js bridge script is the actual process that runs persistently. Each process:
- Initializes `GeminiChat` once on first message
- Maintains conversation state internally (via `GeminiChat`)
- Listens for JSON requests on stdin
- Sends JSON responses on stdout

**Key Code:**

```javascript
// Module-level variables - persist across all messages in this process
let chat = null;  // One GeminiChat instance per process
let config = null;

async function initializeChat() {
  if (chat) {
    return chat;  // Reuse existing chat instance
  }
  
  // Create Config and GeminiChat instance
  config = new Config({ ... });
  await config.initialize();
  chat = new GeminiChat(config);  // This maintains conversation history
  
  return chat;
}

async function handleMessage(request) {
  // Ensure chat is initialized (only once per process)
  if (!chat) {
    await initializeChat();
  }
  
  // Send message - GeminiChat maintains history automatically
  const stream = await chat.sendMessageStream(effectiveModel, { message: content });
  
  // Collect response and return
}
```

**Why Context Is Maintained:**

- `GeminiChat` from `@google/gemini-cli-core` maintains conversation history internally
- Each `sendMessageStream()` call appends to the history automatically
- Since we reuse the same `chat` instance for all messages in the same process, context is preserved

## Process Lifecycle

### Creating a New Process

1. User sends first message to conversation "abc-123"
2. `BridgeManager.get_or_create_session("abc-123")` is called
3. No session exists, so `BridgeSession::new()` is called
4. Node.js process is spawned: `node backend/bridge/gemini-bridge.js`
5. Process starts listening on stdin for JSON requests
6. Session is stored in `BridgeManager.sessions` HashMap
7. Message is sent to the new process
8. Process initializes `GeminiChat` (lazy initialization)
9. Response is returned

### Reusing an Existing Process

1. User sends second message to conversation "abc-123"
2. `BridgeManager.get_or_create_session("abc-123")` is called
3. Session exists in HashMap, checks if process is still running
4. Process is running, so returns existing `BridgeSession`
5. Message is sent to the same process via stdin
6. Process reuses existing `GeminiChat` instance (with history)
7. Response is returned (with full context from previous messages)

### Creating a New Process for Different Conversation

1. User sends message to conversation "xyz-456" (different from "abc-123")
2. `BridgeManager.get_or_create_session("xyz-456")` is called
3. No session exists for "xyz-456"
4. **New Node.js process is spawned** (separate from "abc-123")
5. This new process has its own `GeminiChat` instance
6. This process runs independently from "abc-123" process

### Process Cleanup

- **On conversation delete**: `BridgeManager.kill_process(conversation_id)` kills the specific process
- **On server shutdown**: `BridgeManager.kill_all_processes()` kills all processes
- **On process crash**: When `BridgeSession.is_running()` detects a dead process, it's removed from HashMap and a new one is created on next message

## Context Management

### How Context Is Preserved

**Within a Single Chat (Same Process):**
- Process stays alive between messages
- `GeminiChat` instance persists in memory
- Each `sendMessageStream()` call adds to internal history
- Full conversation context is maintained automatically

**Between Different Chats (Different Processes):**
- Each conversation has its own process
- Each process has its own `GeminiChat` instance
- Processes are isolated - no shared state
- Context is separate per conversation

### SQLite Database Role

The SQLite database (`ChatDb`) is used for:
- **UI Display**: Loading conversation history for the frontend
- **Persistence**: Saving messages across backend restarts
- **Metadata**: Storing conversation titles, timestamps, etc.

**Important**: The database is NOT used for providing context to the bridge processes. Context is maintained by `GeminiChat` in memory. The database is only for UI display and persistence.

## Implementation Details

### Session Storage

```rust
pub struct BridgeManager {
    /// Map from conversation_id to BridgeSession
    /// One entry per conversation = One process per conversation
    sessions: Arc<RwLock<HashMap<String, Arc<BridgeSession>>>>,
    bridge_script_path: PathBuf,
}
```

### Session Lookup

```rust
pub async fn get_or_create_session(
    &self,
    conversation_id: &str,
) -> Result<Arc<BridgeSession>, String> {
    // Check if session exists
    {
        let sessions = self.sessions.read().await;
        if let Some(session) = sessions.get(conversation_id) {
            if session.is_running().await {
                // REUSE: Process exists and is running
                return Ok(session.clone());
            } else {
                // Process died - remove from map
                drop(sessions);
                let mut sessions = self.sessions.write().await;
                sessions.remove(conversation_id);
            }
        }
    }
    
    // CREATE: New process for this conversation
    let session = Arc::new(
        BridgeSession::new(conversation_id.to_string(), self.bridge_script_path.clone())
            .await?
    );
    
    // Store in HashMap
    {
        let mut sessions = self.sessions.write().await;
        sessions.insert(conversation_id.to_string(), session.clone());
    }
    
    Ok(session)
}
```

### Process Spawning

```rust
pub async fn new(conversation_id: String, bridge_script_path: PathBuf) -> Result<Self, String> {
    // Spawn Node.js process
    let mut child = Command::new("node")
        .arg(&bridge_script_path)
        .stdin(Stdio::piped())    // For sending JSON requests
        .stdout(Stdio::piped())   // For receiving JSON responses
        .stderr(Stdio::piped())   // For error logging
        .spawn()?;
    
    // Extract communication handles
    let stdin = child.stdin.take()?;
    let stdout = child.stdout.take()?;
    
    // Process stays alive until killed
    Ok(Self { child, stdin, stdout, conversation_id, ... })
}
```

### Message Sending

```rust
pub async fn send_message(
    &self,
    content: &str,
    model: Option<&str>,
) -> Result<String, String> {
    // Build JSON request
    let request = BridgeRequest {
        request_type: "message".to_string(),
        content: Some(content.to_string()),
        model: model.map(|s| s.to_string()),
    };
    
    // Send to stdin (same process, same GeminiChat instance)
    stdin.write_all(serde_json::to_string(&request)?.as_bytes()).await?;
    stdin.write_all(b"\n").await?;
    
    // Read response from stdout
    let response_line = stdout_reader.read_line(&mut buffer).await?;
    let response: BridgeResponse = serde_json::from_str(&response_line)?;
    
    Ok(response.data.unwrap_or_default())
}
```

## Benefits of This Architecture

1. **Persistent Context**: No need to reload conversation history from database - `GeminiChat` maintains it in memory
2. **Performance**: Faster responses (no history formatting, no DB lookups for context)
3. **Isolation**: Each conversation has its own process - crashes in one don't affect others
4. **Simplicity**: No terminal parsing, no EOF detection, clean JSON protocol
5. **Scalability**: Can handle many concurrent conversations (each has its own process)

## Testing

To verify the architecture works correctly:

1. **One Process Per Chat**:
   ```bash
   # Start two conversations
   curl -X POST http://localhost:8080/api/simple-chat \
     -H "Content-Type: application/json" \
     -d '{"message": "Hello", "conversation_id": "chat-1"}'
   
   curl -X POST http://localhost:8080/api/simple-chat \
     -H "Content-Type: application/json" \
     -d '{"message": "Hello", "conversation_id": "chat-2"}'
   
   # Check processes
   ps aux | grep "node.*gemini-bridge"
   # Should see 2 separate Node.js processes
   ```

2. **Context Preservation**:
   ```bash
   # First message
   curl -X POST http://localhost:8080/api/simple-chat \
     -H "Content-Type: application/json" \
     -d '{"message": "My name is Alice", "conversation_id": "chat-1"}'
   
   # Second message (should remember name)
   curl -X POST http://localhost:8080/api/simple-chat \
     -H "Content-Type: application/json" \
     -d '{"message": "What is my name?", "conversation_id": "chat-1"}'
   # Should respond with "Alice"
   ```

3. **Process Isolation**:
   ```bash
   # Conversation 1
   curl ... -d '{"message": "Context for chat-1", "conversation_id": "chat-1"}'
   
   # Conversation 2 (should NOT see chat-1 context)
   curl ... -d '{"message": "What context do you have?", "conversation_id": "chat-2"}'
   # Should NOT mention chat-1
   ```

## Troubleshooting

### Process Not Found
- **Symptom**: Error "Failed to spawn bridge process"
- **Cause**: Node.js not installed or bridge script path incorrect
- **Fix**: Verify Node.js is installed and `backend/bridge/gemini-bridge.js` exists

### Process Dies Unexpectedly
- **Symptom**: Error "Bridge process exited unexpectedly"
- **Cause**: Bridge process crashed (check stderr logs)
- **Fix**: `BridgeManager` automatically creates a new process on next message

### Too Many Processes
- **Symptom**: Many Node.js processes running
- **Cause**: Processes not cleaned up after conversation deletion
- **Fix**: Verify `kill_process()` is called when conversations are deleted

### Context Lost
- **Symptom**: Model doesn't remember previous messages
- **Cause**: Process was killed and recreated
- **Fix**: Check if process is being killed unintentionally (check logs for process lifecycle)

## Future Improvements

1. **Session Recovery**: On backend restart, reload conversations from DB and recreate processes
2. **Idle Timeout**: Kill processes after X minutes of inactivity
3. **Resource Limits**: Limit max number of concurrent processes
4. **Health Monitoring**: Periodic health checks for all active processes
5. **Metrics**: Track process lifecycle, message counts, response times per session

## Summary

We implemented a sidecar architecture where:
- **One persistent Node.js bridge process per conversation**
- **Every new chat creates a new process** automatically
- **Processes maintain conversation state** via `GeminiChat` from `@google/gemini-cli-core`
- **Context is preserved** within each chat session automatically
- **Processes are isolated** - each conversation has its own independent process

This architecture provides persistent context, better performance, and cleaner code compared to the previous stateless approach.

