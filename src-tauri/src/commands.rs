//! Tauri command surface — what the frontend can `invoke()`.
//!
//! Each command takes `State<AppState>` (a long-lived holder of the Qdrant
//! client + Embedder) and returns `Result<T, String>` so errors can cross the
//! IPC boundary.

use std::path::PathBuf;
use std::sync::Arc;

use qdrant_client::Qdrant;
use tauri::State;

use crate::indexer::{
    self, Embedder, LensWeights, SearchHit, Topology, COLLECTION,
};
use crate::parser;

pub struct AppState {
    pub qdrant: Qdrant,
    pub embedder: Embedder,
}

pub type AppStateArc = Arc<AppState>;

fn stringify<E: std::fmt::Display>(e: E) -> String {
    format!("{e:#}")
}

#[tauri::command]
pub async fn lens_search(
    state: State<'_, AppStateArc>,
    query: String,
    weights: Option<LensWeights>,
    limit: Option<u64>,
) -> Result<Vec<SearchHit>, String> {
    let weights = weights.unwrap_or_default();
    let limit = limit.unwrap_or(20);
    indexer::lens_search(&state.qdrant, &state.embedder, &query, &weights, limit, 60)
        .await
        .map_err(stringify)
}

#[tauri::command]
pub async fn mix_match(
    state: State<'_, AppStateArc>,
    positive: Vec<String>,
    negative: Vec<String>,
    limit: Option<u64>,
) -> Result<Vec<SearchHit>, String> {
    indexer::mix_match(&state.qdrant, &positive, &negative, limit.unwrap_or(20))
        .await
        .map_err(stringify)
}

#[tauri::command]
pub async fn topology(
    state: State<'_, AppStateArc>,
    sample: Option<u32>,
    per_point: Option<u32>,
) -> Result<Topology, String> {
    indexer::topology(
        &state.qdrant,
        sample.unwrap_or(80),
        per_point.unwrap_or(5),
    )
    .await
    .map_err(stringify)
}

#[tauri::command]
pub async fn recall(
    state: State<'_, AppStateArc>,
    error_text: String,
    limit: Option<u64>,
) -> Result<Vec<SearchHit>, String> {
    indexer::recall(
        &state.qdrant,
        &state.embedder,
        &error_text,
        limit.unwrap_or(5),
    )
    .await
    .map_err(stringify)
}

#[tauri::command]
pub async fn get_session(
    state: State<'_, AppStateArc>,
    session_id: String,
) -> Result<Option<serde_json::Value>, String> {
    let payload = indexer::get_session_payload(&state.qdrant, &session_id)
        .await
        .map_err(stringify)?;
    match payload {
        None => Ok(None),
        Some(p) => {
            let mut out = serde_json::Map::new();
            for (k, v) in p {
                out.insert(k, qdrant_value_to_json(v));
            }
            Ok(Some(serde_json::Value::Object(out)))
        }
    }
}

#[tauri::command]
pub async fn get_session_turns(
    state: State<'_, AppStateArc>,
    session_id: String,
) -> Result<serde_json::Value, String> {
    // Pull the payload to find the original source jsonl path, then re-parse
    // it so the replay can stream turn-by-turn without bloating Qdrant payloads.
    let payload = indexer::get_session_payload(&state.qdrant, &session_id)
        .await
        .map_err(stringify)?;
    let Some(payload) = payload else {
        return Err(format!("session {session_id} not in index"));
    };
    let source = payload
        .get("source_path")
        .and_then(|v| v.kind.as_ref())
        .and_then(|k| match k {
            qdrant_client::qdrant::value::Kind::StringValue(s) => Some(s.clone()),
            _ => None,
        })
        .ok_or_else(|| "session payload missing source_path".to_string())?;
    let session = parser::parse_session(std::path::Path::new(&source))
        .map_err(stringify)?;
    serde_json::to_value(&session).map_err(stringify)
}

fn qdrant_value_to_json(v: qdrant_client::qdrant::Value) -> serde_json::Value {
    use qdrant_client::qdrant::value::Kind;
    use serde_json::Value as J;
    match v.kind {
        Some(Kind::NullValue(_)) | None => J::Null,
        Some(Kind::BoolValue(b)) => J::Bool(b),
        Some(Kind::IntegerValue(i)) => J::Number(i.into()),
        Some(Kind::DoubleValue(d)) => serde_json::Number::from_f64(d)
            .map(J::Number)
            .unwrap_or(J::Null),
        Some(Kind::StringValue(s)) => J::String(s),
        Some(Kind::ListValue(l)) => {
            J::Array(l.values.into_iter().map(qdrant_value_to_json).collect())
        }
        Some(Kind::StructValue(s)) => {
            let mut m = serde_json::Map::new();
            for (k, vv) in s.fields {
                m.insert(k, qdrant_value_to_json(vv));
            }
            J::Object(m)
        }
    }
}

