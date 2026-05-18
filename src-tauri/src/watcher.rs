//! Background auto-index daemon.
//!
//! Polls `~/.claude/projects` every `period` seconds. For every top-level
//! `*.jsonl` whose `mtime` advanced since we last looked, re-parse + upsert
//! into Qdrant. Emits a Tauri event `index-updated` with per-tick stats so
//! the frontend can light up a fade-in chip.
//!
//! We poll instead of using `notify`/FSEvents to stay portable, avoid macOS
//! permission prompts, and dodge the duplicate-event firehose that comes
//! with editors writing temp files. On 80+ sessions one tick is well under
//! 100 ms once the mtime cache is warm.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex as AsyncMutex;
use walkdir::WalkDir;

use crate::commands::AppStateArc;
use crate::indexer;
use crate::parser;

/// Per-tick stats payload emitted on the `index-updated` Tauri event.
#[derive(Debug, Clone, Default, Serialize)]
pub struct TickStats {
    pub checked: usize,
    pub reindexed: usize,
    pub new: usize,
    pub errors: usize,
    pub elapsed_ms: u128,
}

/// Spawn the background watcher. Returns immediately; the task runs until the
/// process exits.
pub fn start_watcher(
    state: AppStateArc,
    app: AppHandle,
    root: PathBuf,
    period: Duration,
) {
    let mtimes: Arc<AsyncMutex<HashMap<PathBuf, SystemTime>>> =
        Arc::new(AsyncMutex::new(HashMap::new()));

    tokio::spawn(async move {
        eprintln!(
            "[memex] watcher started · root={} · period={}s",
            root.display(),
            period.as_secs()
        );

        // First tick: short delay so the UI window gets to paint before we
        // potentially load fastembed (~130 MB on first launch).
        let mut delay = Duration::from_secs(5);
        loop {
            tokio::time::sleep(delay).await;
            delay = period;

            if !root.exists() {
                // Don't churn — wait the full period.
                continue;
            }

            let start = std::time::Instant::now();
            let stats = match tick(&state, &app, &root, &mtimes).await {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("[memex] watcher tick failed: {e:#}");
                    continue;
                }
            };

            if stats.reindexed > 0 || stats.new > 0 {
                eprintln!(
                    "[memex] watcher tick · checked={} new={} reindexed={} errors={} ({} ms)",
                    stats.checked,
                    stats.new,
                    stats.reindexed,
                    stats.errors,
                    start.elapsed().as_millis()
                );
                let _ = app.emit("index-updated", &stats);
            }
        }
    });
}

async fn tick(
    state: &AppStateArc,
    _app: &AppHandle,
    root: &Path,
    mtimes: &Arc<AsyncMutex<HashMap<PathBuf, SystemTime>>>,
) -> anyhow::Result<TickStats> {
    let mut stats = TickStats::default();
    let started = std::time::Instant::now();

    // 1. Cheap walk — collect every (path, mtime) that *might* need work.
    //    We do this before touching Qdrant/Embedder so the slow path only
    //    fires when the corpus actually changed.
    let mut candidates: Vec<(PathBuf, SystemTime)> = Vec::new();
    for entry in WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("jsonl") {
            continue;
        }
        if path.components().any(|c| c.as_os_str() == "subagents") {
            continue;
        }
        stats.checked += 1;
        let Ok(meta) = entry.metadata() else { continue };
        let Ok(modified) = meta.modified() else { continue };
        candidates.push((path.to_path_buf(), modified));
    }

    // 2. Filter to files whose mtime is new-to-us OR advanced.
    let mut to_index: Vec<(PathBuf, SystemTime, bool)> = Vec::new();
    {
        let mtimes_guard = mtimes.lock().await;
        for (path, modified) in &candidates {
            match mtimes_guard.get(path) {
                Some(prev) if *prev >= *modified => {
                    // Up to date — skip.
                }
                Some(_) => to_index.push((path.clone(), *modified, false)),
                None => to_index.push((path.clone(), *modified, true)),
            }
        }
    }

    if to_index.is_empty() {
        stats.elapsed_ms = started.elapsed().as_millis();
        return Ok(stats);
    }

    // 3. Lazy-init the heavy state only when we know there's work.
    let qdrant = state.qdrant().await?;
    let embedder = state.embedder().await?;

    let mut to_remember: Vec<(PathBuf, SystemTime)> = Vec::with_capacity(to_index.len());
    for (path, modified, is_new) in to_index {
        let session = match parser::parse_session(&path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[memex] watcher parse_session failed for {}: {:#}", path.display(), e);
                stats.errors += 1;
                continue;
            }
        };
        match indexer::index_session(&qdrant, &embedder, &session).await {
            Ok(()) => {
                if is_new {
                    stats.new += 1;
                } else {
                    stats.reindexed += 1;
                }
                to_remember.push((path, modified));
            }
            Err(e) => {
                eprintln!("[memex] watcher index_session failed for {}: {:#}", path.display(), e);
                stats.errors += 1;
            }
        }
    }

    // 4. Update the mtime cache atomically.
    if !to_remember.is_empty() {
        let mut mtimes_guard = mtimes.lock().await;
        for (path, modified) in to_remember {
            mtimes_guard.insert(path, modified);
        }
    }

    stats.elapsed_ms = started.elapsed().as_millis();
    Ok(stats)
}
