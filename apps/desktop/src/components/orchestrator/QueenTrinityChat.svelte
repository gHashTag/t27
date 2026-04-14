<script lang="ts">
  //! QueenTrinityChat - Main Chat Component
  //!
  //! Displays user, assistant (Queen), and agent messages in the Orchestrator UI.

  import { onMount, onDestroy } from 'svelte';
  import type { ChatMessage, AgentMessage, UserMessage, AssistantMessage } from '$lib/backend/web';
  import {
    connectSse,
    disconnectSse,
    onMessage,
    sendUserMessage,
    sendAgentMessage,
    listMessages
  } from '$lib/backend/web';

  // Props
  export let sessionId: string = '';
  export let backendUrl: string = 'http://localhost:8082';
  export let directory: string = '/tmp/orchestrator';

  // State
  let messages: ChatMessage[] = [];
  let inputMessage: string = '';
  let isConnected = false;
  let messagesContainer: HTMLElement;
  let unsubscribe: (() => void) | null = null;

  // Agent badge colors
  const agentColors: Record<string, string> = {
    Q: '#FFD700', // Gold for Queen
    A: '#4CAF50', // Green
    B: '#2196F3', // Blue
    C: '#9C27B0', // Purple
    D: '#FF5722', // Deep Orange
    E: '#00BCD4', // Cyan
    F: '#FFC107', // Amber
    G: '#8BC34A', // Light Green
    H: '#E91E63', // Pink
    I: '#3F51B5', // Indigo
    J: '#009688', // Teal
    K: '#FF9800', // Orange
    L: '#673AB7', // Deep Purple
    M: '#795548', // Brown
    N: '#607D8B', // Blue Grey
    O: '#F44336', // Red
    P: '#CDDC39', // Lime
    R: '#9E9E9E', // Grey
    S: '#3F51B5', // Indigo
    T: '#8BC34A', // Light Green
    U: '#FFEB3B', // Yellow
    V: '#E040FB', // Purple Accent
    W: '#18FFFF', // Cyan Accent
    X: '#536DFE', // Indigo Accent
    Y: '#FF6E40', // Orange Accent
    Z: '#00E676', // Green Accent
    '27': '#E91E63', // Special 27th agent - Pink
  };

  // Get color for agent badge
  function getAgentColor(agentId: string): string {
    return agentColors[agentId] || '#9E9E9E';
  }

  // Get emoji for message type
  function getMessageEmoji(message: ChatMessage): string {
    if (message.role === 'agent') {
      const agentMsg = message as AgentMessage;
      return agentMsg.data.emoji || '💬';
    }
    return '';
  }

  // Format timestamp
  function formatTime(timestamp: string): string {
    const date = new Date(timestamp);
    return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  }

  // Get message text content
  function getMessageText(message: ChatMessage): string {
    const textContent = message.data.content?.find(c => c.type === 'text');
    return textContent?.text || '';
  }

  // Scroll to bottom of messages
  function scrollToBottom(): void {
    if (messagesContainer) {
      messagesContainer.scrollTop = messagesContainer.scrollHeight;
    }
  }

  // Handle incoming SSE message
  function handleIncomingMessage(message: ChatMessage): void {
    // Only add if not already in list
    if (!messages.find(m => m.id === message.id)) {
      messages = [...messages, message];
      scrollToBottom();
    }
  }

  // Send user message
  async function sendMessage(): Promise<void> {
    const trimmed = inputMessage.trim();
    if (!trimmed) return;

    inputMessage = '';

    try {
      // Create session if needed
      if (!sessionId) {
        const session = await createSession();
        sessionId = session.id;
      }

      const message = await sendUserMessage(sessionId, trimmed, 'root', backendUrl);
      messages = [...messages, message];
      scrollToBottom();
    } catch (error) {
      console.error('Failed to send message:', error);
      // Show error in UI
      inputMessage = trimmed; // Restore input on error
    }
  }

  // Create new session
  async function createSession(): Promise<{ id: string }> {
    const response = await fetch(`${backendUrl}/session`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        directory,
        title: 'Orchestrator Chat'
      })
    });
    if (!response.ok) {
      throw new Error(`Failed to create session: ${response.statusText}`);
    }
    return response.json();
  }

  // Load existing messages on mount
  async function loadMessages(): Promise<void> {
    if (!sessionId) return;

    try {
      const loaded = await listMessages(sessionId, backendUrl);
      messages = loaded;
      scrollToBottom();
    } catch (error) {
      console.error('Failed to load messages:', error);
    }
  }

  // Handle keyboard input
  function handleKeydown(event: KeyboardEvent): void {
    if (event.key === 'Enter' && !event.shiftKey) {
      event.preventDefault();
      sendMessage();
    }
  }

  // Lifecycle
  onMount(async () => {
    // Connect to SSE
    connectSse(
      backendUrl,
      (event) => {
        isConnected = true;
        // Handle SSE events
        if (event.payload.event_type === 'server.connected') {
          console.log('Connected to SSE stream');
        }
      },
      (error) => {
        console.error('SSE error:', error);
        isConnected = false;
      }
    );

    // Subscribe to messages
    unsubscribe = onMessage(handleIncomingMessage);

    // Load existing messages
    await loadMessages();
  });

  onDestroy(() => {
    disconnectSse();
    if (unsubscribe) {
      unsubscribe();
    }
  });
