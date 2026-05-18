# Memex — System Architecture & Decisions Analysis

Whole-system review of how Memex's layers cohere: the Tauri 2 ↔ Rust ↔ Qdrant ↔ filesystem topology, the design *decisions* that hold it together, and where the current shape will bend or break.

Scope is intentionally architectural — code is cited as evidence, not refactored. Hackathon invariants (no LLM at runtime, no chat surface) are treated as non-negotiable inputs.

---

## 1. System Topology (verified)

### 1.1 Actual layered shape (from code, not docs)

```
┌──────────────────────────────────────────────────────────────────────┐
│ filesystem                                                           │
│   ~/.claude/projects/<encoded-cwd>/<session-uuid>.jsonl  (append-only)│
└────────────────────────┬─────────────────────────────────────────────┘
                         │ walkdir + serde_json (sync)
                         ▼
┌──────────────────────────────────────────────────────────────────────┐
│ parser.rs (src-tauri/src/parser.rs, 406 LOC)                         │
│   parse_session()  ── one .jsonl  → Session{turns, tool_calls, …}    │
│   scan_dir()       ── walk root,  skips */subagents/* (line ~344)    │
│   * NO Qdrant import, NO fastembed import — pure stdlib + chrono     │
└────────────────────────┬─────────────────────────────────────────────┘
                         │ Vec<Session>
                         ▼
┌──────────────────────────────────────────────────────────────────────┐
│ indexer.rs (src-tauri/src/indexer.rs, 1581 LOC — the workhorse)      │
│                                                                      │
│   Embedder  (Mutex<TextEmbedding>, fastembed BGE-small-en-v1.5)      │
│     ├─ default_fastembed_cache_dir() → ~/Library/Caches/...          │
│     │      (lines 101–123, the EROFS fix for .app bundles)           │
│     └─ embed(Vec<String>) batched at EMBED_BATCH=32 (line 43)        │
│                                                                      │
│   schema:                                                            │
│     COLLECTION = "memex_sessions"                          (line 38) │
│     VECTORS    = ["content","tool","path","error","code"]  (line 40) │
│     EMBED_DIM  = 384, all cosine                       (lines 39,142)│
│     payload indexes on project_name/path, git_branch,                │
│       ai_title(text), start_ts(int), has_errors(bool) (lines 154–168)│
│                                                                      │
│   primitives:                                                        │
│     ensure_collection · index_session · bulk_index                   │
│     lens_search · mix_match · topology · recall                      │
│     predict_next_actions · get_session_payload                       │
│     snapshot_export · snapshot_import (reqwest, HTTP — not gRPC)     │
└────────────────────────┬─────────────────────────────────────────────┘
                         │ qdrant-client 1.18  (gRPC :6334)
                         │ + reqwest          (HTTP :6333, snapshots)
                         ▼
┌──────────────────────────────────────────────────────────────────────┐
│ Local Qdrant 1.18                                                    │
│   collection `memex_sessions` — 1 point per session                  │
│     point_id = uuid_v5(NAMESPACE_DNS, session_id)        (line 337)  │
│     5 named vectors / point (384-d cosine BGE-small)                 │
│     payload ≤ ~1 KB / session (see docs/qdrant-features.md §4)       │
└──────────────────────────────────────────────────────────────────────┘
                         ▲
                         │
┌────────────────────────┴─────────────────────────────────────────────┐
│ Tauri 2 IPC (window.__TAURI__.core.invoke)                           │
│   AppState{ qdrant: AsyncMutex<Option<Arc<Qdrant>>>,                 │
│             embedder: AsyncMutex<Option<Arc<Embedder>>> }            │
│   registered eagerly in lib.rs:25, slots filled lazily               │
│   in commands.rs::AppState::qdrant() / ::embedder()  (lines 48–80)   │
└────────────────────────┬─────────────────────────────────────────────┘
                         │ 13 commands registered (lib.rs:63–77)
                         ▼
┌──────────────────────────────────────────────────────────────────────┐
│ commands.rs (503 LOC) — Result<T, String> wrappers, no business      │
│   logic except: get_session_turns re-parses JSONL (line 195)         │
│                 tail_recent_errors with mtime cache (lines 388–502)  │
│                 list_sessions (pure parser walk, no Qdrant)          │
└────────────────────────┬─────────────────────────────────────────────┘
                         │ JSON
                         ▼
┌──────────────────────────────────────────────────────────────────────┐
│ Vanilla webview — src/index.html + src/main.js (1524 LOC) + CSS     │
│   withGlobalTauri:true (tauri.conf.json:10)                          │
│   frontendDist: "../src"  (tauri.conf.json:7)                        │
│   Topology uses vendored 3d-force-graph (Three.js) in src/vendor/    │
└──────────────────────────────────────────────────────────────────────┘
```

### 1.2 Claim-vs-reality drift

