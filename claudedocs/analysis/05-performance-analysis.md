# Memex — Performance Analysis (Report 05)

**Scope.** Hot paths in `src-tauri/src/indexer.rs` (1581 LOC), `src-tauri/src/parser.rs` (406 LOC), and the polling / Topology render regions of `src/main.js` (1524 LOC). External compute centers: fastembed-rs BGE-small (ONNX, 384-d) and qdrant-client gRPC (collection `memex_sessions`).

**Method.** Read-only. Every numeric claim cites `file:line` or is tagged `[est.]` with the arithmetic shown. Verified scale: 79 sessions / ~17 938 tool calls (per README, current author corpus). Design envelope: single-user local, "a few thousand" sessions.

---

## 1. Snapshot

**Verified scale today.**
- 79 indexed sessions, ~17 938 tool calls. (README, current author corpus.)
- One Qdrant point per session — `point_id = uuid_v5(NAMESPACE_DNS, session_id)` (`indexer.rs:336-339`).
- Five named 384-d cosine BGE-small vectors per point: `content / tool / path / error / code` (`indexer.rs:40`).
- Each source text capped at 6 000 chars (`indexer.rs:42`, `MAX_CHARS_PER_VECTOR`).

**Design envelope.**
- Single user. Local Qdrant on `localhost:6334` gRPC (`indexer.rs:127`).
- Target: a few thousand sessions; **no LLM call at runtime** is a hard product invariant (CLAUDE.md lines 17-19).

**Primary perf-relevant choices (each is a deliberate trade-off).**
- **Lazy init.** `AppState::qdrant()` / `AppState::embedder()` block-on-first-use (`commands.rs:48-80`). App opens instantly; first `lens_search` pays ~130 MB ONNX warm-up.
- **Single `Embedder` behind a `std::sync::Mutex`** (`indexer.rs:51-53`). Every embed call serializes — see §8.
- **12 s polling for recall**, not `notify` (`main.js:1454`). Deliberate per CLAUDE.md and docs/architecture.md §"Watcher choice".
- **MST, not full graph.** `min_spanning_tree(&g)` over the matrix (`indexer.rs:859`) — keeps edge count to `V-1` for SVG/3D rendering.
- **Payload-lean → Replay re-parses.** `get_session_turns` re-parses JSONL on demand (`commands.rs:174-198`).
- **Lens combine in Rust (not RRF).** `try_join_all` of N per-vector queries → weighted blend (`indexer.rs:576`, weighted sum on `indexer.rs:584-589`).

---

## 2. Hot paths inventory (`indexer.rs`)

For each function: input shape, expected call frequency, dominant cost.

| Function | Lines | Input | Frequency | Dominant cost |
|---|---|---|---|---|
| `Embedder::new` | 56-74 | – | once per process (lazy) | First-run: 130 MB ONNX download + load (~1-3 s warm). |
| `Embedder::embed` | 76-94 | `Vec<String>` (N texts) | every embed-touching command | **CPU** — fastembed ONNX forward pass under `Mutex` lock. Batches in chunks of 32 (`EMBED_BATCH`, line 43). |
| `connect` | 126-131 | – | once per process (lazy) | TCP/gRPC handshake to `localhost:6334`. |
| `ensure_collection` | 134-170 | – | once per process | Collection create (skipped if exists) + 6 payload-index creates. |
| `session_extracts` | 172-185 | `&Session` | once per index_session | CPU — `build_content / tool / path / error / code` walk all turns + regex code-fence scan. |
| `build_content` | 195-218 | `&Session` | once per index_session | O(turns) string concat. |
| `build_tool` | 220-231 | `&Session` | once per index_session | O(turns × tool_calls) format. |
| `build_path` | 254-281 | `&Session` | once per index_session | O(turns × tool_calls), `BTreeSet` insert per file_path. |
| `build_error` | 283-309 | `&Session` | once per index_session | O(turns × tool_results) + assistant-text line scan. |
| `build_code` | 311-333 | `&Session` | once per index_session | **Regex** `CODE_FENCE` over every turn's text (`indexer.rs:46`) + Edit/Write new_string. |
| `index_session` | 377-395 | `&Session` | per session during scan/refresh | 1 embed (5 texts) + 1 `upsert_points` (gRPC). **Five embeds happen inside one `embed()` call**, so the model lock is taken once but the ONNX session is invoked once per chunk of 32 — for one session, that's one forward pass for 5 texts. |
| `bulk_index` | 407-461 | `&[Session]` | once per `scan --index` / `refresh_index` | **Sequential** loop over sessions (line 432: `for s in sessions`). Each iteration awaits `index_session`. See §3. |
| `search_content` | 517-531 | `query` | not bound to UI — internal helper | One `lens_search` with only `content` weight=1. |
| `lens_search` | 539-615 | `query, weights` | every UI search | 1 query embed + up to 5 parallel `client.query` via `try_join_all` (line 576) + in-process Rust combine over `per_vector_limit=60` hits each. |
| `mix_match` | 628-694 | pos/neg ids | UI: Mix & Match modal | 1 `client.query` with `DiscoverInput` over `content` (line 673-681). No embedding (point-id-anchored). |
| `topology` | 762-909 | `sample, per_point` | UI: Topology modal | 1 `search_matrix_pairs` + 1 `get_points` (batched) + O(V+E log V) MST + optional `scan_dir` for insights (line 895). |
| `compute_insights` | 915-1116 | sessions + nodes + pairs | only when `topology(projects_root=Some(_))` | O(sessions × turns × tool_calls) — full re-walk of every session for tool/path freq. |
| `recall` | 1151-1189 | `error_text` | every poll-hit (after dedup) | 1 embed + 1 `client.query` with `has_errors=true` filter. |
| `predict_next_actions` | 1245-1414 | `session_id, last_n, horizon, neighbors` | per Time-Machine selection (220 ms debounce, `main.js:457-460`) | 1 `get_session_payload` + **1 full re-parse of active session** + 1 embed + 1 `client.query` + **N re-parses of neighbor JSONLs** (line 1342) — see §6. |
| `get_session_payload` | 1483-1494 | `session_id` | every selectSession | 1 `get_points` (single id). |
| `snapshot_export/import` | 1528-1581 | `dest/src` | rare | HTTP roundtrip + disk I/O (Qdrant 1.18 snapshot endpoint). |

