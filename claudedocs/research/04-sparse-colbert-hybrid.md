# Sparse Vectors, BM42/BM25, ColBERT v2, Hybrid Search & MUVERA — Memex Adoption Audit

**Date:** 2026-05-18
**Subject project:** Memex (`/Users/kimsejun/Documents/GitHub/memex`) — Tauri 2 desktop app, Qdrant 1.18, `fastembed = "5"`, 5 named dense vectors per session at 384-d.
**Question:** Is the README's deferral of BM42 sparse-on-`path` and ColBERT v2 on `content` still valid in 2026-05? Where can we materially improve rank quality?

---

## 1. Why this matters for Memex

Memex currently embeds five logically-distinct vectors with the *same* dense encoder — `BAAI/bge-small-en-v1.5`, a 384-d semantic model (`indexer.rs:38-40`, `indexer.rs:61`). BGE-small is genuinely strong for prose similarity, but it is a poor match for two of Memex's five vectors:

- **`path`** (built by `build_path()` at `indexer.rs:254-281`) is a newline-joined list of *literal* file paths and URLs. Path tokens like `src-tauri/src/indexer.rs`, `Cargo.toml`, or `node_modules/@anthropic-ai/sdk` are out-of-distribution for a sentence encoder. Semantic embedding washes out the high-IDF leaf tokens (`indexer`, `embedder`, `predict_next_actions`) that a user actually types into the search box. This is the textbook case for sparse (BM25/BM42).
- **`code`** (built by `build_code()` at `indexer.rs:311-333`) is fenced code blocks plus Edit/Write payloads. Function names (`session_extracts`, `lens_search`), error codes (`EROFS`, `ENOENT`), and CLI flags need *exact-token* recall. Dense embeddings degrade further as the snippet grows (BGE-small has only 512-token context, and `MAX_CHARS_PER_VECTOR = 6_000` at `indexer.rs:42` truncates).
- The `error` vector has similar properties (`EROFS`, panic strings, traceback frames).

`content`, `tool`, and (to a lesser extent) `error` benefit primarily from late-interaction reranking — ColBERT v2 — because the user query is short ("the bug with fastembed cache") while the candidate document is a 6 000-char transcript window. Late interaction's per-token MaxSim handles the asymmetry better than a single 384-d compress-and-cosine.

So the README's two flagged gaps (BM42 on `path`, ColBERT on `content`) are exactly the right ones. The only question is whether the 2026 client tooling can deliver them yet.

---

## 2. Sparse vectors in Qdrant — current state

Qdrant has shipped sparse-vector support as a first-class collection type for some time; it is configured *separately* from the dense `vectors` block via a `sparse_vectors_config` map. The official sparse-vectors article walks through it:

> "Qdrant supports sparse vectors through a separate index, enabling hybrid search by combining sparse and dense vectors in the same collection. Each point can store both vector types simultaneously."
> [Source: `https://qdrant.tech/articles/sparse-vectors/` · fetched 2026-05-18]

Python config (the canonical pattern) coexists with dense named vectors:

```python
client.create_collection(
    collection_name=COLLECTION_NAME,
    vectors_config={
        "text-dense": models.VectorParams(size=1536, distance=models.Distance.COSINE),
    },
    sparse_vectors_config={
        "text-sparse": models.SparseVectorParams(
            index=models.SparseIndexParams(on_disk=False)
        )
    },
)
```
[Source: `https://qdrant.tech/articles/sparse-vectors/` · fetched 2026-05-18]

Storage cost is the headline win: for a 1 M-document corpus at 100 sparse tokens/doc, sparse storage is **~1.12 GB vs 6.144 GB for dense BERT and 12.288 GB for OpenAI 1 536-d** — roughly a 10× memory reduction [Source: `https://qdrant.tech/articles/sparse-vectors/` · fetched 2026-05-18].

For Memex this means **adding a sparse `path` vector alongside the existing 5 dense vectors is essentially free** at our scale (thousands of sessions, not millions). The Qdrant Rust client supports it: `qdrant-client 1.18` exposes `SparseVectorParams`, `SparseIndexParams`, and the unified Query API (`QueryPointsBuilder`) used everywhere in `indexer.rs` [Source: `https://docs.rs/qdrant-client/latest/qdrant_client/qdrant/index.html` · fetched 2026-05-18]. Server-side support is **not the blocker** — client-side embedding generation is.