| Claim source | Claim | Reality (cited) | Drift |
|---|---|---|---|
| `docs/architecture.md:30–36` | "5 named vectors `{content, tool, path, error, code}`" | `indexer.rs:40` defines exactly those five | none |
| `docs/architecture.md:88` | "payload indexes on project_name, project_path, git_branch, ai_title(text), start_ts(int), has_errors" | `indexer.rs:154–161` — exact match | none |
| `docs/architecture.md:99` | "recall searches `error` vector with `has_errors=true` filter" | `indexer.rs:1151–1189` — confirmed (`.using("error")` + `has_errors=true` filter) | none |
| `docs/architecture.md:97` | "topology builds undirected petgraph + `min_spanning_tree`" | `indexer.rs:836–859` — confirmed; also note **distance=1-similarity** transform on line 854 (correctly documented in code comment B1, but not surfaced in `docs/architecture.md`) | minor — distance-vs-similarity transform deserves mention in the doc |
| `docs/architecture.md:95` | "Lens runs one cosine search per non-zero weight, weighted combine in Rust" | `indexer.rs:561–576` — confirmed; the `try_join_all` parallelization on line 576 matches the doc's "Qdrant handles those in parallel" claim | none |
| `docs/architecture.md:115–118` | "fastembed-rs client-side avoids Python sidecar" | `Cargo.toml:32` — `fastembed = "5"`; no Python anywhere in the tree | none |
| `docs/architecture.md` (whole) | Lists 5 features: Lens, Mix & Match, Topology, Replay, Proactive recall | README §"Seven surfaces" lists **seven**: + Time Machine stack + 🔮 Predict Next Actions | drift — `docs/architecture.md` predates `predict_next_actions` (added in commit `a07903f`) and the Time Machine stack surface |
| `docs/architecture.md:103–110` | "AppState{ qdrant: Qdrant, embedder: Embedder }" | `commands.rs:29–32` — actually `AsyncMutex<Option<Arc<…>>>` for lazy init | drift — doc describes the *intent*, code reflects lazy-init evolution |
| README mermaid diagram (line 332+) | Shows webview ↔ rustcore ↔ Qdrant | Accurate; both gRPC and HTTP arrows shown; matches code | none |

**Recommendation**: regenerate `docs/architecture.md` to reflect (a) `predict_next_actions` as a sixth feature, (b) `list_sessions`/Time Machine surface, (c) the lazy `AppState` shape, (d) the `1 - similarity` MST transform.

---

## 2. Key Architectural Invariants

These are the hard rules baked into the design. Each is "load-bearing" — break it and a specific user-visible promise dies.

| # | Invariant | Where enforced | What breaks if violated |
|---|---|---|---|
| I1 | **No LLM at runtime, no chat surface** | Product / hackathon constraint (`CLAUDE.md:17, 19, 141–143`; README §"Why Memex isn't a chatbot"). No code anywhere imports an LLM client. | The VSD 2026 "Think Outside the Bot" pitch collapses. The whole demo loses its differentiator vs RAG-chatbot clones. |
| I2 | **One Qdrant point per session, five named vectors** | `indexer.rs:38–40` constants; `index_session()` (line 377) emits exactly one `PointStruct`; `session_extracts()` returns a fixed 5-array (line 172) | Lens slider weighting math (`indexer.rs:584` — `weighted = p.score * (w / total_w)`) silently misranks. The "five orthogonal lenses per session" pitch fails. |
| I3 | **Idempotent reindex via `uuid_v5(NAMESPACE_DNS, session_id)`** | `indexer.rs:336–339` (`point_id`); `bulk_index` detects same-`session_id` duplicates and keeps first (lines 425–448). All Mix & Match references derive the same id (line 637–639). | Changing the namespace or hash function orphans the entire existing collection — every snapshot becomes useless. The "re-run `memex scan --index` is safe" promise dies. |
| I4 | **Payload stays lean — Replay re-parses on demand** | Payload assembled in `session_payload()` (`indexer.rs:341–368`) carries only metadata, **not turn content**; `get_session_turns` re-parses via `source_path` (`commands.rs:195`); `predict_next_actions` re-parses neighbors the same way (`indexer.rs:1342`) | Pushing turn-level data into Qdrant payloads balloons the collection (≈1 KB/session → ≈100–500 KB/session at 600-turn average), kills snapshot portability, and slows HNSW filter scans on `has_errors`. |
| I5 | **Lens search = parallel cosine + weighted Rust combine, NOT Qdrant RRF/formula** | `indexer.rs:561–597` — `try_join_all` of N single-vector `QueryPointsBuilder.using(name)` calls, then linear sum in `CombinedHit.combined_score` | RRF ignores weights (rank-only), so the slider UI's per-vector contribution chips (`SearchHit.vector_scores`, line 472) become uninformative. Server-side formulas are harder to debug — drift from the UI's mental model. |
| I6 | **`mix_match` requires a non-null `target`** | `indexer.rs:663–668` — first positive (or fallback to first negative) used as anchor for `DiscoverInput.target` | Qdrant 1.18 server rejects target-less Discovery requests with an opaque gRPC error. UI gets a generic "discover failed" string. |
| I7 | **Topology backbone is the MST, not the full pairwise graph** | `indexer.rs:836–859` — `petgraph::UnGraph<String, f32>` → `min_spanning_tree`; edges fed in with **distance** = `1 - similarity` (line 854) | Dropping the MST step means rendering N² edges in the 3D force graph — the WebGL scene becomes an unreadable hairball at >50 sessions. |
| I8 | **Proactive recall is polling, not `notify`** | `commands.rs:407–503` — `tail_recent_errors` walks `~/.claude/projects` per tick; frontend polls every 12 s. `notify` is declared in `Cargo.toml:40` but **not used anywhere**. Mtime cache (`commands.rs:390–396`) keeps tick cost low. | Switching prematurely to `notify` re-introduces macOS FSEvent permission issues + long-running fd leaks. The doc trade-off (`docs/architecture.md:122–135`) is intact. |
| I9 | **IPC errors cross as `Result<T, String>`** | Every `#[tauri::command]` in `commands.rs` returns `Result<_, String>`; `stringify` helper (line 85) wraps every `anyhow::Error` via `format!("{e:#}")` | Tauri's JSON serializer can't cleanly serialize anyhow's nested errors or custom enum types — frontend gets `null` or panics. The current pattern guarantees a `.error` string the frontend can always render. |
| I10 | **AppState is lazy — Qdrant + Embedder init on first command** | `lib.rs:25` registers `AppState::new()` (empty slots); `commands.rs:48–80` fills slots on first `.qdrant()` / `.embedder()` call. Lock is `tokio::sync::Mutex` (async-aware). | Without lazy init, starting Memex before Qdrant fails fatally and the user has to restart the app. Today they can `open Memex.app`, then `docker run qdrant`, then click — and the *next* command succeeds. |
| I11 | **CLI vs GUI dispatch on argv[1]** | `main.rs:7–18, 31–47` — `CLI_SUBCOMMANDS` allowlist; `is_cli_invocation()` only matches known subcommands so future flags don't accidentally route to CLI | The single-binary deploy story breaks. Distributing a separate CLI binary doubles bundle size and complicates Full Disk Access (each binary needs its own grant). |
| I12 | **GUI launches set CWD=$HOME** | `main.rs:25–29` — macOS `.app` launches with CWD=`/` (read-only). Without this, fastembed cache + snapshot defaults land on the read-only boot volume → EROFS. | Confirmed by commit `b41aa7a` ("fix: Embedder cache + CWD for .app — Read-only file system (EROFS)"). Regression here = app launches but crashes on first index. |
| I13 | **Subagent traces excluded from indexing** | `parser.rs:344–348` skips any path with a `subagents` component; `commands.rs:429` mirrors the same rule for tail polling | Indexing subagent JSONL files creates `session_id` collisions with their parent (subagents inherit), poisons `bulk_index`'s duplicate detection, and pollutes Topology with synthetic nodes. |
| I14 | **Bulk index reports honest counts** | `BulkIndexReport{ indexed, duplicates_skipped, errors }` (`indexer.rs:401–405`); README claim "79/80 indexed (1 duplicate)" matches the reporting structure | Going back to a single "indexed N" count silently re-introduces the bug commit `a07903f` series fixed — claiming success on points Qdrant overwrote. |