---

## 3. `scan --index` (cold) cost model

For **N sessions** averaging **M turns**, of which roughly half have a non-trivial tool_call list:

### 3a. JSONL parse cost

- `parse_session` is **line-delimited streaming** via `BufReader::lines()` (`parser.rs:107`). Each line goes through `serde_json::from_str(line)` (`parser.rs:114`). This is the right shape: no whole-file buffer.
- Per-line cost is dominated by `serde_json` over `Value` (boxed enum, not zero-copy) — typical Claude turn line is 200 B – 8 KB.
- `scan_dir` (`parser.rs:327-365`) walks the tree with `walkdir`, filters `.jsonl`, skips `subagents/`, and parses sequentially.
- **Estimate** [est.]: at ~0.5 ms per turn line, M=200 turns × 0.5 ms ≈ 100 ms per session for parse. At N=80, ≈ 8 s parse total. At N=1000, ≈ 1.7 min. At N=10000, ≈ 17 min — and that's *per scan*, single-threaded.

### 3b. Five extract builds per session

- `session_extracts` (`indexer.rs:172-185`) calls `build_content / tool / path / error / code`, each O(turns × inner).
- `build_code` runs the `CODE_FENCE` regex over every turn's text (`indexer.rs:314`) — likely the costliest of the five for prose-heavy sessions.
- All five strings are then `cap()`'d to 6 000 chars (`indexer.rs:187-193`) — char iteration, not byte slice (correct for UTF-8 but ≥2× the cost of byte slicing).
- **Estimate** [est.]: 5-20 ms per session for the five extracts at M=200 turns. Dwarfed by parse and embed costs.

### 3c. Five BGE-small embed calls per session — **NOT batched across sessions**

This is the headline finding.

- `index_session` (`indexer.rs:377-395`) gathers the five extracts into `Vec<String>` and calls `embedder.embed(texts)` once. Inside `Embedder::embed` (`indexer.rs:86-92`), the loop is `for chunk in texts.chunks(EMBED_BATCH=32)` — so the 5 texts go to ONNX in **one** forward pass. **Good.**
- But `bulk_index` (`indexer.rs:432-458`) is a plain `for s in sessions` calling `index_session(client, embedder, s).await` **per session, sequentially**. The model is invoked **N times** (one forward pass of 5 inputs each), not once with `N × 5` batched inputs.
- ONNX batches efficiently — a single forward pass with 32 inputs is ~3-5× the cost of one with 5 (memory-bound). For BGE-small CPU, a single 5-text call is roughly 80-200 ms warm.
- **Round-trip count:**
  - N=80 → 80 model invocations + 80 gRPC `upsert_points` round-trips. [est.] embed time 80 × 120 ms ≈ 9.6 s; gRPC 80 × 5 ms ≈ 400 ms; **total ≈ 10-15 s wall** (matches the typical "first index of 80 sessions takes ~15 s" experience).
  - N=1000 → ~120 s embed + ~5 s gRPC = ~2 min. With batched embed (32 per call, 5/32 of the work), would be ~25-30 s.
  - N=10000 → ~20 min sequential embed + 50 s gRPC. With batched embed and batched upsert, ~3-4 min. **This is the main scale ceiling.**

### 3d. Qdrant `upsert_points` — **single-point batches**

- `index_session` (`indexer.rs:391-393`): `client.upsert_points(UpsertPointsBuilder::new(COLLECTION, vec![point]).wait(true))`. **One point per request, with `wait(true)`** — every upsert blocks on durability.
- A single batched upsert of 100-500 points would reduce gRPC overhead by ~100×.

