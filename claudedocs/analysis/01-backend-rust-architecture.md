# Memex Backend — Rust Architecture Analysis

Scope: `src-tauri/src/{parser,indexer,commands,lib,main}.rs` and `src-tauri/Cargo.toml`.
Read-only review. Every claim cites a specific file:line.

---

## 1. Snapshot

Memex's Rust backend is a single Cargo crate (`src-tauri/Cargo.toml:2`) that produces one binary serving two roles: a Tauri 2 desktop GUI and a `clap`-based CLI, dispatched in `main.rs:32-48` on whether `argv[1]` matches `CLI_SUBCOMMANDS` (`main.rs:7-10`). At its core, `parser.rs` reads Claude Code session JSONL files into a `Session`/`Turn`/`ToolCall` model (`parser.rs:21-77`), `indexer.rs` derives five named text extracts per session (`indexer.rs:172-185`), embeds each with fastembed BGE-small-en-v1.5 at 384-d cosine (`indexer.rs:38-46`), and upserts one Qdrant point per session keyed by `uuid_v5(session_id)` (`indexer.rs:336-339`) so re-indexing is idempotent. All higher-level surfaces — Lens, Mix & Match, Topology MST, Recall, Predict — are pure Qdrant queries plus in-process aggregation; there is **no runtime LLM call** anywhere in the codebase, consistent with the project's "Think Outside the Bot" invariant. Lazy initialization of the Qdrant client and the embedder behind `AsyncMutex<Option<Arc<…>>>` slots in `AppState` (`commands.rs:29-32, 48-80`) lets the GUI window open instantly and self-heal when Qdrant comes up later.

---

## 2. Module map

Listed in dependency order. Line counts from `wc -l` on each file.

### `parser.rs` — 406 LOC

- The pure layer. Zero IO beyond the file system, zero Qdrant, zero embeddings.
- Defines `Session` (`parser.rs:20-33`), `Turn` (`parser.rs:43-54`), `ToolCall` (`parser.rs:64-69`), `ToolResult` (`parser.rs:71-77`), and the `TurnRole` enum (`parser.rs:56-62`).
- `parse_session()` (`parser.rs:80-192`) walks a JSONL line-by-line, accumulating `user`/`assistant` turns and session-level metadata (`sessionId`, `cwd`, `gitBranch`, `version`, `ai-title`, `timestamp`).
- `parse_turn()` (`parser.rs:194-230`) and `extract_content()` (`parser.rs:232-313`) handle the message content union (string, array of {`text`, `tool_use`, `tool_result`, …}) — `image` and `thinking` blocks are intentionally ignored (`parser.rs:305-307`).
- `scan_dir()` (`parser.rs:327-365`) walks a tree (e.g., `~/.claude/projects`), skipping any path that contains a `subagents` component (`parser.rs:344-349`).
- `summary_line()` (`parser.rs:368-396`) produces the CLI's table row formatting.

### `indexer.rs` — 1581 LOC (the workhorse)

- Owns the entire Qdrant interaction layer plus the embedder wrapper.
- Constants pinning the schema: `COLLECTION = "memex_sessions"` (`indexer.rs:38`), `EMBED_DIM = 384` (`indexer.rs:39`), `VECTORS = ["content", "tool", "path", "error", "code"]` (`indexer.rs:40`), `MAX_CHARS_PER_VECTOR = 6_000` (`indexer.rs:42`), `EMBED_BATCH = 32` (`indexer.rs:43`).
- `Embedder` (`indexer.rs:51-95`) wraps fastembed's `TextEmbedding` behind a `std::sync::Mutex` because `embed()` needs `&mut self` (ONNX session state). Resolves cache dir via `default_fastembed_cache_dir()` (`indexer.rs:101-123`) which special-cases macOS to `~/Library/Caches/dev.sgwannabe.memex/fastembed`.
- Five named-vector text extractors: `build_content` (`indexer.rs:195-218`), `build_tool` (`indexer.rs:220-231`), `build_path` (`indexer.rs:254-281`), `build_error` (`indexer.rs:283-309`), `build_code` (`indexer.rs:311-333`).
- Indexing pipeline: `ensure_collection` (`indexer.rs:134-170`) → `session_extracts` (`indexer.rs:172-185`) → `index_session` (`indexer.rs:377-395`) → `bulk_index` (`indexer.rs:407-461`) with duplicate-sessionId detection (`indexer.rs:420-448`).
- Search primitives: `lens_search` (`indexer.rs:539-615`), `mix_match` (`indexer.rs:628-694`), `topology` (`indexer.rs:762-909`) plus `compute_insights` (`indexer.rs:915-1116`), `recall` (`indexer.rs:1151-1189`), `predict_next_actions` (`indexer.rs:1245-1414`).
- Snapshot HTTP layer: `snapshot_export` (`indexer.rs:1528-1559`), `snapshot_import` (`indexer.rs:1561-1581`).

### `commands.rs` — 503 LOC

- The IPC adapter. Wraps each indexer primitive in a `#[tauri::command]` returning `Result<T, String>` so errors serialize cleanly across the IPC boundary (`commands.rs:85-87`).
- `AppState` holds the two lazy slots (`commands.rs:29-32`) plus accessor methods `qdrant()` (`commands.rs:48-63`) and `embedder()` (`commands.rs:65-80`); the embedder loads under `spawn_blocking` so the ~130 MB first-time ONNX download doesn't park a tokio worker (`commands.rs:73`).
- Pure-parser commands that don't need Qdrant: `list_sessions` (`commands.rs:362-376`), `tail_recent_errors` (`commands.rs:407-503`). The latter maintains a per-path mtime cache in a global `Mutex<HashMap>` (`commands.rs:395-396`) so polling stays cheap.
- `qdrant_value_to_json` (`commands.rs:200-222`) bridges Qdrant's typed `Value` to `serde_json::Value` for inspector display.

