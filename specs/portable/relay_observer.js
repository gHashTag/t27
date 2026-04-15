// SPDX-License-Identifier: Apache-2.0
// relay_observer.js 0 WebSocket Relay Observer for BrowserOS A2A Integration
// Ring 32 — Cloud Orchestration
// 12 + 1/34 = 3 | TRINITY
// JavaScript implementation of relay_observer.t27 specification

// ============================================================================
// Constants - Connection States
// ============================================================================

const WS_READY_STATE = 0;
const WS_CONNECTING_STATE = 1;
const WS_ERROR_STATE = 2;
const WS_CLOSED_STATE = 3;

const NEGONE = -1;
const ZERO = 0;
const ONE = 1;

const MESSAGE_TYPE_EVENT = 0;
const MESSAGE_TYPE_DATA = 1;
const MESSAGE_TYPE_CONTROL = 2;

const TYPE_BYTE_TO_NAME = {
    [MESSAGE_TYPE_EVENT]: 'event',
    [MESSAGE_TYPE_DATA]: 'data',
    [MESSAGE_TYPE_CONTROL]: 'control'
};

const TYPE_NAME_TO_BYTE = Object.fromEntries(
    Object.entries(TYPE_BYTE_TO_NAME).map(([name, value]) => [value, name])
);

// ============================================================================
// Types - WebSocket States
// ============================================================================

const WebSocketState = {
    READY: WS_READY_STATE,
    CONNECTING: WS_CONNECTING_STATE,
    ERROR: WS_ERROR_STATE,
    CLOSED: WS_CLOSED_STATE
};

// ============================================================================
// Types - Message Types
// ============================================================================

const MessageType = {
    EVENT: MESSAGE_TYPE_EVENT,
    DATA: MESSAGE_TYPE_DATA,
    CONTROL: MESSAGE_TYPE_CONTROL
};

// ============================================================================
// Classes - WebSocket Message
// ============================================================================

class WebSocketMessage {
    /**
     * Create a new WebSocket message
     * @param {number} type - Message type (event, data, control)
     * @param {Uint8Array} data - Message payload
     * @param {number} timestamp - Message timestamp
     */
    constructor(type, data = new Uint8Array(0), timestamp = Date.now()) {
        this.type = type;
        this.data = data;
        this.timestamp = timestamp;
    }

    /**
     * Convert to JSON representation
     */
    toJSON() {
        return {
            type: TYPE_BYTE_TO_NAME[this.type],
            data: Array.from(this.data).map(b => b),
            timestamp: this.timestamp
        };
    }

    /**
     * Create event message
     * @param {string} eventData - Event data string
     */
    static createEvent(eventData) {
        const encoder = new TextEncoder();
        const typeByte = MESSAGE_TYPE_EVENT;
        const dataBytes = encoder.encode(eventData);
        const message = new Uint8Array(1 + dataBytes.length);
        message[0] = typeByte;
        for (let i = 0; i < dataBytes.length; i++) {
            message[i + 1] = dataBytes[i];
        }
        return new WebSocketMessage(typeByte, message);
    }

    /**
     * Create data message
     * @param {string|object} dataPayload - Data payload
     */
    static createData(dataPayload) {
        let payloadBytes;
        if (typeof dataPayload === 'string') {
            const encoder = new TextEncoder();
            payloadBytes = Array.from(encoder.encode(dataPayload));
        } else if (typeof dataPayload === 'object') {
            const jsonString = JSON.stringify(dataPayload);
            const encoder = new TextEncoder();
            payloadBytes = Array.from(encoder.encode(jsonString));
        } else {
            payloadBytes = new Uint8Array(0);
        }

        const typeByte = MESSAGE_TYPE_DATA;
        const message = new Uint8Array(1 + payloadBytes.length);
        message[0] = typeByte;
        for (let i = 0; i < payloadBytes.length; i++) {
            message[i + 1] = payloadBytes[i];
        }
        return new WebSocketMessage(typeByte, message);
    }

    /**
     * Create control message
     * @param {string} controlData - Control data string
     */
    static createControl(controlData) {
        const encoder = new TextEncoder();
        const dataBytes = Array.from(encoder.encode(controlData));
        const typeByte = MESSAGE_TYPE_CONTROL;
        const message = new Uint8Array(1 + dataBytes.length);
        message[0] = typeByte;
        for (let i = 0; i < dataBytes.length; i++) {
            message[i + 1] = dataBytes[i];
        }
        return new WebSocketMessage(typeByte, message);
    }
}

// ============================================================================
// Classes - Observer Config
// ============================================================================