### Server-side BM25 tokenization (new in 1.15.2)

A relevant late-2025 development: since **Qdrant 1.15.2 the server itself can compute BM25 sparse vectors** from raw input text, removing the need for any client-side sparse encoder for plain BM25:

> "Since Qdrant's release 1.15.2, the conversion to BM25 sparse vectors happens directly in Qdrant, for all the supported Qdrant clients."
> [Source: `https://qdrant.tech/articles/sparse-vectors/` (via search summary) · fetched 2026-05-18]

This is enabled by the `Qdrant/bm25` HuggingFace artifact (a tokenizer + IDF table, not a neural model) and Qdrant's built-in `Modifier.IDF` on the sparse index:

> "BM25 (Best Matching 25) is a ranking function used by search engines to estimate the relevance of documents to a given search query."
> "This model is supposed to be used with Qdrant. Vectors have to be configured with `Modifier.IDF`."
> [Source: `https://huggingface.co/Qdrant/bm25` · fetched 2026-05-18]

For Memex this is significant: **plain BM25 on `path` requires zero changes to `fastembed-rs`** and zero new ONNX downloads. We send raw path strings; the server tokenizes + IDF-weights them.

---

## 3. BM42 — current state

BM42 is Qdrant's experimental "BM25 + transformer attention" sparse scoring approach. It was announced 2024-07-01 and shipped with Qdrant v1.10.0 [Source: `https://qdrant.tech/articles/bm42/` · fetched 2026-05-18].

**Production status:** still flagged experimental by Qdrant itself. The article includes an honest disclaimer added after community pushback on the original benchmarks:

> "BM42 does not outperform BM25 implementation of other vendors. Please consider BM42 as an experimental approach, which requires further research."
> [Source: `https://qdrant.tech/articles/bm42/` · fetched 2026-05-18]

The corrected benchmark shows BM42 at precision@10 = 0.49 vs BM25's 0.45, but Qdrant concedes "When used properly, BM25 with tantivy achieves the best results" [Source: same].

**Inference path:** client-side via the **Python** `fastembed` package, specifically `SparseTextEmbedding`. The article's examples never mention `fastembed-rs` [Source: `https://qdrant.tech/articles/bm42/` · fetched 2026-05-18].

**Is BM42 in `fastembed-rs`?** **No.** As of fastembed-rs **v5.13.4** (the current crates.io release, dated 2026-04-27), the `SparseModel` enum has exactly two variants:

> "**SPLADEPPV1** — `prithivida/Splade_PP_en_v1`"
> "**BGEM3** — `BAAI/bge-m3`"
> [Source: `https://docs.rs/fastembed/latest/fastembed/enum.SparseModel.html` · fetched 2026-05-18]

The crate's Cargo features list confirms no BM42/ColBERT toggle — only `qwen3`, `nomic-v2-moe`, `image-models`, and hardware accelerators [Source: `https://github.com/Anush008/fastembed-rs/blob/main/Cargo.toml` · fetched 2026-05-18]. Recent releases (5.13.0 → 5.13.4) are bug fixes for Qwen3 and DirectML — **none mention ColBERT, BM42, BM25, SPLADE, late interaction, multivector, or sparse vectors** [Source: `https://github.com/Anush008/fastembed-rs/releases` · fetched 2026-05-18].

So the BM42 client path for a pure-Rust app is: (a) wait for fastembed-rs upstream, (b) run ONNX directly via the `ort` crate against the Qdrant/BM42 model, or (c) skip BM42 entirely and use the much simpler server-side BM25 (Qdrant 1.15.2+, no client embedder needed).

Given (a) BM42 itself is flagged experimental, (b) the precision lift over BM25 is marginal at best, and (c) server-side BM25 needs zero new Rust code, **BM42 is not the right battle for Memex in 2026-05**.

---

## 4. ColBERT v2 / late-interaction multivectors in Qdrant

Qdrant shipped native late-interaction multivector support in **v1.10**, around the same time as BM42 [Source: `https://qdrant.tech/blog/qdrant-1.10.x/` (titled "Universal Query, Built-in IDF & ColBERT Support") · fetched 2026-05-18].

