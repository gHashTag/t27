//! Trios Chrome Extension — Rust + Wasm-bindgen
//!
//! ## Architecture
//! - Background: WebSocket client for MCP to trios-server (port 9005)
//! - UI: 3-tab interface (Chat, Agents, MCP Tools)
//! - Brand: TRIOS colors (#161616, #F5D3F2)
//!
//! ## Trinity Stack Law Compliance
//! All code is Rust → Wasm, no JavaScript outside dist/ directory.

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{console, Document, Element, HtmlElement, HtmlInputElement, Window, WebSocket, CloseEvent, Event, KeyboardEvent, MessageEvent};
use web_sys::wasm_bindgen::JsValue;
use js_sys::Array;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value, to_string, to_string_pretty};
use std::cell::RefCell;
use std::rc::Rc;

// ============================================================================
// TRIOS Brand Constants
// ============================================================================

const BG_DEEP_MIDNIGHT: &str = "#161616";
const BG_ELEVATED: &str = "#1E1E24";
const BG_INPUT: &str = "#0A0A0F";
const TEXT_PRIMARY: &str = "#E0E0E0";
const TEXT_SECONDARY: &str = "#A0A0A0";
const ACCENT_GOLD: &str = "#F5D3F2";
const ACCENT_BLUE: &str = "#5D3FF2";
const BORDER_COLOR: &str = "#2A2A32";
const SUCCESS_COLOR: &str = "#28A745";
const ERROR_COLOR: &str = "#E74C3C";

const MCP_SERVER_URL: &str = "ws://localhost:9005/ws";

// ============================================================================
// Application State
// ============================================================================