class ObserverConfig {
    /**
     * Create observer configuration
     * @param {object} options - Configuration options
     * @param {string} options.serverUrl - WebSocket server URL
     * @param {string} options.agentName - Agent identifier
     * @param {number} options.reconnectDelay - Milliseconds between reconnect attempts (default: 3000)
     * @param {number} options.maxReconnectAttempts - Maximum reconnect attempts (default: 10)
     */
    constructor({
        serverUrl = 'ws://localhost:3001',
        agentName = '',
        reconnectDelay = 3000,
        maxReconnectAttempts = 10
    } = {}) {
        this.serverUrl = serverUrl || 'ws://localhost:3001';
        this.agentName = agentName || '';
        this.reconnectDelay = reconnectDelay;
        this.maxReconnectAttempts = maxReconnectAttempts;
    }

    /**
     * Validate configuration
     */
    isValid() {
        return this.serverUrl.length > 0 && this.agentName.length > 0;
    }

    /**
     * Convert to JSON
     */
    toJSON() {
        return {
            serverUrl: this.serverUrl,
            agentName: this.agentName,
            reconnectDelay: this.reconnectDelay,
            maxReconnectAttempts: this.maxReconnectAttempts
        };
    }
}

// ============================================================================
// Classes - Relay Observer
// ============================================================================

export class RelayObserver {
    /**
     * Create a new Relay Observer
     * @param {ObserverConfig} config - Observer configuration
     */
    constructor(config) {
        if (!config.isValid()) {
            // Return default config for empty values
            this.config = new ObserverConfig({});
        } else {
            this.config = config;
        }

        this.ws = null;
        this.state = WebSocketState.CLOSED;
        this.reconnectAttempts = 0;
        this.messageQueue = [];
        this.eventHandlers = new Map();

        // Auto-reconnect timer
        this.reconnectTimer = null;
    }

    /**
     * Initialize the observer
     */
    init() {
        if (this.state === WebSocketState.CLOSED) {
            this.connect();
        }
    }

    /**
     * Connect to WebSocket server
     */
    connect() {
        if (this.ws) {
            this.ws.close();
        }

        this.state = WebSocketState.CONNECTING;
        this.ws = new WebSocket(this.config.serverUrl);
        this.ws.binaryType = 'arraybuffer';

        this.ws.onopen = () => {
            this.state = WebSocketState.READY;
            this.reconnectAttempts = 0;
            console.log('[RelayObserver] Connected to', this.config.serverUrl);
            this.processQueue();
        };

        this.ws.onmessage = (event) => {
            this.handleMessage(event.data);
        };

        this.ws.onerror = (error) => {
            console.error('[RelayObserver] WebSocket error:', error);
            this.state = WebSocketState.ERROR;
        };

        this.ws.onclose = (event) => {
            console.log('[RelayObserver] Connection closed, code:', event.code, 'reason:', event.reason);
            this.state = WebSocketState.CLOSED;
            this.ws = null;
            this.scheduleReconnect();
        };
    }

    /**
     * Disconnect from WebSocket server
     */
    disconnect() {
        if (this.reconnectTimer) {
            clearTimeout(this.reconnectTimer);
            this.reconnectTimer = null;
        }

        if (this.ws) {
            this.ws.close();
            this.ws = null;
        }

        this.state = WebSocketState.CLOSED;
    }

    /**
     * Handle incoming WebSocket message
     * @param {ArrayBuffer} data - Raw message data
     */
    handleMessage(data) {
        try {
            const uint8Array = new Uint8Array(data);
            const messageType = TYPE_BYTE_TO_NAME[uint8Array[0]];

            switch (messageType) {
                case 'event':
                    this.handleEventMessage(uint8Array);
                    break;
                case 'data':
                    this.handleDataMessage(uint8Array);
                    break;
                case 'control':
                    this.handleControlMessage(uint8Array);
                    break;
                default:
                    console.warn('[RelayObserver] Unknown message type:', messageType);
                    break;
            }
        } catch (error) {
            console.error('[RelayObserver] Error handling message:', error);
        }
    }

    /**
     * Handle event message
     * @param {Uint8Array} data - Message data (skip type byte)
     */
    handleEventMessage(data) {
        const eventData = new TextDecoder().decode(data.slice(1));
        console.log('[RelayObserver] Event:', eventData);

        this.emit('event', { data: eventData });
    }

    /**
     * Handle data message
     * @param {Uint8Array} data - Message data (skip type byte)
     */
    handleDataMessage(data) {
        const payload = new TextDecoder().decode(data.slice(1));

        // Check if this message is for this agent
        if (this.config.agentName && this.isAgentMessage(payload)) {
            console.log('[RelayObserver] Data for agent:', payload);
            this.emit('data', { data: payload });
            return;
        }

        // Forward to all registered handlers
        this.emit('data', { data: payload });
    }

