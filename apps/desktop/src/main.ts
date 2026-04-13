//! Main entry point for Trinity Orchestrator Desktop
//!
//! Bootstraps the Svelte application.

import QueenTrinityChat from './components/orchestrator/QueenTrinityChat.svelte';

// Create root component
const app = new QueenTrinityChat({
  target: document.getElementById('app')!,
  props: {
    sessionId: '',
    backendUrl: 'http://localhost:8082',
    directory: '/tmp/orchestrator'
  }
});

export default app;
