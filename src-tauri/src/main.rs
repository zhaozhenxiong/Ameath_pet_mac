#![windows_subsystem = "windows"]

mod codex;
mod http;
mod mcp;
mod state;
mod tray;

#[cfg(target_os = "macos")]
use objc2_app_kit::{NSColor, NSWindow};
use state::StateChangeEvent;
use state::StateManager;
use std::sync::Arc;
use std::time::Duration;
use tauri::utils::config::Color;
use tauri::{Emitter, Manager};
use tokio::sync::{broadcast, Mutex};

#[tokio::main]
async fn main() {
    if codex::handle_cli_event() {
        return;
    }

    let state_manager = Arc::new(Mutex::new(StateManager::new()));
    let (tx, _rx) = broadcast::channel::<StateChangeEvent>(32);

    // Build Tauri app
    tauri::Builder::default()
        // This must be the first plugin registered. A port probe alone is not
        // an atomic single-instance check and allowed simultaneous hooks or
        // double-clicks to create two windows.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("aemeath") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .invoke_handler(tauri::generate_handler![start_drag, set_pet_scale])
        .setup(move |app| {
            // Start background services only after the single-instance plugin
            // owns the application identity. Retry transient port conflicts so
            // the window cannot remain alive with a silently dead API server.
            let sm_http = state_manager.clone();
            let tx_http = tx.clone();
            tokio::spawn(async move {
                serve_with_retry("HTTP", "127.0.0.1:9527", move || {
                    http::create_router(sm_http.clone(), tx_http.clone())
                })
                .await;
            });

            let sm_mcp = state_manager.clone();
            let tx_mcp = tx.clone();
            tokio::spawn(async move {
                serve_with_retry("MCP", "127.0.0.1:9528", move || {
                    mcp::create_mcp_router(sm_mcp.clone(), tx_mcp.clone())
                })
                .await;
            });

            // Force transparent window on macOS using native API
            if let Some(w) = app.get_webview_window("aemeath") {
                #[cfg(target_os = "macos")]
                {
                    let _ = w.set_background_color(Some(Color(0, 0, 0, 0)));
                    if let Ok(ns_window_ptr) = w.ns_window() {
                        let ns_window: &NSWindow = unsafe { &*ns_window_ptr.cast() };
                        // Make NSWindow transparent
                        ns_window.setOpaque(false);
                        ns_window.setBackgroundColor(Some(&NSColor::clearColor()));
                    }
                }
            }

            // Listen to broadcast channel, forward state changes to frontend
            let handle = app.handle().clone();
            let mut rx = tx.subscribe();
            let handle2 = handle.clone();
            tokio::spawn(async move {
                while let Ok(event) = rx.recv().await {
                    let _ = handle2.emit("state-change", event);
                }
            });

            // Send initial waving state
            let _ = handle.emit(
                "state-change",
                StateChangeEvent {
                    animation: "waving".to_string(),
                    bubble: "爱弥斯已连接 Codex~".to_string(),
                },
            );

            // Enable system tray
            if let Err(e) = tray::setup(app) {
                eprintln!("Failed to setup tray: {}", e);
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Aemeath Pet");
}

async fn serve_with_retry<F>(name: &'static str, address: &'static str, mut router: F)
where
    F: FnMut() -> axum::Router,
{
    loop {
        match tokio::net::TcpListener::bind(address).await {
            Ok(listener) => {
                println!("{name} server listening on http://{address}");
                if let Err(error) = axum::serve(listener, router()).await {
                    eprintln!("{name} server stopped: {error}; retrying");
                }
            }
            Err(error) => {
                eprintln!("{name} server could not bind {address}: {error}; retrying");
            }
        }

        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}

#[tauri::command]
fn start_drag(window: tauri::Window) {
    let _ = window.start_dragging();
}

#[tauri::command]
fn set_pet_scale(window: tauri::WebviewWindow, factor: f64) -> Result<(), String> {
    if ![1.0, 1.5, 2.0]
        .iter()
        .any(|supported| (factor - supported).abs() < f64::EPSILON)
    {
        return Err(format!("unsupported pet scale: {factor}"));
    }

    tray::apply_scale(&window, factor).map_err(|error| error.to_string())
}
