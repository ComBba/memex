# Qdrant Official Blog & Articles — Memex Research Catalog

**Researcher**: Claude (Opus 4.7, 1M ctx)
**Fetch date**: 2026-05-18
**Sources scoped**: `qdrant.tech/blog/*`, `qdrant.tech/articles/*`, `qdrant.tech/customers/*`, `qdrant.tech/recommendations/*` (Qdrant-staff bylines only)
**Project under analysis**: Memex — Tauri 2 desktop app, Qdrant 1.18, 5 named vectors (content / tool / path / error / code, all 384-d BGE-small), seven non-chat surfaces. Built for Vector Space Day 2026 ("Think Outside the Bot").

---

## 0. Executive snapshot

Qdrant's 2024–2026 publishing has decisively shifted away from "RAG chatbot" framing toward three themes that Memex is already on top of:

1. **Vector search as a recommendation / discovery / exploration primitive** (not a Q&A retriever).
2. **Multi-vector / named-vector / late-interaction** as the dominant relevance technique.
3. **Long-term agent memory** as the canonical "non-chatbot" Qdrant use case for 2025–26.

The official VSD 2026 hackathon prompt ("Forget the classical RAG chatbot!") confirms Memex's product framing — every one of Memex's seven surfaces aligns directly with an officially blogged Qdrant API:

| Memex surface | Officially-blogged Qdrant primitive |
|---|---|
| Time Machine stack | Payload filter + dense `content` named vector |
| Topology galaxy (MST) | Distance Matrix API (`/points/search/matrix/pairs`) — Qdrant 1.12 |
| Mix & Match | Discovery API (positive/negative anchors) — `discovery-search` article |
| Proactive Recall | Recommend API on `error` named vector + payload filter |
| Predict Next Actions | Recommend `best_score` strategy + neighbor pivot |
| Replay engine | Just payload ordering — no Qdrant magic needed |
| Lens slider | Named-vector weighted query via Universal Query API (1.10) |

The single biggest under-utilized idea from the official blog is the **"Any Embedding Model Can Become a Late Interaction Model"** technique — Memex is already shipping BGE-small-en, the exact model the article uses as its hero example, and could add MaxSim re-ranking on top of the existing 5 named vectors with no new model.

---

## 1. Cluster: Recommendation systems & Discovery API
*Highest-priority cluster for VSD 2026 fit.*

**Cluster framing**. Across 2023–2026, Qdrant has consistently positioned the Recommend, Discover, and Context APIs as the official "not-a-RAG-chatbot" toolbox. Every entry below was authored by a Qdrant employee (Łukawski, Cossío, Vasnetsov, Sukhodolskaya) and every one explicitly contrasts itself with retrieve-then-LLM Q&A. For Memex this cluster is the single most important reference — five of the seven product surfaces are direct applications of one of these APIs, and the verbatim quotes here are the copy Memex should reuse in the VSD pitch and submission write-up.

**API decision matrix as Qdrant currently teaches it**:

| Need | Officially-blessed API | Memex surface |
|---|---|---|
| "Show me things like X" with implicit dislikes | `recommend` with `average_vector` | Predict Next Actions (warm start) |
| "Score every candidate vs each positive AND each negative" | `recommend` with `best_score` | Predict Next Actions (cold start) |
| "Constrain space with (+,−) pairs, then target a third point" | `discover` | Mix & Match (target known) |
| "Constrain space, but no single target — just diverse picks" | `discover` (context-only mode) | Mix & Match (no target / exploration) |
| "Iteratively refine based on user signals, no LLM" | 1.17 Relevance Feedback | Lens slider learning

### 1.1 Deliver Better Recommendations with Qdrant's new API

- **Author**: Kacper Łukawski (Qdrant DevRel)
- **Date**: 2023-10-25
- **URL**: https://qdrant.tech/articles/new-recommendation-api/ (fetched 2026-05-18)
- **Summary**: Introduces the Recommendation API as a "multi-aim search" — find items close to positives AND far from negatives, with two strategies: `average_vector` (default; collapses examples to a single query vector) and `best_score` (evaluates each candidate against every positive and negative during HNSW traversal). Both vector IDs and raw embeddings can be passed. Demoed via Food Discovery: like/dislike images mixed with optional text queries.
- **Relevance for Memex**: `Foundational`. Memex's **Predict Next Actions** surface is essentially this API applied to neighbor-action embeddings — `best_score` strategy is the right default because the user has very few liked sessions.
- **Key quote**: "Recommendations might be seen as a multi-aim search, where we want to find items close to positive and far from negative examples." — https://qdrant.tech/articles/new-recommendation-api/

### 1.2 Discovery needs context

- **Author**: Luis Cossío (Qdrant engineer)
- **Date**: 2024-01-31
- **URL**: https://qdrant.tech/articles/discovery-search/ (fetched 2026-05-18)
- **Summary**: Defines the Discovery API as a triplet-loss-inspired exploration tool: each "context pair" is a (positive, negative) vector pair that carves a hyperplane in vector space; the search target then explores the intersection of all positive half-spaces. Distinct from Recommend: Recommend selects from points; Discovery partitions space. Two modes: **Discovery Search** (target + context pairs) and **Context Search** (context only → diverse results in a constrained zone).
- **Relevance for Memex**: `Foundational`. Memex's **Mix & Match** surface is directly Context Search — and the article's "fish pizza" example is the canonical demo of the same concept Memex sells as "what would the past-you do with these competing signals."
- **Key quote**: "With a context, we can define hyperplanes within the vector space, which always prefer the positive over the negative vectors." — https://qdrant.tech/articles/discovery-search/

### 1.3 Food Discovery Demo

- **Author**: Kacper Łukawski
- **Date**: 2023-09-05
- **URL**: https://qdrant.tech/articles/food-discovery-demo/ (fetched 2026-05-18; canonical slug at `practicle-examples`)
- **Summary**: The reference "no-text-query" exploration UX. Three tricks worth stealing: (1) **negated-vector** trick — if a user has only dislikes, negate the mean and use it as a query (works because cosine of −v gives the antipode); (2) **cold-start via random sampling** of vector space; (3) **search groups API** to enforce diversity (one hit per restaurant).
- **Relevance for Memex**: `Suggests new surface`. The negated-vector trick is a free upgrade for Memex's Proactive Recall when the user has marked sessions as "not what I meant." The diversity-via-grouping pattern also gives Memex a clean way to dedupe Topology nodes by repo path.
- **Key quote**: "You might be craving something sweet, but you don't know what." — https://qdrant.tech/articles/food-discovery-demo/

