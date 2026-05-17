pub mod cli;
pub mod commands;
pub mod indexer;
pub mod parser;

use std::sync::Arc;

use tauri::Manager;

use crate::commands::{AppState, AppStateArc};

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // Initialize Qdrant client + embedder once, share via State.
            // setup() runs on the main thread; we block briefly on the runtime
            // to get the connection up before windows show.
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                match init_app_state().await {
                    Ok(state) => {
                        handle.manage::<AppStateArc>(Arc::new(state));
                        eprintln!("[memex] AppState ready (qdrant + embedder)");
                    }
                    Err(e) => {
                        eprintln!("[memex] AppState init FAILED: {e:#}");
                    }
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            commands::lens_search,
            commands::mix_match,
            commands::topology,
            commands::recall,
            commands::get_session,
            commands::snapshot_export,
            commands::snapshot_import,
            commands::collection_info,
            commands::refresh_index,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

async fn init_app_state() -> anyhow::Result<AppState> {
    let qdrant = indexer::connect().await?;
    indexer::ensure_collection(&qdrant).await?;
    let embedder = indexer::Embedder::new()?;
    Ok(AppState { qdrant, embedder })
}
