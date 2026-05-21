# Memex — Multi-Agent Audit & Research Dossier

**Generated**: 2026-05-18
**Workspace**: `/Users/kimsejun/Documents/GitHub/memex`
**Method**: 10 specialist agents dispatched in parallel — 5 read-only codebase analysts + 5 web-research analysts. Every report enforces a single citation rule (`file:line` for code, `URL · fetched 2026-05-18` for external sources).
**Total output**: 5 160 lines across 10 reports.

---

## I. Codebase analysis (5 reports, 2 226 lines)

| # | Report | Lines | Lead specialist | Top finding |
|---|--------|------:|-----------------|-------------|
| 01 | [Backend Rust architecture](analysis/01-backend-rust-architecture.md) | 403 | `backend-architect` | `indexer.rs` is 1581 LOC carrying 5 disjoint concerns; mechanical split into a module folder would not break any external API |
| 02 | [System architecture & invariants](analysis/02-system-architecture-decisions.md) | 456 | `system-architect` | 14 hard invariants enforceable in code; doc drift bounded (docs claim 5 features, code has 7 surfaces); scales comfortably to ~2 000 sessions |
| 03 | [Frontend + Tauri IPC integration](analysis/03-frontend-tauri-integration.md) | 562 | `frontend-architect` | 12/13 IPC commands wired — single orphan is `snapshot_import` (no GUI surface); no-LLM / no-chat invariant verified intact via keyword grep |
| 04 | [Security audit](analysis/04-security-audit.md) | 250 | `security-engineer` | 12 findings (0 Critical / **3 High** / 2 Medium / 4 Low / 3 Info); zero-network claim FALSE on first launch (fastembed model download from huggingface.co) |
| 05 | [Performance analysis](analysis/05-performance-analysis.md) | 555 | `performance-engineer` | Top 3 quick wins identified — cross-session embed batching (3-4× indexing throughput), Topology insights cache (Topology at N=10k: 17 min → 200 ms), LRU for predict's pivot parse |

### High-severity items to act on (from §04)

| ID | Title | Location | Impact |
|----|-------|----------|--------|
| SEC-002 | `csp: null` + `withGlobalTauri: true` | `tauri.conf.json` | Any HTML injection = full IPC access |
| SEC-003 | `source_path` from payload → `File::open` unchecked | `commands.rs:195`, `indexer.rs:1268`, `indexer.rs:1342` | Malicious snapshot import → arbitrary file read |
| SEC-004 | `snapshot_export` accepts raw `PathBuf` from webview | `commands.rs` | Webview can write Qdrant snapshot to any writable path |

---

## II. Web research (5 reports, 2 934 lines)

| # | Report | Lines | Scope | Most actionable takeaway |
|---|--------|------:|-------|--------------------------|
| 01 | [Qdrant official docs (1.18+)](research/01-qdrant-official-docs-2026.md) | 900 | qdrant.tech/documentation + GitHub releases | Memex **already on latest** (Qdrant 1.18.0 shipped 2026-05-11, seven days before this audit). Top unused feature: native sparse-BM25 on `path`/`tool` + MMR reranking on dense lens |
| 02 | [Qdrant official blog catalog](research/02-qdrant-official-blog.md) | 507 | qdrant.tech/articles + /blog (31 articles cataloged) | Add a 6th named vector `content_late` (per-token BGE-small late-interaction MaxSim rerank) — uses Memex's *exact* existing model, no new LLM, no new download |
| 03 | [VSD 2026 hackathon ("Think Outside the Bot")](research/03-qdrant-vsd-2026-hackathon.md) | 421 | try.qdrant.tech/hackathon-vsd + announcement post | Submission gaps: (1) ≤3-min demo video **missing**, (2) Google Form **not filed**. Deadline 2026-06-01. Memex's no-LLM stance hits 4/5 patterns the 2025 jury rewarded |
| 04 | [Sparse / ColBERT / hybrid (deferral re-validation)](research/04-sparse-colbert-hybrid.md) | 385 | fastembed-rs 5.13.4 + Qdrant 1.15+ BM25 | README ColBERT deferral **still valid** (fastembed-rs Rust port has no late-interaction). README BM42 deferral **partially stale** — Qdrant 1.15.2+ server-side BM25 sidesteps the client-embedder gap entirely |
| 05 | [Rust + Tauri + fastembed ecosystem](research/05-rust-tauri-fastembed-ecosystem.md) | 721 | crates.io + GitHub release notes | Biggest payoff: wrap ONNX inference in `tokio::task::spawn_blocking + Semaphore(num_cpus/2)` (1-2h). No OSS project combines Qdrant + Tauri + fastembed-rs for AI session indexing — Memex is genuinely novel |

