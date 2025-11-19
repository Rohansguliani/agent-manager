# Chat Context Management

## Overview

This document explains how chat context is managed in the simple chat flow and how production applications typically handle conversation history.

## Current Implementation

### How Context Works

The simple chat endpoint (`/api/simple-chat`) maintains conversation context using **in-memory storage**:

1. **Conversation ID**: Each conversation gets a unique ID (UUID). If not provided, one is generated.
2. **Message History**: Messages are stored in a `HashMap<String, Vec<(role, content)>>` keyed by conversation_id.
3. **Context Formatting**: When a new message arrives:
   - Previous conversation history is retrieved
   - History is formatted as: "Previous conversation:\n\nUser: ...\nAssistant: ...\n\nCurrent question: ..."
   - The formatted prompt is sent to Gemini CLI
4. **History Limits**: Conversation history is limited to the last 20 messages (10 turns) to avoid token limits.

### Code Flow

```rust
// Backend: simple_chat.rs
1. Receive request with message + optional conversation_id
2. Generate or use conversation_id
3. Retrieve conversation history from HashMap
4. Format history + current message
5. Call Gemini CLI with formatted prompt
6. Store new message pair in history
7. Return response + conversation_id
```

```typescript
// Frontend: SimpleChat.tsx
1. User types message
2. Send message + conversation_id (if exists)
3. Receive response + conversation_id
4. Store conversation_id for next message
5. Display response
```

## Environment Variables (.env)

### Docker Compose Setup

Docker Compose **automatically loads** `.env` files from the same directory as `docker-compose.yml`.

1. **Create `.env` file** in `agent-manager-gui/` directory:
   ```bash
   GEMINI_API_KEY=your-api-key-here
   ```

2. **Docker Compose uses it**: The `docker-compose.yml` references it:
   ```yaml
   environment:
     - GEMINI_API_KEY=${GEMINI_API_KEY:-}
   ```

3. **Backend receives it**: The backend passes environment variables to Gemini CLI:
   ```rust
   cmd.envs(env::vars()); // Includes GEMINI_API_KEY from .env
   ```

### Verification

To verify `.env` is loaded:
```bash
# Check docker-compose sees the variable
docker-compose config | grep GEMINI_API_KEY

# Or check inside container
docker-compose exec backend env | grep GEMINI_API_KEY
```

## Production Context Management Patterns

### 1. **In-Memory Storage** (Current Implementation)

**Pros:**
- ✅ Fast (no database queries)
- ✅ Simple to implement
- ✅ Good for development/testing

**Cons:**
- ❌ Lost on server restart
- ❌ Doesn't scale across multiple servers
- ❌ Memory usage grows over time

**Use Cases:**
- Development environments
- Single-server deployments
- Short-lived sessions

---

### 2. **Database Storage** (Recommended for Production)

**How it works:**
- Store conversations and messages in a database (PostgreSQL, MySQL, SQLite)
- Query previous messages by `conversation_id`
- Include conversation history in API requests

**Implementation:**
```rust
// Example using SQLite (you already have this!)
async fn get_conversation_history(
    db: &ChatDb,
    conversation_id: &str,
) -> Result<Vec<Message>> {
    sqlx::query_as!(
        Message,
        "SELECT * FROM messages WHERE conversation_id = ? ORDER BY created_at",
        conversation_id
    )
    .fetch_all(&db.pool)
    .await
}
```

**Pros:**
- ✅ Persistent across restarts
- ✅ Can query/search history
- ✅ Scales better than in-memory
- ✅ Can implement conversation management (delete, archive, etc.)

**Cons:**
- ⚠️ Database queries add latency
- ⚠️ Requires database setup/maintenance

**Use Cases:**
- Production applications
- Multi-user systems
- Long-term conversation storage

---

### 3. **Redis/Cache Layer**

**How it works:**
- Store recent conversations in Redis (fast, in-memory)
- Use database for long-term storage
- Implement cache-aside pattern

**Implementation:**
```rust
// Check Redis first
if let Some(history) = redis.get(&conversation_id).await? {
    return history;
}

// Fallback to database
let history = db.get_conversation_history(&conversation_id).await?;

// Store in Redis for next time
redis.set(&conversation_id, &history, ttl: 3600).await?;
```

