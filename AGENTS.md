# AGENTS.md

This file provides guidance to Codex CLI (and other coding agents that respect `AGENTS.md`) when working with code in this repository.

> Sibling document: `CLAUDE.md` is the equivalent guide for Claude Code. Keep this file and `CLAUDE.md` semantically aligned — when one is updated, mirror the same change here.

## 🔴 ABSOLUTE RULE — Remote target

**모든 git 처리는 `https://github.com/ComBba/memex` 에만 수행된다. 예외 없음.**

- 작업 시작 시 반드시 `git remote -v`로 origin이 `https://github.com/ComBba/memex(.git)` 인지 먼저 확인한다. 다르면 즉시 중단하고 사용자에게 확인 요청.
- README와 `docs/`에는 `sgwannabe/memex`가 과거 표기로 남아있다 — **무시한다.** 모든 `git push` / `gh pr create` / `gh issue` / `gh release` / `gh repo` 호출은 `ComBba/memex`를 대상으로 한다.
- `gh` 명령은 `--repo ComBba/memex`를 명시하거나, 현재 디렉토리의 origin이 ComBba/memex임을 검증한 뒤에만 호출한다.
- 새 remote 추가, upstream 변경, `origin` URL 재설정은 **사용자 명시적 승인 없이 절대 금지**. 다른 fork(예: `sgwannabe/memex`)로의 push/PR 생성도 금지.
- 의심스러우면 행동하지 말고 묻는다. "어느 repo로 push하나요?"는 절대 추측하지 않는다.

## What this project is

Memex is a **single-binary Tauri 2 desktop app** (macOS arm64) that indexes coding-agent session transcripts from BOTH of the following corpora into a **local Qdrant 1.18** instance, and exposes seven non-chat surfaces (Time Machine stack, Topology galaxy, Mix & Match, Proactive Recall, Predict Next Actions, Replay, Lens slider) over named vectors per session point:

- `~/.claude/projects/**/*.jsonl` — Claude Code sessions (`<encoded-cwd>/<session-uuid>.jsonl`)
- `~/.codex/sessions/YYYY/MM/DD/rollout-*.jsonl` — Codex CLI sessions (`{timestamp,type,payload}` envelope)

Memex unifies both agents' history into a single semantic space (KICK `KH-01` multi-agent ingest, v3.2 + multi-agent addendum). Each session point carries `source_agent: "claude_code" | "codex"` in the payload so users can filter / blend / contrast across agents.

Built for Qdrant VSD 2026 — the prompt was *"Think Outside the Bot,"* so by design there is no chat window and **no LLM call at runtime**: all retrieval is `fastembed-rs` BGE-small-en-v1.5 + `qdrant-client` gRPC, both running locally.

Treat the "no LLM at runtime / no chat surface" constraint as a hard product invariant — adding chat-style features defeats the entire pitch.

## Build & run

External prereqs that must be running before the app/CLI is useful:

- **Qdrant 1.18 on `localhost:6333` (HTTP) + `6334` (gRPC)** — either prebuilt binary in `./.qdrant/` or `docker run -d -p 6333:6333 -p 6334:6334 qdrant/qdrant:v1.18.0`
- **macOS Full Disk Access for `Memex.app`** (Sequoia/Tahoe) so it can read `~/.claude/projects` AND `~/.codex/sessions`

```bash
# JS deps (only @tauri-apps/cli — frontend is vanilla)
npm install

# Dev (hot-reload webview)
npm run tauri dev

# Production bundle → src-tauri/target/release/bundle/macos/Memex.app + .dmg
npm run tauri build

# Rust-only build (used by README quick-start to get the CLI without bundling)
cargo build --release --manifest-path src-tauri/Cargo.toml
```

The single produced binary at `src-tauri/target/release/memex` is **both the GUI and the CLI** — `main.rs` dispatches on `argv[1]` (see `CLI_SUBCOMMANDS`). The GUI path also switches CWD to `$HOME` because macOS launches `.app` bundles with `CWD=/` (read-only) and that breaks the fastembed cache + any default relative paths.

## CLI surface

Same binary, every subcommand has `--help`:

