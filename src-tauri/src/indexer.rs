//! Qdrant indexing for Memex.
//!
//! One point per session, 5 named dense vectors (BGE-small-en-v1.5, 384-d):
//!
//! - `content` — full conversation transcript text (user+assistant prose only)
//! - `tool`    — tool call descriptors (`<ToolName>: <key-input>` lines)
//! - `path`    — file paths mentioned anywhere (tool inputs, text references)
//! - `error`   — tool_result text where `is_error=true` + "Error:" phrases
//! - `code`    — fenced code blocks + Edit/Write contents
//!
//! BGE-small is used for all 5 vectors in this MVP. Plan §2 calls for BM42
//! sparse on `path` and ColBERT multi-vector on `content` — those are deferred
//! to Phase 3+ once the search loop is wired end-to-end.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;

use anyhow::{Context, Result};
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use once_cell::sync::Lazy;
use qdrant_client::{
    qdrant::{
        vectors_config, CreateCollectionBuilder, CreateFieldIndexCollectionBuilder, Distance,
        FieldType, PointStruct, Query, QueryPointsBuilder, UpsertPointsBuilder,
        VectorParamsBuilder, VectorParamsMap, VectorsConfig,
    },
    Payload, Qdrant,
};
use regex::Regex;
use serde_json::json;

use crate::parser::{Session, ToolCall, TurnRole};

pub const COLLECTION: &str = "memex_sessions";
pub const EMBED_DIM: u64 = 384;
pub const VECTORS: &[&str] = &["content", "tool", "path", "error", "code"];

const MAX_CHARS_PER_VECTOR: usize = 6_000;
const EMBED_BATCH: usize = 32;

static CODE_FENCE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"```[\w+-]*\n([\s\S]*?)```").unwrap());

/// Wraps a fastembed `TextEmbedding` (BGE-small-en-v1.5). The model needs
/// `&mut self` to embed (internal ONNX session state), so we serialize access
/// via a `Mutex` and let callers use `&Embedder`.
pub struct Embedder {
    inner: Mutex<TextEmbedding>,
}

impl Embedder {
    pub fn new() -> Result<Self> {
        let model = TextEmbedding::try_new(
            InitOptions::new(EmbeddingModel::BGESmallENV15).with_show_download_progress(true),
        )
        .context("loading BGE-small-en-v1.5 fastembed model")?;
        Ok(Self {
            inner: Mutex::new(model),
        })
    }

    pub fn embed(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }
        let mut out = Vec::with_capacity(texts.len());
        let mut model = self
            .inner
            .lock()
            .map_err(|e| anyhow::anyhow!("embedder mutex poisoned: {e}"))?;
        for chunk in texts.chunks(EMBED_BATCH) {
            let chunk_refs: Vec<&str> = chunk.iter().map(|s| s.as_str()).collect();
            let batch = model
                .embed(chunk_refs, None)
                .context("fastembed embed() failed")?;
            out.extend(batch);
        }
        Ok(out)
    }
}

/// Connect to local Qdrant (default `http://localhost:6334` gRPC).
pub async fn connect() -> Result<Qdrant> {
    let url = std::env::var("MEMEX_QDRANT_URL").unwrap_or_else(|_| "http://localhost:6334".into());
    Qdrant::from_url(&url)
        .build()
        .with_context(|| format!("connecting to qdrant at {url}"))
}

/// Create the collection + payload indexes if not present (idempotent).
pub async fn ensure_collection(client: &Qdrant) -> Result<()> {
    if client.collection_exists(COLLECTION).await? {
        return Ok(());
    }
    let mut params: HashMap<String, _> = HashMap::new();
    for name in VECTORS {
        params.insert(
            (*name).to_string(),
            VectorParamsBuilder::new(EMBED_DIM, Distance::Cosine).build(),
        );
    }
    let vectors_cfg: VectorsConfig =
        vectors_config::Config::ParamsMap(VectorParamsMap { map: params }).into();

    client
        .create_collection(
            CreateCollectionBuilder::new(COLLECTION).vectors_config(vectors_cfg),
        )
        .await?;

    for (field, ftype) in [
        ("project_name", FieldType::Keyword),
        ("project_path", FieldType::Keyword),
        ("git_branch", FieldType::Keyword),
        ("ai_title", FieldType::Text),
        ("start_ts", FieldType::Integer),
        ("has_errors", FieldType::Bool),
    ] {
        // Best-effort: indexes already exist on re-run.
        let _ = client
            .create_field_index(
                CreateFieldIndexCollectionBuilder::new(COLLECTION, field, ftype).build(),
            )
            .await;
    }
    Ok(())
}