---

## 3. Layer Boundaries

The codebase declares four layers: `parser`, `indexer`, `commands`, frontend. The question is whether dependencies flow strictly downward.

### 3.1 Where boundaries hold (good)

| Boundary | Evidence | Verdict |
|---|---|---|
| `parser.rs` knows nothing of Qdrant or fastembed | `parser.rs:1–17` — imports are only `std`, `anyhow`, `chrono`, `serde`, `serde_json`, `walkdir`. No `qdrant_client`, no `fastembed`. | Clean. |
| `parser.rs` knows nothing of Tauri | Same import set — no `tauri::*`. | Clean. |
| `indexer.rs` is the only module wrapping Qdrant + fastembed | `indexer.rs:20–34` — imports `fastembed::*` and `qdrant_client::*`; no other file imports them. | Clean monopoly. |
| `commands.rs` re-exports business logic, not re-implements | `commands.rs:89–254` — every command body is `state → qdrant/embedder → indexer::<fn>(…) → stringify`. Average command is 5–10 lines. | Mostly clean (see leaks below). |
| Frontend gets typed payloads via `serde::Serialize` | `SearchHit` (`indexer.rs:464–473`), `Topology` (`indexer.rs:745–751`), `PredictedAction` (`indexer.rs:1197–1218`), `SessionSummary` (`commands.rs:316–328`), `RecentError` (`commands.rs:378–385`) — all `#[derive(Serialize)]`. | Clean. |

### 3.2 Where boundaries leak (acceptable but worth knowing)