#[wasm_bindgen]
pub struct AppState {
    agents: RefCell<Vec<Agent>>,
    tools: RefCell<Vec<McpTool>>,
    chat_messages: RefCell<Vec<ChatMessage>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Agent {
    id: String,
    name: String,
    status: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct McpTool {
    name: String,
    description: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    role: String,  // "you", "agent", "system", "error"
    content: String,
}

#[wasm_bindgen]
impl AppState {
    pub fn new() -> Self {
        Self {
            agents: RefCell::new(Vec::new()),
            tools: RefCell::new(Vec::new()),
            chat_messages: RefCell::new(Vec::new()),
        }
    }
}

thread_local! {
    static STATE: RefCell<Option<AppState>> = RefCell::new(None);
    static WS_CLIENT: RefCell<Option<WebSocket>> = RefCell::new(None);
}

fn get_state() -> AppState {
    STATE.with_borrow(|state| {
        state.as_ref().unwrap_or_else(|| {
            let new_state = AppState::new();
            STATE.set(Some(new_state.clone()));
            new_state
        })
        .clone()
    })
}

fn with_state<F, R>(f: F) -> R
where
    F: FnOnce(&AppState) -> R,
{
    let state = get_state();
    f(&state)
}

// ============================================================================
// WebSocket Client
// ============================================================================

#[wasm_bindgen]
pub fn connect_websocket() -> JsValue {
    WS_CLIENT.with_borrow(|ws| {
        if let Some(_) = ws.as_ref() {
            console::log_1(&"[TRIOS] WebSocket already connected".into());
            return JsValue::TRUE;
        }
    });

    let window = web_sys::window().unwrap_throw();
    let ws = WebSocket::new(MCP_SERVER_URL).unwrap_throw();

    // Set binary type
    ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

    {
        let ws_clone = ws.clone();

        // On open handler
        let onopen = Closure::wrap(Box::new(move || {
            console::log_1(&format!("[TRIOS] WebSocket connected to {}", MCP_SERVER_URL).into());
            // Send initial request for agents
            let req = json!({
                "type": "MCP_REQUEST",
                "payload": {
                    "method": "agents/list",
                    "params": {}
                }
            });
            let json = to_string(&req).unwrap_or_default();
            let _ = ws_clone.send_with_str(&json);
        }) as Box<dyn Fn()>);

        ws.set_onopen(Some(onopen.as_ref().unchecked_ref()));
        onopen.forget();
    }

    {
        let ws_clone = ws.clone();

        // On message handler
        let onmessage = Closure::wrap(Box::new(move |event: MessageEvent| {
            if let Ok(data) = event.data().dyn_into::<js_sys::JsString>() {
                let text = String::from(data);
                console::log_1(&format!("[TRIOS] Received: {}", text).into());
                handle_mcp_response(&text);
            }
        }) as Box<dyn Fn(_)>);

        ws.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
        onmessage.forget();
    }

    {
        let ws_clone = ws.clone();

        // On error handler
        let onerror = Closure::wrap(Box::new(move || {
            console::error_1(&"[TRIOS] WebSocket error".into());
            with_state(|s| {
                s.chat_messages.borrow_mut().push(ChatMessage {
                    role: "error".to_string(),
                    content: "Connection failed — is trios-server running?".to_string(),
                });
            });
            render_chat_messages();
        }) as Box<dyn Fn()>);

        ws.set_onerror(Some(onerror.as_ref().unchecked_ref()));
        onerror.forget();
    }

    {
        let ws_clone = ws.clone();

        // On close handler
        let onclose = Closure::wrap(Box::new(move |_: CloseEvent| {
            console::log_1(&"[TRIOS] WebSocket closed".into());
            WS_CLIENT.with_borrow(|ws| *ws.borrow_mut() = None);
        }) as Box<dyn Fn(_)>);

        ws.set_onclose(Some(onclose.as_ref().unchecked_ref()));
        onclose.forget();
    }

    WS_CLIENT.with_borrow(|ws| *ws.borrow_mut() = Some(ws));

    JsValue::TRUE
}

fn send_request(payload: Value) -> JsValue {
    WS_CLIENT.with_borrow(|ws| {
        if let Some(ws) = ws.as_ref() {
            let json = to_string(&payload).unwrap_or_else(|e| {
                console::error_1(&format!("[TRIOS] JSON error: {}", e).into());
                String::new()
            });

            match ws.send_with_str(&json) {
                Ok(_) => JsValue::TRUE,
                Err(e) => {
                    console::error_1(&format!("[TRIOS] Send error: {:?}", e).into());
                    JsValue::FALSE
                }
            }
        } else {
            console::error_1(&"[TRIOS] WebSocket not connected".into());
            JsValue::FALSE
        }
    })
}

fn handle_mcp_response(text: &str) {
    // Try to parse as JSON
    if let Ok(value) = serde_json::from_str::<Value>(text) {
        // Handle MCP response format: { "type": "MCP_RESPONSE", "data": ... }
        if let Some(msg_type) = value.get("type").and_then(|v| v.as_str()) {
            if msg_type == "MCP_RESPONSE" {
                if let Some(data) = value.get("data") {
                    // Handle agents list
                    if let Some(agents) = data.get("agents").or_else(|| data.get("tools")).and_then(|v| v.as_array()) {
                        let mut agent_list = Vec::new();
                        for agent in agents.iter().filter_map(|v| v.as_object()) {
                            let id = agent.get("id")
                                .and_then(|v| v.as_str())
                                .unwrap_or("unknown")
                                .to_string();
                            let name = agent.get("name")
                                .and_then(|v| v.as_str())
                                .unwrap_or("Unknown Agent")
                                .to_string();
                            let status = agent.get("status")
                                .and_then(|v| v.as_str())
                                .unwrap_or("unknown")
                                .to_string();

                            agent_list.push(Agent { id, name, status });
                        }

                        with_state(|s| {
                            *s.agents.borrow_mut() = agent_list;
                            render_agent_list();
                        });
                    }

                    // Handle tools list
                    if let Some(tools) = data.get("tools").and_then(|v| v.as_array()) {
                        let mut tool_list = Vec::new();
                        for tool in tools.iter().filter_map(|v| v.as_object()) {
                            let name = tool.get("name")
                                .and_then(|v| v.as_str())
                                .unwrap_or("Unknown Tool")
                                .to_string();
                            let description = tool.get("description")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();

                            tool_list.push(McpTool { name, description });
                        }

                        with_state(|s| {
                            *s.tools.borrow_mut() = tool_list;
                            render_tools_list();
                        });
                    }

                    // Handle chat response
                    if let Some(response) = data.get("response").or_else(|| data.get("result")) {
                        let content = if let Some(s) = response.as_str() {
                            s.to_string()
                        } else {
                            to_string_pretty(&response).unwrap_or_else(|_| "Unknown response".to_string())
                        };

                        with_state(|s| {
                            s.chat_messages.borrow_mut().push(ChatMessage {
                                role: "agent".to_string(),
                                content,
                            });
                            render_chat_messages();
                        });
                    }
                }
            }
        }
    } else {
        console::error_1(&format!("[TRIOS] Failed to parse JSON: {}", text).into());
    }
}

// ============================================================================
// UI Rendering
// ============================================================================

#[wasm_bindgen]
pub fn render_app() -> JsValue {
    let window = web_sys::window().unwrap_throw();
    let document = window.document().unwrap_throw();
    let body = document.body().unwrap_throw();

    // Clear and rebuild app
    body.set_inner_html("");

    let app = create_element(&document, "div");
    app.set_class_name("app-container");
    body.append_child(&app).unwrap_throw();

    // Header
    let header = create_element(&document, "header");
    header.set_class_name("header");

    let logo = create_element(&document, "img");
    logo.set_class_name("logo");
    logo.set_attribute("src", "icons/icon-128.png").unwrap_throw();
    header.append_child(&logo).unwrap_throw();

    let title = create_element(&document, "h1");
    title.set_class_name("title");
    title.set_text_content(Some("Trios"));
    header.append_child(&title).unwrap_throw();
    app.append_child(&header).unwrap_throw();

    // Tabs
    let nav = create_element(&document, "nav");
    nav.set_class_name("tabs");

    let tabs = [("chat", "💬 Chat"), ("agents", "🤖 Agents"), ("tools", "🔧 MCP Tools")];

    for (i, (tab_id, tab_name)) in tabs.iter().enumerate() {
        let btn = create_element(&document, "button");
        btn.set_class_name("tab");
        btn.set_attribute("data-tab", tab_id).unwrap_throw();
        btn.set_text_content(Some(tab_name));
        if i == 0 {
            btn.class_list().add_1("active").unwrap_throw();
        }
        nav.append_child(&btn).unwrap_throw();

        // Event listener
        let tab_id_clone = tab_id.to_string();
        let btn_clone = btn.clone();
        let closure = Closure::<dyn Fn(_)>::new(move |_| {
            switch_tab(&tab_id_clone);
        });
        btn_clone.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref()).unwrap_throw();
        closure.forget();
    }

    app.append_child(&nav).unwrap_throw();

    // Tab content containers
    let chat_content = create_element(&document, "section");
    chat_content.set_id("tab-chat");
    chat_content.set_class_name("tab-content active");
    chat_content.set_inner_html(r#"<div id="messages"></div><input id="chat-input" type="text" placeholder="Press Enter to send..." />"#);
    app.append_child(&chat_content).unwrap_throw();

    let agents_content = create_element(&document, "section");
    agents_content.set_id("tab-agents");
    agents_content.set_class_name("tab-content");
    agents_content.set_inner_html(r#"<div id="agent-list">Loading agents...</div>"#);
    app.append_child(&agents_content).unwrap_throw();

    let tools_content = create_element(&document, "section");
    tools_content.set_id("tab-tools");
    tools_content.set_class_name("tab-content");
    tools_content.set_inner_html(r#"<div id="tool-list">Loading tools...</div>"#);
    app.append_child(&tools_content).unwrap_throw();

    // Add styles
    add_styles(&document);

    JsValue::TRUE
}

#[wasm_bindgen]
pub fn switch_tab(tab_id: &str) {
    let window = web_sys::window().unwrap_throw();
    let document = window.document().unwrap_throw();

    // Remove active from all tabs
    if let Some(tabs) = document.query_selector_all(".tab") {
        for i in 0..tabs.length() {
            if let Some(tab) = tabs.item(i) {
                tab.class_list().remove_1("active").ok();
            }
        }
    }

    // Remove active from all content panels
    if let Some(panels) = document.query_selector_all(".tab-content") {
        for i in 0..panels.length() {
            if let Some(panel) = panels.item(i) {
                panel.class_list().remove_1("active").ok();
            }
        }
    }

    // Add active to clicked tab
    if let Some(tab) = document.get_element_by_id(tab_id) {
        tab.class_list().add_1("active").ok();
    }

    // Show corresponding content
    let content_id = format!("tab-{}", tab_id);
    if let Some(content) = document.get_element_by_id(&content_id) {
        content.class_list().add_1("active").ok();
    }

    if tab_id == "chat" {
        setup_chat_input();
    } else if tab_id == "agents" {
        load_agents_from_server();
    } else if tab_id == "tools" {
        load_tools_from_server();
    }
}

#[wasm_bindgen]
pub fn render_chat_messages() -> JsValue {
    let window = web_sys::window().unwrap_throw();
    let document = window.document().unwrap_throw();

    if let Some(el) = document.get_element_by_id("messages") {
        let messages = with_state(|s| s.chat_messages.borrow().clone());

        el.set_inner_html("");

        for msg in messages {
            let msg_div = create_element(&document, "div");
            let role_class = format!("message {}", msg.role);
            msg_div.set_class_name(&role_class);
            msg_div.set_text_content(Some(&msg.content));
            el.append_child(&msg_div).ok();
        }

        // Scroll to bottom
        el.set_scroll_top(el.scroll_height());
    }

    JsValue::TRUE
}

#[wasm_bindgen]
pub fn render_agent_list() -> JsValue {
    let window = web_sys::window().unwrap_throw();
    let document = window.document().unwrap_throw();

    if let Some(el) = document.get_element_by_id("agent-list") {
        let agents = with_state(|s| s.agents.borrow().clone());

        if agents.is_empty() {
            el.set_text_content(Some("No agents connected."));
        } else {
            let html = agents
                .iter()
                .map(|a| format!(
                    "<div class='agent-card'><div class='agent-header'><span class='agent-name'>{}</span><span class='agent-status {}'>{}</span></div><div class='agent-meta'>ID: {}</div></div>",
                    escape_html(&a.name),
                    a.status,
                    a.id
                ))
                .collect::<Vec<_>>()
                .join("\n");
            el.set_inner_html(&html);
        }
    }

    JsValue::TRUE
}

#[wasm_bindgen]
pub fn render_tools_list() -> JsValue {
    let window = web_sys::window().unwrap_throw();
    let document = window.document().unwrap_throw();

    if let Some(el) = document.get_element_by_id("tool-list") {
        let tools = with_state(|s| s.tools.borrow().clone());

        if tools.is_empty() {
            el.set_text_content(Some("No tools available."));
        } else {
            let html = tools
                .iter()
                .map(|t| format!(
                    "<div class='tool-card'><div class='tool-header'>{}</div><div class='tool-desc'>{}</div></div>",
                    escape_html(&t.name),
                    escape_html(&t.description)
                ))
                .collect::<Vec<_>>()
                .join("\n");
            el.set_inner_html(&html);
        }
    }

    JsValue::TRUE
}

#[wasm_bindgen]
pub fn setup_chat_input() -> JsValue {
    let window = web_sys::window().unwrap_throw();
    let document = window.document().unwrap_throw();

    if let Some(input) = document.get_element_by_id("chat-input") {
        let input_el: HtmlElement = input.dyn_into().unwrap_throw();
        input_el.focus();

        let closure = Closure::<dyn Fn(_)>::new(move |event: Event| {
            if let Some(key_event) = event.dyn_ref::<KeyboardEvent>() {
                if key_event.key() == "Enter" {
                    if let Some(input) = document.get_element_by_id("chat-input") {
                        if let Some(input_el) = input.dyn_into::<HtmlInputElement>().ok() {
                            if let Some(value) = input_el.value() {
                                let msg = value.trim();
                                if !msg.is_empty() {
                                    send_chat_message(&msg);
                                    input_el.set_value("");
                                }
                            }
                        }
                    }
                }
            }
        });

        input_el
            .add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())
            .unwrap_throw();
        closure.forget();
    }

    JsValue::TRUE
}

#[wasm_bindgen]
pub fn send_chat_message(message: &str) -> JsValue {
    add_chat_message("you", message);

    with_state(|s| {
        let _ = send_request(json!({
            "type": "MCP_REQUEST",
            "payload": {
                "method": "agents/chat",
                "params": {
                    "message": message
                }
            }
        }));
    });

    JsValue::TRUE
}

#[wasm_bindgen]
pub fn add_chat_message(role: &str, content: &str) -> JsValue {
    with_state(|s| {
        s.chat_messages.borrow_mut().push(ChatMessage {
            role: role.to_string(),
            content: content.to_string(),
        });
    });
    render_chat_messages();
    JsValue::TRUE
}

#[wasm_bindgen]
pub fn load_agents_from_server() -> JsValue {
    send_request(json!({
        "type": "MCP_REQUEST",
        "payload": {
            "method": "agents/list",
            "params": {}
        }
    }))
}

#[wasm_bindgen]
pub fn load_tools_from_server() -> JsValue {
    send_request(json!({
        "type": "MCP_REQUEST",
        "payload": {
            "method": "tools/list",
            "params": {}
        }
    }))
}

// ============================================================================
// Utilities
// ============================================================================

fn create_element(doc: &Document, tag: &str) -> Element {
    doc.create_element(tag).unwrap_throw()
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn add_styles(doc: &Document) {
    if let Ok(Some(head)) = doc.query_selector("head") {
        let style = create_element(doc, "style");
        // set_type not needed for style elements
        style.set_text_content(Some(STYLE));
        let _ = head.append_child(&style);
    }
}

// ============================================================================
// Embedded CSS
// ============================================================================

const STYLE: &str = r#"
* { margin: 0; padding: 0; box-sizing: border-box; }
body {
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    background: #161616;
    color: #E0E0E0;
    min-height: 100vh;
}
.header {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 12px 16px;
    background: #1E1E24;
    border-bottom: 1px solid #2A2A32;
}
.logo {
    width: 32px;
    height: 32px;
}
.title {
    font-size: 16px;
    color: #F5D3F2;
}
.tabs {
    display: flex;
    background: #1E1E24;
    border-bottom: 1px solid #2A2A32;
}
.tab {
    flex: 1;
    padding: 10px;
    border: none;
    background: transparent;
    color: #A0A0A0;
    cursor: pointer;
    font-size: 13px;
    transition: color 0.2s, border-bottom 0.2s;
}
.tab.active {
    color: #F5D3F2;
    border-bottom: 2px solid #F5D3F2;
}
.tab-content {
    display: none;
    padding: 16px;
}
.tab-content.active {
    display: block;
}
#chat-input {
    width: 100%;
    padding: 10px;
    border: 1px solid #2A2A32;
    border-radius: 6px;
    background: #0A0A0F;
    color: #E0E0E0;
    font-size: 14px;
    margin-top: 12px;
}
#messages {
    max-height: 400px;
    overflow-y: auto;
    margin-bottom: 12px;
}
.message {
    padding: 8px 12px;
    margin: 8px 0;
    border-radius: 8px;
    line-height: 1.4;
}
.message.you {
    background: #5D3FF2;
    color: #161616;
    margin-left: 20%;
}
.message.agent {
    background: #1E1E24;
    color: #E0E0E0;
    margin-right: 20%;
}
.message.error {
    background: #E74C3C;
    color: white;
}
.message.system {
    background: #28A745;
    color: white;
    text-align: center;
}
.agent-card {
    background: #1E1E24;
    border: 1px solid #2A2A32;
    border-radius: 8px;
    padding: 12px;
    margin: 8px 0;
}
.agent-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 8px;
}
.agent-name {
    font-weight: bold;
    color: #F5D3F2;
}
.agent-status {
    font-size: 11px;
    padding: 2px 8px;
    border-radius: 4px;
}
.agent-status.running {
    background: #28A745;
    color: white;
}
.agent-status.stopped {
    background: #E74C3C;
    color: white;
}
.agent-status.unknown {
    background: #A0A0A0;
    color: white;
}
.agent-meta {
    font-size: 11px;
    color: #A0A0A0;
}
.tool-card {
    background: #1E1E24;
    border: 1px solid #2A2A32;
    border-radius: 8px;
    padding: 12px;
    margin: 8px 0;
}
.tool-header {
    font-weight: bold;
    color: #F5D3F2;
    margin-bottom: 4px;
}
.tool-desc {
    font-size: 13px;
    color: #A0A0A0;
}
"#;

// ============================================================================
// wasm-bindgen entry point
// ============================================================================

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    console::log_1(&"[TRIOS] Extension starting...".into());