pub fn session_extracts(session: &Session) -> [(String, String); 5] {
    let content = build_content(session);
    let tool = build_tool(session);
    let path = build_path(session);
    let error = build_error(session);
    let code = build_code(session);
    [
        ("content".into(), cap(&content)),
        ("tool".into(), cap(&tool)),
        ("path".into(), cap(&path)),
        ("error".into(), cap(&error)),
        ("code".into(), cap(&code)),
    ]
}

fn cap(s: &str) -> String {
    if s.chars().count() <= MAX_CHARS_PER_VECTOR {
        s.to_string()
    } else {
        s.chars().take(MAX_CHARS_PER_VECTOR).collect()
    }
}

fn build_content(s: &Session) -> String {
    let mut buf = String::new();
    if let Some(t) = &s.ai_title {
        buf.push_str("title: ");
        buf.push_str(t);
        buf.push('\n');
    }
    for turn in &s.turns {
        if turn.text.is_empty() {
            continue;
        }
        match turn.role {
            TurnRole::User => buf.push_str("U: "),
            TurnRole::Assistant => buf.push_str("A: "),
            TurnRole::System => continue,
        }
        buf.push_str(&turn.text);
        buf.push('\n');
    }
    if buf.is_empty() {
        buf.push_str(s.project_name.as_deref().unwrap_or("session"));
    }
    buf
}

fn build_tool(s: &Session) -> String {
    let mut lines = Vec::new();
    for turn in &s.turns {
        for tc in &turn.tool_calls {
            lines.push(format!("{}: {}", tc.name, tool_input_snippet(tc)));
        }
    }
    if lines.is_empty() {
        lines.push("(no tool calls)".to_string());
    }
    lines.join("\n")
}

fn tool_input_snippet(tc: &ToolCall) -> String {
    let preview_keys = [
        "command",
        "file_path",
        "url",
        "query",
        "pattern",
        "path",
        "description",
    ];
    for k in preview_keys {
        if let Some(v) = tc.input.get(k).and_then(|x| x.as_str()) {
            if !v.is_empty() {
                return v.chars().take(160).collect();
            }
        }
    }
    let s = serde_json::to_string(&tc.input).unwrap_or_default();
    s.chars().take(160).collect()
}

fn build_path(s: &Session) -> String {
    use std::collections::BTreeSet;
    let mut paths: BTreeSet<String> = BTreeSet::new();
    if let Some(p) = &s.project_path {
        paths.insert(p.clone());
    }
    for turn in &s.turns {
        for tc in &turn.tool_calls {
            for k in ["file_path", "path", "notebook_path"] {
                if let Some(p) = tc.input.get(k).and_then(|x| x.as_str()) {
                    if !p.is_empty() {
                        paths.insert(p.to_string());
                    }
                }
            }
            if let Some(url) = tc.input.get("url").and_then(|x| x.as_str()) {
                paths.insert(url.to_string());
            }
        }
    }
    if paths.is_empty() {
        return s
            .project_path
            .clone()
            .unwrap_or_else(|| "(no paths)".into());
    }
    paths.into_iter().collect::<Vec<_>>().join("\n")
}