#[tauri::command]
pub async fn snapshot_export(path: PathBuf) -> Result<String, String> {
    indexer::snapshot_export(&path).await.map_err(stringify)
}

#[tauri::command]
pub async fn snapshot_import(path: PathBuf) -> Result<(), String> {
    indexer::snapshot_import(&path).await.map_err(stringify)
}

/// Returns a quick collection-level health summary for the splash screen.
#[tauri::command]
pub async fn collection_info(
    state: State<'_, AppStateArc>,
) -> Result<serde_json::Value, String> {
    let info = state
        .qdrant
        .collection_info(COLLECTION)
        .await
        .map_err(stringify)?;
    let r = info.result.unwrap_or_default();
    Ok(serde_json::json!({
        "collection": COLLECTION,
        "points_count": r.points_count.unwrap_or(0),
        "indexed_vectors_count": r.indexed_vectors_count.unwrap_or(0),
        "status": r.status,
        "segments_count": r.segments_count,
    }))
}

/// Lightweight scan/refresh — re-reads `~/.claude/projects`, indexes anything
/// new. Returns how many sessions are now in the collection.
#[tauri::command]
pub async fn refresh_index(
    state: State<'_, AppStateArc>,
    path: Option<PathBuf>,
) -> Result<usize, String> {
    let root = path.unwrap_or_else(default_projects_root);
    let sessions = parser::scan_dir(&root).map_err(stringify)?;
    indexer::ensure_collection(&state.qdrant)
        .await
        .map_err(stringify)?;
    let ok = indexer::bulk_index(&state.qdrant, &state.embedder, &sessions)
        .await
        .map_err(stringify)?;
    Ok(ok)
}

fn default_projects_root() -> PathBuf {
    if let Ok(home) = std::env::var("HOME") {
        let mut p = PathBuf::from(home);
        p.push(".claude");
        p.push("projects");
        p
    } else {
        PathBuf::from(".claude/projects")
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct RecentError {
    pub session_id: String,
    pub project_name: String,
    pub error_text: String,
    pub source_path: String,
    pub seen_at_iso: String,
}

/// Phase 6 polling-style recall trigger. Walks `~/.claude/projects`, finds any
/// `*.jsonl` modified within `since_seconds`, re-parses, and surfaces the most
/// recent `tool_result.is_error` (or assistant-text "Error:" line). Frontend
/// polls every ~10s; on hit it calls `recall(error_text)` and animates the
/// banner.
///
/// We trade real OS file watching for portability — polling is reliable, has
/// no permission edge cases, and on 80 sessions costs <50 ms per tick.
#[tauri::command]
pub async fn tail_recent_errors(
    path: Option<PathBuf>,
    since_seconds: Option<u64>,
) -> Result<Vec<RecentError>, String> {
    use chrono::Utc;
    use walkdir::WalkDir;

    let root = path.unwrap_or_else(default_projects_root);
    let cutoff = std::time::SystemTime::now()
        - std::time::Duration::from_secs(since_seconds.unwrap_or(60));
    let now_iso = Utc::now().to_rfc3339();
    let mut out: Vec<RecentError> = Vec::new();

    for entry in WalkDir::new(&root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let p = entry.path();
        if p.extension().and_then(|e| e.to_str()) != Some("jsonl") {
            continue;
        }
        if p.components().any(|c| c.as_os_str() == "subagents") {
            continue;
        }
        let Ok(meta) = entry.metadata() else { continue };
        let Ok(modified) = meta.modified() else { continue };
        if modified < cutoff {
            continue;
        }
        let Ok(session) = parser::parse_session(p) else { continue };
        let mut latest_err: Option<String> = None;
        for turn in session.turns.iter().rev().take(6) {
            if latest_err.is_some() {
                break;
            }
            if let Some(err) = turn.tool_results.iter().rev().find(|r| r.is_error) {
                let head: String = err.content.chars().take(800).collect();
                latest_err = Some(head);
                break;
            }
            for line in turn.text.lines().rev() {
                let lower = line.to_ascii_lowercase();
                if lower.contains("error:") || lower.contains("traceback") || lower.contains("panic") {
                    latest_err = Some(line.trim().to_string());
                    break;
                }
            }
        }
        if let Some(err) = latest_err {
            out.push(RecentError {
                session_id: session.session_id,
                project_name: session.project_name.unwrap_or_default(),
                error_text: err,
                source_path: p.to_string_lossy().to_string(),
                seen_at_iso: now_iso.clone(),
            });
        }
    }
    Ok(out)
}
