#!/usr/bin/env tsx
//! Agent Bridge CLI
//!
//! CLI utility for agents (A-Z, Q, 27) to send messages
//! to the Orchestrator Chat UI via HTTP API.

interface CreateMessageRequest {
  role: 'agent';
  directory?: string;
  parent_id?: string;
  content?: Array<{
    type: 'text' | 'thinking' | 'tool_use' | 'tool_result';
    id?: string;
    text?: string;
    thinking?: string;
    name?: string;
    input?: Record<string, unknown>;
    tool_use_id?: string;
    tool_content?: string;
  }>;
  agent_id?: string;
  message_type?: string;
  emoji?: string;
}

// Parse command line arguments
const args = process.argv.slice(2);
if (args.length < 3) {
  console.error('Usage: npx tsx scripts/agent-say.ts <agent-id> <message-type> <content> [emoji]');
  console.error('');
  console.error('Arguments:');
  console.error('  agent-id      : Agent identifier (A-Z, Q, or 27)');
  console.error('  message-type   : message | test_result | status | error');
  console.error('  content        : Message content (wrap in quotes)');
  console.error('  emoji          : Optional emoji (default based on type)');
  console.error('');
  console.error('Examples:');
  console.error('  npx tsx scripts/agent-say.ts A message "Hello from Agent A" 💬');
  console.error('  npx tsx scripts/agent-say.ts Q test_result "All agents healthy" 🧪');
  console.error('  npx tsx scripts/agent-say.ts 27 status "Building spec..." 📡');
  process.exit(1);
}

const agentId = args[0];
const messageType = args[1];
const content = args[2];
const emoji = args[3];

// Validate agent ID
const validAgentIds = ['Q', ...Array.from({ length: 26 }, (_, i) => String.fromCharCode(65 + i)), '27'];
if (!validAgentIds.includes(agentId)) {
  console.error(`Invalid agent ID: ${agentId}`);
  console.error(`Valid IDs: ${validAgentIds.join(', ')}`);
  process.exit(1);
}

// Validate message type
const validTypes = ['message', 'test_result', 'status', 'error'];
if (!validTypes.includes(messageType)) {
  console.error(`Invalid message type: ${messageType}`);
  console.error(`Valid types: ${validTypes.join(', ')}`);
  process.exit(1);
}

// Get default emoji if not provided
const defaultEmojis: Record<string, string> = {
  message: '💬',
  test_result: '🧪',
  status: '📡',
  error: '❌'
};
const finalEmoji = emoji || defaultEmojis[messageType];

// Get backend URL from environment or use default
const backendUrl = process.env.TRINITY_BACKEND_URL || 'http://localhost:8082';

async function sendAgentMessage(): Promise<void> {
  try {
    // First, create a session for the agent if needed
    const sessionResponse = await fetch(`${backendUrl}/session`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        directory: `/tmp/agent-${agentId.toLowerCase()}`,
        title: `Agent ${agentId} Session`
      })
    });

    if (!sessionResponse.ok) {
      console.error(`Failed to create session: ${sessionResponse.statusText}`);
      process.exit(1);
    }

    const session = await sessionResponse.json();
    const sessionId = session.id;

    // Create agent message
    const messageRequest: CreateMessageRequest = {
      role: 'agent',
      directory: `/tmp/agent-${agentId.toLowerCase()}`,
      parent_id: 'root',
      content: [
        {
          type: 'text',
          text: `${finalEmoji} [${agentId}] ${content}`
        }
      ]
    };

    const messageResponse = await fetch(`${backendUrl}/session/${sessionId}/message`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(messageRequest)
    });

    if (!messageResponse.ok) {
      console.error(`Failed to send message: ${messageResponse.statusText}`);
      process.exit(1);
    }

    const message = await messageResponse.json();
    console.log(`✓ Message sent from Agent ${agentId}`);
    console.log(`  Type: ${messageType}`);
    console.log(`  Content: ${content}`);
    console.log(`  Message ID: ${message.data?.id || message.id || 'N/A'}`);

  } catch (error) {
    console.error(`Failed to connect to backend at ${backendUrl}`);
    console.error(error instanceof Error ? error.message : String(error));
    console.error('');
    console.error('Make sure Trinity Core backend is running:');
    console.error('  cd backend/trinity-core && cargo run --release');
    process.exit(1);
  }
}

sendAgentMessage();