### `lib.rs` — 80 LOC

- Tauri `Builder` entry point. Registers `Arc<AppState>` eagerly with empty lazy slots in `setup()` (`lib.rs:24-26`).
- Builds a minimal tray menu (Open / Snapshot / Quit) wired to webview actions (`lib.rs:29-59`) — the Snapshot item synthesizes a click on `#btn-snapshot` via `w.eval()` (`lib.rs:45-49`).
- Lists every exposed IPC command in `generate_handler!` (`lib.rs:63-77`) — 13 commands.

### `main.rs` — 49 LOC

- The dispatch shim. `CLI_SUBCOMMANDS` array enumerates every CLI verb (`main.rs:7-10`).
- Performs the macOS HOME-CWD switch for non-CLI launches (`main.rs:19-29`) — the bundled `.app` is launched with `CWD=/`, which is read-only on Sequoia/Tahoe and breaks every relative-path default.
- Hands off to either `memex_lib::cli::run(args)` or `memex_lib::run()`.

### `Cargo.toml` — 45 lines

- Pins the major-version surface: `tauri = 2` with `tray-icon` (`Cargo.toml:21`), `qdrant-client = 1` (`Cargo.toml:30`), `fastembed = 5` (`Cargo.toml:32`), `tokio = 1` with `rt-multi-thread, macros, fs, io-util, time, sync` (`Cargo.toml:31`), `petgraph = 0.6` for MST (`Cargo.toml:39`), `reqwest = 0.12` with `rustls-tls, stream, multipart` for the snapshot HTTP path (`Cargo.toml:37`).
- `notify` 6 and `notify-debouncer-full` 0.3 are declared (`Cargo.toml:40-41`) but currently unused — the proactive-recall surface is poll-based, which is documented as intentional.
- `[lib] name = "memex_lib"` (`Cargo.toml:14`) so the binary (`memex`) and library don't collide on Windows; `crate-type = ["staticlib", "cdylib", "rlib"]` (`Cargo.toml:15`).

---

## 3. Data shapes

### `Session` — `parser.rs:20-33`

The unit indexed in Qdrant. Top-level metadata is derived during parse from whichever line first carries it:

- `session_id: String` — falls back to filename stem if no `sessionId` field appears (`parser.rs:85-89`). The "first sessionId wins" logic (`parser.rs:119-124`) handles JSONLs whose early lines lack the field; once seen, it's pinned for the rest of the parse.
- `source_path: PathBuf` — preserved so Replay can re-parse on demand (`commands.rs:188-196`, see §5).
- `project_path / project_name` — first `cwd` seen (`parser.rs:127-132`), with `project_name_from_cwd` (`parser.rs:316-322`) taking the basename for display.
- `git_branch / claude_version / ai_title` — first occurrences (`parser.rs:133-152`). `ai_title` is special-cased on the `ai-title` event type (`parser.rs:146-152`) rather than appearing on every event.
- `start_time / end_time: Option<DateTime<Utc>>` — min/max over every event timestamp (`parser.rs:155-167`). The comparison is RFC 3339 string-parsed via `chrono::DateTime::parse_from_rfc3339` then normalized to UTC.
- `turns: Vec<Turn>` — push order is JSONL order; sidechain status is preserved but not used for filtering.
- `event_counts: EventCounts` — tallies of user/assistant/system/other (`parser.rs:35-41`). The `other` bucket catches `attachment`, `permission-mode`, `ai-title`, and any future event types — only `user` and `assistant` produce turns.

### `Turn` — `parser.rs:43-54`

- `text: String` — concatenated `text`-block content only; tool blobs are split out (`parser.rs:51`).
- `tool_calls: Vec<ToolCall>` — `tool_use` blocks.
- `tool_results: Vec<ToolResult>` — `tool_result` blocks, with `is_error: bool` (`parser.rs:71-77`) used downstream as the `has_errors` payload signal.
- `is_sidechain: bool` (`parser.rs:49`) — captured but unused so far.

### `ToolCall` — `parser.rs:64-69`

- `id`, `name`, `input: serde_json::Value` (`parser.rs:65-68`). Input stays as `Value` so `build_path` can look up known keys (`file_path`, `path`, `notebook_path`, `url`) without re-parsing (`indexer.rs:262-272`).

### Qdrant payload schema (`indexer.rs:341-368`)

Pushed once per `index_session`:

```
session_id, source_path, project_name, project_path, git_branch,
claude_version, ai_title, start_iso, end_iso,
start_ts (int), end_ts (int),
user_turns (usize), assistant_turns (usize),
tool_count (usize), has_errors (bool)
```

Field indexes created in `ensure_collection` (`indexer.rs:154-167`):

- `project_name`, `project_path`, `git_branch` — `Keyword`
- `ai_title` — `Text`
- `start_ts` — `Integer` (range queries on time)
- `has_errors` — `Bool` (the filter used by `recall`, see `indexer.rs:1160-1165`)