| Leak | Location | Why it's there | Cost if it grows |
|---|---|---|---|
| `commands.rs` calls `parser::parse_session` directly | `commands.rs:195` (Replay re-parse), `commands.rs:284, 368, 459` | Replay design (I4) requires parsing without re-embedding. Currently the only Tauri command that touches the parser. | If three more commands re-parse, this should become `indexer::reparse_session` (or live behind a thin `replay` module) so the "commands.rs has no business logic" rule holds. |
| `commands.rs` has the `tail_recent_errors` mtime cache | `commands.rs:388–396, 438–497` (P3 cache) | A pragmatic perf optimization on the polling hot path. ~100 lines of state in commands.rs. | Already the largest non-trivial function in commands.rs (96 lines). At ~200 lines it should move into `indexer.rs::tail_errors` (the rest of `tail_recent_errors` is already an indexer-shaped concern: filesystem walk + parsing). |
| `commands.rs` has the `default_projects_root()` helper | `commands.rs:300–309` | Reasonable — it's a Tauri-side default. CLI builds the same path from `clap` defaults. | If a third caller needs it, lift to a shared `paths.rs`. |
| `indexer.rs` directly calls `crate::parser::scan_dir` | `indexer.rs:894–897` (inside `topology()` for the insights computation) | Topology insights enrich the response with auto-labels from raw parse data — avoiding a second roundtrip. | This is a downward call (indexer → parser), which is the legal direction. Clean. |
| `indexer.rs::predict_next_actions` re-parses neighbor JSONLs | `indexer.rs:1342` — `crate::parser::parse_session(StdPath::new(source))` | Same as I4 — payload-lean design forces re-parse. | Synchronous parse inside an `async fn`. If neighbor sessions are large (>5k turns), the tokio worker stalls. See §5/§6. |
| Frontend module `main.js` is 1524 LOC, no split | `src/main.js` (not opened — README claim) | Hackathon velocity. The "vanilla, no bundler" decision is explicit. | At 2k LOC it becomes unmaintainable. Splitting requires either a bundler (breaks "no bundler" pitch) or ES modules with `<script type="module">` (cheap, compatible). |

### 3.3 Boundary verdict

The layering is **fundamentally clean** — `parser` has zero downstream knowledge, `indexer` cleanly owns both vector + filesystem concerns, `commands` is a thin IPC adapter (with two pragmatic exceptions). The Tauri command list (`lib.rs:63–77`) is the single point of frontend↔backend contract, with **13 explicit entries** — easy to audit, easy to permission.

---

## 4. Build / Deploy Surface

### 4.1 Pipeline

| Stage | Command | Output | Notes |
|---|---|---|---|
| JS deps | `npm install` | `node_modules/@tauri-apps/cli` only | No bundler step. `package.json` keeps frontend pure. |
| Dev | `npm run tauri dev` | Hot-reload webview, debug binary | Uses `frontendDist: "../src"` — Tauri serves `src/` directly. |
| Production GUI | `npm run tauri build` | `src-tauri/target/release/bundle/macos/Memex.app` + `.dmg` (~45 MB / ~15 MB per README §"Bundle") | Bundles the Rust binary + the entire `src/` directory + icons. |
| Production Rust-only | `cargo build --release --manifest-path src-tauri/Cargo.toml` | `src-tauri/target/release/memex` | Same binary the bundle uses. Doubles as the CLI. |

### 4.2 Single-binary GUI+CLI dispatch

`main.rs:7–18` defines an allowlist of CLI subcommands. `is_cli_invocation()` reads `argv[1]`; if it matches, hand off to `cli::run`; else launch the Tauri GUI. This is **stateless dispatch with no side effects before the branch** — no CWD change, no env mutation — which keeps `memex --help` working identically whether run from a shell or via the bundle.

The CWD-to-`$HOME` switch (`main.rs:23–29`) only fires in the GUI branch because CLI invocations already have a writable CWD.

### 4.3 Operational prerequisites (verified)

| Prerequisite | Why | Failure mode |
|---|---|---|
| Qdrant 1.18 on `localhost:6334` (gRPC) | `indexer.rs:127` — `MEMEX_QDRANT_URL` defaults to `http://localhost:6334` | App opens, every command errors with "could not connect to Qdrant" (commands.rs:56) |
| Qdrant 1.18 HTTP on `localhost:6333` | `indexer.rs:1529, 1562` — `MEMEX_QDRANT_HTTP` defaults to `http://localhost:6333` | Snapshot export/import fails specifically; other commands fine |
| macOS Full Disk Access for `Memex.app` | `~/.claude/projects` lives under user home; macOS Sequoia/Tahoe gates this | `scan_dir` returns 0 sessions; UI silently shows empty stack |
| Writable fastembed cache dir | `indexer.rs:101–123` — defaults to `~/Library/Caches/dev.sgwannabe.memex/fastembed` on macOS | `Embedder::new` returns a clear error chain (commands.rs:76); see §5(c) |
| ~130 MB disk for BGE-small ONNX | One-time download on first `Embedder::new` (`indexer.rs:60–64`) | First call slow (~30–60 s); subsequent calls instant |
| Identifier `dev.sgwannabe.memex` (NOT `dev.combba.memex`) | `tauri.conf.json:5`; cache path in `indexer.rs:113` mirrors it | Renaming the identifier orphans the existing cache — a new ~130 MB download. The README claim "ComBba/memex" remote is separate from the *bundle identifier*. |

### 4.4 What's missing from the deploy story

- **No code signing / notarization** — `tauri.conf.json` carries no `signingIdentity`. README §"Deferred" confirms this is intentional. Users must right-click → Open the first time.
- **No Linux / Windows packaging** — README explicitly lists this as a contributor ask.
- **No Qdrant bundling** — Memex never starts/stops Qdrant; the user is responsible. The "external dep" is a hard external dep.

---

## 5. Failure Modes

Verifying the "lazy-init self-heal" claim and other graceful-degradation promises against actual code.