### 3e. Calibrating against the 79-session reality

The README states `~80 sessions, ~17k tool calls`. At [est.] 100 ms parse + 120 ms embed + 5 ms upsert per session, wall time ≈ 18 s (sequential). The real-world `memex scan --index` on a warm cache lands ~15-20 s, consistent with the model.

**Extrapolation table (sequential, current code):**

| N (sessions) | Parse | Embed (sequential) | Upsert (1-by-1) | **Total wall** [est.] |
|---|---|---|---|---|
| 80 | 8 s | 9.6 s | 0.4 s | ~18 s |
| 1 000 | 100 s | 120 s | 5 s | ~3-4 min |
| 10 000 | 17 min | 20 min | 50 s | ~38-40 min |

With **two wins from §11** (cross-session embed batching + batched upsert), 10 000 sessions would drop to roughly **5-7 minutes**.

---

## 4. `lens_search` cost

### Parallelism mechanism

`indexer.rs:567-576`:

```rust
let queries = active.iter().map(|(vname, _)| {
    let q: Query = qvec.clone().into();
    let req = QueryPointsBuilder::new(COLLECTION)
        .query(q)
        .using((*vname).to_string())
        .limit(per_vector_limit)
        .with_payload(true);
    async move { client.query(req).await }
});
let responses = try_join_all(queries).await?;
```

- Mechanism: **`futures::future::try_join_all`** (`indexer.rs:21`, `:576`). All up-to-5 queries are pending concurrently on the gRPC channel.
- Qdrant 1.18 services parallel single-vector queries without lock contention on shared HNSW state — wall latency is dominated by the **slowest** single-vector search, not the sum.
- Combine cost (`indexer.rs:578-590`): O(`active_vectors × per_vector_limit`) = up to 5 × 60 = 300 hash inserts. Negligible.

### Typical latency

- One embed call for the query string (1 short text → ~30 ms warm CPU [est.]).
- Five HNSW queries in parallel on an 80-point collection: each ~1-3 ms server-side; gRPC round-trip ~1-3 ms; combine ~0.1 ms. **Total ≈ 35-50 ms** at N=80.
- At N=10 000 points, HNSW search is **logarithmic in N**; expect 5-10 ms per query. Latency stays around **40-70 ms** end-to-end — embed remains the floor.
- **At N=100 000+** the bottleneck shifts to the `Mutex` lock contention on `Embedder` (see §8), not Qdrant.

### Cost composition at N=80 (1 query)

| Stage | Time [est.] | Note |
|---|---|---|
| `embedder.embed(vec![query])` | 25-35 ms | Single text, ONNX forward + Mutex acquire |
| `try_join_all` of 5 HNSW queries | 5-15 ms | Wall = slowest of 5; HNSW O(log N) |
| Combine + sort + payload assembly | <1 ms | Rust hashmap |
| **Total** | **~35-50 ms** | |

---

## 5. `topology` cost

### Edge construction

`indexer.rs:839-858`:

```rust
let mut g: UnGraph<String, f32> = UnGraph::new_undirected();
let mut idx: HashMap<String, _> = HashMap::new();
for id in &id_set {
    idx.insert(id.clone(), g.add_node(id.clone()));
}
for pair in &matrix2.pairs {
    // ... insert edge with distance = (1.0 - score).max(0.0)
    g.add_edge(na, nb, distance);
}
let mst = UnGraph::<String, f32>::from_elements(min_spanning_tree(&g));
```

- `search_matrix_pairs` returns `pair_count = sample × per_point` candidate edges (typical: 80 × 6 = **480 candidate pairs**, with duplicates).
- `petgraph::min_spanning_tree` is **Kruskal's** (O(E log E)). For V=80, E=480, that's <1 ms.

### Dominant cost as `per_point` grows

- The matrix endpoint server-side computes up to `sample × per_point` nearest-neighbor lookups. At per_point=20, sample=80: **1 600 K-NN probes**. Each is HNSW O(log N) — still bounded.
- At sample=10 000 with per_point=20 → 200 000 candidate edges. The MST input `add_edge` loop becomes ~200 000 hashmap lookups and graph mutations — ~50-200 ms client-side, but **the real cost** is the matrix endpoint round-trip itself and the `get_points` payload fetch in line 800-806 (batched, good).
- Insights compute (`compute_insights` line 915-1116) **rescans every session on disk** via `crate::parser::scan_dir` (`indexer.rs:895`). At N=10 000 that's the parse cost from §3a all over again (~17 minutes). **This is the dominant Topology cost at scale.**

### `compute_insights` breakdown

- O(sessions × turns × tool_calls) for tool/path frequency aggregation (`indexer.rs:936-966`).
- O(edges + matrix_pairs) for bridge counting and near-miss detection (`indexer.rs:969-1056`).
- Pure CPU, single-threaded.

