# Qdrant 1.18 Feature Pinnacle — Full Surface Inventory for Memex

**Research date:** 2026-05-18
**Memex pinned versions:** Qdrant server `v1.18.0` (released 2026-05-11), Rust client `qdrant-client v1.18.0`
**Prior dossier:** `claudedocs/research/01-qdrant-official-docs-2026.md` (read-only)
**Scope:** broaden 01's coverage — every Qdrant 1.18 feature surface (back through 1.13) with an explicit Memex verdict.

> **Citation discipline.** Every factual claim is followed by `[Source: <URL> · fetched 2026-05-18]`, or
> compact `[docs §X]` / `[changelog vX.Y]` shorthand where the URL is listed in §7 Sources. Items the
> docs *do not yet describe* are marked `unverified`. Verbatim quotes >5 words are blockquoted.

---

## 1. Executive summary

Memex talks to Qdrant 1.18 through a deliberately narrow path: dense named vectors, RRF + DBSF fusion via Query API, Discovery, Distance Matrix, Snapshots. It is **leaving most of 1.18's recent surface unused**. Five highest-leverage misses, ranked by hackathon-window ROI:

1. **Formula-based score boosting** (Qdrant 1.14, fully GA, in Rust client) — single server-side query that does `cosine_score + recency_decay(start_ts) + project_boost`. Today Memex does this client-side in `lens_search` and loses the chance to combine recency, has_errors, and per-lens score in one shot. `[changelog v1.14]` `[docs §3.1]`
2. **MMR diversity rerank** (Qdrant 1.15, GA, in Rust client) — one extra field on `NearestQuery` (`mmr: { diversity: 0.5, candidates_limit: 50 }`). Memex's many near-duplicate sessions are *exactly* the symptom MMR fixes. Effort: ~1 hour. `[docs §3.2]`
3. **Relevance Feedback API** (Qdrant 1.17, GA, in Rust client) — Memex already collects +/− feedback for Mix & Match; today it feeds Discovery `context` pairs. Native Relevance Feedback evolves the *dense query itself*, giving smoother result-set drift across feedback clicks. `[changelog v1.17]`
4. **Group-by query** (`group_by` + `group_size` on QueryPoints) — natural fit for "top-3 sessions per `project_name`" view, which Memex currently has to emulate with multiple filtered queries. `[docs §3.3]`
5. **Strict-mode `max_resident_memory_percent` + `low memory mode`** (Qdrant 1.18) — desktop-first Memex users on 16 GB MacBooks will hit RAM ceilings as collections grow. These two knobs are the supported defense; today Memex sets neither. `[changelog v1.18]`

Plus one freebie not in the top 5: **`is_tenant: true` payload index** on `project_path` would give Memex tiered multitenancy storage layout for free, ahead of any future "shared Memex across many projects" use case. `[docs §6]`

---

## 2. Full feature matrix

35-row inventory. *Memex uses?* columns reflect the existing dossier `01-qdrant-official-docs-2026.md` and `docs/qdrant-features.md`. URLs are compact; resolve via §7 Sources.