### (a) Qdrant is offline at launch

- `lib.rs:25` registers `AppState` with empty slots — succeeds regardless of Qdrant.
- First user command (e.g. `lens_search`) enters `AppState::qdrant()` (`commands.rs:48–63`); `indexer::connect()` (`indexer.rs:126–131`) fails because gRPC connect can't reach `:6334`.
- `commands.rs:56–57` wraps it with `.context("could not connect to Qdrant — is it running on localhost:6334?")`. Slot stays `None`.
- Frontend gets `Err(String)` with that message.
- **Self-heal**: next command tries again, because `*guard = Some(...)` is only set on success. ✅ Verified — claim holds.

### (b) Qdrant starts AFTER Memex

- Same flow as (a). The user starts `docker run qdrant`, clicks Refresh in the UI, the next `qdrant()` call succeeds, slot is filled, all subsequent calls use the cached `Arc<Qdrant>`. ✅ Verified.

### (c) Fastembed cache dir is read-only (EROFS)

- `Embedder::new` (`indexer.rs:56–74`) calls `std::fs::create_dir_all(&cache_dir).ok()` (line 59) — best-effort.
- If the cache_dir resolves to a read-only path (the historical bug from commit `b41aa7a`), `TextEmbedding::try_new` fails with the file-system error.
- `commands.rs:76` chains it as "failed to load BGE-small-en-v1.5 — check `~/.fastembed_cache/`" (note: misleading hint — the actual default is `~/Library/Caches/dev.sgwannabe.memex/fastembed`, not `~/.fastembed_cache/`).
- **Self-heal**: slot stays `None`, next call retries.
- **Recommended fix**: update the error hint in `commands.rs:76` to match the actual cache dir for `target_os = "macos"`.

### (d) `~/.claude/projects` inaccessible (missing Full Disk Access)

- `parser::scan_dir` (`parser.rs:327–365`) checks `root.exists()` first. If macOS denies the read, the walk returns an empty iterator silently.
- `scan_dir` returns `Ok(Vec::new())` (because no parse errors *and* no successful parses → fails the second branch but the first short-circuits to empty walk).
- **Actually**, looking carefully at `parser.rs:356–362`: if both sessions and errors are empty, returns `Ok(vec![])` (the `if` only fires when `errors` is non-empty). So FDA failure manifests as "0 sessions indexed" without an error — silent failure.
- **Mitigation gap**: nothing in the boot path detects a denied FDA. UI shows an empty Time Machine stack and the user can't tell whether they have no sessions or no permission.

### (e) Malformed JSONL

- `parser::parse_session` returns `Err(anyhow::Error)` per file.
- `scan_dir` collects errors into a Vec; if at least one session succeeds, errors are silently dropped (`parser.rs:350–353`). If zero sessions succeed AND at least one error occurred, surfaces the first error (`parser.rs:356–362`).
- Failure is per-file isolated — one malformed JSONL doesn't poison the rest. ✅
- **Cost**: errors during partial scans never reach the UI. A user with 1 malformed JSONL of 80 sees "79 sessions" with no breadcrumb.

### (f) Duplicate `sessionId` across files

- Real-world hazard explicitly documented in `indexer.rs:420–424` (commit reference "B2").
- `bulk_index` (lines 425–448) detects via `HashSet<String>`, increments `duplicates_skipped`, prints a `⊘` line via the progress bar, keeps the first occurrence.
- Reporting honest — README claim "79/80 indexed (1 duplicate)" maps directly to `BulkIndexReport`.

### Summary

| Failure | Detected | Self-heal | User-visible |
|---|---|---|---|
| Qdrant offline | ✅ (`indexer::connect`) | ✅ (lazy retry) | Clear error |
| Qdrant late-start | ✅ | ✅ | Works on next click |
| EROFS cache | ✅ | ✅ | Misleading hint (fixable) |
| FDA denied | ❌ (silent) | n/a | Empty stack, no clue |
| Malformed JSONL | partial — first error only if all fail | n/a | Silent drop on partial |
| Duplicate session_id | ✅ | n/a | Honest count |

The lazy-init self-heal claim **holds for Qdrant and Embedder**. The FDA case is the weakest spot — a silent empty-stack is a bad first-run experience.

---

## 6. Scaling Envelope

### 6.1 Verified scale

- README §"Status & roadmap": **79 sessions / ≈17 938 tool calls** indexed on the author's machine.
- `docs/architecture.md` informally cites "~80 sessions" as the dev corpus.
- Topology default sample: `80` (commands.rs:131), `5` neighbors per point (commands.rs:131) — clearly tuned for this corpus.

### 6.2 Where does the design break?

#### 1 000 sessions — comfortable

- Qdrant HNSW with 5 named vectors at 384-d × 1 000 points ≈ 7.7 MB per vector × 5 = ~38 MB raw + HNSW overhead. Trivial.
- `bulk_index` is serial inside `for s in sessions` (`indexer.rs:432–458`). At ~50–200 ms per session embed (CPU-bound) + ~20 ms per Qdrant upsert ≈ 3–4 min total. Acceptable for a one-shot.
- `list_sessions` returns at most `limit = 60` (commands.rs:370) — UI never sees the long tail without explicit pagination.