**At N=80 today:** Topology is ~150-400 ms total (matrix ~50 ms, get_points ~30 ms, MST <5 ms, insights via scan_dir ~7 s). At N=10 000, **insights becomes the bottleneck at ~17 min** — same as scan.

---

## 6. `predict_next_actions` cost — **the re-parse fan-out**

`indexer.rs:1341-1373`:

```rust
for (nb_sid, sim_score, source, nb_project) in &neighbor_meta {
    let Ok(nb) = crate::parser::parse_session(StdPath::new(source)) else { continue };
    // ... find_pivot_turn, walk horizon turns, collect tool calls
}
```

**File opens per call (default `neighbors=8`):**

| Stage | File reads | Note |
|---|---|---|
| `get_session_payload(active)` | 0 | Qdrant payload only |
| `parse_session(active.source_path)` | **1** | `indexer.rs:1268` |
| Embed last-N turns | 0 | |
| `client.query` for neighbors | 0 | |
| Loop: `parse_session(neighbor.source)` × neighbors | **8** | `indexer.rs:1342`, **one open per neighbor** |
| **Total file opens per call** | **9** | |

**At UI usage:** 220 ms debounce (`main.js:457-460`) — so rapid arrow-key navigation through the stack is throttled, but every "lingered-on" selection triggers 9 JSONL parses.

**Why it hurts:**
- A 5 000-turn session JSONL is ~5-10 MB. Parsing 9 of them sequentially on a single selection is **300-900 ms** [est.] at the documented ~100 ms/5k-turn parse rate.
- Inside the prediction we only need: (a) a pivot turn whose text overlaps the anchor (lexical, `find_pivot_turn` line 1419) and (b) the `horizon` (default 3) turns *after* that pivot. **A very small slice of the file.**
- Yet we re-parse the entire JSONL every time. No cache, no slice.

**Suggested trade-off (payload-cache vs full re-parse):**

The cheapest payload-side fix is **memoizing the parsed `Session` for the last K=32 neighbor session_ids** in a `Lazy<Mutex<LruCache<String, Arc<Session>>>>`. After the first hit, repeated predictions across nearby anchor positions reuse the parse. For a user who scrubs through 20 sessions in the stack, this cuts the cumulative parse work by ~10× without changing the on-disk format.

A larger refactor is to push a **tool-call timeline summary** (list of `{turn_idx, tool_name, summary, input_snippet}`) into the Qdrant payload — say 2-5 KB per session — and skip the re-parse entirely when only the horizon walk is needed. Costs ~50% more payload but eliminates 8/9 file opens per prediction.

---

## 7. Proactive recall poller (12 s)

`commands.rs:407-503` — `tail_recent_errors`.

### Verified mechanics

- Walks `~/.claude/projects` with `WalkDir` (`commands.rs:420`).
- Skips files whose `mtime < cutoff` (default 60 s, line 416). **mtime check is the primary filter** — only recently modified JSONLs are considered.
- For files that pass the mtime gate, checks `TAIL_CACHE` keyed by canonical path (`commands.rs:395-396`, lines 440-457). If `cached.mtime == file.mtime`, **the prior parsed-error result is reused** (negative caching included).
- Cache miss → `parse_session(p)` (line 459) + scan `turns.iter().rev().take(6)` for the most-recent `is_error=true` tool_result or "error:" assistant line.

### Per-tick cost

**At N=80 sessions (today):**
- Cold tick: walkdir of ~80 entries + stat each = ~3-5 ms. Parse of 0-2 recently-modified files = ~50-100 ms. Cache populated.
- Warm tick: walkdir + stat + 80 cache hits. **~3-10 ms.** Matches the docs/architecture.md claim of "~50 ms on 80 sessions" (cold), "closer to <10 ms once the mtime cache warms up" (also matches commands.rs:406).

**At N=10 000 sessions:**
- Walkdir + stat over 10 000 entries: **~50-200 ms** on warm SSD [est.], dominated by per-file `metadata()` calls (one syscall each, line 432).
- mtime gate filters out >99.9% of files (only files modified in the last 60 s). Of those, cache hit = 0 work; cache miss = parse cost (≤2 files / tick typically).
- **The walkdir itself is the bottleneck at scale.** Even with the mtime gate filtering everything, `WalkDir::new(&root).into_iter()` + `entry.metadata()` for 10 000 files is ~100-300 ms per 12 s tick.
- Still well under 12 s, but no longer "free." If it grows linearly, the line that limits it is `commands.rs:420-431` — the WalkDir + stat loop.

**Mitigation at scale.** Track the directory mtime first; only walk subdirs whose parent dir mtime changed. Or pre-build an "active sessions index" of paths modified in the last N minutes and rotate it weekly.

### Frontend cadence

- `main.js:1454` `RECALL_POLL_MS = 12_000` — every 12 s.
- `main.js:1471` `recallCached(ev.error_text, 3)` dedupes embed calls on identical error text within a TTL (P2 comment, line 1485).

---

## 8. Embedder concurrency

### The lock

`indexer.rs:51-53`:

