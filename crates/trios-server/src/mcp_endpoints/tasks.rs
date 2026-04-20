use crate::ws_handler::{AppState, BusEvent, TaskEntry};
use serde_json::{json, Value};
use uuid::Uuid;

pub async fn assign(state: &AppState, params: Option<Value>) -> Value {
    let params = params.unwrap_or(json!({}));
    let agent_id = params.get("agent_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let task = params.get("task").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let context = params.get("context").and_then(|v| v.as_str()).unwrap_or("").to_string();

    if agent_id.is_empty() {
        return json!({"error": "agent_id is required"});
    }
    if task.is_empty() {
        return json!({"error": "task is required"});
    }

    let task_id = format!("task-{}", Uuid::new_v4());
    let entry = TaskEntry {
        id: task_id.clone(),
        agent_id: agent_id.clone(),
        task: format!("{}{}", task, if context.is_empty() { String::new() } else { format!(" | ctx: {}", context) }),
        status: "assigned".to_string(),
    };

    state.tasks.lock().await.push(entry);

    // Broadcast task assignment event to all connected clients
    state.broadcast_event(BusEvent::TaskAssigned {
        task_id: task_id.clone(),
        agent_id: agent_id.clone(),
    });

    json!({"task_id": task_id, "status": "assigned"})
}

pub async fn status(state: &AppState, params: Option<Value>) -> Value {
    let params = params.unwrap_or(json!({}));
    let task_id = params.get("task_id").and_then(|v| v.as_str()).unwrap_or("");

    if task_id.is_empty() {
        return json!({"error": "task_id is required"});
    }

    let tasks = state.tasks.lock().await;
    match tasks.iter().find(|t| t.id == task_id) {
        Some(task) => serde_json::to_value(task).unwrap_or(json!({"error": "serialize failed"})),
        None => json!({"error": format!("task {} not found", task_id)}),
    }
}

/// Update task status and broadcast event
pub async fn update_status(state: &AppState, params: Option<Value>) -> Value {
    let params = params.unwrap_or(json!({}));
    let task_id = params.get("task_id").and_then(|v| v.as_str()).unwrap_or("");
    let new_status = params.get("status").and_then(|v| v.as_str()).unwrap_or("");

    if task_id.is_empty() {
        return json!({"error": "task_id is required"});
    }
    if new_status.is_empty() {
        return json!({"error": "status is required"});
    }

    let mut tasks = state.tasks.lock().await;
    if let Some(task) = tasks.iter_mut().find(|t| t.id == task_id) {
        task.status = new_status.to_string();
        let task_id = task_id.to_string();
        let new_status = new_status.to_string();
        drop(tasks); // Release lock before broadcast

        // Broadcast task update event
        state.broadcast_event(BusEvent::TaskUpdated {
            task_id: task_id.clone(),
            status: new_status.clone(),
        });

        json!({"task_id": task_id, "status": new_status})
    } else {
        json!({"error": format!("task {} not found", task_id)})
    }
}