```bash
memex scan [--index] [--path PATH] [--agent claude|codex|all] [--limit N]
memex search "query"
memex lens "query" --content 2 --tool 1.5 --code 0.5
memex mix --pos <session_id> --neg <session_id>
memex topology --sample 80 --per-point 6 --out topo.json
memex recall "error text"
memex predict <session_id> --last-n 3 --horizon 3 --neighbors 8
memex snapshot export ./memex.snapshot
memex snapshot import ./memex.snapshot
```

When `--agent` is omitted (default `all`), both Claude and Codex corpora are scanned. Use `--agent claude` / `--agent codex` to restrict.

First `scan --index` downloads the ~130 MB BGE-small ONNX model into `.fastembed_cache/`.

## Tests

```bash
cargo test --manifest-path src-tauri/Cargo.toml             # all tests
cargo test --manifest-path src-tauri/Cargo.toml --test parser   # parser suite only
cargo test --manifest-path src-tauri/Cargo.toml --test parser parse_minimal_one_turn  # single test
```

The suite is **parser-focused** — `src-tauri/tests/parser.rs` + (after KH-01 lands) `src-tauri/tests/codex_parser.rs` against fixtures in `src-tauri/tests/fixtures/{claude,codex}/*.jsonl`. There are no `indexer.rs` or end-to-end tests; integration is verified by running `memex scan --index` against the author's real corpora.

Adding new payload/vector fields requires updating the matching fixture assertions on both parsers or test expectations break silently because nothing else exercises those paths.

## Architecture

```
~/.claude/projects/**/*.jsonl       ~/.codex/sessions/**/*.jsonl
        │  walkdir + serde_json              │  walkdir + serde_json
        ▼                                    ▼
parser.rs (claude schema)        codex_parser.rs (codex schema)
        │                                    │
        └────────────┬───────────────────────┘
                     │  normalize to common Session / Turn / ToolCall structs
                     ▼
            indexer.rs (1581 LOC — the workhorse)
              Embedder (fastembed BGE-small, behind a Mutex)
              ensure_collection · index_session · bulk indexing
              lens_search · mix_match · topology · recall
              get_session_turns (re-parses JSONL on demand for Replay)
              predict_next_actions · snapshot_export · snapshot_import
                     │  qdrant-client 1.18 (gRPC)  +  reqwest (HTTP, snapshots only)
                     ▼
            local Qdrant 1.18  ·  collection `memex_sessions_v3`
              point_id = uuid_v5(NAMESPACE_DNS, session_id)   ← idempotent re-index
              5 named vectors per point (all 384-d cosine BGE-small):
                content · tool · path · error · code
              payload indexes: project_name (tenant), project_path, git_branch,
                               ai_title (text), start_ts_dt (datetime), has_errors,
                               source_agent (keyword)                ← KH-01
                     ▲
                     │ Tauri IPC — window.__TAURI__.core.invoke('lens_search', …)
                     │ withGlobalTauri:true → no bundler needed
                     │
            commands.rs — thin Result<T, String> wrappers (503 LOC)
                     │
                     ▼
            src/{index.html, main.js, styles.css}  — vanilla HTML/CSS/JS, no framework
              Topology surface uses vendored 3d-force-graph (Three.js)
```

### Key design decisions to respect when modifying code

