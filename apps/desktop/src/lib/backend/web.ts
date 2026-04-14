//! Backend API & SSE Client
//!
//! Handles communication with Trinity Core backend via SSE and REST API.

const DEFAULT_BACKEND_URL = 'http://localhost:8082';

export interface SseEvent {
  directory: string;
  payload: {
    event_type: string;
    properties: Record<string, unknown>;
  };
}

export interface AgentMessage {
  id: string;
  role: 'agent';
  data: {
    id: string;
    session_id: string;
    parent_id: string;
    time: {
      created: string;
    };
    content: Array<{
      type: 'text' | 'thinking' | 'tool_use' | 'tool_result';
      id?: string;
      text?: string;
      thinking?: string;
      name?: string;
      input?: Record<string, unknown>;
      tool_use_id?: string;
      content?: string;
    }>;
    agent_id: string;
    message_type: string;
    emoji: string;
  };
}

export interface UserMessage {
  id: string;
  role: 'user';
  data: {
    id: string;
    session_id: string;
    parent_id: string;
    time: {
      created: string;
    };
    content: Array<{
      type: 'text' | 'thinking' | 'tool_use' | 'tool_result';
      id?: string;
      text?: string;
      thinking?: string;
      name?: string;
      input?: Record<string, unknown>;
      tool_use_id?: string;
      content?: string;
    }>;
  };
}

export interface AssistantMessage {
  id: string;
  role: 'assistant';
  data: {
    id: string;
    session_id: string;
    time: {
      created: string;
      completed?: string;
    };
    error?: {
      name: string;
      message: string;
    };
    parent_id: string;
    model_id: string;
    provider_id: string;
    mode: string;
    agent: string;
    path: {
      cwd: string;
      root: string;
    };
    cost: number;
    tokens: {
      input: number;
      output: number;
      reasoning: number;
      cache: {
        read: number;
        write: number;
      };
    };
  };
}

export type ChatMessage = AgentMessage | UserMessage | AssistantMessage;

let sseSource: EventSource | null = null;
let messageListeners: Array<(message: ChatMessage) => void> = [];

/**
 * Connect to SSE stream for real-time updates
 */
export function connectSse(
  backendUrl: string = DEFAULT_BACKEND_URL,
  onEvent: (event: SseEvent) => void,
  onError: (error: Event) => void
): void {
  if (sseSource) {
    sseSource.close();
  }

  const eventsUrl = `${backendUrl}/events`;
  sseSource = new EventSource(eventsUrl);

  sseSource.onopen = () => {
    console.log('SSE connection opened');
  };

  sseSource.onmessage = (event) => {
    try {
      const data = JSON.parse(event.data) as SseEvent;
      onEvent(data);

      // Handle agent-message events
      if (data.payload.event_type === 'message.updated') {
        const message = data.payload.properties as unknown;
        if (message && typeof message === 'object' && 'role' in message) {
          const chatMessage = message as ChatMessage;
          notifyMessageListeners(chatMessage);
        }
      }
    } catch (e) {
      console.error('Failed to parse SSE event:', e);
    }
  };

  sseSource.onerror = onError;
}

/**
 * Disconnect from SSE stream
 */
export function disconnectSse(): void {
  if (sseSource) {
    sseSource.close();
    sseSource = null;
  }
}

/**
 * Subscribe to new messages
 */
export function onMessage(listener: (message: ChatMessage) => void): () => void {
  messageListeners.push(listener);

  // Return unsubscribe function
  return () => {
    messageListeners = messageListeners.filter(l => l !== listener);
  };
}

/**
 * Notify all message listeners
 */
function notifyMessageListeners(message: ChatMessage): void {
  for (const listener of messageListeners) {
    listener(message);
  }
}

/**
 * Create a session
 */
export async function createSession(
  directory: string,
  title?: string,
  backendUrl: string = DEFAULT_BACKEND_URL
): Promise<{ id: string; title?: string }> {
  const response = await fetch(`${backendUrl}/session`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ directory, title })
  });

  if (!response.ok) {
    throw new Error(`Failed to create session: ${response.statusText}`);
  }

  return response.json();
}

/**
 * List sessions
 */
export async function listSessions(
  directory?: string,
  backendUrl: string = DEFAULT_BACKEND_URL
): Promise<Array<{ id: string; title?: string }>> {
  const url = new URL(`${backendUrl}/session`);
  if (directory) {
    url.searchParams.set('directory', directory);
  }

  const response = await fetch(url.toString());

  if (!response.ok) {
    throw new Error(`Failed to list sessions: ${response.statusText}`);
  }

  return response.json();
}

/**
 * List messages for a session
 */
export async function listMessages(
  sessionId: string,
  backendUrl: string = DEFAULT_BACKEND_URL
): Promise<ChatMessage[]> {
  const response = await fetch(`${backendUrl}/session/${sessionId}/message`);

  if (!response.ok) {
    throw new Error(`Failed to list messages: ${response.statusText}`);
  }

  return response.json();
}

/**
 * Send a user message
 */
export async function sendUserMessage(
  sessionId: string,
  content: string,
  parentId: string = 'root',
  backendUrl: string = DEFAULT_BACKEND_URL
): Promise<ChatMessage> {
  const response = await fetch(`${backendUrl}/session/${sessionId}/message`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      role: 'user',
      parent_id: parentId,
      content: [{
        type: 'text',
        id: `text_${Date.now()}`,
        text: content
      }]
    })
  });

  if (!response.ok) {
    throw new Error(`Failed to send message: ${response.statusText}`);
  }

  return response.json();
}

/**
 * Send an agent message
 */
export async function sendAgentMessage(
  sessionId: string,
  agentId: string,
  messageType: 'message' | 'test_result' | 'status' | 'error',
  content: string,
  emoji?: string,
  parentId: string = 'root',
  directory?: string,
  backendUrl: string = DEFAULT_BACKEND_URL
): Promise<ChatMessage> {
  const defaultEmojis: Record<string, string> = {
    message: '💬',
    test_result: '🧪',
    status: '📡',
    error: '❌'
  };

  const response = await fetch(`${backendUrl}/session/${sessionId}/message`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      role: 'agent',
      parent_id: parentId,
      directory,
      agent_id: agentId,
      message_type: messageType,
      emoji: emoji || defaultEmojis[messageType],
      content: [{
        type: 'text',
        id: `text_${Date.now()}`,
        text: content
      }]
    })
  });

  if (!response.ok) {
    throw new Error(`Failed to send agent message: ${response.statusText}`);
  }

  return response.json();
}

/**
 * Update message (mark complete, add cost, etc.)
 */
export async function updateMessage(
  sessionId: string,
  messageId: string,
  updates: {
    completed?: boolean;
    cost?: number;
    tokens?: {
      input: number;
      output: number;
      reasoning: number;
    };
    error?: {
      name: string;
      message: string;
    };
  },
  backendUrl: string = DEFAULT_BACKEND_URL
): Promise<void> {
  const response = await fetch(`${backendUrl}/session/${sessionId}/message/${messageId}`, {
    method: 'PATCH',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(updates)
  });

  if (!response.ok) {
    throw new Error(`Failed to update message: ${response.statusText}`);
  }
}