### 1.4 Vector Similarity: Going Beyond Full-Text Search

- **Author**: Luis Cossío
- **Date**: 2023-08-08
- **URL**: https://qdrant.tech/articles/vector-similarity-beyond-search/ (fetched 2026-05-18)
- **Summary**: A taxonomy of non-RAG uses of vector similarity: (a) **dissimilarity / anomaly** — invert the search objective to find outliers; (b) **diversity sampling** — pick points to maximize pairwise distance for unbiased overviews; (c) **dual recommendation modes** — feature-averaging vs relative-distance; (d) **discovery as reversed triplet loss**.
- **Relevance for Memex**: `Foundational`. This is the manifesto Memex's positioning sentence ("seven non-chat surfaces") is built on. The dissimilarity trick is a cheap addition to the Replay engine for "find a session that's the opposite of this one."
- **Key quote**: "We can easily achieve a dissimilarity search by inverting the search objective." — https://qdrant.tech/articles/vector-similarity-beyond-search/

### 1.5 Relevance Feedback in Qdrant

- **Author**: Evgeniya Sukhodolskaya (Qdrant solutions / research engineer)
- **Date**: 2026-02-20
- **URL**: https://qdrant.tech/articles/relevance-feedback/ (fetched 2026-05-18)
- **Summary**: Native-to-the-engine feedback loop introduced in 1.17: at query time, Qdrant applies a learned formula `f(retriever_score, feedback_confidence, delta)` during HNSW traversal — adjusting scores *across the whole index*, not just re-ordering top-k. Needs only 50–300 labelled queries to calibrate. Modality-agnostic. **No LLM at query time** — purely a scoring tweak.
- **Relevance for Memex**: `Suggests new surface`. Memex's Lens slider could expose a "this was useful / not useful" thumbs UI per session card; collect ~100 signals; calibrate; the Lens permanently learns the user's session-relevance bias without any model retraining. Honors the "no LLM at runtime" invariant.
- **Key quote**: "It is cheap to use, as the time and resources spent on obtaining relevance feedback are minimal." — https://qdrant.tech/articles/relevance-feedback/

---

## 2. Cluster: Multi-vector / named-vectors / late interaction

**Cluster framing**. The arc of this cluster is: 2024 introduced ColBERT/multi-vector as a native primitive (1.10); 2024-Q4 demonstrated that *any* dense encoder can be retrofitted to late interaction by keeping its per-token outputs; 2025 expanded the visual side (ColPali, ColQwen); 2025-Q3 brought MUVERA as the canonical scaling fix. The current Qdrant position is unambiguous: **single-vector-per-document is no longer the default**. Memex's "5 named vectors per session" is one valid interpretation of the multi-vector philosophy (separate semantic facets), and a complementary interpretation — per-token late interaction inside each named vector — is a free upgrade because Memex already uses the exact model the official article (§2.1) demonstrates with.

**Two readings of "multi-vector" Memex should keep distinct**:

1. **Named vectors** — multiple *independent* vector spaces per point, each indexed separately with its own HNSW. Optimal when each space has different semantics. *This is what Memex does today (content/tool/path/error/code).*
2. **Multi-vector value** (a.k.a. ColBERT-style) — a *list of vectors* inside a single named slot, scored with MaxSim. Optimal for token-level relevance. *This is what Memex could add as a 6th named slot.*

These can be combined: Memex's hypothetical future `content_late` would be a named vector whose value is itself a multi-vector list.

### 2.1 Any* Embedding Model Can Become a Late Interaction Model

- **Author**: Kacper Łukawski
- **Date**: 2024-08-14
- **URL**: https://qdrant.tech/articles/late-interaction-models/ (fetched 2026-05-18)
- **Summary**: Standard dense models discard per-token embeddings when pooling to a single vector. If you instead **store the per-token output embeddings**, you get a free late-interaction reranker via MaxSim — and the article shows `BAAI/bge-small-en` (22M parameters) beating purpose-built late-interaction models and SPLADE on multiple BEIR datasets. Scalar quantization recovers 75% of the storage at negligible quality loss.
- **Relevance for Memex**: `Suggests new surface` — and this is the **highest-leverage idea in the whole corpus** for Memex, because Memex is already using `BAAI/bge-small-en`. Memex could add a 6th named vector `content_late` storing per-token embeddings (HNSW disabled, used only at rerank time) and instantly upgrade Time Machine relevance with no new model and no LLM at runtime.
- **Key quote**: "Standard dense embedding models can be effectively adapted for late interaction scenarios using output token embeddings." — https://qdrant.tech/articles/late-interaction-models/

### 2.2 Master Multi-Vector Search With Qdrant (course launch)

- **Author**: Neil Kanungo (Qdrant)
- **Date**: 2026-03-24
- **URL**: https://qdrant.tech/blog/multi-vector-search-course/ (fetched 2026-05-18)
- **Summary**: Announces the official 4-module course covering ColBERT text multi-vectors, ColPali for visual docs, MaxSim, and billion-scale optimization (incl. MUVERA). Confirms that as of 2026, Qdrant officially treats multi-vector / late interaction as the "default next step" beyond single-vector search.
- **Relevance for Memex**: `Reinforces existing surface`. Memex's 5-named-vector design isn't multi-vector-per-vector — but the course's framing ("most tutorials stop at one document, one vector") is exactly the talking point Memex should use at VSD.
- **Key quote**: "Most vector search tutorials stop at single-vector embeddings: one document, one vector, one similarity score." — https://qdrant.tech/blog/multi-vector-search-course/

### 2.3 MUVERA: Making Multivectors More Performant

