use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};

use crate::state::{PetState, SharedState, StateChangeEvent};

#[derive(Debug, Deserialize)]
pub struct StateRequest {
    pub s: String,
    #[serde(default)]
    pub tool: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CurrentResponse {
    pub animation: String,
    pub bubble: String,
    pub provider: String,
}

pub fn create_router(state: SharedState, tx: broadcast::Sender<StateChangeEvent>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/api/state", post(handle_state))
        .route("/api/heartbeat", get(handle_heartbeat))
        .route("/api/current", get(handle_current))
        .route("/api/codex/:event", post(handle_codex_event))
        .route("/api/event/:provider/:event", post(handle_provider_event))
        .route("/api/hook/thinking", post(handle_hook_thinking))
        .route("/api/hook/working", post(handle_hook_working))
        .route("/api/hook/done", post(handle_hook_done))
        .route("/api/hook/idle", post(handle_hook_idle))
        .route("/api/hook/permission", post(handle_hook_permission))
        .layer(cors)
        .with_state(AppState { state, tx })
}

#[derive(Clone)]
struct AppState {
    state: SharedState,
    tx: broadcast::Sender<StateChangeEvent>,
}

async fn handle_state(State(app): State<AppState>, Json(body): Json<StateRequest>) -> StatusCode {
    let pet_state = PetState::from_hook(&body.s, body.tool.as_deref());
    let tool = body.tool.clone();
    let animation = pet_state.animation_name().to_string();
    let bubble = pet_state.bubble_text(tool.as_deref()).to_string();

    println!(
        "[HTTP] state: s={} tool={:?} => animation={} bubble={}",
        body.s, tool, animation, bubble
    );

    {
        let mut mgr = app.state.lock().await;
        mgr.set_state_from(pet_state, tool, "api");
    }

    let _ = app.tx.send(StateChangeEvent { animation, bubble });
    StatusCode::OK
}

async fn handle_heartbeat() -> StatusCode {
    StatusCode::OK
}

async fn handle_current(State(app): State<AppState>) -> Json<CurrentResponse> {
    let mgr = app.state.lock().await;
    let current = mgr.current_state();
    let tool = mgr.current_tool();
    let animation = current.animation_name().to_string();
    let bubble = mgr
        .current_bubble()
        .unwrap_or_else(|| current.bubble_text(tool))
        .to_string();
    let provider = mgr.current_provider().to_string();
    Json(CurrentResponse {
        animation,
        bubble,
        provider,
    })
}

async fn handle_codex_event(
    State(app): State<AppState>,
    Path(event): Path<String>,
    body: String,
) -> StatusCode {
    apply_provider_event(&app, "codex", &event, &body).await
}

async fn handle_provider_event(
    State(app): State<AppState>,
    Path((provider, event)): Path<(String, String)>,
    body: String,
) -> StatusCode {
    apply_provider_event(&app, &provider, &event, &body).await
}

async fn apply_provider_event(
    app: &AppState,
    provider: &str,
    event: &str,
    body: &str,
) -> StatusCode {
    let payload = serde_json::from_str::<serde_json::Value>(body).unwrap_or_default();
    let tool = codex_tool_name(&payload);
    let normalized_event = event.replace('-', "_").to_ascii_lowercase();
    let normalized_provider = provider.to_ascii_lowercase();

    let state = match normalized_event.as_str() {
        "session_start" => PetState::Waving,
        "session_end" => PetState::Idle,
        "user_prompt_submit" => PetState::Chatting,
        "pre_tool_use" => state_for_codex_tool(tool.as_deref()),
        "post_tool_use" | "subagent_stop" | "post_compact" => PetState::Thinking,
        "permission_request" => PetState::Waving,
        "pre_compact" | "subagent_start" => PetState::Analyzing,
        "stop" | "agent_turn_complete" => PetState::Celebrating,
        _ => PetState::Thinking,
    };

    println!(
        "[{}] event={} tool={:?} => {:?}",
        normalized_provider, normalized_event, tool, state
    );
    let protect_permission = normalized_event == "permission_request";
    set_pet_state(app, state, tool, &normalized_provider, protect_permission).await;
    StatusCode::OK
}

fn codex_tool_name(payload: &serde_json::Value) -> Option<String> {
    ["tool_name", "toolName", "tool"]
        .iter()
        .find_map(|key| payload.get(key).and_then(|value| value.as_str()))
        .map(String::from)
}

fn state_for_codex_tool(tool: Option<&str>) -> PetState {
    let name = tool.unwrap_or_default().to_ascii_lowercase();
    if name.contains("web") || name.contains("search") || name.contains("browser") {
        PetState::Searching
    } else if name.contains("fetch") || name.contains("read") || name.contains("view_image") {
        PetState::Fetching
    } else if name.contains("apply_patch") || name.contains("write") || name.contains("edit") {
        PetState::Building
    } else if name.contains("collaboration") || name.contains("agent") || name.contains("delegate")
    {
        PetState::Analyzing
    } else {
        PetState::Running
    }
}

async fn handle_hook_thinking(State(app): State<AppState>) -> StatusCode {
    set_pet_state(&app, PetState::Chatting, None, "legacy", false).await;
    StatusCode::OK
}

async fn handle_hook_working(State(app): State<AppState>, body: String) -> StatusCode {
    let tool = serde_json::from_str::<serde_json::Value>(&body)
        .ok()
        .and_then(|v| {
            v.get("tool_name")
                .and_then(|t| t.as_str().map(String::from))
        });

    // Map specific tools to specific animations
    let state = match tool.as_deref() {
        Some("WebFetch") => PetState::Fetching,
        Some("WebSearch") => PetState::Searching,
        Some("Write") | Some("Edit") => PetState::Building,
        Some("Agent") | Some("TaskCreate") | Some("TaskUpdate") => PetState::Analyzing,
        _ => PetState::Running,
    };

    set_pet_state(&app, state, tool, "legacy", false).await;
    StatusCode::OK
}

async fn handle_hook_done(State(app): State<AppState>) -> StatusCode {
    set_pet_state(&app, PetState::Celebrating, None, "legacy", false).await;
    StatusCode::OK
}

async fn handle_hook_idle(State(app): State<AppState>) -> StatusCode {
    set_pet_state(&app, PetState::Idle, None, "legacy", false).await;
    StatusCode::OK
}

async fn handle_hook_permission(State(app): State<AppState>) -> StatusCode {
    {
        let mut mgr = app.state.lock().await;
        mgr.set_state_from(PetState::Waving, None, "legacy");
        mgr.lock_permission();
    }
    let _ = app.tx.send(StateChangeEvent {
        animation: "waving".to_string(),
        bubble: "等待指示...".to_string(),
    });
    StatusCode::OK
}

async fn set_pet_state(
    app: &AppState,
    state: PetState,
    tool: Option<String>,
    provider: &str,
    protect_permission: bool,
) {
    {
        let mgr = app.state.lock().await;
        // Protect permission bubble from being overwritten
        if state != PetState::Waving && mgr.is_permission_locked() {
            return;
        }
    }
    let animation = state.animation_name().to_string();
    let bubble = state.bubble_text(tool.as_deref()).to_string();
    {
        let mut mgr = app.state.lock().await;
        mgr.set_state_from(state, tool, provider);
        if protect_permission {
            mgr.lock_permission();
        }
    }
    let _ = app.tx.send(StateChangeEvent { animation, bubble });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_codex_tools_to_animations() {
        assert_eq!(
            state_for_codex_tool(Some("exec_command")),
            PetState::Running
        );
        assert_eq!(
            state_for_codex_tool(Some("apply_patch")),
            PetState::Building
        );
        assert_eq!(state_for_codex_tool(Some("web.run")), PetState::Searching);
        assert_eq!(state_for_codex_tool(Some("view_image")), PetState::Fetching);
        assert_eq!(
            state_for_codex_tool(Some("collaboration.spawn_agent")),
            PetState::Analyzing
        );
    }
}