- **One Qdrant point per session, five named vectors.** Each vector has its own source text extract (capped at 6 000 chars to stay under BGE-small's ≈512-token limit while leaving room for partial truncation). The extraction rules live in `indexer.rs` and are documented in `docs/architecture.md` — match them when adding a new vector or you'll silently change rank quality.
- **Payload stays lean — Replay re-parses the JSONL on demand** via `source_path` in the payload. Don't push turn-level data into Qdrant payloads "to save a roundtrip"; the inspector explicitly trades a parse for a smaller collection.
- **Two parsers, one normalized Session shape.** `parser.rs` reads Claude schema; `codex_parser.rs` (KH-01) reads Codex `{timestamp,type,payload}` envelope. Both emit the same `Session`/`Turn`/`ToolCall` structs so indexer.rs stays agent-agnostic. `source_agent` is the only payload-level discriminator.
- **`AppState` is lazy.** `lib.rs::run()` registers `Arc<AppState>` with **empty** Qdrant/Embedder slots; both init on first command. This is what lets the app open instantly and self-heal if the user starts Qdrant *after* Memex.
- **Lens search runs N parallel cosine queries + weighted combine in Rust**, intentionally *not* Qdrant's RRF/formula APIs — RRF ignores weights and server-side formulas are harder to debug. The 5 round-trips are parallelized server-side, so this is a wash on latency and a win on per-vector contribution chips in the UI.
- **`mix_match` requires a `target`** in Qdrant 1.18's `DiscoverInput` — we pass the first positive as the anchor. Don't drop this when refactoring; the server rejects target-less requests.
- **`topology` builds a `petgraph::UnGraph<String, f32>` from `search_matrix_pairs` results, then `min_spanning_tree`** — keep it MST so the SVG/3D layout doesn't get drowned in N² edges.
- **Proactive recall is polling, not `notify`.** `notify` and `notify-debouncer-full` are already in `Cargo.toml` (deferred) — current 12 s poll is intentional (no fd leaks on long-running sessions, no macOS FSEvent permission edge cases). If you swap, do it as a one-channel-deep change behind the same handler.
- **Errors cross the IPC boundary as `Result<T, String>`** (`format!("{e:#}")`). Tauri serializes that cleanly; structured error types do not.
- **Re-indexing must remain idempotent** because the point id is `uuid_v5(session_id)`. New extracts/payload fields are fine; changing the id derivation orphans the entire existing collection.
- **`SandboxRoot` is multi-root.** P1 KF-01 takes `Vec<PathBuf>` for `~/.claude/projects` and `~/.codex/sessions`; canonicalize each and reject any input path that falls outside ALL roots.

### Frontend wiring

`src/index.html` + `src/main.js` (1524 LOC) + `src/styles.css` is plain vanilla — no bundler, no TS, no framework. `tauri.conf.json` sets `withGlobalTauri: true` so the bridge is at `window.__TAURI__.core.invoke`. The Topology surface uses the vendored 3d-force-graph in `src/vendor/`. `frontendDist` is `../src` (Tauri serves the directory directly).

## Repo layout (only the non-obvious bits)

- `src-tauri/src/main.rs` — CLI vs GUI dispatcher (read `CLI_SUBCOMMANDS` + the HOME-CWD comment before touching launch behavior)
- `src-tauri/src/lib.rs` — `tauri::Builder`, tray menu, the `generate_handler![…]` list of every exposed command
- `src-tauri/src/indexer.rs` — biggest file by far (1581 LOC); every Qdrant primitive lives here
- `src-tauri/src/parser.rs` — Claude Code JSONL parser
- `src-tauri/src/codex_parser.rs` — Codex CLI JSONL parser (KH-01, to be added)
- `src-tauri/tests/fixtures/{claude,codex}/*.jsonl` — sanitized minimal-shape fixtures; reuse these rather than committing real session data
- `docs/architecture.md` — authoritative per-feature backend trace; update it when changing the indexing schema
- `docs/qdrant-features.md` — per-primitive engineer's tour
- `index.html` (repo root, not `src/index.html`) — single-file public landing page for github.io; unrelated to the app

## Deferred items (do not "fix" without a separate decision)

These are listed deferred in the README and are intentional, not bugs:

- ColBERT v2 / BM42 sparse on `path` — blocked on `fastembed-rs` 5.x not exposing the model
- Real `notify` file watcher — see polling rationale above
- Native file picker for snapshots — currently `window.prompt()`, swap needs `tauri-plugin-dialog`
- Code signing / notarization — local-only MVP
- Indexing of `~/.codex/history.jsonl` (the flat user-input log, NOT per-session) and `~/.codex/logs_2.sqlite` (internal tracing) — out of scope; only `sessions/**/*.jsonl` is the canonical corpus.

## Hackathon constraint (project invariant)

The product brief is *Qdrant Vector Space Day 2026 — "Think Outside the Bot."* Surface designs that introduce a chat box, a "ask a question" affordance, or a runtime LLM call break the pitch. The multi-agent ingest (Claude + Codex unified) doubles down on this theme: instead of building one more bot, Memex gives the user spatial control over the bot history they already have.

When in doubt, prefer adding a spatial/recommendation/replay affordance over a generative one.