fn build_error(s: &Session) -> String {
    let mut chunks = Vec::new();
    for turn in &s.turns {
        for r in &turn.tool_results {
            if r.is_error {
                chunks.push(r.content.chars().take(800).collect::<String>());
            }
        }
        if matches!(turn.role, TurnRole::Assistant) {
            for line in turn.text.lines() {
                let lower = line.to_ascii_lowercase();
                if lower.contains("error:")
                    || lower.contains("failed")
                    || lower.contains("traceback")
                    || lower.contains("panic")
                    || lower.contains("exception")
                {
                    chunks.push(line.trim().to_string());
                }
            }
        }
    }
    if chunks.is_empty() {
        chunks.push("(no errors)".to_string());
    }
    chunks.join("\n")
}

fn build_code(s: &Session) -> String {
    let mut blobs = Vec::new();
    for turn in &s.turns {
        for cap in CODE_FENCE.captures_iter(&turn.text) {
            if let Some(m) = cap.get(1) {
                blobs.push(m.as_str().to_string());
            }
        }
        for tc in &turn.tool_calls {
            for k in ["new_string", "content"] {
                if let Some(v) = tc.input.get(k).and_then(|x| x.as_str()) {
                    if !v.is_empty() {
                        blobs.push(v.chars().take(800).collect());
                    }
                }
            }
        }
    }
    if blobs.is_empty() {
        blobs.push("(no code)".to_string());
    }
    blobs.join("\n---\n")
}

/// Deterministic point ID derived from `session_id` so reindex is idempotent.
pub fn point_id(session_id: &str) -> String {
    let ns = uuid::Uuid::NAMESPACE_DNS;
    uuid::Uuid::new_v5(&ns, session_id.as_bytes()).to_string()
}

fn session_payload(s: &Session) -> Payload {
    let mut tool_count = 0usize;
    let mut has_errors = false;
    for turn in &s.turns {
        tool_count += turn.tool_calls.len();
        if turn.tool_results.iter().any(|r| r.is_error) {
            has_errors = true;
        }
    }
    let payload = json!({
        "session_id": s.session_id,
        "source_path": s.source_path.to_string_lossy(),
        "project_name": s.project_name.as_deref().unwrap_or(""),
        "project_path": s.project_path.as_deref().unwrap_or(""),
        "git_branch": s.git_branch.as_deref().unwrap_or(""),
        "claude_version": s.claude_version.as_deref().unwrap_or(""),
        "ai_title": s.ai_title.as_deref().unwrap_or(""),
        "start_iso": s.start_time.map(|t| t.to_rfc3339()).unwrap_or_default(),
        "end_iso": s.end_time.map(|t| t.to_rfc3339()).unwrap_or_default(),
        "start_ts": s.start_time.map(|t| t.timestamp()).unwrap_or(0),
        "end_ts": s.end_time.map(|t| t.timestamp()).unwrap_or(0),
        "user_turns": s.event_counts.user,
        "assistant_turns": s.event_counts.assistant,
        "tool_count": tool_count,
        "has_errors": has_errors,
    });
    Payload::try_from(payload).expect("payload conversion")
}

pub fn build_point(session: &Session, vectors_by_name: Vec<(String, Vec<f32>)>) -> PointStruct {
    let id = point_id(&session.session_id);
    let payload = session_payload(session);
    let vec_map: HashMap<String, Vec<f32>> = vectors_by_name.into_iter().collect();
    PointStruct::new(id, vec_map, payload)
}

pub async fn index_session(
    client: &Qdrant,
    embedder: &Embedder,
    session: &Session,
) -> Result<()> {
    let extracts = session_extracts(session);
    let texts: Vec<String> = extracts.iter().map(|(_, t)| t.clone()).collect();
    let vectors = embedder.embed(texts)?;
    let named: Vec<(String, Vec<f32>)> = extracts
        .into_iter()
        .map(|(k, _)| k)
        .zip(vectors.into_iter())
        .collect();
    let point = build_point(session, named);
    client
        .upsert_points(UpsertPointsBuilder::new(COLLECTION, vec![point]).wait(true))
        .await?;
    Ok(())
}