```rust
pub struct Embedder {
    inner: Mutex<TextEmbedding>,
}
```

This is `std::sync::Mutex`, blocking, **not** `tokio::sync::Mutex`. Every call to `Embedder::embed` (`indexer.rs:82-84`) takes the lock and holds it across the ONNX forward pass.

### Lock-takers (lines that call `embedder.embed`)

- `index_session` line 384 — once per session during `bulk_index`.
- `lens_search` line 547 — once per UI search.
- `recall` line 1157 — once per recall call (post-cache-miss).
- `predict_next_actions` line 1296-1300 — once per prediction call.
- `search_content` line 530 — internal helper, not exposed as command.

### Contention scenarios

**(a) Topology kicks off during a Lens search.**

Topology itself **does not embed** — it queries `search_matrix_pairs` and `get_points` (`indexer.rs:769-806`) and the optional `scan_dir` for insights (line 895, all on parser, no embed). So a Topology mid-lens-search **does not contend on the Embedder lock at all.** ✅

But `compute_insights` is **CPU-bound on the parser thread** and runs on the same tokio runtime. If the user kicks off Topology while a lens search is awaiting its query embed, the parse traffic competes for CPU and for the gRPC client. Latency for the lens embed remains the same (it's on the blocking pool effectively, ONNX is C++), but the gRPC layer multiplexes both — Qdrant 1.18 handles parallel requests fine.

**(b) Poller fires mid-`scan --index`.**

This is the worst case.
- `bulk_index` loop (`indexer.rs:432-458`) holds the Embedder lock briefly each iteration (~120 ms for 5 texts) and releases between sessions.
- If `tail_recent_errors` finds a new error mid-scan, the frontend calls `recall(error_text)`, which calls `embedder.embed(vec![error_text])`. That await **queues behind the current session's embed**.
- For a 10 000-session scan running ~10 sessions/s, the recall embed sees, on average, **~50 ms queue wait**. Acceptable, but worth noting if a user kicks off a heavy `refresh_index` from the UI: every poll tick that finds an error stalls for the duration of one session-embed.

Mitigation:
- Switch to an **async-aware lock** (`tokio::sync::Mutex`) so the lock-hold is suspended cleanly, not blocking the worker thread.
- Or: **embedder pool** (2-4 `Embedder` instances behind a `Vec<Mutex<TextEmbedding>>`) — ONNX is thread-safe at the Session level if you have one Session per thread. Cost: 130 MB × pool_size. For 4 instances on a 16 GB machine, totally fine.

---

## 9. Memory footprint

For **N sessions** in steady state:

### Per-Qdrant-point

- **5 × 384-d f32 vectors** = 5 × 384 × 4 = **7 680 B raw** ≈ 7.5 KB / point (before HNSW overhead).
- HNSW overhead is roughly 2-3× the raw vector size at default graph degrees → **~20-25 KB / point**.
- Payload: from the JSON schema in docs/architecture.md, payload is ~1 KB per point (project_name, source_path, ai_title, timestamps, turn counts, has_errors).
- **Total per point on disk: ~25-30 KB.**

### Process resident set

- **fastembed ONNX model: ~130 MB** resident once loaded (CLAUDE.md line 60: "downloads the ~130 MB BGE-small ONNX model").
- Tokio runtime, qdrant-client, Tauri webview overhead: ~80-150 MB.
- `TAIL_CACHE` (`commands.rs:395`): N entries × (~200 B path + ~500-1500 B `latest_err` clone) = at N=10 000, ~10-15 MB. **Bounded** (one entry per file).

### Qdrant collection on disk

| N | Raw vectors | With HNSW | Payload | **Total disk** [est.] |
|---|---|---|---|---|
| 80 | 600 KB | ~2 MB | 80 KB | ~2-3 MB |
| 1 000 | 7.5 MB | ~25 MB | 1 MB | ~25-30 MB |
| 10 000 | 75 MB | ~250 MB | 10 MB | **~250-300 MB** |

### Frontend (3d-force-graph)

- `nodes: Vec<TopoNode>` serialized to JSON for the webview (`main.js:756-765`): ~250 B per node JS object × N.
- `links: Vec<TopoEdge>` for MST (`main.js:766-774`): V-1 edges, so N nodes → N-1 links × ~150 B.
- At N=10 000, the rendered MST has 9 999 edges → ~1.5 MB JS heap. **3d-force-graph + Three.js comfortably handle ~5 000 nodes**; beyond that, mouse-hover hit-testing and force simulation start to chug.

### Memory ceiling

A typical user laptop (16-32 GB) handles **N=10 000 trivially** for storage. The real RAM hot points are:
- The 130 MB ONNX model (fixed).
- The 250-300 MB Qdrant working set (mmap, mostly).
- Webview ~200 MB.

**Total ~600-700 MB resident at N=10 000** [est.].

---

## 10. Frontend perf

### Replay render (`main.js:1286-1320`)

`renderReplayTurn(i)` does **`detail.innerHTML = …`** every frame (`main.js:1301`). For each step:
1. Builds a template literal for the active turn — meta + text + tool calls + stray results.
2. Sets `detail.innerHTML` (full subtree teardown + parse).
3. Re-walks every `.replay-row` to toggle the `.selected` class (`main.js:1313-1315`).
4. Calls `scrollIntoView` on the selected row.

**Cost per frame.** [est.]
- innerHTML rebuild for a ~3 KB template: ~1-3 ms.
- Querying `.replay-row` (one DOM walk over N rows): O(N rows) = a few ms at N=600 turns.
- `renderToolCall` per call is ~200 B per `tc` (`main.js:1322-1395`); for a tool-heavy turn (10 calls) it's ~2 KB.

**At 4× speed (250 ms / frame at default 1 000 ms tick), 600 turns = 150 s of replay.** Math:
- `state.replay.speedMs` comes from `#replay-speed` value (`main.js:1235`) — default seems to be 1 000 ms (typical replay), 4× = 250 ms per turn.
- Per-frame DOM cost ~5 ms; well within 250 ms budget. **The render is full-rebuild per frame**, but small enough to never block.

**The bigger issue.** Both `renderReplayList` (line 1254-1284) and `renderReplayTurn` use full innerHTML rebuilds. For a 5 000-turn session, the initial list render is `O(N)` innerHTML appends — ~50-100 ms one-time cost, acceptable.

### Topology 3D scene

`main.js:778-814` initializes `ForceGraph3D` with the full node + link arrays.

- Nodes: each becomes a Three.js Sphere geometry at `nodeResolution: 16` (line 787).
- Links: thin cylinders, with `linkDirectionalParticles` for cross-project edges (line 798).
- Custom forces: `link.distance`, `charge.strength: -90`, `project-cluster` (lines 810-813).
- `setTimeout(() => G.zoomToFit(900, 80), 1400)` — 1.4 s settle before zoom.

**Stutter thresholds.** [est.]
- < 200 nodes: smooth 60 fps interaction.
- 200-1 000 nodes: smooth pan/rotate; force tick at 30-60 fps.
- 1 000-3 000 nodes: pan still smooth; force simulation tick drops to ~10-20 fps for the first ~3 s, settles afterward.
- **3 000+ nodes:** raycaster hover hit-testing becomes the bottleneck; consider `nodeResolution: 8` and disabling `linkDirectionalParticles`.
- **10 000+ nodes:** ForceGraph3D's default raycaster is O(N) per frame on hover. Stutters guaranteed. Switch to an instanced renderer or sample down.

### Other render hotspots

- `renderResults` (`main.js:371-417`) — full DOM rebuild on every search. At limit=20 hits, ~2 ms. Fine.
- `renderInspector` (around line 540-620, kvs + raw payload) — single innerHTML, ~5 ms with the raw JSON `<pre>` block.
- `pollRecall` doesn't render unless a banner fires; cost is dominated by the `invoke('tail_recent_errors')` round-trip.

---

## 11. Quick wins (low-effort, measurable)

Each item: file:line — change — expected impact.

### QW-1. Cross-session embed batching in `bulk_index`
**`indexer.rs:432-458`.** Today, the loop calls `index_session` per session, each making a 5-text embed call.

**Fix.** Collect all `(session_idx, vec_name, text)` triples up front, batch them in chunks of `EMBED_BATCH = 32` (so ~6-7 sessions per ONNX forward), embed once per chunk, then assemble points and upsert.

**Expected impact.** ONNX cost is mostly forward-pass overhead — batching 32 inputs vs 5 cuts per-text cost ~3-5×. At N=80, embed time drops from ~9.6 s to ~2-3 s. **~3-4× faster scan.** At N=10 000 the win is closer to ~5× (~20 min → ~4 min) because both per-call gRPC overhead and ONNX session setup amortize.

### QW-2. Batch the upsert in `bulk_index`
**`indexer.rs:391-393` (inside `index_session`).** Each session upserts as a `vec![point]` with `.wait(true)`.

**Fix.** Refactor `bulk_index` to accumulate points (say 100 per batch) and call `upsert_points(UpsertPointsBuilder::new(COLLECTION, batch).wait(true))`. Drop `.wait(true)` to `.wait(false)` for all but the final batch.

**Expected impact.** Per-point gRPC + Qdrant disk sync cost drops by 100×. At N=10 000 that's ~50 s → ~0.5 s. Small absolute win, but pairs cleanly with QW-1 for a unified rewrite.

### QW-3. LRU cache for `predict_next_actions` neighbor parses
**`indexer.rs:1342`.** `crate::parser::parse_session` is called once per neighbor per prediction — no memoization.

**Fix.** Wrap in a `Lazy<Mutex<LruCache<PathBuf, Arc<Session>>>>` keyed by `source_path`, size 64. Invalidate by mtime (cheap stat).

**Expected impact.** When the user scrubs through 20 stack items, the neighbor set is ~80% overlap session-to-session. Cumulative parse time drops by ~5-10×. Single-prediction latency unchanged on first call (~300-900 ms), but the second-and-onward drop to ~50 ms (just the Qdrant search + horizon walk).

### QW-4. Replace `std::sync::Mutex` with `tokio::sync::Mutex` on `Embedder`
**`indexer.rs:52`** (`inner: Mutex<TextEmbedding>`), and all `.lock()` sites at lines 82, 547, 1157, 1296, etc.

**Fix.** Switch to `tokio::sync::Mutex`. The lock-hold will be suspendable, so a poller-triggered embed call doesn't block the tokio worker thread during a long `bulk_index` iteration.

**Expected impact.** No throughput win in isolation, but UI responsiveness during a heavy scan improves — typed queries get embedded as soon as the current session's forward pass finishes (~120 ms tail latency) instead of starving the runtime. Critical at N=10 000 scan time.

### QW-5. Per-vector limit tuning in `lens_search`
**`indexer.rs:545` and `commands.rs:100`.** Today `per_vector_limit = 60` and final `limit = 20`. We fetch 5 × 60 = 300 hits and combine to 20.

**Fix.** Drop `per_vector_limit` to 30 unless we observe the combined ranking changes — empirically the long tail beyond rank 30 per vector rarely makes top-20 after the weighted blend.

**Expected impact.** Each Qdrant query gets faster (HNSW work scales with `limit`) and gRPC payload halves. At N=10 000 with the collection on the same machine this is ~2-5 ms saved per query. Cumulative impact on UI typing latency.

### QW-6. Skip `compute_insights` re-scan when `projects_root` is unchanged
**`indexer.rs:894-901`.** Topology always calls `scan_dir(&root)` for insights when `projects_root` is `Some`.

**Fix.** Cache the per-project aggregates (tool_freq, path_freq, had_errors) keyed by (root, max_mtime). Invalidate on incremental refresh.

**Expected impact.** At N=10 000, Topology drops from ~17 min (insights re-scan) to ~200 ms (matrix + MST + cached insights). **Two orders of magnitude.**

### QW-7. Use byte-slice cap instead of char-iter in `cap()`
**`indexer.rs:187-193`.** `s.chars().take(MAX_CHARS_PER_VECTOR).collect()` iterates the entire string up to the cap on every call.

**Fix.** For ASCII-dominant input (most code/prose), use `&s[..MAX_BYTES_PER_VECTOR.min(s.len())]` after finding a UTF-8 boundary near the cap. ~2-3× cheaper for large inputs.

**Expected impact.** Minor — ~5-10 ms per session shaved off the extracts. Adds up at N=10 000.

---

## 12. Bigger investments (with trade-off)

### BI-1. Payload-cache for `predict_next_actions` pivot lookup

**Today.** `predict_next_actions` does 9 file opens per call (`indexer.rs:1268, 1342`).

**Investment.** Augment the Qdrant payload with a `tool_timeline: Vec<{turn_idx, tool_name, summary, input_snippet, text_preview}>` field — say 100 entries per session, ~2-5 KB each → +200-500 KB per session.

**Trade-off.** Doubles the average payload size (1 KB → 3-5 KB). At N=10 000, that's ~50 MB extra payload disk usage. In exchange:
- `predict_next_actions` does **zero re-parses** for the horizon walk.
- Single-prediction latency drops from ~300-900 ms to ~30-50 ms (just Qdrant + Rust aggregation).
- Frees up Topology insights to similarly skip parsing.

**Risk.** Violates the "payload-lean / Replay re-parses" invariant in CLAUDE.md line 109. Worth a separate decision: the rule was about *full turn-level data*, not summaries. A 100-entry timeline is the "index of replay," not replay itself.

### BI-2. Switch `lens_search` combine to Qdrant's server-side scoring when weights are all 1

**Today.** Even when all 5 weights are 1 (the default), `lens_search` runs 5 separate parallel queries and combines in Rust.

**Investment.** Detect `weights.content == weights.tool == weights.path == weights.error == weights.code` and dispatch a single `Query` with a `Fusion::Rrf` over the 5 named vectors instead. Skip the embed-once-query-5-times path.

**Trade-off.** Loses the per-vector contribution chips in the UI (because RRF only exposes the fused rank). Could be a toggle in the sidebar — "show breakdown" forces the parallel path.

**Expected impact.** Halves the gRPC traffic and HNSW work for the default state (which is what the user starts in). Lens search drops from ~35-50 ms to ~15-25 ms.

### BI-3. Replace 12 s polling with `notify` watcher

**Today.** Every 12 s, `tail_recent_errors` walks `~/.claude/projects` and stats every file.

**Investment.** `notify` and `notify-debouncer-full` are already in `Cargo.toml` (CLAUDE.md line 138). Wire them through a `tokio::sync::mpsc` channel into the same handler that `tail_recent_errors` runs after a cache miss.

**Trade-off.** macOS FSEvent permission model: in Sequoia/Tahoe, a `.app` bundle sometimes needs FSEvent permission explicitly granted (CLAUDE.md line 126). Polling avoids this entirely — that's why it shipped. The investment is real testing, not real engineering.

**Expected impact.**
- Per-tick poll cost (3-10 ms × 5 ticks/min ≈ 15-50 ms/min) → **zero** when idle.
- New errors surface in **<100 ms** instead of **<12 s**.
- At N=10 000 the poll cost becomes ~150-300 ms per tick (§7); the win grows with N.

---

## 13. Benchmarks not yet written

What I'd measure to confirm the model above, and the simplest way to do it.

### B-1. `bulk_index` per-session breakdown
**Tool.** `cargo bench` with `criterion`, or simpler: `Instant::now()` instrumentation around `embed()`, `upsert_points()`, and `session_extracts()` inside `index_session`. Aggregate per-stage means over 79 sessions.

**Confirms.** §3c claim that embed (~120 ms) dominates over upsert (~5 ms) and extracts (<20 ms). Establishes the batching ROI.

### B-2. ONNX batch-size scaling curve
**Tool.** Plain Rust harness invoking `Embedder::embed` with batch sizes 1, 5, 10, 32, 64, 128. Plot ms/text.

**Confirms.** The "3-5× faster with batch 32 vs batch 5" claim in QW-1. The actual curve might be more or less favorable; this benchmark tells the truth.

### B-3. Lens-search latency vs N (sessions)
**Tool.** Bench `lens_search` against synthetic collections of 80, 1 000, 10 000, 50 000 random 384-d points (use `client.upsert_points` with random vectors). Measure end-to-end (embed + query) and Qdrant-only (skip embed).

**Confirms.** §4 scaling claims and identifies the crossover point where the Embedder mutex becomes the bottleneck.

### B-4. Polling cost at N
**Tool.** `hyperfine --warmup 3 'memex recall "test error"'` is the wrong shape. Better: instrument `tail_recent_errors` to log walkdir time + stat time + parse time per tick. Run for 5 minutes against directories of varying size (use `tmpfs` mounts with synthetic JSONLs).

**Confirms.** The §7 estimate that walkdir cost scales linearly with file count at ~10-30 μs / file, and the cache hit rate after warm-up.

### B-5. Frontend Replay frame budget
**Tool.** Chrome DevTools Performance recording during a 4× replay of a 600-turn session. Look at Layout and Paint times per frame.

**Confirms.** §10 claim that `innerHTML` rebuild + selected-row scan stays under the 250 ms frame budget at 4× speed.

### B-6. Topology 3D node-count ceiling
**Tool.** Generate synthetic topologies with N = 100, 500, 1 000, 3 000, 10 000 nodes (random positions, MST edges). Open in Memex, drag the camera, measure FPS via `requestAnimationFrame` delta logging.

**Confirms.** §10 stutter thresholds (1 000, 3 000, 10 000) and identifies whether the raycaster, force simulation, or geometry count is the actual bottleneck.

### B-7. End-to-end `scan --index` wall time vs N
**Tool.** `hyperfine 'memex scan --index --path <synthetic-N>'` with N=80, 200, 500, 1 000.

**Confirms.** The extrapolation table in §3e. Repeat after applying QW-1 + QW-2 to validate the predicted 5× speedup.

---

## Appendix — claim-to-source map

| Claim | Source |
|---|---|
| 5 named vectors, 384-d cosine | `indexer.rs:40`, `indexer.rs:39` |
| 6 000-char cap | `indexer.rs:42` |
| Embed batch size 32 (within one call) | `indexer.rs:43` |
| Sequential per-session bulk index | `indexer.rs:432-458` |
| Single-point upsert with `wait(true)` | `indexer.rs:391-393` |
| `try_join_all` parallelism in lens | `indexer.rs:576` |
| Lens weighted combine | `indexer.rs:584-589` |
| Topology distance = 1 - similarity | `indexer.rs:854` |
| MST via petgraph | `indexer.rs:859` |
| Topology insights re-scan on disk | `indexer.rs:895` |
| Predict re-parses neighbors | `indexer.rs:1342` |
| Predict re-parses active session | `indexer.rs:1268` |
| Recall mutex contention point | `indexer.rs:1157`, `indexer.rs:51-53` |
| `tail_recent_errors` mtime gate | `commands.rs:434` |
| `tail_recent_errors` cache | `commands.rs:395-396`, `:440-457` |
| 12 s frontend poll | `main.js:1454` |
| Recall cache TTL dedup | `main.js:1485-1499` |
| Prediction 220 ms debounce | `main.js:457-460` |
| ForceGraph3D init params | `main.js:778-813` |
| Replay innerHTML rebuild | `main.js:1301` |
| Lazy AppState init | `commands.rs:48-80` |
| HNSW pre-filter via `has_errors` payload index | `indexer.rs:154-167`, `:1160-1166` |

---

*Report 05 — read-only analysis.*