- **Author**: Kacper Łukawski
- **Date**: 2025-09-05
- **URL**: https://qdrant.tech/articles/muvera-embeddings/ (fetched 2026-05-18)
- **Summary**: Fixed-Dimensional Encoding for multi-vector retrieval: SimHash + random projection compress variable-length token sequences into a single 40,960-dim vector. ~7x faster than full MaxSim, 99% accuracy when paired with a final-stage rerank. Document-side and query-side aggregation differ (avg vs sum).
- **Relevance for Memex**: `Tangential` today (Memex doesn't yet have multi-vector per session). Becomes `Suggests new surface` IF Memex adopts §2.1 (late-interaction per token from BGE) — then MUVERA is the standard speed-up.
- **Key quote**: "The paper recommends using it as an initial retrieval step followed by reranking with the original multi-vector representation." — https://qdrant.tech/articles/muvera-embeddings/

### 2.4 Advanced Retrieval with ColPali & Qdrant Vector Database

- **Author**: Sabrina Aquino (Qdrant DevRel)
- **Date**: 2024-11-05
- **URL**: https://qdrant.tech/blog/qdrant-colpali/ (fetched 2026-05-18)
- **Summary**: ColPali (vision LM) emits ~1,030 patch embeddings per PDF page; late-interaction MaxSim retrieves over the patch grid. Binary Quantization gives ~50% search speedup over Scalar on this workload.
- **Relevance for Memex**: `Tangential`. Memex is code-not-PDF, but the **patch-grid → MaxSim** pattern is the cleanest mental model for "Lens slider weights the contribution of each named vector."
- **Key quote**: "ColPali generates contextualized multivector embeddings directly from an image of a document page." — https://qdrant.tech/blog/qdrant-colpali/

### 2.5 Optimizing ColPali for Retrieval at Scale, 13x Faster Results

- **Authors**: Evgeniya Sukhodolskaya & Sabrina Aquino
- **Date**: 2024-11-27
- **URL**: https://qdrant.tech/blog/colpali-qdrant-optimization/ (fetched 2026-05-18)
- **Summary**: Mean-pool the 32×32 patch grid → 38 vectors per page, retrieve top 200 with the pooled set, then rerank top 200 with the full 1,030-vector representation. NDCG@20 = 0.952, almost no quality loss.
- **Relevance for Memex**: `Reinforces existing surface`. The two-stage retrieve-then-rerank pattern is the right architecture for any future Memex re-ranker.
- **Key quote**: "ColPali generates 1,030 vectors for just one page of a PDF. While this is manageable for small-scale tasks…" — https://qdrant.tech/blog/colpali-qdrant-optimization/

---

## 3. Cluster: Distance Matrix API + similarity graphs

**Cluster framing**. Only two articles in the entire qdrant.tech corpus speak directly to the Distance Matrix API — one is the original 1.12 release note, and the other is the CTO's own explainer five months later. Both confirm that this API exists precisely because Qdrant believes vector exploration (UMAP, clustering, graph viz, MST) should be a *server-side* operation, not a client-side post-processing step. Memex's Topology galaxy is the only product the researcher could find in the published Qdrant universe that actually uses `search_matrix_pairs` for an end-user interactive visualization — meaning Memex has a clean differentiation story at VSD: "we are the reference consumer of the API the CTO blogged about."

### 3.1 Distance-based data exploration

- **Author**: Andrey Vasnetsov (Qdrant CTO)
- **Date**: 2025-03-11
- **URL**: https://qdrant.tech/articles/distance-based-exploration/ (fetched 2026-05-18)
- **Summary**: The CTO's own piece on why Distance Matrix exists. Three downstream uses that Qdrant officially endorses: (1) feed pairwise distances into **UMAP** with `metric="precomputed"`; (2) feed into **KMeans**/clustering; (3) build an **interactive similarity graph** using sampling + spanning trees to keep the visualization tractable.
- **Relevance for Memex**: `Foundational` — this is literally the playbook Memex's Topology galaxy implements. The article's "interactive graph visualization … spanning tree" guidance is the official endorsement of MST as the right scaling strategy.
- **Key quote**: "Adding new data points to the graph is as straightforward as inserting new nodes and edges without the need to re-run any training steps." — https://qdrant.tech/articles/distance-based-exploration/

### 3.2 Qdrant 1.12 — Distance Matrix, Facet Counting & On-Disk Indexing

- **Author**: David Myriel (Qdrant DevRel)
- **Date**: 2024-10-08
- **URL**: https://qdrant.tech/blog/qdrant-1.12.x/ (fetched 2026-05-18)
- **Summary**: The release that *introduced* Distance Matrix. Two endpoints: `/points/search/matrix/pairs` (graph-friendly) and `/points/search/matrix/offsets` (CSR-friendly for SciPy). Also brought Facet Counting and on-disk text/geo indexes.
- **Relevance for Memex**: `Foundational`. Memex's Topology uses the `pairs` endpoint; this post is the canonical citation for *why* that endpoint exists.
- **Key quote**: "A retail company with 10,000 customers wants to segment them by purchasing behavior … without a dedicated API, clustering would need 10,000 separate batch requests." — https://qdrant.tech/blog/qdrant-1.12.x/

---

## 4. Cluster: Hybrid search, sparse vectors, BM42

**Cluster framing**. Hybrid search is Qdrant's biggest *publicly-corrected* line of work in the period under review: BM42 (§4.1) was launched as a "new baseline" in July 2024, then officially retracted as not-yet-production-grade by the same author. The lesson Qdrant has internalized — visible in the 2024-10 modern sparse retrieval survey (§4.3) and the 2024-07 hybrid revamp (§4.2) — is that **dense + (BM25 OR SPLADE++) fused by RRF, optionally reranked by ColBERT-style late interaction**, is the durable recipe. Memex doesn't currently run a sparse channel; this cluster is here primarily so the team has the official guidance if/when a sparse channel is added (most likely candidate: a BM25 channel over the *raw prompt text* of a session, to complement the BGE `content` vector for exact-phrase queries like "S3 EROFS").

### 4.1 BM42: New Baseline for Hybrid Search

- **Author**: Andrey Vasnetsov (CTO)
- **Date**: 2024-07-01
- **URL**: https://qdrant.tech/articles/bm42/ (fetched 2026-05-18)
- **Summary**: BM42 uses transformer [CLS] attention as the IDF-weighted term importance signal, reversing WordPiece tokenization to recover whole-word weights. Fast (no token expansion), interpretable, multilingual. **Important correction**: Qdrant later acknowledged BM42 did NOT in fact outperform BM25 in their own benchmarks and is "experimental, requiring further development." Effectively a partial reversal.
- **Relevance for Memex**: `Tangential`. Memex is dense-only today; if a sparse channel is ever added, the official 2026 guidance is to prefer BM25 over BM42 and watch the miniCOIL line of work (§4.3).
- **Key quote**: "BM42 can support any natural language as long as there is a transformer model for it." — https://qdrant.tech/articles/bm42/

### 4.2 Hybrid Search Revamped — Building with Qdrant's Query API

- **Author**: Kacper Łukawski
- **Date**: 2024-07-25
- **URL**: https://qdrant.tech/articles/hybrid-search/ (fetched 2026-05-18)
- **Summary**: Establishes the modern hybrid recipe: (1) retrieve with dense, (2) retrieve with sparse, (3) fuse with **Reciprocal Rank Fusion** (linear combination of raw scores doesn't work — sparse and dense scores aren't on the same scale), (4) optionally rerank with late interaction. The Universal Query API does all four in one round-trip.
- **Relevance for Memex**: `Reinforces existing surface`. Memex's Lens slider isn't hybrid (no sparse channel) — but the principle that scores from different modalities aren't linearly comparable is the same warning Memex should heed when blending its 5 named-vector scores.
- **Key quote**: "None of the linear formulas would be able to distinguish between them." — https://qdrant.tech/articles/hybrid-search/

### 4.3 Modern Sparse Neural Retrieval: From Theory to Practice

- **Author**: Evgeniya Sukhodolskaya
- **Date**: 2024-10-23
- **URL**: https://qdrant.tech/articles/modern-sparse-neural-retrieval/ (fetched 2026-05-18)
- **Summary**: Survey article. Walks from DeepCT → SPLADE → SPLADE++ → COIL → miniCOIL. Key practical points: internal document expansion beats external (`docT5query`); SPLADE++ is the production-grade sparse model for medical/legal/e-commerce; COIL's 32-dim per token captures homonyms but storage is the gating factor.
- **Relevance for Memex**: `Tangential` (Memex is dense-only). Useful read if Memex ever indexes long Markdown blobs from session prompts.
- **Key quote**: "Sparse neural retrieval can be a valuable option for scaling, especially when working with large datasets." — https://qdrant.tech/articles/modern-sparse-neural-retrieval/

### 4.4 What is a Sparse Vector?

- **Author**: Nirant Kasliwal (formerly Qdrant)
- **Date**: 2023-12-09
- **URL**: https://qdrant.tech/articles/sparse-vectors/ (fetched 2026-05-18)
- **Summary**: Primer. 20–200 non-zero dims per doc, ~10x memory reduction vs dense, SPLADE-style term expansion. Hybrid sparse+dense covers both exact-keyword and semantic matches.
- **Relevance for Memex**: `Tangential`.
- **Key quote**: "Sparse vectors focus on relative word weights per document, with most values being zero." — https://qdrant.tech/articles/sparse-vectors/

---

## 5. Cluster: Agent / chat memory / long-term memory patterns

**Cluster framing**. This is the cluster Qdrant has been doubling down on most aggressively in 2025–2026: the 2025 recap titled itself "Powering the Agentic Era," the dedicated Skills release (§5.3) targets Cursor and Claude Code by name, and the Agentic Builders Guide (§5.2) reads like an SLO checklist for memory-backed agents. Memex's positioning thesis — "your Claude Code session history is the agent's externalized long-term memory" — is the most literal possible reading of how Qdrant currently markets itself.

**Five concrete patterns from the cluster Memex can borrow without violating its no-LLM-at-runtime invariant**:

1. **Layer split** (CrewAI taxonomy in §5.1): short-term / long-term / entity / contextual. Memex's existing payload schema already maps cleanly — `tool_name` is entity memory, `error_seen` is contextual.
2. **Recency decay** (§5.2): apply a `created_at`-derived multiplier in a Score-Boosting expression (§6.3) so Predict Next Actions naturally prefers recent sessions.
3. **Millisecond upserts** (§5.2): Memex's indexer should write a new session's 5 vectors as soon as the JSONL file changes, not on app launch — the latency budget is single-digit ms per upsert.
4. **Co-located storage** (§5.4 Fieldy): Memex's embedded Qdrant satisfies this for free; emphasize this in the demo (no network hop, zero p99 surprise).
5. **Hybrid filter + vector** (§5.2 TripBuilder pattern): the canonical query shape Qdrant teaches agents is `must=[payload filter] + vector=[semantic anchor]`. Every Memex surface should be expressible in that shape, and most already are.

### 5.1 What is Agentic RAG? Building Agents with Qdrant

- **Author**: Kacper Łukawski
- **Date**: 2024-11-22
- **URL**: https://qdrant.tech/articles/agentic-rag/ (fetched 2026-05-18)
- **Summary**: Official taxonomy of agent memory layers. Qdrant officially serves two roles: (a) **short-term workflow checkpoints** (used inside LangGraph runs), and (b) **long-term semantic memory layer shared across runs** (CrewAI). The article also names the four-layer CrewAI model: short-term, long-term, entity, contextual.
- **Relevance for Memex**: `Foundational`. Memex's whole pitch is "your Claude Code session history *is* the agent's long-term memory." This article is the canonical Qdrant text Memex should cite at VSD.
- **Key quote**: "Qdrant with its semantic search capabilities is often used as a long-term memory layer." — https://qdrant.tech/articles/agentic-rag/

### 5.2 Building Performant, Scaled Agentic Vector Search with Qdrant

- **Author**: Thierry Damiba (Qdrant)
- **Date**: 2025-10-26
- **URL**: https://qdrant.tech/articles/agentic-builders-guide/ (fetched 2026-05-18)
- **Summary**: Production guide for agents. Five operational claims: (1) agent latency *compounds* — each ~75ms search matters; (2) hybrid filter+vector is the dominant pattern (TripBuilder example); (3) **agents need millisecond upserts for fresh memory** plus a recency decay function for time-weighted scoring; (4) RBAC/multitenancy is non-negotiable; (5) context retrievability is the binding constraint on capability.
- **Relevance for Memex**: `Reinforces existing surface`. The recency-decay function is a directly-applicable upgrade to Predict Next Actions (favor recent sessions). The compounded-latency point reinforces Memex's "no-LLM-at-runtime" invariant — every saved RTT matters.
- **Key quote**: "Your agent's ability to complete complex tasks is only as good as the context it can retrieve." — https://qdrant.tech/articles/agentic-builders-guide/

### 5.3 Qdrant Skills for AI Agents

- **Author**: Thierry Damiba
- **Date**: 2026-03-31
- **URL**: https://qdrant.tech/blog/qdrant-skills-release/ (fetched 2026-05-18)
- **Summary**: Releases an official Qdrant "Skills" pack (symptom-organized decision trees) for Cursor, Claude Code, and other agents. Internal eval: 96% assertion pass rate with Skills vs 65% without on production scaling scenarios.
- **Relevance for Memex**: `Tangential` for Memex itself, but **highly relevant to the demo narrative** — Memex is a Claude Code companion, and Qdrant is now officially shipping Claude Code-targeted tooling. Memex can name-drop this at VSD as "we're a downstream of Qdrant Skills."
- **Key quote**: "Documentation says: 'Here's how to enable scalar quantization.' A solutions architect says: 'First check if the vectors are actually the problem.'" — https://qdrant.tech/blog/qdrant-skills-release/

### 5.4 How Fieldy AI Achieved Reliable AI Memory with Qdrant (case study, also see §8.2)

- **Author**: Daniel Azoulai (Qdrant)
- **Date**: 2025-09-04
- **URL**: https://qdrant.tech/blog/case-study-fieldy/ (fetched 2026-05-18)
- **Summary**: Co-located Qdrant; HNSW+BM25 fused via RRF; tens of millions of embeddings; the framing language is "AI Memory" (not "RAG"). Migrated from Weaviate after 5xx-rate-driven outages.
- **Relevance for Memex**: `Reinforces existing surface`. Fieldy's positioning vocabulary ("AI memory," "instantly retrievable") is exactly the brand language Memex should adopt for VSD copy.
- **Key quote**: "Capture every relevant spoken interaction, transcribe it with high accuracy, and make it instantly retrievable." — https://qdrant.tech/blog/case-study-fieldy/

---

## 6. Cluster: Performance & scaling (recent only)

**Cluster framing**. The unifying performance theme across 1.10 → 1.17 (about 13 months of releases) is "**push the work to the server, and lean on quantization**." Every major release moved another piece of orchestration that used to live in client code (multi-API stitching → 1.10 Universal Query; client-side re-scoring → 1.14 Score-Boosting; client-side relevance learning → 1.17 Relevance Feedback) into the engine, and every release also expanded the quantization menu (1.13 Delta-Encoding HNSW; 1.15 1.5/2-bit + asymmetric; 1.16 Inline Storage; 2026-05 TurboQuant). For Memex, the highest-yield reads in this cluster are the ones that affect the *Lens slider* and the *named-vector storage budget* — specifically 1.10 (Universal Query), 1.14 (Score-Boosting), 1.16 (ACORN for filter+vector), and 1.17 (Weighted RRF + Relevance Feedback).

### 6.1 Qdrant 1.10 — Universal Query, Built-in IDF & ColBERT Support

- **Author**: David Myriel
- **Date**: 2024-07-01
- **URL**: https://qdrant.tech/blog/qdrant-1.10.x/ (fetched 2026-05-18)
- **Summary**: Universal Query API consolidates `search`, `recommend`, `discover`, hybrid, multi-vector into one endpoint. Built-in IDF (server-side). Native ColBERT/multi-vector storage. Sparse vector compression to float16 + bit-packing.
- **Relevance for Memex**: `Foundational`. Memex's Lens slider is essentially a Universal Query call with per-named-vector weights.
- **Key quote**: "Query API will consolidate all search APIs into a single request. Previously, you had to work outside of the API to combine different search requests." — https://qdrant.tech/blog/qdrant-1.10.x/

### 6.2 Qdrant 1.13 — GPU Indexing, Strict Mode & New Storage Engine

- **Author**: David Myriel
- **Date**: 2025-01-23
- **URL**: https://qdrant.tech/blog/qdrant-1.13.x/ (fetched 2026-05-18)
- **Summary**: GPU HNSW (Vulkan; NVIDIA/AMD/Intel). Strict Mode caps unindexed filters, batch sizes, search params. New custom storage engine (constant-time R/W) replacing RocksDB. HNSW Delta Encoding compresses the graph ~30%. New `has_vector` filter for named-vector presence.
- **Relevance for Memex**: `Reinforces existing surface`. `has_vector` is a free win — Memex can use it to find sessions that *have* an `error` vector for Proactive Recall instead of doing a null-check in payload.
- **Key quote**: "Indexing over GPU now delivers speeds up to 10x faster than CPU-based methods for the equivalent hardware price." — https://qdrant.tech/blog/qdrant-1.13.x/

### 6.3 Qdrant 1.14 — Reranking Support & Resource Optimizations

- **Author**: David Myriel
- **Date**: 2025-04-22
- **URL**: https://qdrant.tech/blog/qdrant-1.14.x/ (fetched 2026-05-18)
- **Summary**: **Score-Boosting Reranker** — official server-side mechanism to combine vector similarity with payload-derived business logic at query time. Incremental HNSW indexing. 57% faster batch queries in single-segment setups.
- **Relevance for Memex**: `Suggests new surface`. Memex can replace whatever client-side scoring tweaks the Lens slider does today with a Score-Boosting expression — keeps it on the server, no LLM, no extra round-trip.
- **Key quote**: "The Score-Boosting Reranker allows you to combine vector-based similarity with business or domain-specific logic by applying a rescoring step." — https://qdrant.tech/blog/qdrant-1.14.x/

### 6.4 Qdrant 1.15 — Smarter Quantization & Better Text Filtering

- **Author**: Derrick Mwiti (Qdrant)
- **Date**: 2025-07-18
- **URL**: https://qdrant.tech/blog/qdrant-1.15.x/ (fetched 2026-05-18)
- **Summary**: **1.5-bit and 2-bit quantization** (16× compression). **Asymmetric quantization** — stored vectors binary, queries scalar — recovers most of binary's quality loss at the same storage cost. Multilingual tokenization (CJK). Stopwords/stemming/phrase matching. MMR for diversity.
- **Relevance for Memex**: `Suggests new surface`. Memex ships 5 named vectors per session — quantization budget multiplies. 1.5-bit on the cold `path` vector is a clean optimization.
- **Key quote**: "2-bit quantization addresses this by explicitly representing zeros using an efficient scoring mechanism." — https://qdrant.tech/blog/qdrant-1.15.x/

### 6.5 Qdrant 1.16 — Tiered Multitenancy & Disk-Efficient Vector Search

- **Author**: Abdon Pijpelink (Qdrant)
- **Date**: 2025-11-19
- **URL**: https://qdrant.tech/blog/qdrant-1.16.x/ (fetched 2026-05-18)
- **Summary**: Tiered multitenancy (small tenants share, large get promoted shards). **ACORN** filtered-HNSW algorithm — explores second-hop neighbors to recover recall on low-selectivity filters. **Inline Storage** mode embeds quantized vectors directly into HNSW graph nodes for disk efficiency. `text_any` condition. Conditional updates.
- **Relevance for Memex**: `Reinforces existing surface`. ACORN is the answer to "filter by `error_seen=true` AND search by `content` vector" — exactly Memex's Proactive Recall query shape.
- **Key quote**: "With inline storage enabled, the quantized vectors are directly embedded into the HNSW graph nodes, alongside neighbor IDs." — https://qdrant.tech/blog/qdrant-1.16.x/

### 6.6 Qdrant 1.17 — Relevance Feedback & Search Latency Improvements

- **Author**: Abdon Pijpelink
- **Date**: 2026-02-20
- **URL**: https://qdrant.tech/blog/qdrant-1.17.x/ (fetched 2026-05-18)
- **Summary**: Native **Relevance Feedback** query (see §1.5). Update-queue + `prevent_unoptimized` mode + delayed fan-out reduce p99 latency under load. **Weighted RRF**. Audit logging. Upsert update modes.
- **Relevance for Memex**: `Suggests new surface`. Weighted RRF is the production-grade replacement for the Lens slider's current naïve score-blending — and the relevance-feedback query lets the Lens learn over time without retraining.
- **Key quote**: "Retrieval systems can leverage this relevance feedback to iteratively refine results toward user intent." — https://qdrant.tech/blog/qdrant-1.17.x/

### 6.7 Vector Search Resource Optimization Guide

- **Author**: David Myriel
- **Date**: 2025-02-09
- **URL**: https://qdrant.tech/articles/vector-search-resource-optimization/ (fetched 2026-05-18)
- **Summary**: The canonical knob-by-knob tuning guide as of 2025. `m`/`ef_construct`/`ef`. Scalar quantization (4× compression, <1% recall hit). Binary quantization (32× compression, 40× faster, requires oversample+rescore). Multitenancy via `group_id` payload index with `is_tenant=true`. `on_disk=true` memmap storage for RAM-bound deployments.
- **Relevance for Memex**: `Foundational` for any future scale work. The `is_tenant=true` pattern is the right design when Memex eventually supports multiple Claude Code installs / multiple users on the same Qdrant.
- **Key quote**: "Compression cuts memory usage by a factor of 4. Qdrant compresses 32-bit floating-point values (float32) into 8-bit unsigned integers (uint8)." — https://qdrant.tech/articles/vector-search-resource-optimization/

### 6.8 TurboQuant in Qdrant

- **Authors**: Ivan Pleshkov & Jonas Schulz (Qdrant engineering)
- **Date**: 2026-05-13
- **URL**: https://qdrant.tech/articles/turboquant-quantization/ (fetched 2026-05-18)
- **Summary**: New rotation-based quantization (published 2026-05, days before this research). Random orthogonal rotation + Lloyd-Max codebook + length renormalization + per-coordinate anisotropy calibration. **TurboQuant 4-bit matches Scalar at half the storage; TurboQuant 2-bit beats Binary by 9–24 pp**. No per-dataset training (codebook is fixed from standard normal). All three distance metrics supported with the same kernels.
- **Relevance for Memex**: `Suggests new surface`. Memex's 5 named vectors × millions of sessions is the exact workload TurboQuant targets. Drop-in for current Scalar.
- **Key quote**: "If you currently run SQ or BQ, try the equivalent TurboQuant configuration on a test subset of your data." — https://qdrant.tech/articles/turboquant-quantization/

### 6.9 Qdrant 2025 Recap: Powering the Agentic Era

- **Author**: Daniel Azoulai
- **Date**: 2025-12-17
- **URL**: https://qdrant.tech/blog/2025-recap/ (fetched 2026-05-18)
- **Summary**: Year-end summary. Themes: Advanced Retrieval (Score-Boosting, ACORN, MMR, multilingual), Performance (GPU HNSW, inline storage, incremental indexing, 1.5/2-bit, asymmetric), Enterprise (tiered multitenancy, SSO, RBAC, granular API keys, Terraform), Deployment (Cloud Inference, Edge beta).
- **Relevance for Memex**: `Foundational` as a one-stop "what does Qdrant officially care about in 2025" citation.
- **Key quote**: "Speed alone was no longer enough. Production systems now require precise relevance control, predictable performance at scale." — https://qdrant.tech/blog/2025-recap/

---

## 7. Cluster: Vector Space Day 2026 / Think Outside the Bot

**Cluster framing**. The official VSD 2026 announcement (§7.1) and the 2025 recap (§7.2) together establish the criteria a winning submission has to satisfy. Reading the 2025 grand-prize and category winners against Memex's seven surfaces, the alignment is unusually tight: 2025's grand prize was a "3D e-commerce platform" combining vector search with another modality (3D rendering); 2025 had category recognition for "adaptive robotics with memory systems" and "NPCs featuring spatio-temporal recall capabilities." Both of those phrases would describe Memex with minor wording tweaks. The 2026 prompt explicitly elevates "intelligent recommendations" to first-class status, which is the exact slot Predict Next Actions occupies.

**Submission-criteria hypothesis** (deadlines / judging not yet published, so this is inference from the 2025 recap + 2026 announcement):

- **Beyond-RAG framing required**: the prompt opens with "Forget the classical RAG chatbot" — any submission that pitches itself as a Q&A retriever is on the wrong side of the prompt. Memex's seven non-chat surfaces explicitly answer this.
- **At least one of the three tracks** (Search/Retrieval, Agents/Memory, Edge/Robotics): Memex hits all three.
- **Use of advanced Qdrant primitives** is implied by the 2025 winners (multi-modal, multi-vector, named-vector, Discovery): Memex uses 5 named vectors + Discovery + Distance Matrix + Recommend, i.e. ≥4 advanced primitives in one app.
- **Demonstrable, runnable artifact**: 2025's grand prize was a working immersive demo; Memex is a runnable Tauri desktop bundle.

### 7.1 Announcing Vector Space Day 2026 in San Francisco

- **Author**: Qdrant (corporate byline)
- **Date**: 2026-04-21
- **URL**: https://qdrant.tech/blog/vector-space-day-sf-2026/ (fetched 2026-05-18; the alternative slug `vector-space-day-2026-sf/` returns HTTP 404 as of this date)
- **Summary**: Confirms the event: **June 11, 2026, The Midway, San Francisco**. Early-bird $99 through May 11, then $199. Confirmed speaker orgs: **LlamaIndex, mem0, Neo4j**, plus community. Topic tracks: Search & AI Retrieval, **Agents & Memory**, Edge & Robotics AI. The "Think Outside the Bot" global virtual hackathon submission link is `https://try.qdrant.tech/hackathon-vsd`. **Prize structure, judging criteria, and submission deadline are not published on the announcement page as of 2026-05-18**.
- **Relevance for Memex**: `Foundational`. Memex's product framing should explicitly mirror the three event tracks — Memex hits all three (Search & AI Retrieval = Time Machine/Lens; Agents & Memory = the whole premise; Edge = it's a local Tauri desktop app with embedded Qdrant).
- **Key quote**: "Forget the classical RAG chatbot! Explore multi-modal applications, intelligent recommendations, and advanced vector search that go far beyond conversational interfaces." — https://qdrant.tech/blog/vector-space-day-sf-2026/

