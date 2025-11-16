// API client for backend communication

const API_URL = import.meta.env.VITE_API_URL || 'http://localhost:8080';

export type AgentType = 'Gemini' | 'ClaudeCode' | 'Generic' | { Other: string };
export type AgentStatus = 'Idle' | 'Running' | 'Stopped' | 'Error';

export interface Agent {
  id: string;
  name: string;
  agent_type: AgentType;
  status: AgentStatus;
}

export interface AgentsListResponse {
  agents: Agent[];
  count: number;
}

export interface CreateAgentRequest {
  name: string;
  agent_type: Agent['agent_type'];
}

export interface UpdateAgentRequest {
  name?: string;
  agent_type?: Agent['agent_type'];
  status?: Agent['status'];
}

export interface MessageResponse {
  message: string;
  status: string;
  version?: string; // Optional version field from backend
}

export interface QueryRequest {
  query: string;
}

export interface QueryResponse {
  response: string;
  agent_id: string;
  execution_time_ms: number;
}

export class ApiError extends Error {
  constructor(
    message: string,
    public status: number,
    public response?: unknown
  ) {
    super(message);
    this.name = 'ApiError';
  }
}

async function handleResponse<T>(response: Response): Promise<T> {
  if (!response.ok) {
    const error = await response.json().catch(() => ({ error: response.statusText }));
    throw new ApiError(
      error.error || `HTTP ${response.status}: ${response.statusText}`,
      response.status,
      error
    );
  }
  return response.json();
}

export const api = {
  // Health check
  async healthCheck(): Promise<MessageResponse> {
    const response = await fetch(`${API_URL}/api/health`);
    return handleResponse<MessageResponse>(response);
  },

  // List all agents
  async listAgents(): Promise<AgentsListResponse> {
    const response = await fetch(`${API_URL}/api/agents`);
    return handleResponse<AgentsListResponse>(response);
  },

  // Get a specific agent
  async getAgent(id: string): Promise<Agent> {
    const response = await fetch(`${API_URL}/api/agents/${id}`);
    return handleResponse<Agent>(response);
  },

  // Create a new agent
  async createAgent(request: CreateAgentRequest): Promise<Agent> {
    const response = await fetch(`${API_URL}/api/agents`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(request),
    });
    return handleResponse<Agent>(response);
  },

  // Update an agent
  async updateAgent(id: string, request: UpdateAgentRequest): Promise<Agent> {
    const response = await fetch(`${API_URL}/api/agents/${id}`, {
      method: 'PUT',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(request),
    });
    return handleResponse<Agent>(response);
  },

  // Delete an agent
  async deleteAgent(id: string): Promise<MessageResponse> {
    const response = await fetch(`${API_URL}/api/agents/${id}`, {
      method: 'DELETE',
    });
    return handleResponse<MessageResponse>(response);
  },

  // Start an agent
  async startAgent(id: string): Promise<Agent> {
    const response = await fetch(`${API_URL}/api/agents/${id}/start`, {
      method: 'POST',
    });
    return handleResponse<Agent>(response);
  },

  // Stop an agent
  async stopAgent(id: string): Promise<Agent> {
    const response = await fetch(`${API_URL}/api/agents/${id}/stop`, {
      method: 'POST',
    });
    return handleResponse<Agent>(response);
  },

  // Query an agent
  async queryAgent(id: string, query: string): Promise<QueryResponse> {
    const response = await fetch(`${API_URL}/api/agents/${id}/query`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ query }),
    });
    return handleResponse<QueryResponse>(response);
  },

  // File system API
  async listFiles(path?: string): Promise<{ files: FileInfo[]; path: string }> {
    // If path is empty string, don't include it in URL (backend will use default)
    const url = path && path !== ''
      ? `${API_URL}/api/files?path=${encodeURIComponent(path)}`
      : `${API_URL}/api/files`;
    const response = await fetch(url);
    return handleResponse<{ files: FileInfo[]; path: string }>(response);
  },

  async getWorkingDirectory(): Promise<WorkingDirectoryResponse> {
    const response = await fetch(`${API_URL}/api/files/working-directory`);
    return handleResponse<WorkingDirectoryResponse>(response);
  },

  async setWorkingDirectory(path: string | null): Promise<WorkingDirectoryResponse> {
    const response = await fetch(`${API_URL}/api/files/working-directory`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ path }),
    });
    return handleResponse<WorkingDirectoryResponse>(response);
  },

  // Orchestration API - uses SSE like query_stream
  async orchestratePoem(goal: string = ''): Promise<Response> {
    const response = await fetch(`${API_URL}/api/orchestrate/poem`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ goal }),
    });

    if (!response.ok) {
      throw new ApiError(
        `HTTP ${response.status}: ${response.statusText}`,
        response.status
      );
    }

    return response;
  },

  // Dynamic Orchestration API - uses planner agent and executes plan
  async orchestrate(goal: string): Promise<Response> {
    const response = await fetch(`${API_URL}/api/orchestrate`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ goal }),
    });

    if (!response.ok) {
      throw new ApiError(
        `HTTP ${response.status}: ${response.statusText}`,
        response.status
      );
    }

    return response;
  },
};

// File system types
export interface FileInfo {
  name: string;
  path: string;
  is_directory: boolean;
  size?: number;
  modified?: number;
}

export interface ListFilesResponse {
  files: FileInfo[];
  path: string;
}

export interface WorkingDirectoryResponse {
  path: string | null;
}

// Orchestration API types
export interface OrchestrationRequest {
  goal: string;
}

export interface OrchestrationStatus {
  step?: number; // Optional for backward compatibility
  step_id: string; // Required for parallel tracking
  message: string;
  status: 'running' | 'completed' | 'error' | 'pending'; // Added 'pending' for steps waiting on dependencies
}

// WebSocket message type
export interface WebSocketMessage {
  [key: string]: unknown;
}

// WebSocket connection helper
export function createWebSocketConnection(
  onMessage: (message: WebSocketMessage) => void,
  onError?: (error: Event) => void,
  onClose?: () => void
): WebSocket {
  const wsUrl = API_URL.replace('http://', 'ws://').replace('https://', 'wss://');
  const ws = new WebSocket(`${wsUrl}/ws`);

  ws.onmessage = (event) => {
    try {
      const data = JSON.parse(event.data) as WebSocketMessage;
      onMessage(data);
    } catch (error) {
      // Call error handler if provided, otherwise silently fail
      onError?.(error as Event);
    }
  };

  ws.onerror = (error) => {
    onError?.(error);
  };

  ws.onclose = () => {
    onClose?.();
  };

  return ws;
}

