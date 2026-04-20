chrome.runtime.onInstalled.addListener(() => {
  chrome.sidePanel.setPanelBehavior({ openPanelOnActionClick: true });
});

chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
  if (message.type === "MCP_REQUEST") {
    const ws = new WebSocket("ws://localhost:9005/mcp");
    ws.onopen = () => {
      ws.send(JSON.stringify(message.payload));
    };
    ws.onmessage = (event) => {
      sendResponse({ type: "MCP_RESPONSE", data: JSON.parse(event.data) });
      ws.close();
    };
    ws.onerror = () => {
      sendResponse({ type: "MCP_ERROR", error: "WebSocket connection failed" });
      ws.close();
    };
    return true;
  }
});