    // Initialize state
    STATE.set(Some(AppState::new()));

    // Render app
    render_app();

    // Connect WebSocket
    connect_websocket();

    // Add welcome message
    add_chat_message("agent", "Welcome to TRIOS Sidebar! Connecting to trios-server...");

    console::log_1(&"[TRIOS] Extension ready".into());

    Ok(())
}

#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    format!("Trios says: Hello, {}!", name)
}

// ============================================================================
// Background Script (for service worker)
// ============================================================================

#[wasm_bindgen]
pub fn init_background() -> Result<(), JsValue> {
    console::log_1(&"[TRIOS] Background script initialized".into());

    let win = web_sys::window()
        .ok_or_else(|| JsValue::from_str("Window not available"))?;

    // Set side panel behavior
    if let Ok(chrome) = js_sys::Reflect::get(&win, &"chrome".into()) {
        if let Ok(side_panel) = js_sys::Reflect::get(&chrome, &"sidePanel".into()) {
            if let Ok(set_behavior) = js_sys::Reflect::get(&side_panel, &"setPanelBehavior".into()) {
                if let Some(func) = set_behavior.dyn_ref::<js_sys::Function>() {
                    let config = js_sys::Object::new();
                    js_sys::Reflect::set(&config, &"openPanelOnActionClick".into(), &true.into()).ok();
                    let _ = func.call1(&side_panel, &config);
                }
            }
        }

        // Setup message listener
        if let Ok(runtime) = js_sys::Reflect::get(&chrome, &"runtime".into()) {
            if let Ok(on_message) = js_sys::Reflect::get(&runtime, &"onMessage".into()) {
                if let Ok(add_listener) = js_sys::Reflect::get(&on_message, &"addListener".into()) {
                    if let Some(func) = add_listener.dyn_ref::<js_sys::Function>() {
                        let listener = Closure::wrap(Box::new(|message: JsValue, _sender: JsValue, send_response: JsValue| {
                            handle_mcp_request(message, send_response);
                        }) as Box<dyn Fn(JsValue, JsValue, JsValue)>);

                        let _ = func.call1(&on_message, listener.as_ref().unchecked_ref());
                        listener.forget();
                    }
                }
            }
        }
    }

    Ok(())
}