Point id is `uuid_v5(NAMESPACE_DNS, session_id)` (`indexer.rs:336-339`). Re-indexing is therefore idempotent — but any change to the id derivation orphans the entire collection, as CLAUDE.md flags.

---

## 4. The five named vectors

All five share the same model (BGE-small-en-v1.5, 384-d cosine, `indexer.rs:139-146`) and the same `MAX_CHARS_PER_VECTOR = 6_000` cap (`indexer.rs:42`). The cap exists because BGE-small tops out at ~512 tokens; at ~4 chars/token, 6 000 chars (~1 500 tokens) means the back of the text is always truncated by the tokenizer, but the front gets through cleanly. `cap()` (`indexer.rs:187-193`) truncates by chars, not bytes — safe for multi-byte UTF-8 but not Unicode-grapheme-aware (a minor concern for emoji-heavy transcripts).

### `content` — `indexer.rs:195-218`

- Built from `ai_title` (if present) + every `User`/`Assistant` turn's `.text`, prefixed `U: ` or `A: `.
- `System` turns are skipped (`indexer.rs:209`).
- Falls back to `project_name` if everything is empty so the embedder never sees an empty string.

### `tool` — `indexer.rs:220-231`

- One line per `ToolCall` formatted as `"<ToolName>: <key-input>"`.
- `tool_input_snippet()` (`indexer.rs:233-252`) pulls the first non-empty value from a fixed priority list (`command, file_path, url, query, pattern, path, description`), truncated to 160 chars; otherwise falls back to a 160-char JSON dump. The 160-char cap is per-tool-call, so a session with 100 tool calls can fill ~16 000 chars before `cap()` clips at 6 000.

### `path` — `indexer.rs:254-281`

- `BTreeSet` of `project_path` + every `file_path` / `path` / `notebook_path` / `url` from any tool input (`indexer.rs:262-272`).
- Sorted/deduped (BTreeSet) so the embedded text is stable across re-indexes — important for the idempotency claim because different file orderings would produce different embeddings even though the point id is fixed.
- Newline-joined for the embedder.

### `error` — `indexer.rs:283-309`

- Two sources: every `tool_result.content` where `is_error == true` (capped at 800 chars per result, `indexer.rs:288`), and every assistant-text line matching one of `error:`, `failed`, `traceback`, `panic`, `exception` (case-insensitive, `indexer.rs:294-302`).
- Sentinel `"(no errors)"` when the session is clean (`indexer.rs:305-307`) — preserves a non-empty embedding but means a query for the literal "no errors" would falsely rank clean sessions; this is offset by the `has_errors` filter on `recall` (`indexer.rs:1160-1165`).

### `code` — `indexer.rs:311-333`