**Collection config** uses a per-vector `multivector_config` with the `MAX_SIM` comparator:

```python
"output-token-embeddings": models.VectorParams(
    size=384,
    distance=models.Distance.COSINE,
    multivector_config=models.MultiVectorConfig(
        comparator=models.MultiVectorComparator.MAX_SIM,
    ),
)
```
[Source: `https://qdrant.tech/articles/late-interaction-models/` · fetched 2026-05-18]

The Rust client mirrors this: `qdrant-client 1.18` exposes `MultiVectorConfigBuilder` and `MultiVectorComparator` [Source: `https://docs.rs/qdrant-client/latest/qdrant_client/qdrant/index.html` · fetched 2026-05-18].

**Storage cost — the painful number.** A ColBERT document is *N tokens × 128 dims*, not a single 384-d vector. The article compares the canonical 384-d MiniLM dense baseline (with uint8 quantization) against `colbert-ir/colbertv2.0`:

> "with uint8 scalar quantization, the dense model achieves a 25% reduction in memory usage compared to ColBERT's 512 bytes per vector"
> [Source: `https://qdrant.tech/articles/late-interaction-models/` · fetched 2026-05-18]

That "512 bytes per vector" is *per token*, and ColBERT emits up to ~256 tokens per doc, so the per-document overhead vs a single 384-d dense vector is **roughly 50–100×** in the worst case before quantization. For Memex's ~thousands of sessions this is still tolerable on a laptop (tens of MB → low GB), but it is the only Memex vector type where storage would meaningfully grow.

**Is ColBERT v2 in `fastembed-rs`?** **No.** The Rust crate's public surface, verified against docs.rs for v5.13.4:

> "**Structs:** `TextEmbedding`, `ImageEmbedding`, `SparseTextEmbedding`, `TextRerank`, `UserDefinedEmbeddingModel`, `UserDefinedImageEmbeddingModel`, `UserDefinedRerankingModel`, and `UserDefinedSparseModel`.
> **Enums:** `EmbeddingModel`, `ImageEmbeddingModel`, `RerankerModel`, `SparseModel`, `OnnxSource`, `Pooling`, and `QuantizationMode`."
> [Source: `https://docs.rs/fastembed/latest/fastembed/` · fetched 2026-05-18]

No `LateInteractionTextEmbedding`, no `MultiVectorTextEmbedding`, no `ColBERT*` variant anywhere. The Python `qdrant/fastembed` *does* have `LateInteractionTextEmbedding` with `colbert-ir/colbertv2.0` as the default [Source: `https://qdrant.github.io/fastembed/examples/ColBERT_with_FastEmbed/` · fetched 2026-05-18], but the Rust port has not picked it up.

This is the **central finding**: the README's claim that "`fastembed-rs` 5.x doesn't yet ship the model" (`README.md:113`, `README.md:440`) is **still factually correct on 2026-05-18**. The fallback path the README names — `ort` crate + ONNX Jina-ColBERT-v2 — remains the only pure-Rust route, and it requires writing the token-pooling, MaxSim, and ONNX session-management code that fastembed-rs would normally hand us.

There *is* an escape hatch worth noting: fastembed-rs exposes `UserDefinedEmbeddingModel` and accepts an `OnnxSource`. In principle we could load a ColBERT ONNX export through that path, but we would still need to write the per-token output handling ourselves; it is not a turnkey solution.

---

## 5. MUVERA — multivector compressed into a single vector

This is the most important new development since the Memex README was written. Qdrant published "MUVERA: Making Multivectors More Performant" on **2025-09-05**, authored by Kacper Łukawski [Source: search summary for `https://qdrant.tech/articles/muvera-embeddings/` · fetched 2026-05-18].

**What MUVERA does.** It converts a variable-length multi-vector (ColBERT-style) into a *single, fixed-dimensional vector* called an FDE (Fixed Dimensional Encoding) via three steps: SimHash clustering, cluster-wise aggregation (averaged for docs, summed for queries), and random projection. The single FDE can be indexed with normal HNSW; the original multivector is kept only for reranking the top-N FDE hits [Source: `https://qdrant.tech/articles/muvera-embeddings/` · fetched 2026-05-18].

