# Simple Chat Flow Documentation

## Overview

This document describes the simplified chat flow that uses Gemini CLI locally. The flow is straightforward: **User types message → Frontend → Backend → Gemini CLI → Response → Frontend**.

## Architecture

### Files Created

1. **Backend:**
   - `backend/src/api/simple_chat.rs` - New API endpoint handler
   - Updated `backend/src/api/mod.rs` - Added simple_chat module
   - Updated `backend/src/main.rs` - Added `/api/simple-chat` route

2. **Frontend:**
   - `frontend/src/components/SimpleChat.tsx` - Simple chat UI component
   - `frontend/src/SimpleChatPage.tsx` - Page wrapper for SimpleChat
   - Updated `frontend/src/api.ts` - Added `simpleChat()` API function
   - Updated `frontend/src/main.tsx` - Switched to SimpleChatPage

## Flow Diagram

```
┌─────────────┐
│   User      │
│  (Browser)  │
└──────┬──────┘
       │ Types message
       │
       ▼
┌─────────────┐
│  Frontend   │  SimpleChat.tsx component
│  (React)    │  - User types in textarea
│             │  - Clicks Send or presses Enter
└──────┬──────┘
       │ POST /api/simple-chat
       │ { message: "..." }
       │
       ▼
┌─────────────┐
│  Backend    │  simple_chat.rs handler
│  (Rust)     │  - Receives JSON request
│             │  - Validates message
└──────┬──────┘
       │ Executes: gemini "<message>"
       │
       ▼
┌─────────────┐
│ Gemini CLI  │  Local CLI tool
│  (npm)      │  - Processes message
│             │  - Returns response
└──────┬──────┘
       │ Response text
       │
       ▼
┌─────────────┐
│  Backend    │  Returns JSON response
│  (Rust)     │  { response: "...", success: true }
└──────┬──────┘
       │ HTTP 200 OK
       │
       ▼
┌─────────────┐
│  Frontend   │  Updates UI
│  (React)    │  - Displays assistant message
│             │  - Ready for next message
└─────────────┘
```

## Detailed Flow

### 1. User Input (Frontend)

- User types a message in the textarea component
- Presses Enter (without Shift) or clicks Send button
- `handleSend()` function is called

### 2. Frontend API Call

```typescript
// frontend/src/components/SimpleChat.tsx
const response = await api.simpleChat(message);
```

- Makes POST request to `/api/simple-chat`
- Sends JSON: `{ message: "user's message" }`
- Waits for response

### 3. Backend Receives Request

```rust
// backend/src/api/simple_chat.rs
pub async fn simple_chat(
    Json(request): Json<SimpleChatRequest>,
) -> Result<Json<SimpleChatResponse>, StatusCode>
```

- Axum router matches `/api/simple-chat` POST route
- Handler validates message is not empty
- Prepares to call Gemini CLI

### 4. Backend Calls Gemini CLI

```rust
let mut cmd = Command::new("gemini");
cmd.arg(&request.message);
cmd.envs(env::vars()); // Passes through GEMINI_API_KEY
```

- Creates a new process to run `gemini` command
- Passes user message as argument
- Passes through environment variables (including `GEMINI_API_KEY`)
- Executes with 60-second timeout

### 5. Gemini CLI Processing

- Gemini CLI tool runs locally
- Sends request to Gemini API (if API key is set)
- Receives response from Gemini API
- Returns response text to stdout

### 6. Backend Processes Response

```rust
if output.status.success() {
    let response_text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(Json(SimpleChatResponse {
        response: response_text,
        success: true,
    }))
}
```

- Checks if command succeeded
- Extracts response from stdout
- Returns JSON response to frontend

### 7. Frontend Displays Response

```typescript
setMessages((prev) => [...prev, { 
    role: 'assistant', 
    content: response.response 
}]);
```

- Adds assistant message to messages array
- React re-renders UI
- Message appears in chat interface
- User can send another message

## Key Features

- **Fast**: Direct CLI call, no intermediate layers
- **Simple**: Single endpoint, straightforward flow
- **Local**: Uses Gemini CLI installed on your machine
- **Stateless**: Each request is independent (no conversation history)

## Prerequisites

1. **Gemini CLI installed:**
   ```bash
   npm install -g @google/gemini-cli
   ```

2. **GEMINI_API_KEY set:**
   ```bash
   export GEMINI_API_KEY="your-api-key"
   ```

3. **Backend running:**
   ```bash
   cd backend
   cargo run
   ```

4. **Frontend running:**
   ```bash
   cd frontend
   npm run dev
   ```

## Switching Back to Original App

To use the original complex chat interface, edit `frontend/src/main.tsx`:

```typescript
import App from './App'
// import SimpleChatPage from './SimpleChatPage'

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
)
```

## Future Enhancements

- Add conversation history/memory
- Add streaming responses
- Add error handling UI improvements
- Add loading indicators
- Add message timestamps

