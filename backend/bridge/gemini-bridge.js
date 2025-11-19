#!/usr/bin/env node

/**
 * Gemini Bridge Script
 * 
 * A sidecar bridge that uses @google/gemini-cli-core SDK to maintain
 * persistent chat sessions. Communicates with Rust backend via JSON over stdin/stdout.
 * 
 * Protocol:
 * - Input: JSON lines on stdin
 * - Output: JSON lines on stdout
 * 
 * Request format:
 *   { "type": "message", "content": "...", "model": "..." }
 * 
 * Response format:
 *   { "status": "success", "data": "..." }
 *   { "status": "error", "message": "..." }
 */

import { GeminiChat, Config, AuthType, DEFAULT_GEMINI_FLASH_MODEL } from '@google/gemini-cli-core';
import readline from 'readline';

const rl = readline.createInterface({
  input: process.stdin,
  output: process.stdout,
  terminal: false,
});

// Initialize GeminiChat instance once (maintains conversation state)
let chat = null;
let config = null;

async function initializeChat() {
  if (chat) {
    return chat;
  }

  try {
    // Create a minimal Config instance
    // For bridge usage, we use a simple config with minimal settings
    const targetDir = process.cwd();
    
    // Create Config with minimal required parameters
    // Session ID is just a unique identifier for this bridge instance
    // Use default flash model if not provided via settings
    config = new Config({
      sessionId: `bridge-${Date.now()}`,
      targetDir: targetDir,
      debugMode: false,
      model: DEFAULT_GEMINI_FLASH_MODEL, // Use default flash model
    });
    
    // Initialize the config (loads settings, tools, etc.)
    await config.initialize();
    
    // Set up authentication based on environment variables
    // This is required for the content generator to work
    if (process.env['GEMINI_API_KEY']) {
      // Use API key authentication
      await config.refreshAuth(AuthType.USE_GEMINI);
    } else if (process.env['USE_CCPA'] || process.env['GOOGLE_APPLICATION_CREDENTIALS']) {
      // Use OAuth/CCPA authentication
      await config.refreshAuth(AuthType.LOGIN_WITH_GOOGLE);
    } else {
      // Default to OAuth (will use cached credentials if available)
      await config.refreshAuth(AuthType.LOGIN_WITH_GOOGLE);
    }
    
    // Create GeminiChat instance with empty history
    // History will be maintained internally by GeminiChat
    chat = new GeminiChat(config);
    
    return chat;
  } catch (error) {
    console.error(JSON.stringify({
      status: 'error',
      message: `Failed to initialize GeminiChat: ${error.message}`,
    }));
    process.exit(1);
  }
}

async function handleMessage(request) {
  try {
    // Validate content before trying to initialize chat
    const { content, model } = request;
    
    if (!content || typeof content !== 'string' || content.trim().length === 0) {
      return {
        status: 'error',
        message: 'Invalid request: "content" must be a non-empty string',
      };
    }

    // Ensure chat is initialized (only if content is valid)
    if (!chat) {
      await initializeChat();
    }

    // Get effective model from config or use provided model
    // If model is provided, use it; otherwise use config's model or default
    const effectiveModel = model || config?.getModel() || 'gemini-2.5-flash';

    // Send message using sendMessageStream
    // We'll collect all chunks and return the full response
    let fullResponse = '';
    let stream;
    
    try {
      stream = await chat.sendMessageStream(
        effectiveModel,
        { message: content },
        `bridge-${Date.now()}`
      );
    } catch (error) {
      return {
        status: 'error',
        message: `Failed to send message: ${error.message}`,
      };
    }

    // Collect all chunks from the stream
    for await (const event of stream) {
      if (event.type === 'chunk') {
        // Extract text from chunk
        const chunk = event.value;
        if (chunk.candidates && chunk.candidates[0]?.content?.parts) {
          for (const part of chunk.candidates[0].content.parts) {
            if (part.text) {
              fullResponse += part.text;
            }
          }
        }
      } else if (event.type === 'retry') {
        // Retry signal - ignore for now, will continue with next chunk
        continue;
      }
    }

    return {
      status: 'success',
      data: fullResponse,
    };
  } catch (error) {
    return {
      status: 'error',
      message: error.message || 'Unknown error occurred',
    };
  }
}

// Process requests from stdin
rl.on('line', async (line) => {
  try {
    const request = JSON.parse(line);

    if (!request.type) {
      console.log(JSON.stringify({
        status: 'error',
        message: 'Invalid request: missing "type" field',
      }));
      return;
    }

    let response;
    switch (request.type) {
      case 'message':
        response = await handleMessage(request);
        break;
      default:
        response = {
          status: 'error',
          message: `Unknown request type: ${request.type}`,
        };
    }

    // Send response to stdout
    console.log(JSON.stringify(response));
  } catch (error) {
    // Handle JSON parse errors
    if (error instanceof SyntaxError) {
      console.log(JSON.stringify({
        status: 'error',
        message: `Invalid JSON: ${error.message}`,
      }));
    } else {
      console.log(JSON.stringify({
        status: 'error',
        message: error.message || 'Unknown error occurred',
      }));
    }
  }
});

// Don't initialize on startup - wait for first message
// This allows the bridge to start even if auth is not configured yet

// Handle process exit
process.on('SIGTERM', () => {
  process.exit(0);
});

process.on('SIGINT', () => {
  process.exit(0);
});