### VSD 2026 critical-path checklist (from §03)

1. ✅ **Brief alignment**: "Think Outside the Bot" → Strong (verified verbatim against `https://qdrant.tech/blog/vector-space-day-sf-2026/`)
2. ❌ **Demo video** (≤3 min, was 60s in 2025) — **must record before 2026-06-01**
3. ❌ **Google Form submission** at `https://forms.gle/YDQ2TDUi8MqS9Vx28` — **must file**
4. ✅ Public repo, Apache-2.0, README — all in place
5. **Total work to close gaps: ~17.5h, single-developer-doable in 14 days** per §10 of the hackathon report

---

## III. Cross-report synthesis

Patterns that appeared in **3 or more independent reports**:

| Theme | Reports raising it | Convergent recommendation |
|-------|-------------------|---------------------------|
| **Embedder mutex contention** | 01 (backend), 02 (system), 05 (perf), 05 (ecosystem) | Wrap inference in `spawn_blocking` + `Semaphore`; batch across sessions (3-4× indexing speedup) |
| **`indexer.rs` size (1581 LOC)** | 01 (backend), 02 (system) | Module split — non-breaking, mechanical |
| **Lens search vs Qdrant Query API + RRF** | 01 (backend), 02 (system), 04 (sparse-hybrid), 02 (blog) | Migrating lens to server-side prefetch + RRF unlocks sparse path-BM25 and late-interaction rerank in one Query API call |
| **Snapshot trust boundary** | 02 (system), 04 (security: SEC-003), 01 (official docs) | Validate `source_path` containment after snapshot import; consider snapshot signing or origin marker |
| **Predict's per-call file fan-out** | 02 (system), 05 (perf) | LRU cache the parsed sessions keyed by mtime |
| **Doc drift (docs claim 5 features, code has 7)** | 02 (system), 03 (frontend) | Update `docs/architecture.md` + `docs/qdrant-features.md` to add Predict + Time Machine sections |
| **README's ColBERT/BM42 deferral logic** | 04 (sparse-hybrid), 02 (blog), 01 (official docs), 05 (ecosystem) | ColBERT deferral still valid; BM42 deferral is partially stale — server-side BM25 is available now |

---

## IV. The single highest-impact path forward

Synthesizing across all 10 reports, the **two-week pre-VSD-2026-submission path** that maximizes judging-criteria fit without violating the no-LLM-at-runtime invariant:

1. **Record the ≤3-min demo video** + file the Google Form (§03 hackathon, ~4h, **submission-blocker**)
2. **Fix SEC-003/SEC-004** — path containment + snapshot-payload validation (§04 security, ~3h, **shouldn't ship with the High findings open**)
3. **Add server-side BM25 sparse on `path` + RRF prefetch in `lens_search`** (§04 sparse-hybrid, ~150 LOC, makes "exact filename / function-name" recall *visibly better* in the demo, retires a deferred-item in the README)
4. **Doc-drift fixes** — `docs/architecture.md` adds the 2 surfaces post-dating it (§02 system, ~1h)
5. **Quick-win QW-1 (embed batching)** if there's time — turns the cold `scan --index` demo step from "wait a minute" to "wait ten seconds" (§05 perf, ~2h)

Everything else (`indexer.rs` module split, ColBERT v2 via `ort` crate, `notify` swap, code signing) is post-hackathon.

---

## V. Provenance

- All 10 reports were authored by Claude Code agents dispatched in a single parallel `Agent` invocation block on 2026-05-18.
- Each agent received an isolated read-only brief + an explicit citation rule + a forbidden-actions list (no git operations, no source modifications, no chat-surface additions per the product invariant in `/CLAUDE.md`).
- The two halves of the dossier use different evidence regimes: code analysts cite `file:line`; web researchers cite `URL · fetched YYYY-MM-DD`. Both regimes are auditable.
- No agent had write access outside `claudedocs/`. The source tree was not modified.
- Per the absolute-rule clause in `/CLAUDE.md`, all git remote operations remain limited to `https://github.com/ComBba/memex`; this dossier introduces no remote traffic.