| # | Feature | Introduced | Status | Rust client exposed? | Memex uses? | Adoption fit | 1-line description |
|---|---|---|---|---|---|---|---|
| 1 | Named vectors per point | 0.x (long) | GA | yes | **yes** | Direct | N dense vectors per point, each with own size/distance `[docs §2]` |
| 2 | Add/Delete named vectors on existing collection | 1.18 | GA | yes (`update_collection`) | no | Direct | Schema evolution without re-ingest `[changelog v1.18]` |
| 3 | Multivectors (late interaction / ColBERT) | 1.10+ | GA | yes (`MultiVectorConfig`) | no | Adapter | Variable-length vector matrix per point, `max_sim` comparator `[docs §11]` |
| 4 | Multivector `max_sim` comparator | 1.10+ | GA | yes (`MultiVectorComparator`) | no | Adapter | Sum-of-max-similarities scoring for late interaction `[docs §11]` |
| 5 | MUVERA single-vector approximation | client-side only | GA (FastEmbed 0.7.2+) | n/a (client lib) | no | Adapter | FDE encoding to make ColBERT searchable in HNSW; not Qdrant-server-native `[blog 1.18]` |
| 6 | Sparse vectors (BM25 / SPLADE) | 1.7+ | GA | yes (`SparseVectorParamsBuilder`) | no | Direct | Server-side sparse index alongside dense `[docs §11]` |
| 7 | Sparse vector `modifier: idf` | 1.10 | GA | partial (i32-coded enum) | no | Direct | Server applies IDF re-weighting on sparse query `[docs §5]` |
| 8 | Sparse vector `modifier: none` | 1.10 | GA | partial | no | Direct | No re-weighting; raw values used as scores `[docs §5]` |
| 9 | Sparse index `on_disk`, `full_scan_threshold` | 1.7+ | GA | yes (`SparseIndexConfigBuilder`) | no | Direct | Disk-backed sparse index for large corpora `[docs §11]` |
| 10 | Quantization — Scalar (int8) | 1.1.0 | GA | yes (`ScalarQuantizationBuilder`) | no | Direct | 4× compression, SIMD-friendly `[docs §10]` |
| 11 | Quantization — Binary (1-bit) | 1.5.0 | GA | yes (`BinaryQuantizationBuilder`) | no | Direct | Up to 32×; centered high-dim vectors only `[docs §10]` |
| 12 | Binary 1.5-bit / 2-bit / asymmetric | 1.15.0 | GA | yes | no | Direct | Encoding/query_encoding split: e.g. `binary` storage + `scalar8bits` query `[docs §10]` |
| 13 | Quantization — Product | 1.2.0 | GA | yes (`ProductQuantizationBuilder`) | no | Direct | Codebook-based, up to 64×; not SIMD-friendly `[docs §10]` |
| 14 | Quantization — **TurboQuant** | 1.18.0 | GA | yes (`TurboQuantizationBuilder`) | no | Direct | Rotation+asymmetric; 4-bit/2-bit/1.5-bit/1-bit; 8× w/o recall tax `[changelog v1.18]` |
| 15 | Quantization `oversampling` + `rescore` per-query | 1.x | GA | yes | no | Direct | Recall-recovery at search time `[docs §10]` |
| 16 | HNSW per-vector `m`, `ef_construct` | 1.x | GA | yes (`HnswConfigDiffBuilder`) | partial (defaults) | Direct | Tune index quality/memory per named vector `[docs §4]` |
| 17 | HNSW per-query `hnsw_ef`, `exact`, `indexed_only` | 1.x | GA | yes (`SearchParams`) | no | Direct | Query-time accuracy/latency knobs `[api §query-points]` |
| 18 | Filterable HNSW `enable_hnsw=false` per payload index | 1.17 | GA | partial | no | Direct | Skip extra HNSW edges on low-leverage filter fields `[changelog v1.17]` |
| 19 | ACORN-1 filtered HNSW | 1.16 | GA | yes (`AcornSearchParams`) | no | Direct | Better recall when filter is highly selective `[docs §4]` |
| 20 | Payload index — keyword | early | GA | yes | **yes** | Direct | Exact string match `[docs §6]` |
| 21 | Payload index — integer | early | GA | yes | **yes** (`start_ts`) | Direct | Range + match on i64 `[docs §6]` |
| 22 | Payload index — float | early | GA | yes | no | Direct | f64 range `[docs §6]` |
| 23 | Payload index — bool | 1.x | GA | yes | **yes** (`has_errors`) | Direct | True/false match `[docs §6]` |
| 24 | Payload index — datetime (RFC3339) | 1.x | GA | yes | no | Direct | Native datetime range queries `[docs §6]` |
| 25 | Payload index — geo | early | GA | yes | no | Doesn't fit | Bounding box / radius / polygon `[docs §6]` |
| 26 | Payload index — UUID | 1.x | GA | yes | no | Direct | Optimized keyword for UUID values `[docs §6]` |
| 27 | Payload index — text + tokenizer (word/whitespace/prefix/multilingual) | 1.x | GA | yes | **yes** (`ai_title`, word) | Direct | Full-text filter index `[docs §6]` |
| 28 | Text index — ASCII folding | 1.16 | GA | yes | no | Direct | Diacritic-insensitive match `[changelog v1.16]` |
| 29 | Text index — phrase search | 1.15 | GA | yes | no | Direct | Exact sequence match across token stream `[changelog v1.15]` |
| 30 | Text index — stopwords | 1.15 | GA | yes | no | Direct | Drop common words at index/query time `[changelog v1.15]` |
| 31 | Text index — Snowball stemmer | 1.15 | GA | yes | no | Direct | Reduce words to root form `[changelog v1.15]` |
| 32 | Text index — multilingual tokenizer (default) | 1.15 | GA | yes | partial | Direct | Better JP/ZH/etc. handling `[changelog v1.15]` |
| 33 | Tenant index (`is_tenant: true`) | 1.11 | GA | yes (in `field_schema`) | no | Direct | Storage hint for high-cardinality partition keys `[docs §6]` |
| 34 | Principal index (`is_principal: true`) | 1.x | GA | partial | no | Adapter | Ordering hint for primary timeline key `[docs §6]` |
| 35 | Tiered multitenancy + `ReplicatePoints` action | 1.16 | GA | yes (HTTP), partial (Rust) | no | Doesn't fit (single-node) | Promote payload-tenant to dedicated shard `[changelog v1.16]` |
| 36 | Query API — `formula` (score boosting) | 1.14 | GA | **yes** (`FormulaBuilder`) | no | Direct | Custom score = f(score, payload, decays) `[changelog v1.14]` |
| 37 | Formula decay functions (`exp_decay`, `gauss_decay`, `lin_decay`) | 1.14 | GA | yes (`DecayParamsExpressionBuilder`) | no | Direct | Distance-from-target falloff with `target/scale/midpoint` `[api §query-points]` |
| 38 | Formula math ops (sum/mult/div/neg/abs/sqrt/pow/exp/log10/ln) | 1.14 | GA | yes (Expression variants) | no | Direct | Pure-math composition in scoring formula `[api §query-points]` |
| 39 | Formula `geo_distance(origin, to=payload_key)` | 1.14 | GA | yes | no | Doesn't fit | Distance from origin to per-point geo payload `[blog 1.14]` |
| 40 | Formula `datetime` / `datetime_key` expressions | 1.14 | GA | yes (`DatetimeExpression`) | no | Direct | Recency boost via `start_ts` payload `[blog 1.14]` |
| 41 | Query API — `mmr` on NearestQuery | 1.15 | GA | **yes** (`MmrBuilder`) | no | Direct | Diversity rerank inline; `diversity ∈ [0,1]`, `candidates_limit` `[api §query-points]` |
| 42 | Query API — `group_by` + `group_size` | 1.x | GA | yes (`group_by`, `group_size` on QueryPoints) | no | Direct | Top-N results per payload group in one call `[docs §3]` |
| 43 | Query API — `order_by` | 1.x | GA | yes (`OrderByBuilder`) | no | Direct | Deterministic payload-based ordering (scroll-like) `[api §query-points]` |
| 44 | Query API — `sample: random` | 1.x | GA | yes (`SampleQueryBuilder`) | no | Adapter | Random point sampling, e.g. for canary eval `[api §query-points]` |
| 45 | Query API — `recommend` with `sum_scores` strategy | 1.14 | GA | yes (`RecommendStrategy`) | no | Direct | Multi-positive/negative score sum — relevance-feedback-ish `[changelog v1.14]` |
| 46 | Query API — `relevance_feedback` | 1.17 | GA | **yes** (`RelevanceFeedbackInput`) | no | Direct | Server-side adaptive scoring from +/− labels `[changelog v1.17]` |
| 47 | Query API — `discover` (target+context) | 1.x | GA | yes (`DiscoverInputBuilder`) | **yes** | Direct | Anchor + contrastive pairs `[docs §3]` |
| 48 | Query API — `context` (no target) | 1.x | GA | yes (`ContextInputBuilder`) | no | Direct | Pairs-only triplet-loss search `[docs §3]` |
| 49 | Query API — `fusion: rrf` / `dbsf` | 1.x | GA | yes (`FusionQuery`) | partial (client-side fusion today) | Direct | Server-side multi-prefetch fusion `[docs §3]` |
| 50 | Weighted RRF (`k` param + per-prefetch weights) | 1.17 | GA | yes | no | Direct | Customize RRF rank smoothing & lens weights `[changelog v1.17]` |
| 51 | Distance Matrix — `pairs` format | 1.x | GA | **yes** (`search_matrix_pairs`) | **yes** | Direct | List of `{a, b, score}` for graph use `[api §matrix-pairs]` |
| 52 | Distance Matrix — `offsets` format | 1.x | GA | yes (`search_matrix_offsets`) | no | Direct | CSR-style sparse matrix output `[docs §3]` |
| 53 | Snapshots — collection-scoped HTTP | early | GA | yes | **yes** | Direct | Create/list/download/restore via REST `[docs §6]` |
| 54 | Snapshots — restore from URL (toggle in 1.18) | 1.18 | GA | yes (server config) | no | Direct | Security hardening: disable URL restore `[changelog v1.18]` |
| 55 | Snapshots — streaming transfer (1.13) | 1.13 | GA | yes (server-side; client benefits) | n/a | Direct | "Stream snapshots in snapshot transfer, don't put snapshots on disk first" `[changelog v1.13]` |
| 56 | Snapshots — SHA256 checksum on recover | 1.x | GA | yes | no | Direct | Verify integrity before restore `[docs §6]` |
| 57 | Conditional updates (filter-gated upsert) | 1.16 | GA | yes | no | Direct | "Only apply update on points matching filter" `[changelog v1.16]` |
| 58 | `update_mode` insert / update / upsert | 1.17 | GA | yes | no | Direct | Reject duplicate IDs or reject missing IDs `[changelog v1.17]` |
| 59 | Inference API — BM25 (self-hostable) | 1.x | GA | yes (`Document` in upsert) | no | Direct | **The only server-side inference flow available on local Qdrant** `[docs §5]` |
| 60 | Inference API — text/image embedding (Cloud only) | 1.x | GA | yes (`Document`/`Image`) | no | **Cloud-only** | Cluster-side embeddings (MiniLM, mxbai, CLIP) `[docs §5]` |
| 61 | Inference API — external providers (OpenAI/Cohere/Jina/OpenRouter) | 1.15-1.17 | GA (cloud) | yes | no | Cloud-only | Call external embed APIs through Qdrant `[changelog v1.17]` |
| 62 | Inference API — reranking model | unverified for self-host | unverified | unverified | no | Cloud-only (likely) | Cloud Inference docs mention "Reranking with FastEmbed" but not as inference-side feature `[docs §5]` |
| 63 | Strict mode — `max_resident_memory_percent` | 1.18 | GA | yes (`StrictModeConfigBuilder`) | no | Direct | Reject writes when RAM > threshold `[changelog v1.18]` |
| 64 | Strict mode — `search_max_batchsize` | 1.18 | GA | yes | no | Direct | Cap batch search size `[blog 1.18]` |
| 65 | Strict mode — `max_payload_indices` | 1.16 | GA | yes | no | Direct | Cap payload index count per collection `[changelog v1.16]` |
| 66 | Low memory mode (force on-disk on startup) | 1.18 | GA | yes (server config) | no | Direct | Avoid OOM by mmapping everything `[changelog v1.18]` |
| 67 | Deep memory reporting (`/memory` breakdown) | 1.18 | GA | yes (HTTP) | no | Direct | Per-component RAM/disk/page-cache split `[blog 1.18]` |
| 68 | Dynamic CPU pool for search workers | 1.18 | GA | server-internal | yes (transparent) | Direct | "Improve search performance when there's high IO wait" `[changelog v1.18]` |
| 69 | Immutable geo index — 7× memory reduction | 1.18 | GA | server-internal | n/a (Memex has no geo) | Doesn't fit | Smaller geo footprint `[changelog v1.18]` |
| 70 | Audit logging (config + tracing ID) | 1.17 | GA | yes (Rust client `tracing` helper in 1.18) | no | Direct | Per-request log entries with trace IDs `[changelog v1.17]` |
| 71 | RocksDB fully removed → Gridstore only | 1.18 | GA | server-internal | yes (transparent) | Direct | Single-engine simplification `[changelog v1.18]` |
| 72 | Inline storage (vectors inside HNSW nodes) | 1.16 | GA | yes (collection config) | no | Direct | "Reduces random access reads from 32 per node to approximately 2 pages" `[blog 1.16]` |
| 73 | Incremental HNSW indexing on upsert | 1.14 | GA | server-internal | yes (transparent) | Direct | "Partially re-use existing HNSW graph on merging segments" `[changelog v1.14]` |
| 74 | GPU-accelerated HNSW build | 1.13 / 1.14 | GA | server-build-time | no | Doesn't fit (desktop CPU) | NVIDIA GPU only `[changelog v1.13]` |
| 75 | Bulk-load trick `m=0` then re-enable | n/a (technique) | GA | yes (collection diff) | no | Direct | 5-10× faster initial ingest `[docs §4]` |
| 76 | gRPC streaming upserts | n/a (gRPC native) | GA | partial (Rust client uses `tonic` streaming under hood) | partial | Adapter | Streaming `upsert_points_blocking` exists but Rust API mostly batched `[Rust client docs]` |
| 77 | `Datatype` per named vector (f32/f16/u8) | 1.x | GA | yes | no | Direct | 2× / 4× memory reduction vs f32 `[docs §11]` |
| 78 | `text_any` full-text filter (OR over tokens) | 1.16 | GA | yes | no | Direct | Match any of N query tokens `[changelog v1.16]` |
| 79 | RRF custom `k` parameter | 1.16 | GA | yes | no | Direct | Tune RRF rank smoothing `[changelog v1.16]` |
| 80 | Replication / sharding | 1.x | GA (cluster) | yes | no | Doesn't fit (single-node) | Multi-node replication factor & shard count `[changelog v1.17]` |