### 7.2 All Vectors Lead to Community: Vector Space Day 2025 Recap

- **Author**: Qdrant
- **Date**: 2025-09-30
- **URL**: https://qdrant.tech/blog/vector-space-day-2025-recap/ (fetched 2026-05-18)
- **Summary**: Recap of the 2025 (Berlin) edition. **Grand Prize**: Benedict Counsell, *Vector Vintage* — 3D e-commerce + Qdrant + Mistral + Neo4j. Other notable category winners include **"adaptive robotics with memory systems"** and **"NPCs featuring spatio-temporal recall capabilities"** — i.e. the 2025 judges *already* rewarded memory-machine entries. Memex sits squarely in that lineage.
- **Relevance for Memex**: `Foundational` as positioning evidence. The 2025 winners' shapes (memory, recall, time-aware NPCs) are extremely close to Memex's surfaces — proof of fit, not coincidence.
- **Key quote**: "Think Outside the Bot to create AI solutions with Vector Search beyond simple RAG chatbots." — https://qdrant.tech/blog/vector-space-day-2025-recap/

---

## 8. Cluster: Case studies that mirror Memex

**Cluster framing**. The researcher scanned the Qdrant Customers index for any case study whose architectural shape resembles Memex's (desktop + Rust + local Qdrant + code-or-session data + memory framing). Only two cleared the bar: **Bloop** (Tauri + Rust + Qdrant for code; nearly identical architecture, different domain) and **Fieldy AI** (co-located Qdrant, hybrid HNSW+BM25, framed as "AI memory"). Other case studies on the index page (Dust, TrustGraph, Kakao, Lyzr, Tavus, Voiceflow, Qovery) are agent or chatbot products and don't share Memex's local-first / non-chat shape — they're listed in §8.3 for completeness only.