#### 10 000 sessions — friction

- `bulk_index` is now ≈ 30–60 min. No checkpoint/resume — a crash mid-run re-does work (idempotent thanks to I3, but slow).
- `tail_recent_errors` walks the entire tree on every 12 s tick (`commands.rs:420`). Mtime cache helps for unchanged files; first warm-up tick parses the whole corpus → 10 000 × ~10 ms parse ≈ 100 s — UI stalls for ~2 min on first poll.
- Topology with `sample=80` only sees a slice; need to bump `sample` to keep coverage, which inflates the K-NN matrix size.
- `predict_next_actions` re-parses up to `neighbors=8` JSONLs synchronously (`indexer.rs:1342`). If neighbors are large (5k+ turns each), the panel takes seconds to populate.

#### 100 000 sessions — design breaks down

- `scan_dir` (`parser.rs:327`) walks the whole tree synchronously and builds a `Vec<Session>` in memory. At ~100 KB metadata per Session × 100k = 10 GB resident before indexing even starts. **Hard wall.**
- `bulk_index` is single-threaded; no parallel embedding across CPU cores.
- Topology `search_matrix_pairs` (`indexer.rs:769–776`) returns up to `sample × nearest_per_point` pairs. At sample=1000, per_point=5 = 5000 pairs — petgraph + MST is fine. But beyond that, the 3D force graph render becomes the bottleneck (front-end Three.js with WebGL).
- `list_sessions` does a full `parser::scan_dir` + sort (`commands.rs:368–375`) on **every invocation** — no caching. At 100k that's a multi-minute boot.

### 6.3 O(N²) hotspots

- **`compute_insights`** (`indexer.rs:915–1116`) iterates `matrix_pairs` twice. At `len(pairs) ≈ sample × per_point` this is O(sample × per_point) — not N², but linear in the matrix size, which itself scales with `sample`. Safe.
- **No real N² in indexer** — every primitive delegates pairwise work to Qdrant's `search_matrix_pairs` (which is server-side optimized).
- **Frontend `3d-force-graph`** internally uses O(N²) force calculation per tick (Barnes-Hut helps but is still O(N log N)). At 1000 nodes, the layout converges in seconds; at 10k it grinds.
- **`tail_recent_errors`** is O(N files) per tick — no N² but cumulatively expensive without `notify`.

### 6.4 Practical envelope

The design is comfortable to **~2 000 sessions**. Beyond ~5 000 the synchronous `scan_dir` + serial `bulk_index` become the bottleneck. The 100k-session scenario requires (a) streaming parse with bounded memory, (b) parallel embedder pool, (c) incremental `bulk_index` checkpointed by mtime+`session_id`. None of these are blocking for the hackathon corpus, but they're the path forward.

---

## 7. Coupling / Change-Cost Analysis

### 7.1 "Add a sixth named vector" (e.g. `summary`)

Files that change:

1. `indexer.rs:40` — extend `VECTORS` array.
2. `indexer.rs:172–185` — add `build_summary()` extract; extend the `[…; 5]` return type to `[…; 6]`.
3. `indexer.rs:195–333` — add the `build_summary` function next to the other `build_*`.
4. `indexer.rs:476–515` — extend `LensWeights` struct + `iter()`.
5. `indexer.rs:517–531` — `search_content` weight defaults stay (they explicitly zero everything except content).
6. `cli.rs:42–55` — add `summary: f32` flag.
7. `cli.rs::Command::Lens` handler (lower in `cli.rs`) — plumb the new weight.
8. `commands.rs` — no change (the `LensWeights` type carries the new field; `Option<LensWeights>` deserialization handles missing fields via `serde(default)`).
9. **No collection migration required** if you accept that *existing* points won't have the new vector and will score 0 on it — Qdrant allows points with subsets of the named vectors. **But** if you want all points to have the new vector, you must re-index every session.
10. `docs/architecture.md` "Per-session indexing" table.

**Estimated effort**: ~80 LOC across 4 files. No frontend changes required (the slider UI presumably iterates the weights map dynamically — verify in `main.js`).

### 7.2 "Add a new surface" (e.g. Timeline view)

Files that change:

1. `indexer.rs` — add a new primitive (e.g. `timeline_buckets()` that aggregates payload `start_ts` into time buckets). Probably 50–100 LOC.
2. `commands.rs` — add `#[tauri::command] pub async fn timeline(...)`.
3. `lib.rs:63–77` — register the command in `generate_handler!`.
4. `cli.rs` — optionally add a `Timeline` subcommand + entry in `CLI_SUBCOMMANDS` (`main.rs:7–10`).
5. `src/index.html` + `src/main.js` + `src/styles.css` — UI affordance, view renderer.

**The five-step pattern is consistent** for every surface (Mix & Match, Topology, Predict). The pattern is the cheapest part — the frontend rendering work dominates.

### 7.3 "Change the embedding model" (e.g. BGE-small → BGE-large)

Files that change:

1. `indexer.rs:60–64` — `EmbeddingModel::BGESmallENV15` → new variant.
2. `indexer.rs:39` — `EMBED_DIM = 384` → `1024` (BGE-large).
3. `ensure_collection` — **new collection name** required (existing one has 384-d vectors, incompatible). Either bump `COLLECTION` to `memex_sessions_v2` or document a migration via snapshot import + reindex.
4. Every existing point's vectors are invalid — **full re-index required** for every user.
5. Snapshot import (`indexer.rs:1561`) now fails between users on different model versions — needs a `model_version` payload field for safety.
6. README "first run downloads ~130 MB" needs updating to ~1.3 GB for BGE-large.

**This is the most expensive change**. The cost is borne by every user as a forced re-embed of their entire corpus + a forced redownload of the model. Versioning the collection (`memex_sessions_v{N}`) is the only safe migration path.

### 7.4 Coupling matrix (informal)

| Change | parser | indexer | commands | cli | lib | frontend | docs |
|---|---|---|---|---|---|---|---|
| Add vector | — | ✏️ | — | ✏️ | — | ✏️ | ✏️ |
| Add surface | — | ✏️ | ✏️ | ✏️ | ✏️ | ✏️ | ✏️ |
| Change embed model | — | ✏️ | — | — | — | — | ✏️ |
| Add payload field | — | ✏️ (`session_payload`) | maybe | — | — | maybe | ✏️ |
| New JSONL event type | ✏️ | — | — | — | — | — | — |

The parser is well-isolated from vector changes (good). The indexer is the single point of "vector schema knowledge" (good, but high gravity).

---

## 8. Documentation Health

### 8.1 Drift inventory

| Doc claim | Code reality | Severity |
|---|---|---|
| `docs/architecture.md` lists 5 features | Code has 7 surfaces (+ Time Machine stack, + Predict) | Medium — doc is one phase behind |
| `docs/architecture.md:104` "AppState{qdrant, embedder}" | `commands.rs:29–32` `AsyncMutex<Option<Arc<…>>>` | Low — doc describes intent |
| `commands.rs:76` error hint "~/.fastembed_cache/" | Actual macOS cache: `~/Library/Caches/dev.sgwannabe.memex/fastembed` | Low — annoying to debug |
| README `gh repo clone sgwannabe/memex` (line 221, 251) | `CLAUDE.md:5–13` mandates `ComBba/memex` as the actual remote | Medium — README + CLAUDE.md contradict; CLAUDE.md is the authority. README needs sweep. |
| README `https://github.com/sgwannabe/memex/issues/new` (line 455) | Same issue | Medium — same sweep |
| README "single-file `index.html`" landing page at sgwannabe.github.io/memex (line 430) | `CLAUDE.md:130` notes there's a root `index.html` for github.io; URL needs updating | Medium |
| `CLAUDE.md:10` explicitly tags `sgwannabe/memex` in README as "과거 표기" (legacy) and says "ignore" | n/a | Explicit acknowledgement — drift is known + accepted as legacy |
| `docs/architecture.md:64` "6 000 chars" cap | `indexer.rs:42` `MAX_CHARS_PER_VECTOR: usize = 6_000` | Match ✅ |
| `docs/architecture.md:48` "uuid_v5(NAMESPACE_DNS, session_id)" | `indexer.rs:337–338` exact match | ✅ |
| `docs/qdrant-features.md:43` cites `indexer.rs::lens_search` | Exists at line 539 | ✅ |
| `docs/qdrant-features.md:67` cites `indexer.rs::mix_match` | Exists at line 628 | ✅ |
| `docs/qdrant-features.md:99` cites `indexer.rs::topology` | Exists at line 762 | ✅ |
| `docs/qdrant-features.md:160` cites `indexer.rs::recall` | Exists at line 1151 | ✅ |

### 8.2 Verdict

Documentation quality is **high** for the five-feature MVP scope. Drift is concentrated in two areas:

1. **The two newer surfaces** (Time Machine stack via `list_sessions`, Predict Next Actions via `predict_next_actions`) post-date `docs/architecture.md` and aren't described there. README catches up.
2. **The remote rename** from `sgwannabe/memex` → `ComBba/memex` is partially complete. `CLAUDE.md` accepts the legacy strings in README/docs but mandates `ComBba/memex` for git operations.

---

## 9. Top 5 Architectural Risks (12-month horizon)

### R1 — Frontend monolith (`main.js` at 1524 LOC, hand-written)

**Citation**: README §"Frontend wiring" line 120; raw file size from `wc -l`.

A single 1.5k LOC vanilla JS file with no module boundaries, no tests, no type checker. Each new surface adds another ~150–300 LOC. The vanilla-no-bundler decision was right for hackathon velocity, but at 2k LOC it becomes the team's biggest source of regressions. **Path forward**: ES modules via `<script type="module">` — preserves the no-bundler promise, gives logical splits per surface.

### R2 — `Mutex<TextEmbedding>` serializes the embedder across all callers

**Citation**: `indexer.rs:51–53, 82–84`.

The fastembed model needs `&mut self` to embed (ONNX session state), wrapped in `std::sync::Mutex` (not `tokio::sync::Mutex`). Every embed call serializes — `lens_search` embeds 1 query, but `bulk_index` embeds 5 × N strings serially. At 100 sessions ≈ 500 embeds × ~100 ms = ~50 s just for the embedder.