fn handle_mcp_request(message: JsValue, send_response: JsValue) {
    // Clone for use in multiple closures
    let send_response1 = send_response.clone();
    let send_response2 = send_response.clone();
    let send_response3 = send_response.clone();

    // Check if this is an MCP request
    if let Ok(msg_type) = js_sys::Reflect::get(&message, &"type".into()) {
        if let Some(type_str) = msg_type.as_string() {
            if type_str == "MCP_REQUEST" {
                // Get payload
                let payload = js_sys::Reflect::get(&message, &"payload".into())
                    .unwrap_or_else(|_| js_sys::Object::new().into());

                let payload_str = js_sys::JSON::stringify(&payload)
                    .and_then(|s| s.as_string().ok_or(JsValue::from_str("not a string")))
                    .unwrap_or_else(|_| "{}".to_string());

                // Create WebSocket connection
                let ws = WebSocket::new(MCP_SERVER_URL);
                match ws {
                    Ok(ws) => {
                        let ws_clone = ws.clone();

                        let onopen = Closure::wrap(Box::new(move || {
                            console::log_1(&"[TRIOS] Background WebSocket connected".into());
                            let _ = ws_clone.send_with_str(&payload_str);
                        }) as Box<dyn Fn()>);

                        ws.set_onopen(Some(onopen.as_ref().unchecked_ref()));
                        onopen.forget();

                        let onmessage = Closure::wrap(Box::new(move |event: MessageEvent| {
                            if let Ok(response) = event.data().dyn_into::<js_sys::JsString>() {
                                let text = String::from(response);
                                // Parse and send response
                                if let Some(func) = send_response1.dyn_ref::<js_sys::Function>() {
                                    let resp = js_sys::Object::new();
                                    js_sys::Reflect::set(&resp, &"type".into(), &"MCP_RESPONSE".into()).ok();
                                    if let Ok(parsed) = js_sys::JSON::parse(&text) {
                                        js_sys::Reflect::set(&resp, &"data".into(), &parsed).ok();
                                    } else {
                                        js_sys::Reflect::set(&resp, &"data".into(), &JsValue::from_str(&text)).ok();
                                    }
                                    let _ = func.call1(&JsValue::NULL, &resp);
                                }
                            }
                        }) as Box<dyn Fn(_)>);

                        ws.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
                        onmessage.forget();

                        let onerror = Closure::wrap(Box::new(move || {
                            console::error_1(&"[TRIOS] Background WebSocket error".into());
                            if let Some(func) = send_response2.dyn_ref::<js_sys::Function>() {
                                let err = js_sys::Object::new();
                                js_sys::Reflect::set(&err, &"type".into(), &"MCP_ERROR".into()).ok();
                                js_sys::Reflect::set(&err, &"error".into(), &"WebSocket connection failed".into()).ok();
                                let _ = func.call1(&JsValue::NULL, &err);
                            }
                        }) as Box<dyn Fn()>);

                        ws.set_onerror(Some(onerror.as_ref().unchecked_ref()));
                        onerror.forget();
                    }
                    Err(e) => {
                        console::error_1(&format!("[TRIOS] Background WebSocket creation error: {:?}", e).into());
                        if let Some(func) = send_response3.dyn_ref::<js_sys::Function>() {
                            let err = js_sys::Object::new();
                            js_sys::Reflect::set(&err, &"type".into(), &"MCP_ERROR".into()).ok();
                            js_sys::Reflect::set(&err, &"error".into(), &"WebSocket creation failed".into()).ok();
                            let _ = func.call1(&JsValue::NULL, &err);
                        }
                    }
                }
            }
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_html() {
        assert_eq!(escape_html("<script>"), "&lt;script&gt;");
        assert_eq!(escape_html("&"), "&amp;");
        assert_eq!(escape_html("\""), "&quot;");
    }

    #[test]
    fn test_agent_serialization() {
        let agent = Agent {
            id: "test-id".to_string(),
            name: "Test Agent".to_string(),
            status: "running".to_string(),
        };
        let json = to_string(&agent).unwrap();
        assert!(json.contains("test-id"));
        assert!(json.contains("Test Agent"));
    }

    #[test]
    fn test_chat_message_creation() {
        let msg = ChatMessage {
            role: "you".to_string(),
            content: "Hello".to_string(),
        };
        let json = to_string(&msg).unwrap();
        assert!(json.contains("you"));
        assert!(json.contains("Hello"));
    }
}