**Concrete trade-off (BeIR nfcorpus benchmark, ColBERTv2):**

- MUVERA-only retrieval: NDCG@10 = 0.242 (about 67% of full multi-vector performance, NDCG@10 = 0.347)
- MUVERA initial + ColBERT reranking: NDCG@10 = 0.343 (nearly identical to full ColBERT)
- Speed: **~7× faster end-to-end** — search time drops from 1.27 s to 0.18 s
[Source: `https://qdrant.tech/articles/muvera-embeddings/` · fetched 2026-05-18]

**Storage caveat.** The FDE is *not* a small vector. Default parameters (`k_sim=6` clusters, `dim_proj=32`, `r_reps=20`, 128-d token vectors) yield **40 960-dimensional FDEs** [Source: `https://qdrant.tech/articles/muvera-embeddings/` · fetched 2026-05-18]. Compression via product quantization can reduce this **~32×** with minimal recall loss, but you still need to store both the FDE and the original multivector representation if you want the reranking quality.

**fastembed support.** "FastEmbed version 0.7.2+ supports MUVERA embeddings" [Source: search summary for `https://qdrant.tech/articles/muvera-embeddings/` · fetched 2026-05-18]. That refers to the Python package. The **Rust** fastembed-rs has no MUVERA primitives (since it has no ColBERT primitives to compress in the first place).

**When to use MUVERA vs raw ColBERT vs dense:**

| Need | Choose |
|---|---|
| Hard wall on storage, single-vector index is the constraint | Dense (BGE-small-style) |
| Want best possible ranking, storage budget OK, latency budget OK | Raw ColBERT (MaxSim over multivector) |
| Want ColBERT-quality ranking, single-vector retrieval speed | MUVERA for retrieval + ColBERT for rerank |