- Two sources: fenced markdown blocks matched by `CODE_FENCE = ```` ```\n(.*?)``` ```` (`indexer.rs:45-46`, applied at `indexer.rs:314-318`), and `Edit`/`Write` payload fields `new_string` and `content` (`indexer.rs:319-326`), each capped at 800 chars per occurrence.
- Joined with `"\n---\n"` separator so the embedder treats them as related-but-distinct chunks rather than one continuous file.

Where extracted: `session_extracts()` (`indexer.rs:172-185`) calls all five builders and returns a fixed-size array `[(String, String); 5]` so downstream `index_session` (`indexer.rs:377-395`) can zip extract names with embedding outputs in a single pass.

---

## 5. Qdrant primitives in use

### `lens_search` — `indexer.rs:539-615`

- API: parallel `client.query(QueryPointsBuilder::new(COLLECTION).query(qvec).using(<vname>).limit(per_vector_limit))` — one per non-zero-weight vector (`indexer.rs:567-575`), dispatched via `try_join_all` (`indexer.rs:576`).
- Combine: weighted sum of cosine scores in Rust (`indexer.rs:578-590`), with weights normalized by `total_w` (`indexer.rs:584`). Each session's per-vector contributions are kept in `SearchHit.vector_scores` for the UI's lens-inspector chips.
- Trade-off vs alternatives: Qdrant supports server-side RRF and arbitrary `Prefetch` formulas, but RRF (`Reciprocal Rank Fusion`) is rank-based and ignores weight magnitudes, while server formulas obscure per-vector contributions. The 5-roundtrip cost is bounded by `try_join_all` and Qdrant handles parallel single-vector queries without HNSW contention.

### `mix_match` — `indexer.rs:628-694`

- API: Qdrant 1.18 Discovery — `ContextInputBuilder` over `ContextInputPair { positive, negative }` pairs (`indexer.rs:646-660`), wrapped in `DiscoverInput { target: …, context: … }` (`indexer.rs:668-671`), passed to `QueryPointsBuilder::query(discover_input).using("content")` (`indexer.rs:673-681`).
- The `target` field is **mandatory in Qdrant 1.18**; the code picks the first positive (or the first negative as fallback) (`indexer.rs:663-667`). CLAUDE.md flags this as a refactor hazard — dropping the target makes the server reject the request.
- Pairing logic when positive/negative count differs: pads with the first element of the shorter side (`indexer.rs:647-657`). A pathological case is "one positive, zero negatives" — the negative gets stand-in'd to the positive itself, which is a degenerate context but doesn't error.
- Always uses the `content` vector (`indexer.rs:677`); there's no per-lens variation of Mix & Match yet.

### `topology` — `indexer.rs:762-909`

- API: `client.search_matrix_pairs(SearchMatrixPointsBuilder::new(COLLECTION).using("content").sample(N).limit(per_point))` (`indexer.rs:769-776`) returns pairwise cosine *similarities* for a sampled subset.
- Edge weight transform: `distance = (1.0 - pair.score).max(0.0)` (`indexer.rs:854`) — critical because MST is a *minimum* spanning tree; similarity must be inverted to distance or the MST picks the *worst* pairs as the backbone (the codebase calls this out at `indexer.rs:844-847`).
- Graph: `petgraph::UnGraph<String, f32>` (`indexer.rs:839`), `min_spanning_tree` + `from_elements` (`indexer.rs:837-859`).
- Payload batch: a single `client.get_points(GetPointsBuilder)` to fetch all node payloads at once (`indexer.rs:800-806`) rather than one round-trip per node.
- Bonus: `compute_insights` (`indexer.rs:915-1116`) does a separate `scan_dir` pass to attach per-project tool/path stats, then identifies "isolated" (no cross-project MST edge, `indexer.rs:1071-1085`) and "near-miss" projects (high cross-project avg sim but pruned from MST, `indexer.rs:1087-1113`). Threshold for near-miss is `avg ≥ 0.50` (`indexer.rs:1097`).
- Trade-off: the alternative is O(N²) explicit cosine computation client-side, which becomes painful at scale. `search_matrix_pairs` is sampled, which means the MST is approximate at high N — but the UI never renders more than `sample = 80` nodes by default (`commands.rs:131`) so this is below the perception threshold.

### `recall` — `indexer.rs:1151-1189`

- API: `client.query(QueryPointsBuilder::new(COLLECTION).query(qvec).using("error").limit(N).filter(must has_errors=true))` (`indexer.rs:1167-1176`).
- The `must` filter on `has_errors` (`indexer.rs:1160-1165`) is what makes the `(no errors)` sentinel safe — clean sessions never reach the ranking.
- Trade-off: one Qdrant call, one named vector, simple filter — about as clean as it gets. No multi-lens because users describing an error care about error-text match, not transcript prose.

### `predict_next_actions` — `indexer.rs:1245-1414`

A composite operation, not a single Qdrant call:

1. `get_session_payload` to retrieve `source_path` (`indexer.rs:1257-1267`).
2. Re-parse active session, embed its last `last_n_turns` turn texts (`indexer.rs:1296-1302`). If the tail text is empty (sometimes the most recent turn is a tool-result with no prose), the code falls back to embedding only the very last turn's anchor text (`indexer.rs:1296-1300`).
3. `client.query(...).using("content").limit(neighbors + 1)` (`indexer.rs:1303-1311`) — `+1` so the active session itself can be filtered out at `indexer.rs:1316-1318`. After filtering, the loop also caps at `neighbors` (`indexer.rs:1322-1324`) to handle the case where the active session wasn't in the result set (e.g., its own embedding is far enough from itself due to averaging artifacts).
4. For each neighbor, `parse_session` again on its `source_path` (`indexer.rs:1342`), find a pivot turn via lexical word-overlap (`find_pivot_turn`, `indexer.rs:1419-1444`), walk forward `horizon` turns collecting tool calls (`indexer.rs:1350-1372`). The pivot finder filters out short words (`len > 4`) to avoid stopword noise (`indexer.rs:1335-1339, 1427-1432`) and falls back to "two-thirds in" if no good overlap signal is found (`indexer.rs:1420, 1439-1443`), on the heuristic that the user is mid-flow.
5. Aggregate by tool name, rank by `0.55 * frequency + 0.45 * confidence` (`indexer.rs:1397-1401`), truncate to 6 (`indexer.rs:1402`). Each surviving prediction also carries a representative example — the action whose source neighbor had the highest similarity score (`indexer.rs:1387-1392`) — so the UI shows a concrete `Bash` command or `Read` file path rather than just a tool-name histogram.

- Trade-off: the alternative would be to push per-turn tool sequences into Qdrant as additional vectors/payloads, doubling or quadrupling the collection size. The current design keeps payload lean and re-parses on demand — same pattern as `get_session_turns`. Pivot detection is intentionally lexical (Jaccard-flavored, `indexer.rs:1424-1438`), not embedding-based, because doing one extra embedding pass per neighbor would dominate the runtime.
- `summarize_tool_input` (`indexer.rs:1446-1480`) hand-codes the per-tool argument-preview rules (e.g., `Bash` shows `command`, `Edit` shows `file_path`, `WebFetch` shows `url`). This is duplicated logic with `tool_input_snippet` (`indexer.rs:233-252`) — both pick a "primary" input field per tool — but the priority lists differ slightly. See refactor note in §11.

### `get_session_turns` — `commands.rs:174-198`

- Not a Qdrant primitive — Qdrant is only used to look up `source_path` from the payload (`commands.rs:181-194`), then `parser::parse_session` re-reads the JSONL (`commands.rs:195`).
- This is the *only* code path that exposes turn-level data to the frontend, and it does so by re-parsing on demand — the explicit trade-off documented in CLAUDE.md (don't push turns into Qdrant payloads "to save a roundtrip").

### `snapshot_export` / `snapshot_import` — `indexer.rs:1528-1581`

- These are the only two primitives that bypass `qdrant-client` and use `reqwest` against Qdrant's HTTP API (`indexer.rs:1530-1532`, `1561-1564`). gRPC `qdrant-client` 1.x does not expose snapshot CRUD; HTTP `POST /collections/{name}/snapshots` does.
- Export: POST to create, GET to download, write to disk (`indexer.rs:1532-1558`). The chosen filename is returned to the frontend so it can echo the server-side name to the user.
- Import: multipart-form POST with `priority=snapshot` (`indexer.rs:1566-1578`). The `priority=snapshot` query parameter tells Qdrant to prefer the uploaded snapshot's payloads over any in-memory state for any overlapping point ids — important because import without that flag has surprising merge semantics.
- The HTTP base URL is overridable via `MEMEX_QDRANT_HTTP` (default `localhost:6333`, `indexer.rs:1529-1530, 1562-1563`), parallel to the gRPC override `MEMEX_QDRANT_URL` (`indexer.rs:127`). Two env vars rather than one because Qdrant uses two ports.
- Trade-off: one extra dependency on the HTTP port (6333) being open in addition to gRPC (6334). Acceptable because the dev/prod Qdrant container exposes both anyway. The alternative — waiting for `qdrant-client` to expose snapshot gRPC — would block this feature indefinitely.

---

## 6. AppState + lazy init pattern

`lib.rs::run()` does `app.manage::<AppStateArc>(Arc::new(AppState::new()))` (`lib.rs:25`) — eager registration of a state container whose internal slots are empty (`commands.rs:42-46`). Each accessor implements a "check-then-init" pattern under an `AsyncMutex` so that:

1. On first call, the slot is `None`, the accessor runs `indexer::connect()` and `ensure_collection()` (`commands.rs:53-62`) or `Embedder::new` (`commands.rs:73-77`).
2. On success, it caches the `Arc<…>` into the slot and returns a clone.
3. On failure, the slot stays `None` and the **next call retries** — this is the self-heal property. If the user launches Memex first and Qdrant second, no restart is needed: the next IPC invocation will find Qdrant alive and cache it.

The two slot types differ for a reason:
- `qdrant: AsyncMutex<Option<Arc<Qdrant>>>` (`commands.rs:30`) — Qdrant client setup is async (gRPC dial + `collection_exists` round-trip).
- `embedder: AsyncMutex<Option<Arc<Embedder>>>` (`commands.rs:31`) — but `Embedder::new()` is synchronous CPU-bound work loading an ONNX model, so it runs under `tokio::task::spawn_blocking` (`commands.rs:73-75`) to avoid parking a worker thread for the ~130 MB first-run download.

There's an embedded second mutex inside `Embedder` itself (`indexer.rs:51-53`) — `std::sync::Mutex<TextEmbedding>` — because fastembed's `embed` requires `&mut self`. So the call stack is effectively: `AsyncMutex` (slot acquisition) → `Arc<Embedder>` (cheap clone) → `std::sync::Mutex` (serialized ONNX inference). The inner sync mutex is held only for the duration of `embed()`, so concurrent IPC calls don't deadlock — but they do serialize.

---

## 7. IPC contract

All commands follow the shape `async fn name(state: State<'_, AppStateArc>, …) -> Result<T, String>` (see `commands.rs:89-103, 105-116, 118-137`, etc.). The `stringify` helper at `commands.rs:85-87` does `format!("{e:#}")` to flatten any error chain into a single string — `anyhow`'s alternate-Display walks the cause chain, which matters for "could not connect to Qdrant" producing "connecting to qdrant at http://localhost:6334: transport error: …".

Tauri serializes `Result<T, String>` directly; structured error enums would require manual serde derives plus discriminator handling in the frontend, and CLAUDE.md explicitly calls this out as a deliberate choice.

Command → indexer mapping:

| Command (`lib.rs:63-77`) | Indexer entry | Lazy slots touched |
| --- | --- | --- |
| `lens_search` | `indexer::lens_search` | qdrant + embedder |
| `mix_match` | `indexer::mix_match` | qdrant |
| `topology` | `indexer::topology` (+ `parser::scan_dir`) | qdrant |
| `recall` | `indexer::recall` | qdrant + embedder |
| `get_session` | `indexer::get_session_payload` | qdrant |
| `get_session_turns` | `get_session_payload` + `parser::parse_session` | qdrant |
| `predict_next_actions` | `indexer::predict_next_actions` | qdrant + embedder |
| `snapshot_export` | `indexer::snapshot_export` (HTTP) | neither |
| `snapshot_import` | `indexer::snapshot_import` (HTTP) | neither |
| `collection_info` | `client.collection_info` | qdrant |
| `refresh_index` | `parser::scan_dir` + `indexer::bulk_index` | qdrant + embedder |
| `tail_recent_errors` | parser-only, mtime-cached | neither |
| `list_sessions` | `parser::scan_dir` | neither |

`withGlobalTauri: true` (per CLAUDE.md) means the bridge is at `window.__TAURI__.core.invoke('command_name', { argKey: … })` — no bundler step is needed in the frontend, and the argument names must match the Rust parameter names exactly (`commands.rs:90-95` declares `query`, `weights`, `limit`; the frontend must send those keys). Type juggling: `Option<u64>` arrives as either omitted or a number; `Vec<String>` as a JS array; `LensWeights` as an object whose missing fields default to `1.0` via `#[serde(default = "default_weight")]` (`indexer.rs:476-491`).

---

## 8. CLI/GUI single-binary dispatch

`CLI_SUBCOMMANDS` is a `&[&str]` of 10 entries (`main.rs:7-10`): the seven actual subcommands plus `help`, `--help`, `-h`. The dispatch logic is the same boolean evaluated twice — once before the CWD switch (`main.rs:25`, calling `is_cli_invocation()`) and once after (`main.rs:32-35`, recomputed inline). The duplication is mild and self-contained.

### HOME-CWD workaround (`main.rs:19-29`)

The comment block here is the most important non-obvious piece of launch behavior:

- macOS launches `.app` bundles with `CWD = /` (the system root, read-only on Sequoia/Tahoe).
- Any code path that uses a relative path or default-relative file (the fastembed cache used to be `./.fastembed_cache/`, snapshot prompts, log files) breaks with `EROFS`.
- Solution: when **not** a CLI invocation, `env::set_current_dir(&$HOME)`. CLI launches keep their existing CWD because the user ran the binary from somewhere writable.

This is paired with the cache-dir override inside `default_fastembed_cache_dir()` (`indexer.rs:101-123`), which is a belt-and-braces approach: even if the HOME-CWD trick fails or the user has set an exotic `MEMEX_FASTEMBED_CACHE_DIR`, the embedder lands in a writable directory.

### CLI surface

The handler lives in `cli.rs` (447 LOC, out of scope by the task brief but worth mentioning) and is dispatched at `main.rs:38`. The same `memex_lib` library backs both the GUI commands and the CLI subcommands, so any indexer change is automatically reflected in both surfaces.

---

## 9. Strengths

1. **Pure parser, no IO leakage** (`parser.rs` end-to-end). `Session`/`Turn`/`ToolCall` are plain `Serialize + Deserialize` structs (`parser.rs:20-77`); they have no Qdrant, no embedding, no async. This is what lets `tests/parser.rs` exist as a clean unit suite and lets `list_sessions` (`commands.rs:362-376`) and `tail_recent_errors` (`commands.rs:407-503`) work without Qdrant being up.

2. **Idempotent re-index via deterministic point id** (`indexer.rs:336-339`). `uuid_v5(NAMESPACE_DNS, session_id)` means `bulk_index` can be re-run any number of times without producing duplicates. The duplicate-sessionId detection at `indexer.rs:420-448` is honest about a real edge case (two JSONL files with the same `sessionId`) instead of silently overwriting.

3. **Self-healing lazy init** (`commands.rs:48-80`). Re-checks the slot every call, retries on failure. The user's mental model — "I just need to start Qdrant" — matches the code's behavior without an explicit reconnect path.

4. **Sampling-based topology that scales** (`indexer.rs:769-776`). Using `search_matrix_pairs` with `sample = 80` (`commands.rs:131`) means the MST stays cheap even if the collection grows to thousands of sessions. The MST itself caps at `nodes - 1` edges, so the 3D graph never gets visually busy.

5. **Snapshot via HTTP, search via gRPC** (`indexer.rs:1528-1581` vs everything else). gRPC `qdrant-client` doesn't expose snapshot CRUD; rather than blocking on that limitation, the code uses `reqwest` for the two endpoints that need it and keeps gRPC for everything else. This is a textbook "use the right transport per operation" choice.

6. **Payload index plan matches actual queries** (`indexer.rs:154-167`). The `Bool` index on `has_errors` is *the* index `recall` filters on (`indexer.rs:1162`); the `Integer` index on `start_ts` is the one a future time-range query would use. No speculative indexes, no missing critical ones.

7. **Parallel-by-default lens search** (`indexer.rs:567-576`). `try_join_all` over up to five `client.query` futures collapses wall-clock latency to the slowest single-vector search. The naive sequential implementation would have multiplied that by 5.

---

## 10. Risks / smells

### 1. `indexer.rs` is 1581 LOC in a single file — `indexer.rs:1-1581`

Five logically distinct concerns sit in one module: embedding wrapper (51-95), schema/extracts (134-333), CRUD/bulk indexing (335-461), search primitives (463-1189), prediction (1191-1444), payload helpers (1483-1524), snapshots (1528-1581). The file works because each section is well-commented, but every new contributor reads 1.5k lines before they can confidently add a primitive. **Suggestion:** split into a `indexer/` module — `embed.rs`, `schema.rs`, `crud.rs`, `search.rs`, `predict.rs`, `snapshot.rs` — each ~250 LOC. Public surface (`COLLECTION`, `Embedder`, `SearchHit`, `LensWeights`, `Topology`, `PredictionContext`, the async fns) re-exported from `mod.rs`. No external API changes.

### 2. `std::sync::Mutex` on `Embedder` serializes inference — `indexer.rs:51-53, 82-94`

Multiple concurrent IPC calls (e.g., user scrolls the Topology while typing in the Lens box) will queue through one mutex. fastembed's underlying ONNX session is stateful and not thread-safe, so the mutex is correct, but it's a hard concurrency ceiling. **Suggestion:** for a future bump, pool 2-3 `TextEmbedding` instances in a `crossbeam::queue::ArrayQueue` and check one out per `embed()` call. Cost is ~130 MB × N model copies in RAM — measurable but tolerable on dev machines. Not urgent: the BGE-small inference is ~10 ms per query and only one IPC call typically embeds at a time in practice.

### 3. Errors lose structure crossing the IPC boundary — `commands.rs:85-87`

Every Tauri command returns `Result<T, String>`. The frontend cannot distinguish "Qdrant down" from "session not in index" from "embedder ONNX load failed" without parsing the message string. **Suggestion:** keep the current shape for the MVP, but add a structured error envelope `{ kind: "qdrant_down" | "not_indexed" | "embed_failed" | "parser_failed" | "other", message: String }` once the UI starts wanting branchable behavior (e.g., a "Start Qdrant" button shown only on `qdrant_down`). The change is local to `commands.rs` plus the frontend invoke handlers.

### 4. `cap()` truncates by chars without UTF-8 grapheme awareness — `indexer.rs:187-193`

`s.chars().take(MAX_CHARS_PER_VECTOR).collect()` is safe in the codepoint sense but will split combining-character sequences (emoji modifiers, ZWJ sequences, regional indicator pairs). For tool-call-heavy English transcripts this is invisible; for any session with emoji-laden user messages, the embedder sees malformed glyphs at the boundary. **Suggestion:** use the `unicode-segmentation` crate's `.graphemes(true)` for the truncation. Tiny dep, fixes a real edge case.

### 5. `mix_match` always uses the `content` vector — `indexer.rs:677`

The Lens UI lets the user weight each of five vectors for `lens_search`, but Mix & Match is hard-coded to `content`. Two sessions that share tool patterns but not prose (e.g., two debugging sessions for entirely different bugs that both used `Read` + `Edit` + `Bash`) won't be found as similar. **Suggestion:** thread the `LensWeights` through `mix_match` and run one Discovery query per non-zero-weight vector, combining identically to `lens_search`. Same parallelism, similar code shape.

### 6. `(no errors)` / `(no code)` / `(no paths)` sentinels embed into the vector space — `indexer.rs:227-229, 274-279, 305-307, 329-331`

Every session without errors gets the same `error` embedding (the embedding of the literal string `"(no errors)"`); same for `(no code)` and `(no paths)`. This works because `recall` filters by `has_errors=true` (`indexer.rs:1160-1165`) before ranking, so the bogus embeddings never compete. But there's no equivalent filter on `code`-vector queries — if a future feature does "show me sessions with code similar to X", every empty-code session collides at one cluster point. **Suggestion:** skip empty-source vectors entirely (don't insert them in the point's vector map). Qdrant 1.x supports missing named vectors per point; queries on a vector that some points lack simply skip those points.

### 7. `predict_next_actions` re-parses every neighbor session on each call — `indexer.rs:1342`

For `neighbors = 8` (default, `commands.rs:240`) and a typical 200-turn JSONL, each call does ~1.5 MB of redundant disk + parse work. Multiplied by however often the frontend invokes it as the user scrolls. **Suggestion:** an LRU keyed by `source_path + mtime` holding the parsed `Session`. Even a tiny 16-entry cache covers most repeated calls. The `TAIL_CACHE` pattern in `commands.rs:395-396` is a working precedent.

### 8. Tray "snapshot" item couples backend to frontend DOM — `lib.rs:45-49`

`w.eval("document.getElementById('btn-snapshot')?.click();")` ties the tray menu to a specific DOM id. Refactor the frontend's button id, refactor the tray. **Suggestion:** emit a Tauri event (`app.emit("memex://snapshot-requested", ())`) and let the frontend listen; the backend stays DOM-agnostic.

### 9. `predict_next_actions` ranking weights are hard-coded — `indexer.rs:1397-1401`

`frequency * 0.55 + confidence * 0.45` is fine, but it's a magic constant with no test exercising it and no telemetry to tune it. **Suggestion:** expose as an optional parameter on the command (`Option<f32>` for `frequency_weight`) so the UI could A/B test, with the same default. Zero behavioral change today.

### 10. `tail_recent_errors` cache is a `Lazy<Mutex<…>>` with no eviction — `commands.rs:395-396`

It grows monotonically with the file set. For the current ~80-session corpus this is negligible; for a long-lived install over years, it's a slow leak. Each entry is small (a `PathBuf` + `SystemTime` + `Option<RecentError>`), so the leak is measured in KB/year, not MB. **Suggestion:** add a stale-entry sweep — drop entries whose path no longer exists — every Nth call (e.g., once every 50 ticks). Not urgent.

---

## 11. Refactor opportunities (non-urgent)

Items that are clean wins *when* there's an excuse to touch them, but not worth touching now.

1. **Split `indexer.rs` into a module folder** (see Risk #1). The shape is obvious — five sub-modules along the section comments — and every section already imports its own narrow Qdrant types, so the diff is mostly moving code, not rewriting it.

2. **Pull `LensWeights::iter()` (`indexer.rs:506-515`) up into a trait or const array.** The same five names appear in `VECTORS` (`indexer.rs:40`), `session_extracts` (`indexer.rs:178-184`), and `LensWeights` fields. Keeping them in sync is a manual exercise. A `const VECTOR_NAMES: [&str; 5] = ["content", "tool", "path", "error", "code"];` referenced from all three spots would catch drift at compile time.

3. **Extract `SearchHit` construction.** The same `SearchHit { score, session_id: payload_str(...), project_name: payload_str(...), ai_title: payload_str(...), start_iso: payload_str(...), vector_scores: ... }` pattern appears in `lens_search` (`indexer.rs:604-612`), `mix_match` (`indexer.rs:685-692`), `recall` (`indexer.rs:1180-1187`). A `SearchHit::from_scored_point(&p, score, vector_scores)` helper would deduplicate ~30 lines.

4. **Type-alias the gRPC payload.** `HashMap<String, qdrant_client::qdrant::Value>` appears 8+ times in signatures (`indexer.rs:1486, 1497, 1515`, plus inline in `lens_search`). A `pub type PayloadMap = HashMap<String, qdrant_client::qdrant::Value>;` improves grep-ability and signature readability.

5. **`SearchHit.vector_scores` could be a struct.** The `HashMap<String, f32>` with five known keys is type-loose; consumers in the frontend can't compile-check that they're reading `content` vs `tool`. A `struct PerVectorScores { content: f32, tool: f32, path: f32, error: f32, code: f32 }` with `#[serde(default)]` would be safer at zero runtime cost.

### Premature to extract

- **A general "Qdrant repository" trait** abstracting over named-vector queries. There's only one collection, one schema, one client. The abstraction would add layers without enabling anything (no in-memory test double exists, and Qdrant's client is already mockable enough at the function-boundary level).
- **An async `Embedder` API.** fastembed exposes a sync `embed` because the underlying ONNX is sync; wrapping it in an `async fn` would force `spawn_blocking` inside `Embedder` itself, making the API misleading. The current "sync API, callers decide" shape is honest.
- **A configuration struct** for `MAX_CHARS_PER_VECTOR`, `EMBED_BATCH`, the prediction-ranking weights, etc. These are documented constants, not user-tunable parameters. Promoting them to runtime config encourages drift between deployments and complicates the snapshot story.
- **A separate crate for the indexer.** It's tempting because of the size, but splitting it forces `parser::Session` to be re-exported across a crate boundary, and the current single-crate `cargo build` is fast (single-digit seconds incremental). Not worth the Cargo.lock churn.

---

## 12. Cross-cutting concerns

A few patterns worth pulling out because they recur across files:

### Path conventions

- `~/.claude/projects` is the canonical scan root, resolved by `default_projects_root` (`commands.rs:300-309`) and used by `list_sessions`, `refresh_index`, `topology`, and `tail_recent_errors`. It's also a CLI default in `cli.rs`. The single helper means changing the default in one place propagates.
- The fastembed cache path is platform-resolved (`indexer.rs:101-123`) and overridable via `MEMEX_FASTEMBED_CACHE_DIR`. Tests can point at a temp dir without touching the real cache.
- Subagent JSONL files (`*/subagents/*.jsonl`) are filtered out in both `parser::scan_dir` (`parser.rs:344-349`) and `tail_recent_errors` (`commands.rs:429-431`). Two filter sites means two places to update if the directory layout changes — minor duplication.

### Empty-input guards

The embedder, the prediction loop, and the matrix-pair handler all defend against empty inputs:

- `Embedder::embed` short-circuits on empty input (`indexer.rs:78-80`).
- `predict_next_actions` returns an empty `PredictionContext` if the active session has no turns (`indexer.rs:1269-1277`).
- `topology` substitutes an empty `SearchMatrixPairs` if Qdrant returns `None` (`indexer.rs:781-782, 848`) — important because an empty collection (the very first launch) shouldn't crash.
- `build_*` extractors all push a sentinel (`(no errors)`, `(no code)`, `(no paths)`, `(no tool calls)`) when the source list is empty (`indexer.rs:228, 274-279, 305-307, 329-331`) — see Risk #6 for the trade-off.

### Async vs sync boundaries

- Tauri commands are `async fn` (`commands.rs:89, 105, 118, …`) and Tokio runs the runtime.
- Qdrant client calls are `async` and need `.await`.
- fastembed is sync; the embedder's `embed` method is called from inside async commands without `spawn_blocking` (`indexer.rs:76-94` is called at `indexer.rs:384, 547, 1157, 1296`). The reasoning is that one query's embedding is short (~10 ms) and parking a worker briefly is preferable to the spawn overhead. The 130 MB initial **model load** does use `spawn_blocking` (`commands.rs:73-75`) because that's a one-time multi-second operation.
- Parser is sync and called from async contexts directly (`commands.rs:195, 284, 368, 459`; `indexer.rs:895, 1268, 1342`). For ~80 sessions × low-MB JSONLs this is fine; for a 10k-session corpus the `bulk_index` pass would benefit from `tokio::task::spawn_blocking`.

### `unwrap_or_default` patterns

The codebase leans hard on `unwrap_or_default()` to avoid plumbing `Result` through display layers (e.g., `commands.rs:339-355`, `indexer.rs:606-609, 687-690, 1183-1185`). This is fine for display — an empty `project_name` renders as an empty string in the UI — but it does swallow "data missing" vs "data present and empty" distinctions. Not actionable: the alternative is plumbing `Option<String>` through every serde struct, and the UI side would just `?? ""` it back to empty.

---

## Appendix: file:line citation index

For quick navigation during follow-up work:

- Parser entry: `parser.rs:80`
- Five-vector extractor: `indexer.rs:172-185`
- Embedder mutex: `indexer.rs:51-53`
- Point id derivation: `indexer.rs:336-339`
- Lens parallel queries: `indexer.rs:567-576`
- Mix & Match Discovery: `indexer.rs:668-681`
- Topology MST distance fix: `indexer.rs:844-857`
- Predict pivot detection: `indexer.rs:1419-1444`
- Snapshot HTTP path: `indexer.rs:1528-1581`
- AppState lazy slots: `commands.rs:29-32, 48-80`
- IPC error stringify: `commands.rs:85-87`
- HOME-CWD workaround: `main.rs:19-29`
- Tauri handler registration: `lib.rs:63-77`
- Cargo deps pin: `Cargo.toml:20-41`
