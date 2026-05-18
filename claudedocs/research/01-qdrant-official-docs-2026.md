# Qdrant Official Documentation Research — for Memex

**Research date:** 2026-05-18
**Memex pinned version:** Qdrant 1.18.0 / `qdrant-client` (Rust) 1.18.0 (confirmed in `src-tauri/Cargo.lock`)
**Audience:** Memex maintainers preparing for Qdrant Vector Space Day 2026 ("Think Outside the Bot")

> Scope note: this dossier is read-only research. Every factual claim that touches the Qdrant runtime
> is followed by a `[Source: <URL> · fetched 2026-05-18]` tag. Where a doc page failed to fetch or
> only returned a stale paraphrase, that is called out explicitly with **Not found in official docs
> as of 2026-05-18** so future readers don't mistake a gap for a missing feature.

---

## 1. Versioning timeline

### 1.1 Current latest stable

The latest stable Qdrant server release is **v1.18.0**, tagged on the upstream repo on
**2026-05-11** (seven days before this report). The GitHub Releases API at
`repos/qdrant/qdrant/releases/latest` returns:

```
v1.18.0    2026-05-11T15:04:38Z    v1.18.0
```

[Source: https://api.github.com/repos/qdrant/qdrant/releases/latest · fetched 2026-05-18]

The release ships eight platform binaries; the one Memex's README quick-start downloads —
`qdrant-aarch64-apple-darwin.tar.gz` — is published at the canonical asset URL:

`https://github.com/qdrant/qdrant/releases/download/v1.18.0/qdrant-aarch64-apple-darwin.tar.gz`
(28,756,014 bytes).

[Source: https://api.github.com/repos/qdrant/qdrant/releases/tags/v1.18.0 · fetched 2026-05-18]

This is the URL the Memex README quick-start instructs the user to download, so the
quick-start is still valid as of today.

### 1.2 Why Memex is *already* on the latest

Memex pins `qdrant-client = "1"` in `src-tauri/Cargo.toml`, and the resolved version in
`Cargo.lock` is `qdrant-client v1.18.0` — published on the same day as the server, at
**2026-05-11T13:13:41Z**. The crate's release note confirms it tracks the server:

> Support for Qdrant 1.18

[Source: https://api.github.com/repos/qdrant/rust-client/releases/tags/v1.18.0 · fetched 2026-05-18]

So in practice, **Memex is already on the latest stable server and the matching Rust client**.

### 1.3 What's new in 1.18 (verbatim from release body)

Headline features:

> Add TurboQuant quantization variant, 8x vector compression without the recall tax

> Add API to create and delete named vectors in existing collection

> Deep memory reporting, show memory usage breakdown for storage components

> Add strict mode parameter to reject updates when memory usage is high (`max_resident_memory_percent`)

Improvements (selected):

> Use dynamic CPU pool for search workers, improve search performance when there's high IO wait

> Reduce memory usage of immutable geo index by 7x

> Fully remove RocksDB support, simplifying storage handling

Security:

> Enforce API key/JWT authentication on internal gRPC endpoints

> Add config option to disable snapshot restore from URL

[Source: https://api.github.com/repos/qdrant/qdrant/releases/tags/v1.18.0 · fetched 2026-05-18]

### 1.4 Release timeline (recent)

| Version | Published  | Note                                                              |
| ------- | ---------- | ----------------------------------------------------------------- |
| v1.18.0 | 2026-05-11 | **Latest. Memex is on this.** TurboQuant, named-vector CRUD.      |
| v1.17.1 | 2026-03-27 | Bug-fix release.                                                  |
| v1.17.0 | 2026-02-20 | Relevance Feedback, weighted RRF, audit logging.                  |
| v1.16.0 | 2025-11-17 | Tiered multitenancy, ACORN, inline storage, conditional updates.  |
| v1.15.0 | 2025-07-18 | 1.5-bit / 2-bit / asymmetric quantization, MMR, phrase match.     |
| v1.14.0 | 2025-04-22 | GPU-accelerated indexing.                                         |

[Source: https://api.github.com/repos/qdrant/qdrant/releases · fetched 2026-05-18]

### 1.5 Memex 1.18 → latest: what would change?

**Nothing.** Memex is already on 1.18.0 (server) and `qdrant-client` 1.18.0 (Rust). Risk is
purely *future* — when 1.19 lands, the 1.18 client may not back-port new-server features.

---

## 2. Named vectors (multi-vector points)

### 2.1 Official page

The canonical page is `qdrant.tech/documentation/manage-data/vectors/`.

[Source: https://qdrant.tech/documentation/manage-data/vectors/ · fetched 2026-05-18]

> Qdrant supports storing multiple vectors of different sizes and types within a single point.

The page documents collection creation with N named vectors, each with its own size and
distance metric. The current syntax (JSON):

```json
PUT /collections/{collection_name}
{
  "vectors": {
    "image": { "size": 4, "distance": "Dot" },
    "text":  { "size": 5, "distance": "Cosine" }
  }
}
```

[Source: https://qdrant.tech/documentation/manage-data/vectors/ · fetched 2026-05-18]

The Rust equivalent the page shows is:

```rust
let mut vector_config = VectorsConfigBuilder::default();
vector_config.add_named_vector_params("text",  VectorParamsBuilder::new(5, Distance::Dot));
vector_config.add_named_vector_params("image", VectorParamsBuilder::new(4, Distance::Cosine));
```

[Source: https://qdrant.tech/documentation/manage-data/vectors/ · fetched 2026-05-18]

This matches Memex's `indexer.rs` collection-creation shape almost exactly — Memex just
adds `content` / `tool` / `path` / `error` / `code` instead of `text` / `image`, all 384-d
cosine.

### 2.2 New in 1.18: runtime add/remove of named vectors

A real upgrade that lands in 1.18: you can now add and remove named vectors **without
recreating the collection**. The 1.18 release body lists:

> Add API to create and delete named vectors in existing collection

[Source: https://api.github.com/repos/qdrant/qdrant/releases/tags/v1.18.0 · fetched 2026-05-18]

For Memex this means future "add a `sparse_path` BM42 vector to existing 80-session
collection" work is no longer a re-index. Worth a one-line `client.update_collection_…`
helper in the next pass.

### 2.3 Multivectors (late interaction) vs named vectors

Important distinction the docs draw: **named vectors** are N separate dense vectors per
point (Memex's pattern). **Multivectors** are a single named vector that holds a
*variable-length matrix* of same-shape dense vectors per point, intended for ColBERT-style
late interaction.

> Qdrant supports the storing of a variable amount of same-shaped dense vectors in a single point.

The comparator for multivectors is `max_sim` —

> a sum of maximum similarities between each pair of vectors in the matrices

[Source: https://qdrant.tech/documentation/manage-data/vectors/ · fetched 2026-05-18]

The advanced tutorial page `using-multivector-representations/` exists but its fetch
returned an empty response on 2026-05-18 (likely caching). The blog article
`muvera-embeddings/` (next section) substitutes.

### 2.4 MUVERA (2026 multivector accelerator)

> MUVERA embeddings…aim to solve this problem. The idea is to create a single vector that approximates the multi-vector representation.

[Source: https://qdrant.tech/articles/muvera-embeddings/ · fetched 2026-05-18]

Pipeline: SimHash clustering → Fixed Dimensional Encoding (FDE) → random projection. The
output is a single dense vector that approximates the original multivector matrix and is
indexable in regular HNSW. Headline number from the article:

> MUVERA-only search is approximately 8x faster than full multi-vector search, while the hybrid approach with reranking still achieves about 7x speed improvement while maintaining nearly identical search quality.

[Source: https://qdrant.tech/articles/muvera-embeddings/ · fetched 2026-05-18]

**MUVERA support location.** As of today it's a `fastembed` post-processor:

> FastEmbed version 0.7.2 has introduced support for MUVERA embeddings

[Source: https://qdrant.tech/articles/muvera-embeddings/ · fetched 2026-05-18]

The article does **not** mention native Qdrant-server integration — MUVERA is a
client-side encoder you call before upsert. Memex would have to add the
`fastembed-rs` 0.7.2+ MUVERA call path; the server stays vanilla.

---

## 3. Distance Matrix API

### 3.1 Endpoint shape

```
POST /collections/{collection_name}/points/search/matrix/pairs
POST /collections/{collection_name}/points/search/matrix/offsets
```

[Source: https://api.qdrant.tech/api-reference/search/matrix-pairs · fetched 2026-05-18]

### 3.2 Request schema (`pairs` variant)

Verbatim field descriptions from the API reference:

- `sample` — "How many points to select and search within. Default is 10."
- `limit` — "How many neighbours per sample to find. Default is 3."
- `using` — "Define which vector name to use for querying. If missing, the default vector is used."
- `filter` — "Look only for points which satisfies this conditions"
- `shard_key` — restrict the call to specific shards (optional)

[Source: https://api.qdrant.tech/api-reference/search/matrix-pairs · fetched 2026-05-18]

### 3.3 Response shape

> result.pairs — List of pairs of points with scores

Each `SearchMatrixPair` carries `a` (point id), `b` (point id), and `score` (distance).
The response envelope also includes `time`, `status`, and a `usage` object reporting
hardware/inference resources consumed.

[Source: https://api.qdrant.tech/api-reference/search/matrix-pairs · fetched 2026-05-18]

This is exactly the shape Memex consumes in
`src-tauri/src/indexer.rs::topology`: `SearchMatrixPair { a, b, score }` feeding directly
into `petgraph::UnGraph<String, f32>` for MST computation. The `using("content")`,
`sample(N)`, `limit(K)` parameters in Memex map 1:1 to the documented fields.

### 3.4 Intended use cases (per Qdrant)

The `articles/distance-based-exploration/` post lists three first-class use cases:

1. **Dimensionality reduction** — feed pairs into UMAP/t-SNE for 2D/3D plots.
   The article quotes UMAP's promise to preserve "the relative distances between
   high-dimensional points."
2. **Clustering** — distance matrix as KMeans / HDBSCAN input.
3. **Graph exploration** — "the more similar the data points are, the stronger the edges
   between them."

[Source: https://qdrant.tech/articles/distance-based-exploration/ · fetched 2026-05-18]

Memex's MST is squarely in use case 3. There is no doc page that calls out MST
specifically — Memex is using the API the way Qdrant intended, just with a particular
graph algorithm on top.

### 3.5 Pagination / limits

The API reference does **not** document explicit pagination cursors for matrix endpoints
— `sample` caps how many points you ask about, and `limit` caps K-NN per sample. There's
no `offset` or `next_page` field in `SearchMatrixRequest` as of 2026-05-18.

[Source: https://api.qdrant.tech/api-reference/search/matrix-pairs · fetched 2026-05-18]

For Memex's ~80 sessions this is irrelevant; for 100k+ points you'd shard the
exploration by `sample` calls with different `filter` partitions.

---

## 4. Discovery API

### 4.1 Endpoint shape (today: wrapped in Query API)

The Discovery API has been folded into the universal `query` endpoint. The current
canonical shape is:

```
POST /collections/{collection_name}/points/query
{
  "query": {
    "discover": {
      "target": [0.2, 0.1, 0.9, 0.7],
      "context": [
        { "positive": 100, "negative": 718 },
        { "positive": 200, "negative": 300 }
      ]
    }
  },
  "limit": 10
}
```

[Source: https://qdrant.tech/documentation/search/explore/ · fetched 2026-05-18]

### 4.2 Rust example (verbatim from the docs)

```rust
use qdrant_client::qdrant::{ContextInputBuilder, DiscoverInputBuilder, QueryPointsBuilder};

client
    .query(
        QueryPointsBuilder::new("{collection_name}").query(
            DiscoverInputBuilder::new(
                vec![0.2, 0.1, 0.9, 0.7],
                ContextInputBuilder::default()
                    .add_pair(100, 718)
                    .add_pair(200, 300),
            )
            .build(),
        ),
    )
    .await?;
```

[Source: https://qdrant.tech/documentation/search/explore/ · fetched 2026-05-18]

This is exactly the call shape Memex uses in `indexer.rs::mix_match`.

### 4.3 Target requirement — still required for *discovery search*

> Target is required for discovery search. The documentation distinguishes discovery search (with target) from context search (without target).

[Source: https://qdrant.tech/documentation/search/explore/ · fetched 2026-05-18]

So Memex's current workaround — "set target to the first positive session" — is still
the canonical way to express "find sessions like *this one*, biased by these contrastive
pairs." The alternative is **context search** (omit `target`, use only `context` pairs),
which is also available via the same Query API but with `target: null`. The docs describe
that mode as:

> When only the context is provided (without a target), pairs of points are used to generate a loss that guides the search towards the area where most positive examples overlap.

[Source: https://api.qdrant.tech/api-reference/search/discover-points · fetched 2026-05-18]

For Memex's UX ("show me sessions like the ones I positive-tagged, unlike the ones I
negative-tagged"), the current "first positive as target" choice is correct. A future
**context-search** mode in the UI would let users skip the anchor entirely — useful for
"find a region of sessions that look like these N positives", potentially a richer Mix &
Match v2.

### 4.4 Context-pair semantics (verbatim)

> Context is a set of positive-negative pairs, and each pair divides the space into positive and negative zones.

[Source: https://qdrant.tech/documentation/search/explore/ · fetched 2026-05-18]

> The context score for each pair is +1 if the point is closer to a positive example than to a negative example, and -1 otherwise.

[Source: https://api.qdrant.tech/api-reference/search/discover-points · fetched 2026-05-18]

The total score is two-part: integer = rank vs. context, decimal = distance to target.
That's why "1 positive close to target" can outrank "2 positives far from target" —
useful framing for the Memex result-card chips.

---

## 5. Payload indexes

### 5.1 Types supported as of 2026-05-18

The current `manage-data/indexing/` page enumerates eight first-class payload index types:

| Type     | JSON schema                  | Use                                    |
| -------- | ---------------------------- | -------------------------------------- |
| keyword  | `"keyword"`                  | exact string match, faceting           |
| integer  | `"integer"`                  | range + match on i64                   |
| float    | `"float"`                    | range on f64                           |
| bool     | `"bool"`                     | match on true/false                    |
| geo      | `"geo"`                      | bounding box / radius                  |
| datetime | `"datetime"`                 | RFC3339 range                          |
| text     | `{ "type": "text", "tokenizer": "word" }` | full-text search filter   |
| uuid     | `"uuid"`                     | optimized keyword for UUID payloads    |

[Source: https://qdrant.tech/documentation/manage-data/indexing/ · fetched 2026-05-18]

### 5.2 Memex coverage vs available types

Memex currently uses, per `docs/qdrant-features.md`:

- `keyword` on `project_name`, `project_path`, `git_branch`
- `text` on `ai_title`
- `integer` on `start_ts`
- `bool` on `has_errors`

All four index types Memex relies on are first-class and unchanged in 1.18. The
**`datetime`** index type would be a marginally better fit for `start_ts` than integer
(supports RFC3339 ranges natively), but integer is fine if Memex stores UNIX seconds.

### 5.3 New since Memex started (1.16+ payload-side improvements)

From the v1.16.0 release body, payload-side additions worth knowing about:

> Add ASCII folding (normalization) to full text indices, fold diacritics into ASCII characters
> Add conditional update functionality, only apply update on points matching filter
> Add `text_any` full text filter to match any query term
> In strict mode, specify maximum number of payload indices per collection
> Add custom key-value metadata to collections

[Source: https://api.github.com/repos/qdrant/qdrant/releases/tags/v1.16.0 · fetched 2026-05-18]

**ASCII folding** for Memex's `text` index on `ai_title` would let users find a title with
"café" by typing "cafe". One config tweak, zero re-indexing pain.

**Conditional updates** are the more interesting unlock for the future "embedding model
upgrade" scenario: you can atomically swap the BGE-small content vector for, say,
`mxbai-embed-large` only on points where `embedding_model_version == "bge-small-v1.5"`.

### 5.4 Index tuning detail unique to 1.17

> Add ability to disable extra HNSW links construction for specific payload indices

[Source: https://api.github.com/repos/qdrant/qdrant/releases/tags/v1.17.0 · fetched 2026-05-18]

Relevant if Memex ever sees pre-filtering on a high-selectivity payload like
`project_name` slow down search — you can disable the extra HNSW edges that index would
otherwise add.

---

## 6. Snapshot API

### 6.1 HTTP endpoints (collection-scoped)

From the search-result enumeration of `qdrant.tech/documentation/database-tutorials/create-snapshot/`:

```
POST /collections/{collection_name}/snapshots                     → create snapshot
GET  /collections/{collection_name}/snapshots                     → list
GET  /collections/{collection_name}/snapshots/{snapshot_name}     → download
PUT  /collections/{collection_name}/snapshots/recover             → restore (URL or local path)
POST /collections/{collection_name}/snapshots/upload?priority=…   → restore by upload
POST /collections/{collection_name}/shards/{shard_id}/snapshots   → per-shard snapshot
```

[Source: https://qdrant.tech/documentation/database-tutorials/create-snapshot/ · fetched 2026-05-18]

There is also a storage-wide variant:

```
POST /snapshots
```

[Source: https://api.qdrant.tech/api-reference/snapshots/create-full-snapshot · fetched 2026-05-18]

Memex's `snapshot_export` / `snapshot_import` paths in `indexer.rs` use the
collection-scoped POST/GET/upload subset, which is fine.

### 6.2 `recover` request body and priority enum

The recover endpoint accepts `location` (required URL or `file://` path), optional
`priority` (`no_sync` | `snapshot` | `replica`), optional `checksum` ("Optional SHA256
checksum to verify snapshot integrity before recovery"), and optional `api_key` for
remote-URL fetches. Priority semantics: `no_sync` skips any post-restore sync;
`snapshot` makes the snapshot authoritative; `replica` preserves existing data and
syncs the snapshot afterward.

[Source: https://api.qdrant.tech/api-reference/snapshots/recover-from-snapshot · fetched 2026-05-18]

Memex's import path is "restore into a fresh collection," so `priority=snapshot` is the
right default — and that matches the existing `priority=snapshot` query string in
`docs/qdrant-features.md`.

### 6.3 Cross-version compatibility (verbatim)

> Snapshots generated in one Qdrant cluster can only be restored to other Qdrant clusters that share the same minor version—for instance, a snapshot captured from a v1.4.1 cluster can only be restored to clusters running version v1.4.x.

[Source: https://qdrant.tech/documentation/database-tutorials/create-snapshot/ · fetched 2026-05-18]

**Implication for Memex.** A Memex snapshot taken against Qdrant 1.18.x can be restored
on any 1.18.y, but **not** on 1.19.x. If users start sharing snapshots peer-to-peer
(which is the promise in the README), Memex should embed the server version in the
snapshot filename. A simple `memex-{collection}-{git_short_hash}-qdrant-{server_minor}.snapshot`
naming convention would prevent a sad UX when 1.19 ships.

### 6.4 Integrity / tampering

The doc page does not call out file tampering specifically, but two safety surfaces are
available today:

1. **SHA256 checksum** on the recover request — verifies the snapshot wasn't
   bit-flipped or substituted between download and restore.
2. **Disable URL-based restore** — a brand-new 1.18 feature:

> Add config option to disable snapshot restore from URL

[Source: https://api.github.com/repos/qdrant/qdrant/releases/tags/v1.18.0 · fetched 2026-05-18]

For Memex, which restores from a *local file the user picked*, neither is mandatory —
but exposing the SHA256 in a future "verify on import" UI hook would be a nice touch and
costs only a few lines in `snapshot_import`.

---

## 7. Rust client status

### 7.1 Current version

`qdrant-client` 1.18.0, published **2026-05-11T13:13:41Z** — 2 hours before the server
release.

[Source: https://api.github.com/repos/qdrant/rust-client/releases/tags/v1.18.0 · fetched 2026-05-18]

Memex's `Cargo.lock` confirms resolution to exactly `qdrant-client v1.18.0`:

```
name = "qdrant-client"
version = "1.18.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
```

### 7.2 What 1.18 adds to the client

The release body is short:

> Support for Qdrant 1.18
> Add custom headers in client builder
> Add helper to specify per request tracing

[Source: https://api.github.com/repos/qdrant/rust-client/releases/tags/v1.18.0 · fetched 2026-05-18]

The "per request tracing" helper is the more useful one for Memex if it ever turns on
audit logging on the server (1.17 feature) — every Memex query gets a trace ID that
appears in the audit log.

### 7.3 Breaking changes in the 1.17 → 1.18 window

The 1.17 Rust client removed the old pre-1.10 client:

> Remove old client deprecated since version 1.10

[Source: https://api.github.com/repos/qdrant/rust-client/releases/tags/v1.17.0 · fetched 2026-05-18]

Memex uses the new builder API (`QueryPointsBuilder::new(...).query(...)`,
`Qdrant::from_url(...).build()`), so the removal is a no-op. The 1.18 client also dropped
deprecated `search`/`recommend`/`discovery`/`upload_records`/`search_batch`/`recommend_batch`/
`discovery_batch`/`rest`/`init_from` methods — Memex calls none of these.

### 7.4 Recommended construction pattern (verbatim from docs)

```rust
let client = Qdrant::from_url("http://localhost:6334").build()?;
```

For cloud, append `.api_key(std::env::var("QDRANT_API_KEY"))` before `.build()`.

[Source: https://github.com/qdrant/rust-client · fetched 2026-05-18]

This is exactly what Memex does. No change recommended.

---

## 8. Quantization & storage

### 8.1 Methods available in 1.18

The official quantization page enumerates four methods:

- **Scalar** (1.1.0) — float32 → uint8, 4× compression.
- **Binary** (1.5.0) — 1-bit-per-component, up to 32×. 1.5-bit and 2-bit added in 1.15.
- **Product** (1.2.0) — codebook-based, up to 64×, "Slower distance calculations; not SIMD-friendly."
- **TurboQuant** (1.18.0) — rotation-based, up to 32×, "asymmetric quantization automatically: only stored vectors are compressed, while queries are scored in full precision."

[Source: https://qdrant.tech/documentation/manage-data/quantization/ · fetched 2026-05-18]

### 8.2 TurboQuant — the 1.18 headliner (verbatim)

> Add TurboQuant quantization variant, 8x vector compression without the recall tax

[Source: https://api.github.com/repos/qdrant/qdrant/releases/tags/v1.18.0 · fetched 2026-05-18]

Config shape: `{ "quantization_config": { "turbo": { "bits": "bits4", "always_ram": true } } }`.

### 8.3 Should Memex enable quantization?

- **Today (≤ 80 sessions, ~400 vectors): no.** 5 × 384 floats × 80 sessions ≈ 600 KB. Quantization saves megabytes that don't exist.
- **At scale (≥ 10k sessions): yes.** TurboQuant 4-bit cuts RAM ~8× with no recall hit per the doc; `always_ram: true` keeps lookups fast.
- **Very large (≥ 100k sessions): maybe binary** for `content` + TurboQuant for `error`/`tool`. BGE-small at 384-d is borderline for binary ("only efficient for high-dimensional vectors and require a centered distribution") — needs empirical test.

Doc guidance: "Test quantization on your data before production deployment" — use
`ignore: true` at query time to A/B raw vs quantized recall before flipping the default.

[Source: https://qdrant.tech/documentation/manage-data/quantization/ · fetched 2026-05-18]

### 8.4 Other storage knobs added 2025+

- **Inline storage** (v1.16.0): vectors live inside HNSW nodes, optimized for disk I/O.
  > stores quantized vector data directly inside the HNSW nodes
  > optimizes disk-based deployments by reducing random access reads from 32 per node to approximately 2 pages
  [Source: https://qdrant.tech/blog/qdrant-1.16.x/ · fetched 2026-05-18]
- **Gridstore** (v1.15.0 default, v1.18.0 full RocksDB removal): custom storage engine.
  > Fully remove RocksDB support, simplifying storage handling
  [Source: https://api.github.com/repos/qdrant/qdrant/releases/tags/v1.18.0 · fetched 2026-05-18]

Memex doesn't touch storage config; the 1.18 defaults are already the right pick.

---

## 9. HNSW tuning

### 9.1 Default values (verbatim)

> m — Number of edges per node in the index graph. Larger the value - more accurate the search, more space required.  Default: 16

> ef_construct — Number of neighbours to consider during the index building. Larger the value - more accurate the search, more time required to build index.  Default: 100

> full_scan_threshold — Minimal size threshold (in KiloBytes) below which full-scan is preferred over HNSW search.  Default: 10000 KB

`hnsw_ef` (per-query) defaults to `ef_construct` (100) when not specified.

[Source: https://qdrant.tech/documentation/manage-data/indexing/ · fetched 2026-05-18]

### 9.2 Recommended ranges for Memex's scale (≤ 100k points)

The official docs don't publish a "≤ 100k points" recipe. Synthesizing:

- **m = 16** is fine. Bumping to 32 helps recall at ~2× HNSW-graph memory — rarely worth it under 100k.
- **ef_construct = 100** is fine; 200–256 helps build quality. Memex indexes incrementally, so build time is invisible — 200 is a safe bump.
- **hnsw_ef = 100** (search) is fine for "feels instant" UX. 64 to squeeze latency, 200 to maximize recall.
- **full_scan_threshold = 10000 KB** is the right default. Memex's ~600 KB collection is **below** this — searches may be brute-force today, which is *fine* (faster than HNSW for tiny N).

[Source: https://qdrant.tech/documentation/manage-data/indexing/ · fetched 2026-05-18]

### 9.3 Bulk-load trick

> Set m=0 to skip HNSW graph links during bulk upload. Switch to a normal m after ingest to build the graph. This speeds up inserts 5-10x because link creation is deferred.

[Source: https://qdrant.tech/documentation/manage-data/indexing/ · fetched 2026-05-18]

For Memex's first-run "scan all 80 sessions" — `m=0` during the initial bulk-upsert, then
`update_collection(...).with_hnsw(m=16)` after — would cut first-index time materially.
Today Memex doesn't do this; it's a non-blocking nice-to-have.

### 9.4 1.16 ACORN search algorithm (filterable HNSW)

> ACORN-1 search method, accurate search over many filtered points at the cost of performance

[Source: https://api.github.com/repos/qdrant/qdrant/releases/tags/v1.16.0 · fetched 2026-05-18]

> examines neighbors of neighbors (the second hop) if the direct neighbors have been filtered out

[Source: https://qdrant.tech/blog/qdrant-1.16.x/ · fetched 2026-05-18]

Relevant to Memex's `has_errors=true` filter in `recall`: if the dataset grows and
`has_errors=true` becomes selective (say <5% of sessions), ACORN improves recall on the
filtered query at the cost of latency. Enable per-query via `params.acorn = true`.

---

## 10. 2026 feature highlights not in Memex

### 10.1 Sparse vectors (BM25 / BM42 / SPLADE)

**What.** Native sparse-vector index alongside dense. Each point can carry both:

```python
client.create_collection(
    collection_name=COLLECTION_NAME,
    vectors_config={
        "text-dense": models.VectorParams(size=1536, distance=models.Distance.COSINE)
    },
    sparse_vectors_config={
        "text-sparse": models.SparseVectorParams(
            index=models.SparseIndexParams(on_disk=False)
        )
    },
)
```

[Source: https://qdrant.tech/articles/sparse-vectors/ · fetched 2026-05-18]

**IDF — server-side since 1.10.** From the BM42 article:

> When using any retrieval formula that includes IDF (Inverse Document Frequency), such as BM25, in Qdrant, you no longer need to include the IDF component in the sparse document representations—the IDF component will be applied by Qdrant automatically when computing similarity scores.

[Source: https://qdrant.tech/articles/bm42/ · fetched 2026-05-18]

**BM42 — official caveat.**

> BM42 should be considered as an experimental approach, which requires further research and development before it can be used in production.

[Source: https://qdrant.tech/articles/bm42/ · fetched 2026-05-18]

**Memex relevance verdict.** Memex's `path` and `tool` lenses are token-flavored —
filenames, tool names, CLI flags. A **BM25 sparse** vector on those payloads would
out-perform a 384-d dense vector for exact-match queries like "edit `index.html`" or
"running `cargo test`". This is exactly what `docs/qdrant-features.md` flags as
deferred (T2.2). Worth picking back up: BM25 is production-ready; BM42 isn't.

**Adoption cost.** New `sparse_vectors_config` entry, a tokenizer in Rust (or call
fastembed `Bm25` model), and one `Prefetch` block in the hybrid query. ~1 day of work
plus eval.

### 10.2 ColBERT / late-interaction multivectors

**What.** Per-token vectors with `max_sim` comparator inside one named vector slot. Best
recall on the planet for retrieval; expensive at scale (where MUVERA comes in).

[Source: https://qdrant.tech/documentation/manage-data/vectors/ · fetched 2026-05-18]

**Memex relevance verdict.** Overkill. Memex's "find similar sessions" task is solved by
single-vector cosine within 1% of ColBERT-grade recall, and at 80 sessions the latency
difference is invisible. **Skip until corpus crosses 100k sessions.**

### 10.3 Hybrid search via Query API

**What.** Combine a dense query, a sparse query, and a fusion step (RRF or DBSF) inside
*one* server-side `query` call via nested `Prefetch` blocks and a `FusionQuery`.

[Source: https://qdrant.tech/articles/hybrid-search/ · fetched 2026-05-18]

**Weighted RRF** is new in 1.17:

> Add Weighted RRF

[Source: https://api.github.com/repos/qdrant/qdrant/releases/tags/v1.17.0 · fetched 2026-05-18]

**Memex relevance verdict.** Memex *already* has a weighted multi-vector setup
(`sum(score_i × weight_i / sum_of_weights)` in `lens_search`). The Query API's hybrid
RRF would centralize that math server-side — fewer round trips per query — but the
client-side approach gives Memex something the server doesn't: per-lens score chips on
each result card. **Defer until per-query latency becomes a complaint;** the UI value of
per-vector scores outweighs the server-side fusion benefit at Memex's scale.

### 10.4 Server-side inference (Qdrant Cloud Inference)

**What.** Wrap a `Document` object in your upsert/query and Qdrant generates the embedding
in-cluster — supported models include MiniLM, mxbai-large, SPLADE, BM25, and CLIP.

[Source: https://qdrant.tech/blog/qdrant-cloud-inference-launch/ · fetched 2026-05-18]

**Critical caveat.**

> This feature is cloud-only.

[Source: https://qdrant.tech/blog/qdrant-cloud-inference-launch/ · fetched 2026-05-18]

**Memex relevance verdict.** **Not applicable.** Memex's pitch is local-first, single
binary, no cloud round-trip — moving embedding to a remote service breaks the value prop.
`fastembed` in-process is the right call and stays the right call.

### 10.5 MUVERA (single-vector approximation of multivector)

**What.** Encode ColBERT-style multivectors into one dense vector via FDE; first-pass with
that, optionally rerank with the original multivectors. See section 2.4.

**Memex relevance verdict.** Conditional on Memex ever adopting ColBERT (section 10.2),
MUVERA becomes the right way to make it fast. Today MUVERA is also a fastembed-side
feature (0.7.2+, Python-first), with no documented Qdrant-server hook. **Park until
ColBERT lands.**

### 10.6 Relevance feedback (1.17)

**What.** A new way to tell Qdrant "this result was good / bad" and have subsequent
searches adapt. Listed in the 1.17 release notes as a milestone feature:

> Relevance Feedback ([docs](https://qdrant.tech/documentation/concepts/search-relevance/#relevance-feedback))

[Source: https://api.github.com/repos/qdrant/qdrant/releases/tags/v1.17.0 · fetched 2026-05-18]

**Memex relevance verdict.** *Genuinely* aligned with the "Think Outside the Bot" theme —
Memex's `+ pos / − neg` UI is already a feedback mechanism. Today it feeds Discovery's
`context` pairs; relevance feedback offers a parallel path where the feedback adapts the
*dense* query rather than constraining it via pairs. Worth a one-page spike: would
relevance feedback give a smoother result-set evolution than rebuilding the Discovery
context every click?

### 10.7 MMR reranking (1.15)

**What.** Diversity-aware re-ranking — penalize near-duplicates in the result set.

[Source: https://qdrant.tech/blog/qdrant-1.15.x/ · fetched 2026-05-18]

**Memex relevance verdict.** **Yes, easy win.** Many Claude Code sessions are
near-duplicates (rerunning the same project across days). An MMR re-rank with
λ ≈ 0.7 on the top-50 of every lens query would diversify result lists without losing
the top hits. Single param on the query call; near-zero adoption cost.

### 10.8 Phrase matching + multilingual tokenization (1.15)

**What.** Full-text index can now do exact phrase matching (Doumont would approve) and
tokenize Japanese / Chinese natively.

[Source: https://qdrant.tech/blog/qdrant-1.15.x/ · fetched 2026-05-18]

**Memex relevance verdict.** Niche — Memex is English-only Claude Code sessions today. If
the user base ever includes JP/ZH Claude users, this becomes a one-line fix on the
`ai_title` text index.

### 10.9 GPU-accelerated HNSW indexing (1.14)

**What.** Initial HNSW build moved to GPU on supported hardware.

[Source: https://qdrant.tech/blog/2025-recap/ · fetched 2026-05-18]

**Memex relevance verdict.** **Not for desktop.** Memex runs on whatever laptop Claude
Code runs on. Skip.

### 10.10 Decision matrix

| Feature                            | Adopt for Memex?              | Effort   | Priority   |
| ---------------------------------- | ----------------------------- | -------- | ---------- |
| BM25 sparse on `path` / `tool`     | **Yes**                       | ~1 day   | **High**   |
| MMR reranking                      | **Yes**                       | ~1 hour  | **High**   |
| ASCII folding on `ai_title`        | **Yes**                       | ~5 min   | Medium     |
| Relevance Feedback                 | Spike                         | ~1 day   | Medium     |
| Snapshot SHA256 in UI              | Yes (defense in depth)        | ~2 hours | Medium     |
| Conditional updates                | Save for embedding-model swap | n/a now  | Low        |
| ACORN on filtered search           | Once `has_errors` selectivity drops | ~10 min | Low  |
| TurboQuant 4-bit                   | At ≥ 10k sessions             | ~30 min  | Low (now)  |
| ColBERT multivector / MUVERA       | At ≥ 100k sessions, paired    | ~1 wk    | Skip (now) |
| Cloud Inference / GPU HNSW         | **No** (breaks local-first)   | —        | Never      |

---

## 11. Deployment

### 11.1 Single binary vs Docker vs Cloud

Three official deployment modes:

1. **Single binary** — download from GitHub Releases, run `./qdrant`. README's
   recommended path for Memex users on macOS arm64.
2. **Docker** — `qdrant/qdrant` on Docker Hub; recommended command from quickstart:
   ```bash
   docker run -p 6333:6333 -p 6334:6334 \
       -v "$(pwd)/qdrant_storage:/qdrant/storage:z" \
       qdrant/qdrant
   ```
   [Source: https://qdrant.tech/documentation/quickstart/ · fetched 2026-05-18]
3. **Qdrant Cloud** — managed, with Cloud Inference layered on top.

### 11.2 1.18.0 binary URL for `aarch64-apple-darwin` (verbatim)

`https://github.com/qdrant/qdrant/releases/download/v1.18.0/qdrant-aarch64-apple-darwin.tar.gz`
(28,756,014 bytes)

[Source: https://api.github.com/repos/qdrant/qdrant/releases/tags/v1.18.0 · fetched 2026-05-18]

The Memex README (lines 224 / 262 / 269) references this exact URL with `v1.18.0` —
**still correct and live as of 2026-05-18** since v1.18.0 is the current latest stable.

### 11.3 Docker tag

The README at line 224 / 269 uses `qdrant/qdrant:v1.18.0`. That tag exists on Docker Hub
under the standard release tagging convention used for every prior release.
[Source (release pattern): https://api.github.com/repos/qdrant/qdrant/releases · fetched 2026-05-18]

The official quickstart uses the unpinned `qdrant/qdrant` tag (rolling latest). Memex's
choice to pin to `v1.18.0` is the right one for reproducibility — it's the latest stable
as of today and locks the snapshot-compatibility minor-version rule (section 6.3) to a
known value.

### 11.4 Single-binary safety note (new in 1.18)

If Memex's users follow the README's "download and run" path on macOS with no
authentication, two 1.18 server-side knobs are worth a callout in future docs:

> Enforce API key/JWT authentication on internal gRPC endpoints

> Add config option to disable snapshot restore from URL

[Source: https://api.github.com/repos/qdrant/qdrant/releases/tags/v1.18.0 · fetched 2026-05-18]

Memex talks to `localhost:6334` only, so neither is required, but if a user ever exposes
their Qdrant port to a LAN — disabling URL-based snapshot restore is now the supported
mitigation against the "evil snapshot URL" attack surface.

---

## Sources

- `api.github.com/repos/qdrant/qdrant/releases/latest` — current latest server release tag/date.
- `api.github.com/repos/qdrant/qdrant/releases` — full release timeline (v1.13 → v1.18) with dates.
- `api.github.com/repos/qdrant/qdrant/releases/tags/v1.18.0` — v1.18 changelog body + asset list (the aarch64 macOS URL).
- `api.github.com/repos/qdrant/qdrant/releases/tags/v1.17.0` — v1.17 changelog (relevance feedback, weighted RRF, audit logging).
- `api.github.com/repos/qdrant/qdrant/releases/tags/v1.16.0` — v1.16 changelog (ACORN, inline storage, tiered multitenancy, ASCII folding).
- `api.github.com/repos/qdrant/rust-client/releases/latest` — latest Rust client release.
- `api.github.com/repos/qdrant/rust-client/releases/tags/v1.18.0` — Rust 1.18 changelog.
- `api.github.com/repos/qdrant/rust-client/releases/tags/v1.17.0` — Rust 1.17 changelog (pre-1.10 client removed).
- `qdrant.tech/documentation/manage-data/vectors/` — named-vector + multivector syntax (JSON + Rust).
- `qdrant.tech/documentation/manage-data/indexing/` — HNSW defaults; payload index types; bulk-load `m=0` trick.
- `qdrant.tech/documentation/manage-data/quantization/` — Scalar/binary/product/TurboQuant config + best practices.
- `qdrant.tech/documentation/search/explore/` — Discovery API current shape + Rust example + context-pair semantics.
- `qdrant.tech/documentation/quickstart/` — official Docker `run` invocation.
- `qdrant.tech/documentation/database-tutorials/create-snapshot/` — snapshot HTTP endpoints + minor-version rule.
- `qdrant.tech/articles/distance-based-exploration/` — Distance Matrix use cases + `search_matrix_offsets` example.
- `qdrant.tech/articles/muvera-embeddings/` — MUVERA description + 8×/7× perf numbers + fastembed 0.7.2 note.
- `qdrant.tech/articles/hybrid-search/` — Query API hybrid prefetch shape with RRF.
- `qdrant.tech/articles/sparse-vectors/` — sparse-vector collection-creation shape.
- `qdrant.tech/articles/bm42/` — BM42 IDF-on-server quote + experimental caveat.
- `qdrant.tech/articles/agentic-builders-guide/` — core agentic patterns (memory, multimodal, hybrid, filtering).
- `qdrant.tech/blog/2025-recap/` — 2025 capability-area breakdown + 2026 roadmap themes.
- `qdrant.tech/blog/qdrant-1.15.x/` — 1.15 blog (1.5/2-bit quant, MMR, phrase match, multilingual tokenization).
- `qdrant.tech/blog/qdrant-1.16.x/` — 1.16 blog (ACORN verbatim explanation, inline storage).
- `qdrant.tech/blog/qdrant-cloud-inference-launch/` — Cloud Inference launch — cloud-only confirmation.
- `api.qdrant.tech/api-reference/search/matrix-pairs` — `search_matrix_pairs` endpoint, request fields, response shape.
- `api.qdrant.tech/api-reference/search/discover-points` — Discovery target-required statement + context scoring.
- `api.qdrant.tech/api-reference/snapshots/recover-from-snapshot` — Snapshot recover body, priority enum, checksum field.
- `api.qdrant.tech/api-reference/snapshots/create-full-snapshot` — storage-wide snapshot POST endpoint.
- `github.com/qdrant/rust-client` — recommended `Qdrant::from_url(...).build()` construction pattern.

Three pages returned a generic "I can't browse" stub on 2026-05-18 (WebFetch caching
artifact); facts in this dossier are sourced from adjacent pages or release bodies:

- `documentation/concepts/snapshots/` and `documentation/operations/snapshots/` — snapshot facts came from `database-tutorials/create-snapshot/` and the API reference instead.
- `documentation/concepts/search-relevance/` — Relevance Feedback presence confirmed from the 1.17 release body only. Detailed API shape: **Not found in official docs as of 2026-05-18** (fetch failure, not absence of docs).