For Memex MUVERA is the *strategically* interesting option (it directly addresses the README's "ColBERT cost is the blocker" concern), but it is **gated by ColBERT being available in the first place** — and ColBERT itself is gated by fastembed-rs. So MUVERA is downstream of the same blocker.

---

## 6. Hybrid search via Qdrant Query API

The Query API ("Universal Query") was introduced in Qdrant 1.10 and is the official way to combine dense + sparse + multivector lookups server-side. Pattern:

```python
client.query_points(
    collection_name="my_collection",
    prefetch=[
        models.Prefetch(
            query=models.SparseVector(indices=[1, 42], values=[0.22, 0.8]),
            using="sparse",
            limit=20,
        ),
        models.Prefetch(
            query=[0.01, 0.45, 0.67],  # dense vector
            using="dense",
            limit=20,
        ),
    ],
    query=models.FusionQuery(fusion=models.Fusion.RRF),
)
```
[Source: `https://qdrant.tech/documentation/concepts/hybrid-queries/` · fetched 2026-05-18]

Mechanics per the docs:

> "whenever a query has at least one prefetch, Qdrant will: 1) Perform the prefetch query (or queries), 2) Apply the main query over the results of its prefetch(es)."
> [Source: same]

**Fusion modes available** (2026-05):

- **`Fusion.RRF`** — classic reciprocal rank fusion, score = Σ 1 / (k + rank), default k=2.
- **`RrfQuery(rrf=Rrf(k=60))`** — custom RRF constant since Qdrant 1.16.0. Higher k flattens the rank-weight curve.
- **`RrfQuery(rrf=Rrf(weights=[3.0, 1.0]))`** — weighted RRF since Qdrant 1.17.0. "results from the first prefetch are weighted 3× higher than the second."
- **`Fusion.DBSF`** — Distribution-Based Score Fusion. "Normalizes the scores of the points in each query, using the mean +/- the 3rd standard deviation as limits, and then sums the scores."

[Source: `https://qdrant.tech/documentation/concepts/hybrid-queries/` · fetched 2026-05-18]

Rust client support: `qdrant-client 1.18` includes `QueryPointsBuilder` and the prefetch/fusion enums used by these examples [Source: `https://docs.rs/qdrant-client/latest/qdrant_client/qdrant/index.html` · fetched 2026-05-18].

### Compared to Memex's current pattern

Memex's `lens_search()` (`indexer.rs:539-615`) dispatches **one `QueryPointsBuilder` per non-zero-weight vector in parallel via `try_join_all`**, then does a *weighted score combine in Rust* (not RRF). The function's own comment is honest about this:

> "Runs one cosine search per named vector whose weight > 0, then performs a weighted score combine in Rust (true weighted blend — not RRF rank fusion)." (`indexer.rs:535-537`)

This is a defensible design choice for the dense-only lens UI (sliders multiply directly into the combine), but it has two costs:

1. **N round trips instead of 1.** Five dense vectors with weight > 0 = five gRPC calls. With the Query API and `prefetch`, this becomes one call regardless of how many sub-queries.
2. **No native sparse path.** As soon as a sparse vector enters the mix, the Rust-side combine has to normalize across two fundamentally different score scales (cosine ∈ [-1, 1] vs IDF-weighted dot product ∈ [0, ∞)). DBSF or RRF handle this server-side; a hand-rolled weighted sum will under-rank one side or the other depending on corpus statistics.

The migration path is mechanical: keep the per-vector slider UI, but build prefetches dynamically and let Qdrant fuse with `RrfQuery(rrf=Rrf(weights=...))`. This is the right shape once we add even one sparse vector.

---

## 7. fastembed-rs status verification — the load-bearing section

This is the section that determines whether the README's deferral notes are stale or accurate.

**Current crate version on crates.io: `5.13.4`, released 2026-04-27.**
[Source: `https://github.com/Anush008/fastembed-rs/releases` · fetched 2026-05-18; `https://github.com/Anush008/fastembed-rs/blob/main/Cargo.toml` · fetched 2026-05-18 (`version = "5.13.4"`)]

The recent release timeline shows the maintainer is active but focused elsewhere:

- v5.13.4 (2026-04-27) — Qwen3 F16 dtype mismatch fixes
- v5.13.3 (2026-04-21) — Dependency bumps
- v5.13.2 (2026-04-10) — DirectML session options fix
- v5.13.1 (2026-04-06) — Documentation updates
- v5.13.0 (2026-03-16) — Qwen3Model type export

> "The provided release notes contain **no mentions** of ColBERT, BM42, BM25, SPLADE, late interaction, multivector, or sparse vectors."
> [Source: `https://github.com/Anush008/fastembed-rs/releases` · fetched 2026-05-18]

**Public API surface (v5.13.4):**

> "**Structs:** `TextEmbedding`, `ImageEmbedding`, `SparseTextEmbedding`, `TextRerank`, `UserDefinedEmbeddingModel`, `UserDefinedImageEmbeddingModel`, `UserDefinedRerankingModel`, and `UserDefinedSparseModel`.
> **Enums:** `EmbeddingModel`, `ImageEmbeddingModel`, `RerankerModel`, `SparseModel`, `OnnxSource`, `Pooling`, and `QuantizationMode`."
> [Source: `https://docs.rs/fastembed/latest/fastembed/` · fetched 2026-05-18]

**`SparseModel` enum — verbatim:**

> "1. **SPLADEPPV1** — `prithivida/Splade_PP_en_v1`
> 2. **BGEM3** — `BAAI/bge-m3`"
> [Source: `https://docs.rs/fastembed/latest/fastembed/enum.SparseModel.html` · fetched 2026-05-18]

**Dense `EmbeddingModel` registry (abridged, README list):**

> "BAAI/bge-small-en-v1.5 (Default), BAAI/bge-base-en-v1.5, BAAI/bge-large-en-v1.5, BAAI/bge-small-zh-v1.5, BAAI/bge-large-zh-v1.5, BAAI/bge-m3, sentence-transformers/all-MiniLM-L6-v2 […] approximately 20 additional dense embedding models including options from Nomic AI, Alibaba NLP, Jina, Google, and Snowflake. Quantized variants are available for several models (denoted with a 'Q' suffix)."
> [Source: `https://github.com/Anush008/fastembed-rs` · fetched 2026-05-18]

**Verdict on the README's deferral note (`README.md:440-441`):**

| Memex README claim | Status 2026-05-18 |
|---|---|
| "ColBERT v2 inline citations are on the roadmap; `fastembed-rs` 5.x doesn't yet ship the model" | **Still accurate.** No `LateInteractionTextEmbedding`, no `ColBERT*` variant, no MUVERA primitives. Fallback path (`ort` + ONNX Jina-ColBERT-v2) remains the only pure-Rust route. |
| "BM42 sparse on `path` vector — same upstream gap" | **Still accurate** for BM42 specifically. **Partially stale** for the broader goal: plain BM25 no longer needs a client embedder at all (Qdrant 1.15.2 server-side BM25), and SPLADE *is* exposed in fastembed-rs as `SparseModel::SPLADEPPV1`. |

**Concrete actionable conclusion:** the deferral is correct for the specific models named (ColBERT, BM42), but Memex is leaving two alternative paths on the table that *do* work today in pure Rust:

1. **Server-side BM25 on `path`** — needs only a Qdrant collection config change, zero new Rust code, zero new ONNX downloads. The Qdrant server tokenizes the path string itself.
2. **SPLADE on `path` via fastembed-rs** — `SparseModel::SPLADEPPV1` is already in our dependency. It adds ~40 MB of ONNX to the cache and uses an `ort` session we don't yet construct, but it is a turnkey enum variant, not net-new ONNX plumbing.

---

## 8. Recommendation for Memex

| Option | Verdict | Rank-quality impact (qualitative) | Implementation cost | Risk |
|---|---|---|---|---|
| **Sparse on `path`** — server-side BM25 (Qdrant 1.15.2+ tokenizer) | **Adopt now** | Large — exact-token recall for filenames, function names, error codes that BGE-small flattens | Low: 1 new sparse named vector in `ensure_collection()`, 1 new prefetch in `lens_search()`. No new fastembed model, no new ONNX cache. | Low. Server-side BM25 path is stable across Qdrant 1.15.2 → 1.18. Cap upgrade risk by gating behind a `MEMEX_HYBRID=1` env flag for one release. |
| **Sparse on `path`** — SPLADE via `fastembed-rs` | **Wait** (vs the BM25 option) | Modest delta over BM25 for code/path tokens, larger for natural language. Marginal vs BM25 for Memex's `path` vector specifically. | Medium: instantiate a second fastembed model (`SparseTextEmbedding`), wire indices/values into the `PointStruct`, add ~40 MB to the fastembed cache. | Medium. Adds a second long-lived ONNX session in the desktop process. BM25 gets ~80 % of the benefit at ~10 % of the cost. |
| **BM42 on `path`** | **Skip for now** | Qdrant itself admits BM42 does not outperform tantivy BM25 in practice. | High: not in `fastembed-rs`. Would need ONNX export + manual `ort` integration. | High. Experimental, no Rust client support, marginal lift over BM25. Revisit only if upstream `fastembed-rs` adds it. |
| **ColBERT v2 on `content`** | **Wait** | Very large for short-query → long-document ranking, which is Memex's dominant query shape. Best single rank-quality lever available. | Currently high: `fastembed-rs` does not expose `LateInteractionTextEmbedding`. Pure-Rust path requires `ort` + Jina-ColBERT-v2 ONNX + writing the MaxSim pooling. ~3–4 days of focused work. | Medium-high. Storage grows 50–100× *on the content vector specifically* (other 4 vectors unchanged). Watch the fastembed-rs issue tracker; the moment a `LateInteractionTextEmbedding` lands in fastembed-rs, this drops to "trivial" and should be reclassified Adopt. |
| **MUVERA (ColBERT → single FDE)** | **Skip until ColBERT lands** | Closes 70 % → ~99 % of the ColBERT quality gap once ColBERT is enabled, with 7× retrieval speedup. | Strictly downstream of ColBERT being available client-side. No standalone path. | Same dependency as ColBERT. |
| **Hybrid via Query API + Weighted RRF** | **Adopt now** | Moderate quality lift on its own; large lift the moment any sparse vector is added (replaces fragile hand-rolled score combine across mismatched score scales). | Low-Medium: rewrite `lens_search()` to build a `Prefetch` list and submit one Query API call instead of N parallel `query()` calls. Map the existing `LensWeights` to `Rrf(weights=...)`. | Low. Pure refactor — no new dependencies, no new models. Test parity against the current weighted-sum behavior with `weights=[1,1,1,1,1]`. |

**Single most actionable upgrade:** add **server-side BM25 on `path`** + migrate `lens_search()` to the **Query API with weighted RRF**. Together these two changes deliver most of the rank-quality win the README anticipated from BM42, with no new fastembed-rs dependency and no new ONNX downloads. Defer ColBERT (and therefore MUVERA) until fastembed-rs exposes `LateInteractionTextEmbedding` upstream — at which point the README's "Phase 3+" note becomes a one-PR project rather than a multi-day ONNX integration.

---

## 9. What changes in Memex if we adopt the recommended option

The recommended option is: **(a) sparse `path` via server-side BM25 + (b) Query API hybrid via weighted RRF**. Concrete code-level impact:

### 9.1 Collection schema change (`indexer.rs:134-170`, `ensure_collection`)

**Breaking** — Qdrant collections cannot retroactively add a sparse vector to an existing schema. We need to either (i) bump `COLLECTION` from `memex_sessions` to `memex_sessions_v2` and force a one-time reindex from the parsed JSONL files, or (ii) call `update_collection` (which Qdrant does accept for sparse_vectors_config in 1.x — verify against `qdrant-client 1.18` docs before relying on it). Recommendation: bump the collection name and reindex; the parser+indexer pipeline is fast and the user already pays a "first run" indexing cost.

Add to the `CreateCollectionBuilder` call site:

```rust
// existing dense vectors_cfg unchanged
let mut sparse: HashMap<String, SparseVectorParams> = HashMap::new();
sparse.insert(
    "path_bm25".into(),
    SparseVectorParamsBuilder::default()
        .modifier(Modifier::Idf)   // enable server-side IDF
        .build(),
);
client.create_collection(
    CreateCollectionBuilder::new(COLLECTION)
        .vectors_config(vectors_cfg)
        .sparse_vectors_config(SparseVectorConfig { map: sparse }),
).await?;
```

(Exact builder names to confirm against `qdrant-client 1.18`; the concept maps 1:1 onto the Python `SparseVectorParams` + `Modifier.IDF` shown in §3.)

### 9.2 Upsert path (`indexer.rs:370-394`, `build_point` / `index_session`)

For server-side BM25, the client uploads **raw text** under the sparse vector name; the Qdrant server tokenizes. Concretely: keep the existing 5 dense vectors in the `vec_map`, and add a `Document` (or equivalent) under the sparse name with the raw output of `build_path(session)` (`indexer.rs:254-281`). The exact API in `qdrant-client 1.18` for "server-side inference of sparse vector from text" should be verified against the docs (it may require `PointVectors::Document { ... }` or similar). If unavailable in the Rust client, the fallback is to ship a deterministic BM25-tokenize-yourself crate (~150 LOC) but the server-side path is preferable.

`build_path()` itself does not need to change — its newline-joined output is already a clean tokenizable string.

### 9.3 Query path (`indexer.rs:539-615`, `lens_search`)

Replace the `try_join_all` over N parallel `QueryPointsBuilder` calls with a single Query API call using `prefetch`:

```rust
let mut prefetches = Vec::new();
for (vname, w) in active {
    prefetches.push(
        PrefetchQueryBuilder::default()
            .query(Query::from(qvec.clone()))
            .using(vname.to_string())
            .limit(per_vector_limit)
            .build(),
    );
}
// Add the new sparse prefetch — query is the raw query string, not a vector.
if weights.path > 0.0 {
    prefetches.push(
        PrefetchQueryBuilder::default()
            .query(Query::from(Document::new(query)))  // server-side BM25
            .using("path_bm25")
            .limit(per_vector_limit)
            .build(),
    );
}
let req = QueryPointsBuilder::new(COLLECTION)
    .add_prefetches(prefetches)
    .query(Query::new_rrf(active_weights))  // weighted RRF
    .limit(limit)
    .with_payload(true);
let res = client.query(req).await?;
```

This replaces the entire `CombinedHit` / weighted-sum block (`indexer.rs:550-614`). The per-vector score breakdown the UI currently shows in `SearchHit.vector_scores` is no longer trivially available from a fused result; we either drop it or run a separate diagnostic query when the user opens the lens inspector.

### 9.4 Other call sites

- `search_content()` (`indexer.rs:517-531`) is a thin wrapper around `lens_search` and is unaffected as long as we preserve its signature.
- `mix_match()` (`indexer.rs:628`+), `topology()` (`indexer.rs:762`+), `recall()` (`indexer.rs:1151`+), `predict_next_actions()` (`indexer.rs:1245`+) all use independent query APIs and do not need to change in this round.

### 9.5 Cache & deps

- **fastembed cache:** no new ONNX downloads. BGE-small (~130 MB) remains the only cached model. Cache path is unchanged (`indexer.rs:101-123`).
- **Cargo.toml:** no new dependencies — `qdrant-client 1.18` already has the sparse + Query API builders. The existing `fastembed = "5"` line (`Cargo.toml:32`) is unchanged.
- **README update:** strike or amend the entries at `README.md:440-441`. BM42 line can stay (still deferred); ColBERT line should be updated to note that the underlying blocker (fastembed-rs ColBERT support) was re-verified on 2026-05-18 and remains the gating issue.

---

## Sources

- [Qdrant — What is a Sparse Vector? How to Achieve Vector-based Hybrid Search](https://qdrant.tech/articles/sparse-vectors/) · fetched 2026-05-18
- [Qdrant — BM42: New Baseline for Hybrid Search](https://qdrant.tech/articles/bm42/) · fetched 2026-05-18
- [Qdrant — Any* Embedding Model Can Become a Late Interaction Model… If You Give It a Chance!](https://qdrant.tech/articles/late-interaction-models/) · fetched 2026-05-18
- [Qdrant — MUVERA: Making Multivectors More Performant](https://qdrant.tech/articles/muvera-embeddings/) (search summary) · fetched 2026-05-18
- [Qdrant — Hybrid and Multi-Stage Queries (Query API)](https://qdrant.tech/documentation/concepts/hybrid-queries/) · fetched 2026-05-18
- [Qdrant — Vectors documentation](https://qdrant.tech/documentation/concepts/vectors/) · fetched 2026-05-18
- [Qdrant — 1.10 release: Universal Query, Built-in IDF & ColBERT Support](https://qdrant.tech/blog/qdrant-1.10.x/) (search summary) · fetched 2026-05-18
- [GitHub — Anush008/fastembed-rs README](https://github.com/Anush008/fastembed-rs) · fetched 2026-05-18
- [GitHub — Anush008/fastembed-rs releases](https://github.com/Anush008/fastembed-rs/releases) · fetched 2026-05-18
- [GitHub — Anush008/fastembed-rs Cargo.toml](https://github.com/Anush008/fastembed-rs/blob/main/Cargo.toml) · fetched 2026-05-18
- [docs.rs — fastembed crate root (v5.13.4)](https://docs.rs/fastembed/latest/fastembed/) · fetched 2026-05-18
- [docs.rs — fastembed::SparseModel enum](https://docs.rs/fastembed/latest/fastembed/enum.SparseModel.html) · fetched 2026-05-18
- [docs.rs — qdrant-client crate (v1.18)](https://docs.rs/qdrant-client/latest/qdrant_client/qdrant/index.html) · fetched 2026-05-18
- [HuggingFace — Qdrant/bm25 model card](https://huggingface.co/Qdrant/bm25) · fetched 2026-05-18
- [Qdrant — Using SPLADE with FastEmbed (Python)](https://qdrant.tech/documentation/fastembed/fastembed-splade/) · fetched 2026-05-18
- [Qdrant — ColBERT with FastEmbed (Python)](https://qdrant.github.io/fastembed/examples/ColBERT_with_FastEmbed/) · fetched 2026-05-18 (referenced to confirm Python-only ColBERT availability)
- Memex source: `/Users/kimsejun/Documents/GitHub/memex/src-tauri/src/indexer.rs` (lines 38-40, 42, 61, 101-123, 134-170, 254-281, 311-333, 517-615) · read 2026-05-18
- Memex source: `/Users/kimsejun/Documents/GitHub/memex/src-tauri/Cargo.toml` (line 32: `fastembed = "5"`) · read 2026-05-18
- Memex source: `/Users/kimsejun/Documents/GitHub/memex/README.md` (lines 113, 280, 349, 377, 393, 440-441) · read 2026-05-18