### 8.1 Powering Bloop semantic code search

- **Author**: Qdrant Team
- **Date**: 2023-02-28
- **URL**: https://qdrant.tech/blog/case-study-bloop/ (fetched 2026-05-18)
- **Summary**: Bloop ships a **Tauri desktop app for semantic code search**, embedding Qdrant via the Rust `qdrant-client` crate alongside `tantivy`. Handles million-LOC repos (e.g. Rust at 2.8M lines) with low resource use. This is the **single closest officially-documented architectural twin to Memex**: Tauri + Rust + Qdrant + code-shaped data + desktop.
- **Relevance for Memex**: `Foundational`. Memex should cite this prominently — Qdrant has already publicly endorsed exactly this stack shape.
- **Key quote**: "Qdrant is an open-source Vector Search Database written in Rust… providing a search for nearest high-dimensional vectors." — https://qdrant.tech/blog/case-study-bloop/

### 8.2 How Fieldy AI Achieved Reliable AI Memory with Qdrant

(See §5.4 for full entry — appears in both clusters because it doubles as a memory-pattern article and a case study.)

- **URL**: https://qdrant.tech/blog/case-study-fieldy/ (fetched 2026-05-18)

### 8.3 Other case studies on the customers index (not deeply reviewed)

These appear on https://qdrant.tech/customers/ (fetched 2026-05-18) and were inspected from the index page only. None matches Memex's local-first / non-chat / code-data shape closely enough to warrant a full per-article entry, but the URLs are listed for completeness in case the team wants to mine them for additional positioning quotes:

- Dust — https://qdrant.tech/blog/case-study-dust-v2/ (multi-source connector for AI apps; cloud-hosted)
- TrustGraph — https://qdrant.tech/blog/case-study-trustgraph/ (enterprise agentic AI)
- Kakao — https://qdrant.tech/blog/case-study-kakao/ (internal service desk; chatbot-shaped)
- Lyzr — https://qdrant.tech/blog/case-study-lyzr/ (agent performance via vector search optimisation)
- Tavus — https://qdrant.tech/blog/case-study-tavus/ (conversational AI, Edge deployment)
- Voiceflow — https://qdrant.tech/blog/case-study-voiceflow/ (no-code agent builder)
- Qovery — https://qdrant.tech/blog/case-study-qovery/ (DevOps automation w/ embeddings)

Notably, the Customers landing page also calls out **TripAdvisor** ("1B+ reviews powering AI Trip Planner"), **OpenTable** ("AI Concierge filtering 60K+ restaurants with sparse embeddings"), and **HubSpot** ("Breeze AI assistant") as 2025 marquee deployments. None has a per-article post yet as of 2026-05-18.

---

## 9. Cross-cutting synthesis

### 9.1 Top 5 articles every Memex contributor should read (ranked)

1. **Discovery needs context** — https://qdrant.tech/articles/discovery-search/ — the API mental model behind Mix & Match and the language Memex should use in copy.
2. **Distance-based data exploration** (CTO byline) — https://qdrant.tech/articles/distance-based-exploration/ — direct endorsement of Memex's Topology surface architecture, by Andrey Vasnetsov himself.
3. **Any Embedding Model Can Become a Late Interaction Model** — https://qdrant.tech/articles/late-interaction-models/ — the single highest-ROI upgrade Memex can ship before VSD.
4. **What is Agentic RAG?** — https://qdrant.tech/articles/agentic-rag/ — the canonical Qdrant text on long-term memory layers; Memex is *the* embodiment of this.
5. **Building Performant, Scaled Agentic Vector Search with Qdrant** — https://qdrant.tech/articles/agentic-builders-guide/ — the operational playbook (recency decay, ms-latency upserts, filter+vector pattern) that hardens Memex for the live demo.