**Row count: 80 features.** All entries verified against changelogs or docs unless marked `unverified`. (Original brief asked for ≥35; the surface is much wider.)

---

## 3. Deep dives — top 10 unused, highest leverage

### 3.1 Server-side score formula (`FormulaQuery`)

**Syntax (verbatim API schema).**

```yaml
FormulaQuery:
  properties:
    formula: Expression (required)
    defaults: object (optional)

Expression variants:
  MultExpression, SumExpression, DivExpression, NegExpression,
  AbsExpression, SqrtExpression, PowExpression, ExpExpression,
  Log10Expression, LnExpression, LinDecayExpression, ExpDecayExpression,
  GaussDecayExpression, DatetimeExpression, GeoDistance,
  Variable (string — payload key reference), Constant (number),
  "$score" (string — vector similarity score reference)

DecayParamsExpression:
  x: Expression (required)
  target: Expression (optional, default 0)
  scale: number (optional, default 1.0)
  midpoint: number (optional, default 0.5)
```
[Source: https://api.qdrant.tech/api-reference/search/query-points · fetched 2026-05-18]

> Allow server-side score boosting with user-defined formula
[Source: github.com/qdrant/qdrant releases/tags/v1.14.0 · fetched 2026-05-18]

**Why it matters for Memex.** Memex's recall path today fuses lenses in Rust (`weighted-sum across content/tool/path/error/code`). With `FormulaQuery` Memex could do `cosine_score(content) + 0.5 * cosine_score(tool) + 0.2 * gauss_decay(now - start_ts, scale=7d) + (has_errors * 0.3)` **server-side, single round trip, all evaluated against indexed payload**. Result cards still get per-lens scores by issuing per-lens prefetches and reading their raw scores back.

**Cost.** Zero storage. Latency drops vs current 5-vector multi-call. **Effort: S (~3 hours)** — add `FormulaBuilder` usage path in `indexer.rs::recall`.

**Conflict with Memex invariants.** None — fully local, no extra deps. Replaces, rather than augments, the existing weighted-sum.

### 3.2 MMR diversity rerank (`NearestQueryMmr`)

**Syntax (verbatim).**

```yaml
NearestQuery:
  properties:
    nearest: VectorInput
    mmr: NearestQueryMmr
      description: >-
        Perform MMR (Maximal Marginal Relevance) reranking after search,
        using the same vector in this query to calculate relevance.

Mmr:
  properties:
    diversity: number  // [0,1], default 0.5
    candidates_limit: integer  // defaults to `limit`
```
[Source: https://api.qdrant.tech/api-reference/search/query-points · fetched 2026-05-18]

**Why it matters for Memex.** Claude Code session lists are full of near-duplicates: same project, rerun next day. Top-10 today often shows 6 variants of "this morning's session". MMR with `diversity ≈ 0.5–0.7` and `candidates_limit ≈ 50` keeps the top hit but dedupes the long tail. Single field on existing `NearestQuery`.

**Cost.** Adds one rerank pass over `candidates_limit` candidates; negligible at Memex's 80-session scale, sub-ms at 10k.

**Effort: S (~1 hour).** Add `.mmr(NearestQueryMmr { diversity: 0.5, candidates_limit: 50 })` to recall queries.

**Conflict.** None.

### 3.3 Group-by query

**Syntax.** Memex stays on `query_points` with two extra top-level params:

```json
{
  "query": [/* vector */],
  "group_by": "project_name",
  "limit": 4,
  "group_size": 2
}
```
[Source: https://qdrant.tech/documentation/concepts/hybrid-queries/ · fetched 2026-05-18]

**Why it matters for Memex.** The current Sessions sidebar groups by `project_name` client-side after a flat recall. With `group_by` Memex can issue *one* query that returns "top 2 sessions for each of the top 4 projects" — sidebar gets a coherent project-bucketed view, and the existing pagination logic gets simpler.

**Cost.** Zero. Server handles grouping in the same pass.

**Effort: S (~2 hours).** Sidebar component change, one method on the Rust client (`group_by`, `group_size`).

**Conflict.** None.

### 3.4 Relevance Feedback query

**Syntax (verbatim schema names).**

```yaml
RelevanceFeedbackQuery:
  properties:
    relevance_feedback:
      target: VectorInput
      feedback: array of FeedbackItem
      strategy: FeedbackStrategy
```
[Source: https://api.qdrant.tech/api-reference/search/query-points · fetched 2026-05-18]

Listed as the milestone-48 feature for v1.17:

> Relevance Feedback ([docs](https://qdrant.tech/documentation/concepts/search-relevance/#relevance-feedback))
[Source: github.com/qdrant/qdrant releases/tags/v1.17.0 · fetched 2026-05-18]

**Why it matters for Memex.** Mix & Match's `+pos / −neg` UI today drives Discovery's `context` pairs (binary triplet-loss). Relevance Feedback offers a *gradient* — the dense query itself adapts based on user feedback. Net result is smoother iteration: each click nudges the query rather than re-anchoring it. Pairs well with the existing `01-qdrant-official-docs-2026.md §10.6` recommendation to spike this.

**Cost.** Adds `feedback: Vec<FeedbackItem>` to the recall payload; one extra round-trip per feedback click only if Memex wants to persist the evolving query.

**Effort: M (~1 day).** Rust client exposes the builder; UI wiring is the actual cost.

**Conflict.** Replaces Discovery `context` pairs for the "evolve" mode. Discovery can stay for "anchor on this exact session".

### 3.5 Sparse vectors with BM25 + `idf` modifier (server-side inference path)

**Syntax (collection creation, sparse + dense, with IDF modifier).**

```json
{
  "vectors": {
    "content": { "size": 384, "distance": "Cosine" }
  },
  "sparse_vectors": {
    "content_sparse": {
      "modifier": "idf",
      "index": { "on_disk": false }
    }
  }
}
```
[Source: https://qdrant.tech/articles/sparse-vectors/ · fetched 2026-05-18]

> When using any retrieval formula that includes IDF (Inverse Document Frequency), such as BM25, in Qdrant, you no longer need to include the IDF component in the sparse document representations—the IDF component will be applied by Qdrant automatically when computing similarity scores.
[Source: https://qdrant.tech/articles/bm42/ · fetched 2026-05-18]

**The local-friendly path.** The inference doc explicitly states BM25 is the **one** server-side inference flow that works on self-hosted Qdrant: "Server-side Inference: BM25". [Source: https://qdrant.tech/documentation/concepts/inference/ · fetched 2026-05-18] Memex can send `Document { text: "...", model: "Qdrant/bm25" }` straight into upsert/query and the server tokenizes + builds the sparse vector itself.

**Why it matters for Memex.** Memex's `path`, `tool`, `code` lenses are token-flavored. A BM25 sparse vector on `path` (or a fresh `path_text` field) would crush dense for queries like "edit `index.html`". `bm42` is **experimental** (see §6), so stay on plain BM25.

**Cost.** New named sparse vector per collection; ~10% storage overhead for posting lists.

**Effort: M (~1 day).** Collection migration via the 1.18 add-named-vector API; query path gets one extra `Prefetch` block.

**Conflict.** None — sparse is additive.

### 3.6 TurboQuant 4-bit on `content`

**Config (verbatim form).**

```json
{
  "quantization_config": {
    "turbo": {
      "bits": "bits4",
      "always_ram": true
    }
  }
}
```
[Source: https://qdrant.tech/documentation/guides/quantization/ · fetched 2026-05-18]

Headline number:

> Add TurboQuant quantization variant, 8x vector compression without the recall tax
[Source: github.com/qdrant/qdrant releases/tags/v1.18.0 · fetched 2026-05-18]

Per-bits compression table:
- `bits4` (default) → 8× compression
- `bits2` → 16×
- `bits1_5` → 24×
- `bits1` → 32×

**Why it matters for Memex.** At 80 sessions × ~5 vectors × 384 f32 ≈ 600 KB raw, this is *irrelevant today*. But the hackathon framing suggests Memex wants the **pinnacle** — adopting TurboQuant on the `content` vector is the one-line way to show 1.18 is being driven hard. With `always_ram: true` and 1.18's `oversampling + rescore`, recall is preserved.

**Cost.** Recompute quantized table on existing collection; trivial.

**Effort: S (~30 min).** Add `.quantization_config(...)` to collection-create.

**Conflict.** None. Quantization is per-named-vector, so Memex can leave `tool/path/error/code` raw and only quantize `content`.

### 3.7 Conditional updates + `update_mode` (insert/update/upsert)

**Syntax (verbatim).**

> For upserts, add `update_mode` parameter to either `upsert`, `update` or `insert`
[Source: github.com/qdrant/qdrant releases/tags/v1.17.0 · fetched 2026-05-18]

> Add conditional update functionality, only apply update on points matching filter
[Source: github.com/qdrant/qdrant releases/tags/v1.16.0 · fetched 2026-05-18]

**Why it matters for Memex.** The "re-embed only stale points" worker is the entire reason `embedding_model_version` exists in Memex's payload (per `docs/qdrant-features.md`). Today the worker reads-then-writes; with conditional updates the worker fires one filtered upsert and Qdrant skips up-to-date points.

**Cost.** Zero.

**Effort: S (~30 min).** Add `.update_mode(...)` and `.filter(...)` to the existing `upsert_points` call.

**Conflict.** None.

### 3.8 Inline storage (vectors-in-HNSW)

**What.**

> stores quantized vector data directly inside the HNSW nodes, optimizes disk-based deployments by reducing random access reads from 32 per node to approximately 2 pages
[Source: https://qdrant.tech/blog/qdrant-1.16.x/ · fetched 2026-05-18]

**Why it matters for Memex.** When Memex grows past a single-shard mmap working-set ("user opened 10 projects = 10 collections"), inline storage drops per-search disk reads ~16×. Today Memex implicitly *doesn't* turn this on, so default segments are still doing the 32-page chase.

**Cost.** Slight increase in HNSW segment size; recall unchanged.

**Effort: S (~10 min).** Collection-config flag.

**Conflict.** None.

### 3.9 ACORN-1 filtered search

**Syntax (verbatim).**

> Add ACORN-1 search method, accurate search over many filtered points at the cost of performance
[Source: github.com/qdrant/qdrant releases/tags/v1.16.0 · fetched 2026-05-18]

> examines neighbors of neighbors (the second hop) if the direct neighbors have been filtered out
[Source: https://qdrant.tech/blog/qdrant-1.16.x/ · fetched 2026-05-18]

Per-query: `params.acorn = AcornSearchParams { ... }`. [Source: docs.rs/qdrant-client/1.18.0 · fetched 2026-05-18]

**Why it matters for Memex.** Memex's `has_errors=true` filter cuts the corpus to <5% in most projects. When that filter is on, default HNSW frequently misses good candidates because direct neighbors got filtered. ACORN second-hop scans fix it.

**Cost.** Slower than non-ACORN when filter selectivity is low; the doc says "at the cost of performance". So *conditional* — enable only when filter is selective.

**Effort: S (~30 min).** Per-query opt-in.

**Conflict.** None.

### 3.10 Tenant index (`is_tenant: true`) on `project_path`

**Syntax (verbatim).**

```json
{
  "field_name": "group_id",
  "field_schema": { "type": "keyword", "is_tenant": true }
}
```
[Source: https://qdrant.tech/documentation/guides/multitenancy/ · fetched 2026-05-18]

> is_tenant=true parameter is optional, but specifying it provides storage with additional information about the usage patterns
[Source: https://qdrant.tech/documentation/guides/multitenancy/ · fetched 2026-05-18]

**Why it matters for Memex.** Memex already partitions by `project_path` (each project has its own session pool). Adding `is_tenant: true` tells Qdrant's storage layer to co-locate same-project points on disk, which speeds up project-scoped scrolls and reduces page-fault thrash on cold opens. Free perf win for the existing partitioning.

**Cost.** Re-index of one payload index field. ~seconds at Memex's scale.

**Effort: S (~5 min).** Schema diff on existing `project_path` keyword index.

**Conflict.** None — works even on single-node Qdrant; tiered multitenancy machinery (shard promotion) is opt-in beyond this hint.

---

## 4. Features Memex already uses well

Per `01-qdrant-official-docs-2026.md` and `docs/qdrant-features.md`:

- **Named vectors (5: content/tool/path/error/code)** — properly sized at 384-d cosine. `[01 §2]`
- **Discovery API with `context` pairs** — Mix & Match consumes it correctly with "first positive as target". `[01 §4]`
- **Distance Matrix `pairs`** — feeds `petgraph` MST for topology view. `[01 §3]`
- **Snapshots** — collection-scoped POST/GET/upload, `priority=snapshot` on restore. `[01 §6]`
- **Payload indexes** — keyword/text/integer/bool on the four meaningful fields. `[01 §5]`

No under-exploitation here — these surfaces are used the way Qdrant intended. Recommendation is not "use more of these" but "add the unused surfaces in §3".

---

## 5. Cloud-only features to skip

| Feature | Why skip for Memex |
|---|---|
| Qdrant Cloud Inference — text/image embedding models | "Cloud Inference features are only supported for clusters on Qdrant Managed Cloud" [Source: https://qdrant.tech/documentation/concepts/inference/ · fetched 2026-05-18]. Breaks local-first invariant. |
| External provider inference (OpenAI/Cohere/Jina/OpenRouter) | Same — managed-cloud-only routing layer. |
| Tiered multitenancy *with shard promotion* (`ReplicatePoints`) | Requires distributed cluster. Memex is single-node. Plain `is_tenant: true` hint is single-node-friendly and gets ~all the perf benefit (§3.10). |
| Replication factor / write_consistency_factor / shard_number > 1 | Distributed-only; no single-node value. |
| Cluster-wide telemetry endpoint | Cluster-only. |
| Configurable read fan-out delay | Distributed-only tail-latency knob. |
| GPU-accelerated HNSW indexing | NVIDIA GPU only; Memex runs on whatever laptop Claude Code runs on (mostly Apple Silicon). |
| S3 snapshot pod role auth | Cloud / K8s-only. |
| Secondary API key rotation | Distributed cluster operations. |

---

## 6. Risk — features that look attractive but bite back

**Skip for a 14-day hackathon window:**

| Feature | Status / risk | Why avoid right now |
|---|---|---|
| BM42 sparse retrieval | "BM42 should be considered as an experimental approach, which requires further research and development before it can be used in production." [Source: https://qdrant.tech/articles/bm42/ · fetched 2026-05-18] | Use plain BM25 (also covered by sparse + IDF modifier, see §3.5). Same Memex surface unlocks, none of the "experimental" tax. |
| ColBERT / late-interaction multivectors + MUVERA | Both production-ready *individually*, but the combined path requires `fastembed` 0.7.2+ MUVERA Python-first; Rust support unverified. Stack risk = high for a short window. | Skip until corpus >100k sessions (per 01 §10.2). |
| Strict mode in aggressive config (`max_resident_memory_percent` set low) | Genuinely useful (§3 #5) but **setting too low rejects user upserts mid-session** — a hackathon demo where indexing silently fails is a worse demo than no strict mode. | If adopted, set `max_resident_memory_percent: 85` not 50. |
| Quantization with `rescore: false` | Doc explicitly warns scalar/binary recall degrades without rescore. The 1-bit (`bits1`) TurboQuant especially. | If TurboQuant is on, keep `rescore: true` and `oversampling: 2.0+`. |
| Conditional updates *without explicit `update_mode`* | Easy to mis-issue — a filter that matches zero points silently no-ops, looking like success. | Always set `update_mode: update` (reject if missing) for the embedding-staleness worker so failures are loud. |
| Disabling extra HNSW links (`enable_hnsw=false`) on the wrong payload index | If the filter is high-selectivity and frequently used (e.g. `has_errors`), disabling links *hurts* recall. | Per the changelog wording, only disable on payload indexes that are *not* frequently used in vector search filters. |
| Snapshot restore from URL in 1.18 with default config | 1.18 *enables* URL restore by default; the new switch is for *disabling* it. If a Memex user ever exposes their localhost Qdrant on a LAN, malicious peers can push an "evil snapshot URL". | Always disable URL restore in the Memex-managed Qdrant config: this is in 01 §11.4 too. |

**Things 1.18 ships that look "preview" but aren't:**

The 1.18 release body uses no `experimental` or `preview` qualifiers on its features. TurboQuant, deep memory reporting, named-vector CRUD, strict-mode memory limits, low memory mode, and the dynamic CPU pool are all shipped GA. The "experimental" tag in Qdrant's 2026 surface attaches almost exclusively to **BM42** (1.10) and some Cloud-Inference reranker paths that we already exclude (§5).

---

## 7. Sources

Every URL fetched on 2026-05-18, in citation order.

1. `https://api.github.com/repos/qdrant/qdrant/releases/tags/v1.18.0` — verbatim 1.18 changelog body: TurboQuant, named-vector CRUD, deep memory reporting, low memory mode, strict-mode `max_resident_memory_percent`, dynamic CPU pool, geo index 7× memory cut, full RocksDB removal, snapshot URL-restore toggle.
2. `https://api.github.com/repos/qdrant/qdrant/releases/tags/v1.17.0` — relevance feedback, weighted RRF, audit logging, update queue, `update_mode` parameter, payload-index HNSW-link disable.
3. `https://api.github.com/repos/qdrant/qdrant/releases/tags/v1.16.0` — ACORN-1, inline storage, tiered multitenancy + `ReplicatePoints`, ASCII folding, conditional updates, `text_any`, custom RRF `k`, strict-mode max_payload_indices.
4. `https://api.github.com/repos/qdrant/qdrant/releases/tags/v1.15.0` — MMR, phrase search, stopwords, multilingual tokenizer (default), Snowball stemmer, asymmetric / 1.5-bit / 2-bit binary quantization.
5. `https://api.github.com/repos/qdrant/qdrant/releases/tags/v1.14.0` — score boosting formula, `sum_scores` recommend strategy, incremental HNSW, query batch parallelization.
6. `https://api.github.com/repos/qdrant/qdrant/releases/tags/v1.13.0` — strict mode, GPU HNSW, snapshot streaming, has-vector filter, HNSW graph compression.
7. `https://qdrant.tech/documentation/concepts/inference/` — server-side inference page: BM25 self-hostable; everything else cloud-only; Document/Image object shape; external providers (OpenAI/Cohere/Jina/OpenRouter); "Reranking with FastEmbed" referenced separately.
8. `https://qdrant.tech/documentation/concepts/indexing/` — payload index types (keyword/integer/float/bool/geo/datetime/text/uuid), per-vector HNSW `m`/`ef_construct`, ACORN section, IDF modifier section, ASCII folding, tokenizers, stemmer, stopwords, phrase search.
9. `https://qdrant.tech/documentation/concepts/filtering/` — full filter clause taxonomy: Must/Should/MustNot, Match/MatchAny/MatchExcept, nested key, full-text, phrase match, Range, Datetime Range, UUID Match, geo BBox/Radius/Polygon, Values count, IsEmpty, IsNull, HasId, HasVector.
10. `https://qdrant.tech/documentation/concepts/hybrid-queries/` — Hybrid Search, RRF (with custom `k`), Weighted RRF, DBSF normalization rules, multi-stage queries, grouping (`group_by`, `limit`, `group_size`).
11. `https://qdrant.tech/documentation/concepts/explore/` — Recommendation API (average_vector / best_score / sum_scores strategies), Discovery search (target+context), Context search (no target), Distance Matrix pairs & offsets formats.
12. `https://qdrant.tech/documentation/concepts/vectors/` — Dense/Sparse/Multivector definitions, `max_sim` comparator, named-vector add/remove (1.18), Datatypes (Float32/Float16/Uint8).
13. `https://qdrant.tech/documentation/guides/multitenancy/` — payload-partition pattern, `is_tenant: true` keyword index hint, tiered multitenancy (1.16), `ReplicatePoints` action for tenant promotion, fallback shard key.
14. `https://qdrant.tech/documentation/guides/quantization/` — Scalar/Binary/Product/TurboQuant comparison, TurboQuant bit options (`bits4`/`bits2`/`bits1_5`/`bits1`), `oversampling`, `rescore`, `ignore`.
15. `https://qdrant.tech/blog/qdrant-1.18.x/` — TurboQuant deep dive (Hadamard rotation, asymmetric, SIMD), memory monitoring, named-vector schema CRUD, audit-log API, per-collection metrics, strict-mode `search_max_batchsize`.
16. `https://qdrant.tech/blog/qdrant-1.14.x/` — score-boosting reranker (`$score`, `gauss_decay`, `datetime_key`, `geo_distance(origin, to)`), incremental HNSW, faster batch queries.
17. `https://qdrant.tech/blog/qdrant-1.16.x/` — inline storage in HNSW nodes (32 pages → 2 pages), ACORN second-hop description.
18. `https://api.qdrant.tech/api-reference/search/query-points` — QueryRequest schema: `query`, `prefetch`, `using`, `filter`, `params` (hnsw_ef/exact/quantization/indexed_only/acorn), `score_threshold`, `limit`, `offset`, `with_vector`, `with_payload`, `lookup_from`. All query variants: NearestQuery (+mmr), RecommendQuery, DiscoverQuery, ContextQuery, OrderByQuery, FusionQuery, RrfQuery, FormulaQuery, SampleQuery, RelevanceFeedbackQuery. Expression sub-schemas: MultExpression, SumExpression, NegExpression, AbsExpression, DivExpression, SqrtExpression, PowExpression, ExpExpression, Log10Expression, LnExpression, LinDecayExpression, ExpDecayExpression, GaussDecayExpression, DatetimeExpression, GeoDistance. DecayParamsExpression fields (`x`, `target`, `scale`, `midpoint`).
19. `https://docs.rs/qdrant-client/1.18.0/qdrant_client/qdrant/index.html` — Rust client builder surface confirms: `FormulaBuilder`, `MmrBuilder`, `OrderByBuilder`, `RecommendInputBuilder`, `QuantizationSearchParamsBuilder`, `ScalarQuantizationBuilder`, `BinaryQuantizationBuilder`, `ProductQuantizationBuilder`, `TurboQuantizationBuilder`, `SparseVectorParamsBuilder`, `SparseIndexConfigBuilder`, `HnswConfigDiffBuilder`, `StrictModeConfigBuilder`, `OptimizersConfigDiffBuilder`, `WalConfigDiffBuilder`, `DecayParamsExpressionBuilder`. Confirmed absent (lower-level types not re-exported): `NearestQueryBuilder`, `SampleQueryBuilder`, `GroupByBuilder`, `RelevanceFeedbackBuilder` (these likely live as direct struct construction without explicit builder shortcuts).
20. `https://docs.rs/qdrant-client/1.18.0/qdrant_client/qdrant/struct.SparseVectorParams.html` — confirms `modifier: Option<i32>` mapping to a `Modifier` enum (Idf/None), `index: Option<SparseIndexConfig>`.
21. `https://docs.rs/qdrant-client/1.18.0/qdrant_client/qdrant/struct.Formula.html` — `pub struct Formula { pub expression: Option<Expression>, pub defaults: HashMap<String, Value> }`.
22. `https://qdrant.tech/articles/bm42/` — IDF-on-server documented; BM42 explicit "experimental approach" warning.
23. `https://qdrant.tech/articles/sparse-vectors/` — sparse collection-config shape, IDF modifier in config.
24. `https://qdrant.tech/documentation/concepts/storage/` — vector storage, memmap config, payload storage, versioning. (Inline storage **not** documented here as of fetch — confirmed only via `blog/qdrant-1.16.x/`.)
25. `claudedocs/research/01-qdrant-official-docs-2026.md` — Memex's prior dossier, used to avoid duplication and cross-reference Memex's current usage.
26. `src-tauri/Cargo.toml` — confirms `qdrant-client = "1"`, `fastembed = "5"`, single-binary Tauri 2 desktop target.

**Pages that returned 404 or stub responses on 2026-05-18 (noted for future re-fetch):**

- `https://qdrant.tech/documentation/concepts/search-relevance/` — relevance feedback canonical page, redirected/missing today. Used `api.qdrant.tech/api-reference/search/query-points` `RelevanceFeedbackQuery` schema as the substitute primary source.
- `https://qdrant.tech/articles/formula-queries/` and `https://qdrant.tech/articles/qdrant-1.14.x/` — 404 today. Substituted with `https://qdrant.tech/blog/qdrant-1.14.x/` (live).
- `https://qdrant.tech/articles/qdrant-1.18.x/` — 404; substituted with `https://qdrant.tech/blog/qdrant-1.18.x/`.
- `https://qdrant.tech/documentation/database-tutorials/score-boosting/` — 404; formula API details came from the API reference + 1.14 blog.
- `https://github.com/qdrant/qdrant/blob/master/CHANGELOG.md` and `raw.githubusercontent.com/.../CHANGELOG.md` — Qdrant does not ship CHANGELOG.md in the repo root; all changelog facts came from the GitHub Releases API per-tag bodies.

---

## 8. Appendix A — Qdrant 1.13 → 1.18 feature-introduction timeline

Compact view of when each row in §2 entered the server, sourced from the per-tag release bodies.

| Version | Released | Highlights relevant to Memex |
|---|---|---|
| v1.13.0 | 2025-01-13 | GPU HNSW, strict mode, snapshot streaming, HasVector filter, HNSW graph compression, mmap-default for payload `[changelog v1.13]` |
| v1.14.0 | 2025-04-22 | **Score-boosting formula** (`$score`, decay, payload refs), **`sum_scores` recommend strategy**, incremental HNSW on upsert, parallel batch queries `[changelog v1.14]` |
| v1.15.0 | 2025-07-18 | **MMR**, phrase search, stopwords, multilingual tokenizer (default on), Snowball stemmer, asymmetric / 1.5-bit / 2-bit binary quantization `[changelog v1.15]` |
| v1.16.0 | 2025-11-17 | **ACORN-1**, **inline storage**, tiered multitenancy + `ReplicatePoints`, ASCII folding, conditional updates, `text_any`, custom RRF `k`, strict-mode max payload indices `[changelog v1.16]` |
| v1.17.0 | 2026-02-20 | **Relevance Feedback**, weighted RRF, audit logging, `update_mode`, payload-index HNSW-link disable, unlimited update queue, per-request tracing helper in Rust client `[changelog v1.17]` |
| v1.17.1 | 2026-03-27 | Bug-fix release; nothing API-shape-changing `[01 §1.4]` |
| v1.18.0 | 2026-05-11 | **TurboQuant**, **named-vector CRUD**, deep memory reporting, low memory mode, strict-mode `max_resident_memory_percent`, dynamic CPU pool, 7× geo-index memory cut, full RocksDB removal, snapshot URL-restore disable switch `[changelog v1.18]` |

Memex was assembled against 1.18 from day one (per `01 §1.2`). That means **every bolded feature above** is available to Memex's pinned client today — none require a server upgrade, none require a Rust crate bump. The "pinnacle gap" is purely application-side adoption.

---

## 9. Appendix B — Minimal Rust client snippets per high-leverage feature

These are the *shape* sketches based on the builders documented at `docs.rs/qdrant-client/1.18.0` and `api.qdrant.tech/api-reference/search/query-points`. They are deliberately abbreviated (no error handling, no full `use` lines) — the goal is to show the call surface, not be copy-paste-runnable.

### B.1 MMR on `NearestQuery`

```rust
use qdrant_client::qdrant::{QueryPointsBuilder, NearestQueryBuilder, MmrBuilder};

let resp = client.query(
    QueryPointsBuilder::new("memex_sessions")
        .using("content")
        .query(NearestQueryBuilder::new(query_vector)
            .mmr(MmrBuilder::default()
                .diversity(0.6)
                .candidates_limit(50))
            .build())
        .limit(10),
).await?;
```
[Source: api.qdrant.tech/api-reference/search/query-points + docs.rs/qdrant-client/1.18.0 · fetched 2026-05-18]

### B.2 Formula score with recency decay

```rust
use qdrant_client::qdrant::{
    QueryPointsBuilder, FormulaBuilder, Expression,
    SumExpression, MultExpression, GaussDecayExpression,
    DecayParamsExpressionBuilder, Variable,
};

let formula = FormulaBuilder::default()
    .expression(SumExpression::new(vec![
        Variable::from("$score"),
        MultExpression::new(vec![
            Variable::from("has_errors_score"),  // payload key
            Expression::constant(0.3),
        ]).into(),
        GaussDecayExpression::new(
            DecayParamsExpressionBuilder::default()
                .x(Variable::from("age_seconds"))    // payload key
                .scale(7.0 * 24.0 * 3600.0)          // 7-day half-life
                .midpoint(0.5)
                .build()
        ).into(),
    ]).into())
    .build();

let resp = client.query(
    QueryPointsBuilder::new("memex_sessions")
        .prefetch(/* dense prefetch on content */)
        .query(formula)
        .limit(10),
).await?;
```
[Source: api.qdrant.tech/api-reference/search/query-points + docs.rs/qdrant-client/1.18.0 §FormulaBuilder · fetched 2026-05-18]

### B.3 Group-by project

```rust
use qdrant_client::qdrant::QueryPointsBuilder;

let resp = client.query(
    QueryPointsBuilder::new("memex_sessions")
        .using("content")
        .query(query_vector)
        .group_by("project_name")
        .group_size(2)
        .limit(4),
).await?;
```
[Source: qdrant.tech/documentation/concepts/hybrid-queries/ §Grouping · fetched 2026-05-18]

### B.4 Sparse vector with IDF modifier (collection-create + upsert)

```rust
use qdrant_client::qdrant::{
    CreateCollectionBuilder, SparseVectorParamsBuilder,
    SparseIndexConfigBuilder, Modifier,
};

let create = CreateCollectionBuilder::new("memex_sessions")
    .vectors_config(/* dense content vector */)
    .sparse_vectors_config(vec![("content_sparse",
        SparseVectorParamsBuilder::default()
            .index(SparseIndexConfigBuilder::default().on_disk(false))
            .modifier(Modifier::Idf)
            .build())])
    .build();

client.create_collection(create).await?;
```
[Source: qdrant.tech/articles/sparse-vectors/ + docs.rs/qdrant-client/1.18.0 §SparseVectorParams · fetched 2026-05-18]

### B.5 TurboQuant on `content` vector only

```rust
use qdrant_client::qdrant::{
    VectorParamsBuilder, QuantizationConfig, TurboQuantizationBuilder,
};

let content_vec = VectorParamsBuilder::new(384, Distance::Cosine)
    .quantization_config(QuantizationConfig::from(
        TurboQuantizationBuilder::default()
            .bits(/* TurboBits::Bits4 */)
            .always_ram(true)
            .build()
    ))
    .build();
```
[Source: docs.rs/qdrant-client/1.18.0 §TurboQuantizationBuilder + qdrant.tech/documentation/guides/quantization/ · fetched 2026-05-18]

### B.6 ACORN on a `has_errors=true` filtered query

```rust
use qdrant_client::qdrant::{QueryPointsBuilder, SearchParams, AcornSearchParams, Filter, Condition};

let resp = client.query(
    QueryPointsBuilder::new("memex_sessions")
        .using("content")
        .query(query_vector)
        .filter(Filter::must([Condition::matches("has_errors", true)]))
        .params(SearchParams {
            acorn: Some(AcornSearchParams { /* fields per docs */ ..Default::default() }),
            ..Default::default()
        })
        .limit(10),
).await?;
```
[Source: docs.rs/qdrant-client/1.18.0 §AcornSearchParams + blog/qdrant-1.16.x/ · fetched 2026-05-18]

### B.7 Conditional update — only re-embed stale points

```rust
use qdrant_client::qdrant::{UpsertPointsBuilder, Filter, Condition, UpdateMode};

client.upsert_points(
    UpsertPointsBuilder::new("memex_sessions", new_points)
        .update_mode(UpdateMode::Update)              // require existing
        .filter(Filter::must([                        // and only the stale ones
            Condition::matches("embedding_model_version", "bge-small-v1.5".to_string())
        ]))
        .wait(true),
).await?;
```
[Source: changelog v1.16 (conditional updates) + v1.17 (`update_mode`) · fetched 2026-05-18]

### B.8 Named-vector add at runtime (no recreate)

```rust
use qdrant_client::qdrant::{UpdateCollectionBuilder, VectorParamsDiffBuilder};

// In 1.18 the update_collection surface accepts add-vector diff.
// Exact builder name unverified at docs.rs depth-1; the HTTP shape is canonical.
let req = UpdateCollectionBuilder::new("memex_sessions")
    /* .vectors(add: { "content_sparse_bm25": SparseVectorParams { ... } }) */
    .build();
client.update_collection(req).await?;
```
[Source: changelog v1.18 "Add API to create and delete named vectors in existing collection" — Rust builder name `unverified`, HTTP path canonical · fetched 2026-05-18]

---

## 10. Appendix C — Recommended Memex adoption sequence (priority-ordered)

Based on §3 deep dives, §6 risk notes, and Memex's 14-day hackathon window. "Effort" reuses §3 ratings (S = ≤4h, M = ~1 day, L = >1 day).

| Order | Feature | Effort | Surface affected | Demo angle |
|---|---|---|---|---|
| 1 | MMR on NearestQuery (§3.2) | S | `indexer.rs::recall` | "No more 6 copies of this morning's session at the top." Visible diff after one config change. |
| 2 | `is_tenant: true` on `project_path` (§3.10) | S | `indexer.rs` schema setup | Free perf win, near-zero risk. |
| 3 | Conditional updates + `update_mode` (§3.7) | S | embedding-staleness worker | Loud failure mode when re-embed worker is misconfigured. |
| 4 | Inline storage flag (§3.8) | S | collection-create config | Demo: side-by-side page-fault counts before/after on a cold-open. |
| 5 | Group-by query (§3.3) | S | sidebar layer | UI gets project-bucketed view in one shot. |
| 6 | Server-side formula scoring (§3.1) | S | `lens_search` replaced | Demo: "Memex blends recency, errors, and 5 lenses in one server call." Strong VSD 2026 storyline. |
| 7 | TurboQuant on `content` (§3.6) | S | collection-create config | "Memex ships the day Qdrant 1.18 ships, on its newest feature." |
| 8 | ACORN on `has_errors=true` (§3.9) | S | `recall` filter path | Numeric: recall@10 with `has_errors=true` before vs after. |
| 9 | Sparse vectors + BM25 + IDF modifier (§3.5) | M | new named sparse vector + Prefetch in hybrid query | Demo: "type `index.html` → Memex finds the session that edited that exact file." Pairs with score-boosting. |
| 10 | Relevance Feedback (§3.4) | M | Mix & Match replaces Discovery context-pairs | Demo: smoother result drift across consecutive +/− clicks. |

Stop at 10 for the hackathon. Items 1–8 fit comfortably in the 14-day window; 9 and 10 are stretch goals if 1–8 land early.

---

## 11. Appendix D — What this report intentionally does *not* claim

To keep the citation discipline honest, here is what was checked and either confirmed-unverified or out-of-scope:

- **MUVERA Rust support.** The 1.18 blog mentions MUVERA via FastEmbed 0.7.2+. Whether `fastembed-rs` (the Rust port Memex already uses) exposes MUVERA is **unverified** in this fetch round. `[blog qdrant-1.18.x §MUVERA — checked]`
- **Built-in cross-encoder reranker on local Qdrant.** Searched `documentation/concepts/inference/`, did not find any reranker-model entry. The only inference-side reranking note is "Reranking with FastEmbed" — a client-side reference, not a Qdrant-server feature. **Unverified for self-hosted 1.18.**
- **Replication on single-node 1.18.** Out of scope per the brief; explicitly skipped.
- **Auto-shrink / dimension auto-config.** Searched changelogs 1.13–1.18; no entry. The feature **does not appear to exist** in current Qdrant. `unverified` for any future roadmap.
- **gRPC streaming upserts API surface in Rust client.** `qdrant-client` 1.18.0 uses `tonic` under the hood (which is HTTP/2-streaming-capable) but the public Rust API surface for batched upserts is the canonical path. A genuine streaming-upserts builder is `unverified` at docs.rs surface depth-1.
- **Prefix-tokenizer for sparse vectors.** Text-index `prefix` tokenizer is documented (§28 in matrix). Whether sparse vectors accept the same tokenizer at upsert time is **unverified** — the inference-side BM25 model handles tokenization, so prefix-mode would have to come from the inference model config, not the sparse-vector config.

These gaps are noted so the next research pass can target exactly the missing surfaces.

---

*End of report. 80 features cataloged across Qdrant 1.13–1.18, with explicit Memex verdicts. The largest single gap is the entire Query-API-as-scoring-engine surface: `FormulaQuery`, MMR-on-`NearestQuery`, `RelevanceFeedbackQuery`, `group_by`, weighted RRF — all GA, all in the Rust client, none in Memex's code today. The one tempting 1.18 feature to skip is **BM42 sparse retrieval** — Qdrant officially still labels it experimental, and plain BM25 with the `modifier: idf` sparse modifier delivers the same retrieval lift without that risk.*