**Pros:**
- ✅ Very fast (sub-millisecond)
- ✅ Reduces database load
- ✅ Can set TTL for automatic cleanup
- ✅ Works across multiple servers

**Cons:**
- ⚠️ Additional infrastructure
- ⚠️ Still need database for persistence

**Use Cases:**
- High-traffic applications
- Multi-server deployments
- Real-time chat systems

---

### 4. **Client-Side Context** (Stateless)

**How it works:**
- Client sends entire conversation history with each request
- Server doesn't store anything
- Each request is independent

**Implementation:**
```typescript
// Frontend sends all messages
const request = {
  messages: [
    { role: 'user', content: 'Hello' },
    { role: 'assistant', content: 'Hi there!' },
    { role: 'user', content: 'What is 2+2?' },
  ]
};
```

**Pros:**
- ✅ Stateless (scales infinitely)
- ✅ No server storage needed
- ✅ Simple server implementation

**Cons:**
- ❌ Larger request payloads
- ❌ Client must manage history
- ❌ Can't search/analyze conversations server-side

**Use Cases:**
- Serverless architectures
- API-first designs
- When context is short-lived

---

### 5. **Hybrid Approach** (Best Practice)

**How it works:**
- Store conversations in database (long-term)
- Cache recent conversations in Redis (fast access)
- Client sends conversation_id
- Server retrieves from cache or database

**Implementation Flow:**
```
1. Client sends: { message, conversation_id }
2. Server checks Redis cache
3. If miss, query database
4. Format conversation history
5. Call LLM API
6. Store new message in database
7. Update Redis cache
8. Return response
```

**Pros:**
- ✅ Fast (Redis) + Persistent (Database)
- ✅ Scales well
- ✅ Best of both worlds

**Cons:**
- ⚠️ More complex
- ⚠️ Requires both Redis and Database

**Use Cases:**
- Production applications
- High-traffic systems
- Enterprise deployments

---

## Token Management

### Problem
LLMs have token limits (e.g., Gemini: 32K tokens). Long conversations exceed limits.

### Solutions

1. **Sliding Window** (Current Implementation)
   - Keep only last N messages
   - Drop oldest messages
   - Simple but loses early context

2. **Summarization**
   - Summarize old messages
   - Keep summary + recent messages
   - Preserves context, reduces tokens

3. **Semantic Search**
   - Store all messages in vector database
   - Retrieve only relevant past messages
   - Best context retention

4. **Hierarchical Context**
   - Summarize by topic/section
   - Keep summaries + recent messages
   - Good balance

---

## Recommendations for Your Project

### Current State (Development)
✅ **In-memory storage is fine** for now:
- Simple and fast
- Good for testing
- Easy to debug

### Next Steps (Production)
1. **Switch to Database Storage**
   - You already have `ChatDb` and SQLite setup
   - Use existing `Message` and `Conversation` models
   - Query history before calling Gemini CLI

2. **Add Redis Later** (if needed)
   - Only if you need sub-100ms response times
   - Or if running multiple backend instances

3. **Implement Token Management**
   - Add summarization for long conversations
   - Or use semantic search for better context

---

## Example: Migrating to Database Storage

```rust
// In simple_chat.rs, replace HashMap with database calls:

pub async fn simple_chat(
    State((_state, chat_db, _process_manager)): State<RouterState>,
    Json(request): Json<SimpleChatRequest>,
) -> Result<Json<SimpleChatResponse>, StatusCode> {
    let conversation_id = request.conversation_id
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    // Get conversation history from database
    let history = chat_db
        .get_messages(&conversation_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Format history
    let formatted_message = format_conversation_history(&history, &request.message);

    // Call Gemini CLI...
    // ... (rest of implementation)

    // Store new messages in database
    chat_db
        .add_message(&conversation_id, "user", &request.message)
        .await?;
    chat_db
        .add_message(&conversation_id, "assistant", &response_text)
        .await?;

    Ok(Json(SimpleChatResponse { ... }))
}
```

---

## Summary

- **Current**: In-memory HashMap (good for dev)
- **Production**: Database storage (you have the infrastructure!)
- **High-scale**: Redis + Database hybrid
- **Token limits**: Implement sliding window or summarization
- **.env**: Docker Compose automatically loads it ✅