### 9.2 Single biggest unused idea (honors no-LLM-at-runtime)

**Add a 6th named vector `content_late` containing per-token output embeddings from BGE-small-en, with HNSW disabled, used only for MaxSim reranking of the Time Machine top-50.**

- The article (§2.1) literally uses `BAAI/bge-small-en` as its hero example.
- MaxSim is a pure dot-product + max + sum — no inference at query time, no LLM, no model load beyond what Memex already does at ingest.
- Storage is the only cost, and scalar quantization (§6.7) recovers 75% of that.
- Result: Time Machine's relevance jumps to ColBERT-grade quality without changing the model, without adding a runtime dependency, and without violating Memex's "no LLM at runtime" invariant.

Runner-up: enable the **Score-Boosting Reranker** (1.14, §6.3) to express the Lens slider's named-vector weighting *on the server* — saving an RTT and aligning with §5.2's "every 75ms counts" finding.

### 9.3 Memex-surface-to-Qdrant-article crosswalk

Quick-reference: for each of Memex's seven surfaces, the most authoritative Qdrant article(s) Memex should cite in code comments, README, and the VSD submission.

| Memex surface | Primary citation | Secondary citation |
|---|---|---|
| Time Machine stack | §6.1 (Universal Query, 1.10) | §2.1 (late-interaction upgrade path) |
| Topology galaxy (MST) | §3.1 (CTO on distance-based exploration) | §3.2 (1.12 release) |
| Mix & Match | §1.2 (Discovery needs context) | §1.4 (Vector similarity beyond search) |
| Proactive Recall | §1.1 (Recommendation API) | §6.5 (ACORN for filter+vector) |
| Predict Next Actions | §1.1 (`best_score` strategy) | §5.2 (recency decay pattern) |
| Replay engine | §1.4 (dissimilarity / diversity sampling) | §1.3 (search groups for de-dup) |
| Lens slider | §4.2 (no linear blending) + §6.6 (Weighted RRF) | §1.5 (Relevance Feedback for learning) |

