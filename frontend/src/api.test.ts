/**
 * Tests for API client
 * 
 * These tests verify that the API client correctly handles requests and responses.
 * Note: These are unit tests that mock fetch. For integration tests, see the
 * test setup in the project root.
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';
import { api, ApiError } from './api';

// Mock fetch globally
declare const global: any;
global.fetch = vi.fn();

describe('API Client', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('healthCheck', () => {
    it('should return health status', async () => {
      const mockResponse = { message: 'Backend is healthy', status: 'healthy', version: '0.1.0' };
      (global.fetch as any).mockResolvedValueOnce({
        ok: true,
        json: async () => mockResponse,
      });

      const result = await api.healthCheck();
      expect(result).toEqual(mockResponse);
      expect(global.fetch).toHaveBeenCalledWith('http://localhost:8080/api/health');
    });

    it('should throw ApiError on failure', async () => {
      (global.fetch as any).mockResolvedValueOnce({
        ok: false,
        status: 500,
        statusText: 'Internal Server Error',
        json: async () => ({ error: 'Server error' }),
      });

      await expect(api.healthCheck()).rejects.toThrow(ApiError);
    });
  });

  describe('listAgents', () => {
    it('should return list of agents', async () => {
      const mockResponse = {
        agents: [
          { id: '1', name: 'Agent 1', agent_type: 'Generic', status: 'Idle' },
        ],
        count: 1,
      };
      (global.fetch as any).mockResolvedValueOnce({
        ok: true,
        json: async () => mockResponse,
      });

      const result = await api.listAgents();
      expect(result.agents).toHaveLength(1);
      expect(result.count).toBe(1);
    });
  });

  describe('createAgent', () => {
    it('should create a new agent', async () => {
      const mockAgent = {
        id: 'new-id',
        name: 'New Agent',
        agent_type: 'Generic',
        status: 'Idle',
      };
      (global.fetch as any).mockResolvedValueOnce({
        ok: true,
        json: async () => mockAgent,
      });

      const result = await api.createAgent({
        name: 'New Agent',
        agent_type: 'Generic',
      });

      expect(result.name).toBe('New Agent');
      expect(global.fetch).toHaveBeenCalledWith(
        'http://localhost:8080/api/agents',
        expect.objectContaining({
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
        })
      );
    });
  });

  describe('queryAgent', () => {
    it('should query an agent successfully', async () => {
      const mockResponse = {
        response: 'Test response',
        agent_id: 'agent-1',
        execution_time_ms: 1234,
      };
      (global.fetch as any).mockResolvedValueOnce({
        ok: true,
        json: async () => mockResponse,
      });

      const result = await api.queryAgent('agent-1', 'What is Rust?');

      expect(result).toEqual(mockResponse);
      expect(global.fetch).toHaveBeenCalledWith(
        'http://localhost:8080/api/agents/agent-1/query',
        expect.objectContaining({
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ query: 'What is Rust?' }),
        })
      );
    });

    it('should throw ApiError on query failure', async () => {
      (global.fetch as any).mockResolvedValueOnce({
        ok: false,
        status: 500,
        statusText: 'Internal Server Error',
        json: async () => ({ error: 'Execution failed' }),
      });

      await expect(api.queryAgent('agent-1', 'test')).rejects.toThrow(ApiError);
    });

    it('should handle 404 when agent not found', async () => {
      (global.fetch as any).mockResolvedValueOnce({
        ok: false,
        status: 404,
        statusText: 'Not Found',
        json: async () => ({ error: 'Agent not found: agent-1' }),
      });

      await expect(api.queryAgent('agent-1', 'test')).rejects.toThrow(ApiError);
    });
  });
});