</script>

<div class="chat-container">
  <!-- Messages Area -->
  <div bind:this={messagesContainer} class="messages">
    {#each messages as message (message.id)}
      <div class="message-row" class:agent={message.role === 'agent'} class:user={message.role === 'user'} class:assistant={message.role === 'assistant'}>
        {#if message.role === 'agent'}
          {@const agentMsg = message as AgentMessage}
          <div class="agent-message">
            <span class="agent-badge" style="background-color: {getAgentColor(agentMsg.data.agent_id)}">
              {agentMsg.data.agent_id}
            </span>
            <span class="agent-emoji">{getMessageEmoji(message)}</span>
            <span class="message-text">{getMessageText(message)}</span>
            <span class="message-time">{formatTime(message.data.time.created)}</span>
          </div>
        {:else if message.role === 'user'}
          {@const userMsg = message as UserMessage}
          <div class="user-message">
            <span class="message-text">{getMessageText(message)}</span>
            <span class="message-time">{formatTime(userMsg.data.time.created)}</span>
          </div>
        {:else if message.role === 'assistant'}
          {@const asstMsg = message as AssistantMessage}
          <div class="assistant-message">
            <span class="queen-badge">Q</span>
            <span class="message-text">{getMessageText(message)}</span>
            <span class="message-time">{formatTime(asstMsg.data.time.created)}</span>
          </div>
        {/if}
      </div>
    {/each}

    {#if messages.length === 0}
      <div class="empty-state">
        <p>Orchestrator Chat</p>
        <p>Send a message to start</p>
      </div>
    {/if}
  </div>

  <!-- Input Area -->
  <div class="input-area">
    <textarea
      bind:value={inputMessage}
      on:keydown={handleKeydown}
      placeholder="Type a message..."
      rows="1"
    ></textarea>
    <button
      on:click={sendMessage}
      disabled={!inputMessage.trim()}
      class="send-button"
    >
      Send
    </button>
  </div>

  <!-- Connection Status -->
  <div class="status-bar">
    <span class="status-indicator" class:connected={isConnected}></span>
    <span class="status-text">
      {isConnected ? 'Connected' : 'Connecting...'}
    </span>
  </div>
</div>

<style>
  .chat-container {
    display: flex;
    flex-direction: column;
    height: 100%;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, sans-serif;
  }

  .messages {
    flex: 1;
    overflow-y: auto;
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .message-row {
    display: flex;
    width: 100%;
  }

  .message-row.user {
    justify-content: flex-end;
  }

  .message-row.agent,
  .message-row.assistant {
    justify-content: flex-start;
  }

  /* Agent Messages */
  .agent-message {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    background: #f5f5f5;
    border-radius: 12px;
    max-width: 80%;
  }

  .agent-badge {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    border-radius: 50%;
    font-size: 12px;
    font-weight: bold;
    color: white;
    flex-shrink: 0;
  }

  .agent-emoji {
    font-size: 16px;
  }

  /* User Messages */
  .user-message {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 16px;
    background: #007AFF;
    color: white;
    border-radius: 18px;
    max-width: 80%;
  }

  /* Assistant (Queen) Messages */
  .assistant-message {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 16px;
    background: linear-gradient(135deg, #FFD700, #FFA500);
    color: #333;
    border-radius: 18px;
    max-width: 80%;
  }

  .queen-badge {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border-radius: 50%;
    background: #333;
    color: #FFD700;
    font-size: 14px;
    font-weight: bold;
    flex-shrink: 0;
  }

  .message-text {
    flex: 1;
    word-break: break-word;
  }

  .message-time {
    font-size: 10px;
    opacity: 0.6;
    flex-shrink: 0;
  }

  .user-message .message-time {
    color: rgba(255, 255, 255, 0.7);
  }

  /* Empty State */
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: #999;
    text-align: center;
  }

  .empty-state p {
    margin: 4px 0;
  }

  /* Input Area */
  .input-area {
    display: flex;
    gap: 8px;
    padding: 16px;
    border-top: 1px solid #e0e0e0;
  }

  .input-area textarea {
    flex: 1;
    padding: 10px 14px;
    border: 1px solid #e0e0e0;
    border-radius: 20px;
    resize: none;
    font-family: inherit;
    font-size: 14px;
    max-height: 120px;
  }

  .input-area textarea:focus {
    outline: none;
    border-color: #007AFF;
  }

  .send-button {
    padding: 10px 20px;
    background: #007AFF;
    color: white;
    border: none;
    border-radius: 20px;
    font-weight: 600;
    cursor: pointer;
    transition: background 0.2s;
  }

  .send-button:hover:not(:disabled) {
    background: #0051D5;
  }

  .send-button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  /* Status Bar */
  .status-bar {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 8px 16px;
    background: #f9f9f9;
    border-top: 1px solid #e0e0e0;
    font-size: 12px;
  }

  .status-indicator {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: #ccc;
  }

  .status-indicator.connected {
    background: #4CAF50;
  }

  .status-text {
    color: #666;
  }
</style>