### 9.4 Officially deprecated / reversed since Memex's 1.18 choice

1. **BM42** (§4.1) — Qdrant published a correction noting it does *not* outperform BM25 in their own benchmarks and is "experimental." If Memex was considering a sparse BM42 channel, that direction is officially soft-cancelled in favor of plain BM25 or the newer miniCOIL line in §4.3.
2. **RocksDB-backed storage** (§6.2) — replaced in 1.13 by a new custom storage engine with constant-time R/W. Memex 1.18 ships with the new engine by default; any old Memex docs referencing RocksDB tuning are now obsolete.
3. **Linear score combination** for hybrid scoring (§4.2) — officially deprecated guidance; RRF (or Weighted RRF from 1.17, §6.6) is now the only blessed approach. If Memex's Lens slider currently does a linear weighted sum, that approach is on Qdrant's "won't work" list and should move to weighted RRF.
4. **Client-side multi-API stitching** (§6.1) — superseded by the Universal Query API; any Memex code that issues separate `/search` + `/recommend` calls should consolidate into a single `/query` request.

---

## Sources (every URL fetched on 2026-05-18)

1. https://qdrant.tech/articles/new-recommendation-api/
2. https://qdrant.tech/articles/discovery-search/
3. https://qdrant.tech/blog/vector-space-day-sf-2026/
4. https://qdrant.tech/articles/distance-based-exploration/
5. https://qdrant.tech/blog/qdrant-1.12.x/
6. https://qdrant.tech/articles/bm42/
7. https://qdrant.tech/articles/hybrid-search/
8. https://qdrant.tech/articles/agentic-rag/
9. https://qdrant.tech/blog/2025-recap/
10. https://qdrant.tech/blog/multi-vector-search-course/
11. https://qdrant.tech/blog/qdrant-1.15.x/
12. https://qdrant.tech/articles/late-interaction-models/
13. https://qdrant.tech/blog/qdrant-1.10.x/
14. https://qdrant.tech/articles/vector-search-resource-optimization/
15. https://qdrant.tech/blog/vector-space-day-2025-recap/
16. https://qdrant.tech/articles/ (index page)
17. https://qdrant.tech/articles/relevance-feedback/
18. https://qdrant.tech/articles/agentic-builders-guide/
19. https://qdrant.tech/articles/muvera-embeddings/
20. https://qdrant.tech/articles/turboquant-quantization/
21. https://qdrant.tech/articles/food-discovery-demo/
22. https://qdrant.tech/blog/colpali-qdrant-optimization/
23. https://qdrant.tech/blog/qdrant-1.17.x/
24. https://qdrant.tech/articles/vector-similarity-beyond-search/
25. https://qdrant.tech/blog/qdrant-1.16.x/
26. https://qdrant.tech/blog/qdrant-skills-release/
27. https://qdrant.tech/blog/case-study-bloop/
28. https://qdrant.tech/blog/case-study-fieldy/
29. https://qdrant.tech/articles/sparse-vectors/
30. https://qdrant.tech/articles/modern-sparse-neural-retrieval/
31. https://qdrant.tech/blog/qdrant-1.13.x/
32. https://qdrant.tech/blog/qdrant-1.14.x/
33. https://qdrant.tech/blog/qdrant-colpali/
34. https://qdrant.tech/customers/
35. https://qdrant.tech/recommendations/

**Attempted but 404 on 2026-05-18**: https://qdrant.tech/blog/vector-space-day-2026-sf/ (note the slug order — Qdrant's canonical is `vector-space-day-sf-2026`).

**Not catalogued** (out of scope or only seen as search hits): n8n node post, Series A funding, Community Highlights, Quaterion docs, generic course pages, RAG landing page.