    /**
     * Check if message is for this agent
     * @param {string} payload - Message payload
     */
    isAgentMessage(payload) {
        // Format: @AgentName> or @AgentName><
        const pattern = new RegExp(`^@${this.escapeRegex(this.config.agentName)}>`);
        return pattern.test(payload);
    }

    /**
     * Escape special regex characters in agent name
     * @param {string} str - String to escape
     */
    escapeRegex(str) {
        return str.replace(/[.*+?^${}()[\]\\]/g, '\\$&');
    }

    /**
     * Handle control message
     * @param {Uint8Array} data - Message data (skip type byte)
     */
    handleControlMessage(data) {
        const controlData = new TextDecoder().decode(data.slice(1));
        console.log('[RelayObserver] Control:', controlData);

        // Handle CONNECT command
        if (controlData === 'CONNECT') {
            console.log('[RelayObserver] Observer connected');
        }
        // Handle DISCONNECT command
        else if (controlData === 'DISCONNECT') {
            console.log('[RelayObserver] Observer disconnecting');
            this.disconnect();
        }
    }

    /**
     * Emit event to registered handlers
     * @param {string} event - Event name
     * @param {object} data - Event data
     */
    emit(event, data) {
        const handlers = this.eventHandlers.get(event);
        if (handlers) {
            handlers.forEach(handler => {
                try {
                    handler(data);
                } catch (error) {
                    console.error('[RelayObserver] Handler error:', error);
                }
            });
        }
    }

    /**
     * Register event handler
     * @param {string} event - Event name
     * @param {Function} handler - Event handler function
     */
    on(event, handler) {
        if (!this.eventHandlers.has(event)) {
            this.eventHandlers.set(event, new Set());
        }
        this.eventHandlers.get(event).add(handler);
    }

    /**
     * Unregister event handler
     * @param {string} event - Event name
     * @param {Function} handler - Event handler function
     */
    off(event, handler) {
        const handlers = this.eventHandlers.get(event);
        if (handlers) {
            handlers.delete(handler);
        }
    }

    /**
     * Schedule reconnection
     */
    scheduleReconnect() {
        this.reconnectAttempts++;

        if (this.reconnectAttempts >= this.config.maxReconnectAttempts) {
            console.log('[RelayObserver] Max reconnect attempts reached');
            return;
        }

        const delay = this.calculateBackoffDelay(this.reconnectAttempts);
        console.log(`[RelayObserver] Reconnecting in ${delay}ms (attempt ${this.reconnectAttempts}/${this.config.maxReconnectAttempts})`);

        this.reconnectTimer = setTimeout(() => {
            this.connect();
        }, delay);
    }

    /**
     * Calculate exponential backoff delay
     * @param {number} attempt - Current attempt number
     */
    calculateBackoffDelay(attempt) {
        const baseDelay = this.config.reconnectDelay;
        const delay = baseDelay * Math.pow(2, attempt);
        const jitter = Math.floor(delay / 10);
        return Math.min(delay + jitter, 30000); // Max 30 seconds
    }

    /**
     * Process queued messages
     */
    processQueue() {
        while (this.messageQueue.length > 0 && this.state === WebSocketState.READY) {
            const message = this.messageQueue.shift();
            this.sendMessage(message);
        }
    }

    /**
     * Send message via WebSocket
     * @param {WebSocketMessage} message - Message to send
     */
    sendMessage(message) {
        if (this.state !== WebSocketState.READY) {
            this.messageQueue.push(message);
            return;
        }

        this.ws.send(message.data);
    }

    /**
     * Send event message
     * @param {string} eventData - Event data string
     */
    sendEvent(eventData) {
        const message = WebSocketMessage.createEvent(eventData);
        this.sendMessage(message);
    }

    /**
     * Send data message
     * @param {string|object} dataPayload - Data payload
     */
    sendData(dataPayload) {
        const message = WebSocketMessage.createData(dataPayload);
        this.sendMessage(message);
    }

    /**
     * Send control message
     * @param {string} controlData - Control data string
     */
    sendControl(controlData) {
        const message = WebSocketMessage.createControl(controlData);
        this.sendMessage(message);
    }

    /**
     * Get connection state
     */
    getState() {
        return this.state;
    }

    /**
     * Check if ready
     */
    isReady() {
        return this.state === WebSocketState.READY;
    }

    /**
     * Get current config
     */
    getConfig() {
        return this.config;
    }

    /**
     * Destroy the observer
     */
    destroy() {
        this.disconnect();

        if (this.reconnectTimer) {
            clearTimeout(this.reconnectTimer);
            this.reconnectTimer = null;
        }

        this.eventHandlers.clear();
        this.messageQueue = [];
    }
}

// ============================================================================
// Export
// ============================================================================

export default RelayObserver;