pub async fn bulk_index(
    client: &Qdrant,
    embedder: &Embedder,
    sessions: &[Session],
) -> Result<usize> {
    use indicatif::{ProgressBar, ProgressStyle};
    let pb = ProgressBar::new(sessions.len() as u64);
    pb.set_style(
        ProgressStyle::with_template("{wide_bar} {pos}/{len} ({eta}) {msg}")
            .unwrap()
            .progress_chars("=> "),
    );
    let mut ok = 0;
    for s in sessions {
        let label = s
            .project_name
            .clone()
            .unwrap_or_else(|| s.session_id.clone());
        pb.set_message(label);
        match index_session(client, embedder, s).await {
            Ok(()) => ok += 1,
            Err(e) => pb.println(format!("  ⚠ {}: {:#}", s.session_id, e)),
        }
        pb.inc(1);
    }
    pb.finish_with_message("done");
    Ok(ok)
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SearchHit {
    pub score: f32,
    pub session_id: String,
    pub project_name: String,
    pub ai_title: String,
    pub start_iso: String,
}

pub async fn search_content(
    client: &Qdrant,
    embedder: &Embedder,
    query: &str,
    limit: u64,
) -> Result<Vec<SearchHit>> {
    let vecs = embedder.embed(vec![query.to_string()])?;
    let vec = vecs.into_iter().next().context("no embedding for query")?;
    let q: Query = vec.into();
    let res = client
        .query(
            QueryPointsBuilder::new(COLLECTION)
                .query(q)
                .using("content")
                .limit(limit)
                .with_payload(true),
        )
        .await?;

    Ok(res
        .result
        .into_iter()
        .map(|p| SearchHit {
            score: p.score,
            session_id: payload_str(&p.payload, "session_id").unwrap_or_default(),
            project_name: payload_str(&p.payload, "project_name").unwrap_or_default(),
            ai_title: payload_str(&p.payload, "ai_title").unwrap_or_default(),
            start_iso: payload_str(&p.payload, "start_iso").unwrap_or_default(),
        })
        .collect())
}

fn payload_str(
    p: &HashMap<String, qdrant_client::qdrant::Value>,
    key: &str,
) -> Option<String> {
    p.get(key)
        .and_then(|v| v.kind.as_ref())
        .and_then(|k| match k {
            qdrant_client::qdrant::value::Kind::StringValue(s) => Some(s.clone()),
            _ => None,
        })
}

/// Snapshot export — calls Qdrant's HTTP snapshot endpoint and copies the file
/// to `dest`. Returns the chosen filename on the server.
pub async fn snapshot_export(dest: &Path) -> Result<String> {
    let url = std::env::var("MEMEX_QDRANT_HTTP")
        .unwrap_or_else(|_| "http://localhost:6333".into());
    let client = reqwest::Client::new();
    let create_url = format!("{url}/collections/{COLLECTION}/snapshots");
    let resp: serde_json::Value = client
        .post(&create_url)
        .send()
        .await
        .with_context(|| format!("POST {create_url}"))?
        .error_for_status()?
        .json()
        .await?;
    let name = resp
        .get("result")
        .and_then(|r| r.get("name"))
        .and_then(|n| n.as_str())
        .context("snapshot name missing in response")?
        .to_string();

    let download_url = format!("{url}/collections/{COLLECTION}/snapshots/{name}");
    let bytes = client
        .get(&download_url)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;
    tokio::fs::create_dir_all(dest.parent().unwrap_or_else(|| Path::new("."))).await?;
    tokio::fs::write(dest, &bytes).await?;
    Ok(name)
}

pub async fn snapshot_import(src: &Path) -> Result<()> {
    let url = std::env::var("MEMEX_QDRANT_HTTP")
        .unwrap_or_else(|_| "http://localhost:6333".into());
    let bytes = tokio::fs::read(src).await?;
    let client = reqwest::Client::new();
    let upload_url = format!("{url}/collections/{COLLECTION}/snapshots/upload?priority=snapshot");
    let part = reqwest::multipart::Part::bytes(bytes).file_name(
        src.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("memex.snapshot")
            .to_string(),
    );
    let form = reqwest::multipart::Form::new().part("snapshot", part);
    client
        .post(&upload_url)
        .multipart(form)
        .send()
        .await?
        .error_for_status()?;
    Ok(())
}