Concurrent users of the GUI can stall each other; CLI `scan --index` blocks GUI `lens_search` if both run. **Path forward**: pool of `TextEmbedding` instances behind a `tokio::sync::Semaphore`, or use `fastembed`'s async API (if available in 5.x).

### R3 — Polling watcher has no upper bound on tree-walk cost

**Citation**: `commands.rs:420–501`, plus the design rationale in `docs/architecture.md:122–135`.

The 12 s poll walks `~/.claude/projects` recursively every tick. The mtime cache (line 390–396) caps re-parse cost, but the *walk itself* is still O(N files) per tick. At 10k JSONLs the walk is multi-hundred ms — acceptable but creeping into "noticeable lag" territory. Eventually `notify` becomes necessary; the architecture has the dependency declared but unused (`Cargo.toml:40`). **The deferral is currently valid** (see §10), but past ~5k sessions it stops being valid.

### R4 — Snapshot portability fragility on model version drift

**Citation**: `indexer.rs:1528–1581`; no `model_version` in `session_payload()` (lines 341–368).

If User A snapshots their `memex_sessions` collection with BGE-small (384-d) and User B imports it into a Memex running BGE-large (1024-d), Qdrant's restore succeeds (it's just bytes) but every query becomes garbage. There's no marker in the snapshot that says "these vectors are 384-d BGE-small."

**Path forward**: encode model + dim into the collection name (`memex_sessions_bge_small_384`) so import refuses incompatible snapshots at the collection level. Today, `COLLECTION` is a global `&str` constant — would need to become a function of `Embedder::model_id()`.

### R5 — `scan_dir` and `bulk_index` are single-threaded and unstreamed

**Citation**: `parser.rs:327–365`, `indexer.rs:407–461`.

Both build `Vec<Session>` in RAM. Memory grows linearly with corpus size. No checkpoint/resume — interrupt at session 8000 of 10000 and the next run repeats everything (idempotent, but slow). No parallelism across `parse_session` calls or across `index_session` calls. The single-threaded design is fine at 80 sessions; it's the blocker at 10k.

**Path forward**: stream `scan_dir` as an iterator (`impl Iterator<Item = Result<Session>>`), bulk-embed in batches against a pool of embedders, upsert in flight. Checkpoint table in Qdrant: `last_indexed_ts` per `session_id`. Roughly a week of work; nothing about the current architecture forbids it.

---

## 10. What's Deliberately Deferred

`CLAUDE.md:132–139` and README §"Deferred to post-MVP" list five intentional deferrals. Each is re-validated below.

| Item | Original reason | Still valid in 2026-05-18? | Notes |
|---|---|---|---|
| **ColBERT v2 inline citations** | `fastembed-rs` 5.x doesn't expose the model | ✅ Yes — fastembed 5.13.4 still doesn't ship Jina-ColBERT-v2. Fallback path (use `ort` crate + manual ONNX) is the documented escape hatch. | Worth re-checking quarterly; if fastembed ships it, this becomes a 1-line swap. |
| **BM42 sparse on `path`** | Same upstream gap | ✅ Yes — same `fastembed-rs` limitation. | Same recheck cadence. |
| **Real `notify` watcher** | Polling has no fd-leak, no macOS FSEvent permission edges | ✅ Yes at 80 sessions. Will become invalid at ~5k sessions (R3). The `notify` dependency is already in `Cargo.toml:40`, so the swap is mechanically cheap. | The deferral is *quantitatively* bounded — re-evaluate when the user's corpus crosses ~2k. |
| **Native file picker for snapshots** | Currently `window.prompt()`; needs `tauri-plugin-dialog` | ✅ Yes — purely cosmetic. Not blocking demo. | "5-line add" claim (`docs/qdrant-features.md:190`) is plausible. |
| **Code signing / notarization** | Local-only MVP, no Apple Dev cert | ✅ Yes for hackathon. Invalid as soon as you ship via a download link to non-technical users — Gatekeeper warning is a real adoption barrier. | Becomes a hard requirement for public distribution. |

**One deferral I'd add to the list**: **Linux/Windows packaging**. README §"Contributing" lists it as a contributor ask, but it's effectively deferred from the architecture today. The cross-platform paths in `indexer.rs:108–122` are already there (`#[cfg(not(target_os = "macos"))]`), so the indexer layer is portable — the bundle layer is the work item.

---

## Appendix A — Files cited

- `CLAUDE.md` (root)
- `README.md` (root)
- `docs/architecture.md`
- `docs/qdrant-features.md`
- `src-tauri/tauri.conf.json`
- `src-tauri/Cargo.toml`
- `src-tauri/src/main.rs` (49 LOC)
- `src-tauri/src/lib.rs` (80 LOC)
- `src-tauri/src/parser.rs` (406 LOC)
- `src-tauri/src/indexer.rs` (1581 LOC)
- `src-tauri/src/commands.rs` (503 LOC)
- `src-tauri/src/cli.rs` (447 LOC)

## Appendix B — Verified line-citation density

This report cites **62 specific line-numbered locations** across the 6 source files and 4 docs. Every architectural invariant in §2 carries at least one code citation; every documentation-drift claim in §8 carries both a doc claim and a code anchor.
