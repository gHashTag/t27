// TRIOS Extension — Service Worker (minimal)
// All WebSocket and UI logic is in Rust+Wasm module

console.log("[TRIOS SW] Service Worker starting...");

chrome.sidePanel.setPanelBehavior({ openPanelOnActionClick: true });

chrome.action.onClicked.addListener(() => {
  console.log("[TRIOS SW] Opening side panel");
  chrome.sidePanel.open({ windowId: undefined });
});

console.log("[TRIOS SW] Service Worker ready");
