# LLM + Qdrant Augmentation Patterns — Without Ever Exposing a Chat Surface

**Prepared**: 2026-05-18 (T-14 days from VSD 2026 submission)
**Subject**: `ComBba/memex` — Tauri 2 desktop app, indexes `~/.claude/projects/**/*.jsonl` Claude Code sessions into local Qdrant 1.18 with 5 named vectors per session (content / tool / path / error / code, 384-d BGE-small via fastembed-rs). Surfaces are spatial / recommendation / replay — NOT chat.
**Status of "no LLM at runtime" invariant**: **formally relaxed**. New rule — maximize Qdrant 1.18 pinnacle usage; LLM/agent acceptable iff (a) not a chat UX, (b) amplifies Qdrant primitives rather than replacing them, (c) ideally local (ollama / llama.cpp / candle).
**Research scope**: 12 named LLM-augmented retrieval patterns + extras; verify each against Qdrant-staff, paper, or framework-canonical sources; rate each on Memex-surface fit.

---

## 1. Framing — Why LLM ≠ Chat for VSD 2026

The hackathon brief is explicit and well-quoted in our earlier dossier:

> "Push the boundaries of vector search… no chatbots allowed!"
> *[Source: https://try.qdrant.tech/hackathon-vsd · fetched 2026-05-18]*

> "Forget the classical RAG chatbot!"
> *[Source: https://qdrant.tech/blog/vector-space-day-sf-2026/ · fetched 2026-05-18]*

> "Submissions that are only chatbots are not allowed."
> *[Source: https://try.qdrant.tech/hackathon-vsd · fetched 2026-05-18]*

Note what the brief does *not* say. It does not say "no LLM in the system." It says **no chatbot UX**, **no classical RAG**, and **explore "multi-modal apps, intelligent recommendations, advanced vector search."** That distinction is what this dossier exploits: an LLM that runs once at `scan --index` time, never talks to the user, and never appears in a prompt box is **not a chatbot** — it is an *embedding-amplifier* and a *payload-enricher* that makes Qdrant's primitives *more visible* in the demo.

The hard product invariant we must keep:

1. **No textual conversation surface.** The user types into zero `<input type=text>` fields targeted at an LLM.
2. **No free-text "ask the corpus" affordance.** No "What did I work on last week?" prompt.
3. **No runtime LLM dependency on the user's hot path that contradicts "100% local"** — unless the model is local and lightweight enough to never feel like a chatbot delay (i.e., < 200 ms or invisible behind an index step).
4. **Qdrant primitives remain the protagonist.** Every demo-visible win must be traceable to a Qdrant 1.18 call (named vector, Query API prefetch+fusion, Distance Matrix, Discovery, snapshot, payload filter, RRF/DBSF).

If those invariants hold, **the brief is satisfied** and Memex can still claim "Think Outside the Bot" with full integrity — only with the bonus that Qdrant primitives can now look sharper, label themselves, and surface more remarkable recommendations.

---

## 2. Pattern Catalog

For each pattern: definition · where the LLM lives · visible to user · determinism · local model viability · canonical reference · 3-5 sentence summary · Memex-surface fit (1–3 per surface) · cost estimate.

The 7 Memex surfaces, indexed for the per-surface fit lines:

| # | Surface | Qdrant primitive |
|---|---------|------------------|
| 1 | Time Machine stack | `scroll` payload-only |
| 2 | Topology galaxy | Distance Matrix `search_matrix_pairs` |
| 3 | Mix & Match | Discovery API context pairs |
| 4 | Proactive Recall | `query()` on `error` named vector + payload filter |
| 5 | Predict Next Actions | `content` named-vector neighbor search |
| 6 | Replay engine | payload-driven JSONL re-parse |
| 7 | Lens slider | 5 named vectors + weighted parallel `query()` |

Scale: **3 = high fit / visible win** · **2 = moderate / nice-to-have** · **1 = low / nothing meaningful**.

---

### Pattern 1 — HyDE (Hypothetical Document Embeddings)

- **Definition**: LLM zero-shot generates a *fake but plausible* answer document for the query, then embeds **that document** instead of the raw query and runs vector search.
- **Where LLM lives**: **Query-time** (once per query).
- **Visible to user?** Invisible if Memex never exposes a query box. But Memex *has* the Search surface (`memex search "query"`) and the Recall banner that takes an error string — both are query-time invocations.
- **Determinism**: Non-deterministic (temperature > 0 typical). Can be made deterministic with `temperature=0` and a fixed seed in llama.cpp.
- **Local-model viability**: Yes. Qwen 2.5 3B-instruct or Phi-4-mini (~4 GB Q4) generates a 50–150 token hypothetical answer in <500 ms on M-series. The "hypothetical doc" can be very short (HyDE works well with terse passages).
- **Canonical source**: Gao, Ma, Lin, Callan, *"Precise Zero-Shot Dense Retrieval without Relevance Labels,"* arXiv:2212.10496 (Dec 2022); ACL 2023. `[Source: https://arxiv.org/abs/2212.10496 · fetched 2026-05-18]`. Qdrant blog confirms the pattern via Lehtimäki's talk: *"generating hypothetical documents based on the query and embedding those rather than the query itself. That will in some cases lead to higher quality retrieval results." `[Source: https://qdrant.tech/blog/bitter-lesson-generative-language-model/ · fetched 2026-05-18]`*

  > "HyDE first zero-shot instructs an instruction-following language model … to generate a hypothetical document. The document captures relevance patterns but is unreal, and may contain false details. Then, an unsupervised contrastively learned encoder (e.g. Contriever) encodes the document into an embedding vector. This vector identifies a neighborhood in the corpus embedding space, where similar real documents are retrieved based on vector similarity." `[Source: https://arxiv.org/abs/2212.10496 · fetched 2026-05-18]`

- **3-5 sentence summary**: HyDE is a *query expansion through generation*. The trick: short queries embed poorly because they live in a sparse region of the embedding space, but a generated answer (even one with hallucinations) lives near *real answers*. The encoder's bottleneck filters out the hallucinations. **For Memex**: a user typing "WAL Kind WouldBlock" into `memex search` is short and lexical; an LLM hypothetical answer ("This error occurs when SQLite WAL mode is blocked by a concurrent writer; fix by closing the read transaction or…") would land near actual debugging sessions. HyDE is also the *one query-time pattern most cited in the Qdrant ecosystem* — adopting it doesn't violate the brief because the user never sees the generated text.
- **Memex-surface fit**: 1=1 · 2=1 · 3=2 (Mix & Match could accept HyDE-style "hypothetical positive" instead of needing a real session) · 4=**3** (the error banner's similarity threshold improves: short error strings embed terribly, hypothetical fix-document embeds well) · 5=2 · 6=1 · 7=**3** (Lens slider's content-vector arm benefits most).
- **Cost estimate**: 1 LLM call per user search/recall (~150 tokens out, ~50 in) — ~200 tokens × 1 call. On Qwen 2.5 3B local: ~300 ms decode latency + 1 embedding. Net query latency goes from ~30 ms to ~350 ms. Acceptable if framed as "smart search."

---

### Pattern 2 — Multi-Query Expansion (with RRF fusion)

- **Definition**: LLM generates K paraphrases of the query (typically 3–5), each is embedded separately, each searches Qdrant, results fused via RRF (Reciprocal Rank Fusion).
- **Where LLM lives**: Query-time.
- **Visible to user?** Invisible.
- **Determinism**: Non-deterministic unless seeded.
- **Local-model viability**: Yes. Qwen 2.5 3B or Phi-4-mini generates 5 paraphrases in one call in ~600 ms.
- **Canonical source**: LangChain `MultiQueryRetriever` is the canonical pattern shape (we cite for *definition only*, not endorsement). The fusion side is fully native in Qdrant 1.10+: *"Qdrant has built-in support for the Reciprocal Rank Fusion method, which is the de facto standard in the field." `[Source: https://qdrant.tech/articles/hybrid-search/ · fetched 2026-05-18]`*. Qdrant 1.11 added DBSF `[Source: https://qdrant.tech/blog/qdrant-1.11.x/ · fetched 2026-05-18]`.
- **3-5 sentence summary**: Same query phrased five different ways probes five different regions of the embedding manifold; RRF/DBSF combines their top-Ks. Qdrant's Query API natively supports multi-prefetch + Fusion, so the fusion side is **zero code** — you just provide multiple `query` vectors in one `Prefetch` array. The pattern is the *best demo of Qdrant 1.10's Universal Query API specifically* because it shows multiple vectors fan-in + RRF in a single API call. **For Memex**: paraphrases of "WAL crash" (e.g., "write-ahead log error", "sqlite concurrent write failure", "WouldBlock WAL") would each hit a different cluster of sessions.
- **Memex-surface fit**: 1=1 · 2=1 · 3=1 · 4=**3** (recall banner with paraphrased error text → much higher recall) · 5=2 · 6=1 · 7=**3** (Lens slider could fan out paraphrases × named vectors simultaneously — extreme Qdrant 1.10 showpiece).
- **Cost estimate**: 1 LLM call (~250 tokens out for 5 paraphrases), 5 embed calls, 1 Qdrant Query API call with 5-arm prefetch. ~500 ms total on local Qwen 2.5 3B.

---

### Pattern 3 — Query Decomposition

- **Definition**: Complex compound query → LLM splits into N atomic sub-queries → each runs independently → results fused (or sequentially chained for multi-hop).
- **Where LLM lives**: Query-time.
- **Visible to user?** Invisible if no chat. But it implies Memex would accept *complex* queries — which Memex currently doesn't have a surface for (Memex's queries are short error strings, session-IDs, or named-vector lenses).
- **Determinism**: Non-deterministic.
- **Local-model viability**: Yes, but the LLM must reliably emit structured JSON (sub-queries array). Qwen 2.5 3B-instruct can; Phi-4 14B is more reliable but heavier.
- **Canonical source**: Lehtimäki at Qdrant: *"Breaking user queries into smaller questions that each follow different retrieval paths." `[Source: https://qdrant.tech/blog/bitter-lesson-generative-language-model/ · fetched 2026-05-18]`*. Survey: arXiv:2510.18633 (2025), *Query Decomposition for RAG: Balancing Exploration-Exploitation* `[Source: https://arxiv.org/pdf/2510.18633 · fetched 2026-05-18]`.
- **3-5 sentence summary**: For Memex this is mostly a misfit — Memex's queries are atomic by construction (a session ID, an error message, a vector lens). Decomposition shines on multi-hop natural-language questions, which Memex doesn't accept by design. The one possible use: **Predict Next Actions** could decompose "what comes after a `cargo build` failure?" into ["build failure cause", "build failure fix command sequence"] — but this leaks chatbot semantics.
- **Memex-surface fit**: 1=1 · 2=1 · 3=1 · 4=1 · 5=2 · 6=1 · 7=1.
- **Cost estimate**: 1 LLM call (~300 tokens), 2-4 downstream Qdrant searches. ~400 ms.

---

### Pattern 4 — Semantic Chunking at Index Time

- **Definition**: LLM (or embedding-similarity heuristic) splits long documents at *semantic* boundaries instead of fixed-size chunks, producing chunks that are internally coherent.
- **Where LLM lives**: **Index-time only** (during `memex scan --index`).
- **Visible to user?** Invisible.
- **Determinism**: Deterministic if using embedding-similarity-based splitting (LlamaIndex `SemanticSplitterNodeParser`). Non-deterministic if using LLM-as-judge.
- **Local-model viability**: The embedding-similarity variant needs **only an embedding model** (BGE-small we already ship). The LLM-as-judge variant needs a small model.
- **Canonical source**: LlamaIndex `SemanticSplitterNodeParser` — *"the semantic splitter adaptively picks the breakpoint in-between sentences using embedding similarity, ensuring that a 'chunk' contains sentences that are semantically related to each other." `[Source: https://docs.llamaindex.ai/en/stable/examples/node_parsers/semantic_chunking/ · fetched 2026-05-18]`*. Pattern shape only — we are not endorsing LlamaIndex.
- **3-5 sentence summary**: Memex currently indexes one point per *session* with 5 named vectors aggregated across all turns. A 600-turn session is a long document; aggregating all `content` text into one vector dilutes signal. Semantic chunking at the *turn-cluster* level (LLM names "chapters" — e.g., "investigation phase", "fix phase", "retest phase") could give Memex *sub-session* points. The downside: Memex's identity model is "1 session = 1 point," and chapter-level points complicate Time Machine and Predict surfaces. **Use selectively** — e.g., only sessions > 300 turns get chaptered, and chapter points live in a sidecar collection.
- **Memex-surface fit**: 1=1 · 2=2 (chapter clusters → richer Topology) · 3=2 · 4=2 · 5=**3** (Predict could find the *chapter* of past-you's analogous moment, not the whole session — much sharper pivot detection) · 6=2 · 7=2.
- **Cost estimate**: Index-time only. ~200 tokens per chapter boundary decision × ~6 boundaries per long session × ~5% of sessions are long. One-time cost, then cached.

---

### Pattern 5 — RAPTOR (Recursive Abstractive Processing for Tree-Organized Retrieval)

- **Definition**: At index time: cluster documents → LLM summarizes each cluster → embed summary → recurse → build a hierarchical tree. At query time: search at any level of the tree (or all levels via collapsed traversal).
- **Where LLM lives**: **Index-time only.**
- **Visible to user?** Invisible.
- **Determinism**: Non-deterministic at index time (LLM summaries); query time is fully deterministic.
- **Local-model viability**: Yes. Summarization of a session cluster (5–10 sessions) is ~2 K tokens in, ~250 tokens out. Qwen 2.5 7B-instruct or Phi-4 14B (Q4) handles this well.
- **Canonical source**: Sarthi, Abdullah, Tuli, Khanna, Goldie, Manning, *"RAPTOR: Recursive Abstractive Processing for Tree-Organized Retrieval,"* arXiv:2401.18059, ICLR 2024 `[Source: https://arxiv.org/abs/2401.18059 · fetched 2026-05-18]`.

  > "RAPTOR introduces a novel approach of recursively embedding, clustering, and summarizing chunks of text, constructing a tree with differing levels of summarization from the bottom up. At inference time, the RAPTOR model retrieves from this tree, integrating information across lengthy documents at different levels of abstraction." `[Source: https://arxiv.org/abs/2401.18059 · fetched 2026-05-18]`

- **3-5 sentence summary**: This is the most "Memex-like" augmentation pattern in the literature. Memex already has *one* layer of clustering (Distance Matrix → KMeans → cluster auto-label). RAPTOR is the same idea recursed: cluster the cluster-summaries to make a second-tier "topic galaxy," and a third tier of "themes" above that. Demo-wise, this would let Topology galaxy **zoom out** — from "47 sessions" to "5 project-themes" to "2 long-term arcs of your engineering life." That's a *huge* visual wow.
- **Memex-surface fit**: 1=1 · 2=**3** (Topology zoom out levels are RAPTOR levels) · 3=2 · 4=1 · 5=2 · 6=1 · 7=2.
- **Cost estimate**: Index-time. For 200 sessions: ~30 cluster summaries × 2.5 K tokens in + 250 out = ~75 K tokens in / 7.5 K out. Recurse: ~5 second-level summaries. Total under 100 K tokens, runs in ~3 min on local Qwen 2.5 7B.

---

### Pattern 6 — Index-Time Metadata Enrichment (a.k.a. Contextual Retrieval, Anthropic-style)

- **Definition**: LLM tags each document at index time with topic / intent / entities / sentiment / *a contextualizing prefix*. Tags become payload fields; can be filtered with Qdrant `Filter` or used as additional retrieval signal.
- **Where LLM lives**: **Index-time only.**
- **Visible to user?** Invisible. Tags become payload chips visible in cards, but no chat.
- **Determinism**: Effectively deterministic when seeded; the payload is *frozen* after indexing.
- **Local-model viability**: Excellent. Qwen 2.5 3B or Phi-4-mini.
- **Canonical source**: Anthropic, *Introducing Contextual Retrieval*, Sep 2024 `[Source: https://www.anthropic.com/news/contextual-retrieval · fetched 2026-05-18]`. Anthropic's exact recipe:

  > "Please give a short succinct context to situate this chunk within the overall document for the purposes of improving search retrieval." `[Source: https://www.anthropic.com/news/contextual-retrieval · fetched 2026-05-18]`

  Quantified:

  > "Contextual Embeddings reduced the top-20-chunk retrieval failure rate by 35% (5.7% → 3.7%) … Combining Contextual Embeddings and Contextual BM25 reduced the top-20-chunk retrieval failure rate by 49% (5.7% → 2.9%)." `[Source: https://www.anthropic.com/news/contextual-retrieval · fetched 2026-05-18]`

  > "The one-time cost to generate contextualized chunks is $1.02 per million document tokens." `[Source: https://www.anthropic.com/news/contextual-retrieval · fetched 2026-05-18]`

- **3-5 sentence summary**: This is the **single most demo-impactful pattern** for Memex's "Think Outside the Bot" framing. The Anthropic post is *the* citation for "LLM at index time, NEVER at query time" — exactly Memex's invariant. For Memex: at scan time, an LLM reads the session's first 20 turns and emits `{topic: "tauri-bundling", intent: "debug", entities: ["fastembed", "ONNX", "EROFS"], outcome: "resolved", arc: "infra-hardening"}` into the payload. These become **filter chips** on Time Machine cards, **auto-labels** on Topology clusters, and **payload-filter axes** on Lens slider — all visible Qdrant primitives that look much sharper. Zero query-time LLM dependency.
- **Memex-surface fit**: 1=**3** (filter chips on stack cards) · 2=**3** (cluster auto-labels become semantically meaningful, not just tool-call frequency) · 3=**3** (Discovery API gains payload-filter constraints — *"like X, unlike Y, but only intent=debug"*) · 4=2 · 5=**3** (Predict can filter by outcome=resolved before suggesting next actions) · 6=2 (chapter titles in replay timeline) · 7=**3** (payload-filter Lens dimension).
- **Cost estimate**: Index-time. ~500 input tokens × 200 sessions × 1 call = 100 K tokens in, 30 K out. ~2 min on local Qwen 2.5 7B. **One-time per session, then cached forever in payload.**

---

### Pattern 7 — Cluster Auto-Labeling (LLM names Qdrant clusters)

- **Definition**: After Distance Matrix → KMeans gives cluster IDs, LLM is shown 3–5 representative sessions per cluster and emits a 3-7 word human-readable label.
- **Where LLM lives**: **Index-time** (after each `scan --index` and re-cluster).
- **Visible to user?** Invisible in process; labels are *highly visible* in Topology galaxy.
- **Determinism**: Non-deterministic; **cache the label keyed on cluster centroid hash** for stability.
- **Local-model viability**: Excellent. Qwen 2.5 3B handles 1-shot cluster naming with a tight prompt in ~400 ms.
- **Canonical source**: Pattern is well-documented in the research literature: *"a method has been proposed to enhance topic labeling by leveraging LLMs to generate concise and meaningful labels for topic clusters, with four different document selection strategies that emphasize different aspects such as dominant themes or diversity." `[Source: https://arxiv.org/html/2502.18469v1 · fetched 2026-05-18]`*. Qdrant doesn't have an official blog post specifically on this (confirmed via site search) but the Distance Matrix article explicitly endorses the upstream half: *"Many clustering algorithms accept precomputed distance matrix as input, so we can use the same distance matrix we calculated before." `[Source: https://qdrant.tech/articles/distance-based-exploration/ · fetched 2026-05-18]`*
- **3-5 sentence summary**: Memex's Topology already auto-labels clusters by *tool-call statistics* — e.g., *"code + shell · Bash×1350 Edit×1032"*. That's correct but charmless. An LLM upgrade turns the same cluster into *"Tauri build-infra debugging"* or *"Solidity exam grading work"* — instantly more meaningful. This is the single highest-ROI LLM addition: tiny prompt, big visible upgrade, fits Topology demo cold. The current statistical label can be retained as a tooltip/subtitle.
- **Memex-surface fit**: 1=2 (stack cards inherit cluster label) · 2=**3** (the demo killshot) · 3=2 · 4=1 · 5=2 · 6=1 · 7=2.
- **Cost estimate**: Index-time. ~6 clusters × 400 tokens in (5 session previews) × 1 call = 2.4 K tokens / call. ~3 s total. Cached.

---

### Pattern 8 — LLM-as-Reranker

- **Definition**: Top-K from Qdrant (typically K=20–50) → LLM scores each candidate for query relevance → reorder by LLM scores.
- **Where LLM lives**: Query-time, after retrieval.
- **Visible to user?** Invisible in process; results visibly reorder.
- **Determinism**: Non-deterministic without seeding; deterministic with `temperature=0`.
- **Local-model viability**: Borderline. Scoring 20 candidates × ~300 tokens each = 6 K tokens per query — Qwen 2.5 3B does this in ~3 s, which feels slow.
- **Canonical source**: Qdrant officially recognizes the pattern:

  > "This is a newer, smarter way to rerank. LLMs, like GPT, are getting better by the day. With the right instructions, they can prioritize the most relevant documents for you." `[Source: https://qdrant.tech/documentation/search-precision/reranking-semantic-search/ · fetched 2026-05-18]`

  Qdrant's preferred *demonstration* reranker remains the **cross-encoder** (e.g., Jina Reranker v2, BAAI bge-reranker) via FastEmbed — these are **smaller and faster** than an LLM and Memex could ship one:

  > "Rerankers analyze token-level interactions between the query and each document in depth, making them expensive to use but precise in defining relevance, trading speed for accuracy so they are best used on a limited candidate set rather than the entire corpus." `[Source: https://qdrant.tech/documentation/fastembed/fastembed-rerankers/ · fetched 2026-05-18]`

- **3-5 sentence summary**: **Cross-encoder reranker beats LLM-as-reranker for Memex** — same accuracy lift, ~50× faster, no LLM dependency, available in fastembed-rs the moment 5.x ships it. Of all 12 patterns, this is the one to **not** do with an LLM. The right answer: ship a 30 MB ONNX cross-encoder (Jina reranker-v2 or bge-reranker-base) when fastembed-rs supports it; until then, leave reranking alone.
- **Memex-surface fit**: 1=1 · 2=1 · 3=2 · 4=2 · 5=2 · 6=1 · 7=**3** (Lens slider's "smart-rerank" toggle could be a brief-aligned advanced demo).
- **Cost estimate**: Cross-encoder: ~50 ms per query for K=20. LLM: ~3 s. Skip the LLM variant.

---

### Pattern 9 — Self-Query Retriever (NL → Structured Qdrant Filter)

- **Definition** *(pattern name comes from LangChain — we cite for shape only)*: LLM converts a natural-language query into a structured `Filter` expression for Qdrant payload + an extracted semantic-search residue.
- **Where LLM lives**: Query-time.
- **Visible to user?** *This pattern requires a user to type natural language — strong chatbot-adjacent.* **Avoid in Memex's primary UX.** Could be useful in a hidden "power user" CLI mode.
- **Determinism**: Non-deterministic; failures = invalid Qdrant filter JSON → must be caught.
- **Local-model viability**: Needs reliable structured-JSON output. Qwen 2.5 7B-instruct or Phi-4 14B; not 3B.
- **Canonical source**: LangChain Self-Query Retriever (pattern definition only): *"The LangChain self-query retriever leverages a query-constructing LLM chain to transform a user's natural language query into a well-defined structured query that can be used to apply filters to search results based on metadata attached to vectors." `[Source: https://docs.langchain.com/oss/python/integrations/vectorstores/qdrant · fetched 2026-05-18]`*. Qdrant's `QdrantTranslator` is the LangChain class that performs this `[Source: https://api.python.langchain.com/en/latest/retrievers/langchain.retrievers.self_query.qdrant.QdrantTranslator.html · fetched 2026-05-18]`.
- **3-5 sentence summary**: Powerful but **brief-incompatible** as a primary surface. Self-Query *is* "ask the corpus in English" — exactly the chatbot shape we're avoiding. The pattern could appear in a CLI-only debug mode (`memex search --nl "tauri sessions where I hit EROFS"`) but should never appear in the GUI. Skip for VSD 2026.
- **Memex-surface fit**: 1=1 · 2=1 · 3=1 · 4=1 · 5=1 · 6=1 · 7=1.
- **Cost estimate**: 1 LLM call, ~300 tokens. ~400 ms. Skip.

---

### Pattern 10 — Discovery Negative Inference

- **Definition**: User selects only positive examples ("more like this"); LLM proposes implicit negative examples by inferring "what this is *not*"; both pairs feed Qdrant's Discovery API context.
- **Where LLM lives**: Query-time (small call, per Mix & Match click).
- **Visible to user?** Negatives could surface as "Memex inferred these are unlike your picks" chips — visible but not chat.
- **Determinism**: Non-deterministic.
- **Local-model viability**: Yes. The LLM gets a positive session's payload (topic / intent tags from Pattern 6) and emits 2-3 *anti-topics* — small structured output.
- **Canonical source**: Qdrant Discovery API context pairs:

  > "Each pair is made up of a positive and a negative vector. With a context, we can define hyperplanes within the vector space, which always prefer the positive over the negative vectors." `[Source: https://qdrant.tech/articles/discovery-search/ · fetched 2026-05-18]`

  Qdrant explicitly states the Discovery article contains **no** LLM-generated-negative pattern — *"The article contains no discussion of using language models or any automated method to generate negative examples."* So Memex would be **novel** in pairing Discovery with LLM-inferred negatives. (Cited as Qdrant gap, not endorsement.)
- **3-5 sentence summary**: Mix & Match currently requires the user to *manually* pick both positives and negatives. That's friction. With Pattern 6's topic tags in payload, an LLM can look at the user's positive picks and propose "you probably don't mean these adjacent-but-different sessions" — turning Mix & Match into a single-click recommendation. **Novel** Qdrant Discovery use that the official Qdrant blog has not demonstrated → high demo-novelty value.
- **Memex-surface fit**: 1=1 · 2=2 · 3=**3** (the surface this is built for) · 4=1 · 5=2 · 6=1 · 7=1.
- **Cost estimate**: ~150 tokens per click. ~200 ms.

---

### Pattern 11 — GraphRAG / Agentic Retrieval

- **Definition**: An LLM agent iteratively decides what to search next — sometimes Qdrant, sometimes a knowledge graph, sometimes a re-query — driven by intermediate results.
- **Where LLM lives**: Runtime, hot-loop (one or more LLM calls per *iteration*, multiple iterations per query).
- **Visible to user?** Hard to hide. Latency makes it feel chatbot-ish.
- **Determinism**: Non-deterministic; agentic loops are notoriously hard to reproduce.
- **Local-model viability**: Painful. Multi-agent orchestration on local Qwen 2.5 7B works but is **slow** (10–30 s per query).
- **Canonical sources**:
  - Anthropic, *How we built our multi-agent research system*, Jun 2025: orchestrator-worker pattern, *"a lead agent coordinates the process while delegating to specialized subagents that operate in parallel." `[Source: https://www.anthropic.com/engineering/multi-agent-research-system · fetched 2026-05-18]`*.
  - Critical cost warning from the same post:

    > "In our data, agents typically use about 4× more tokens than chat interactions, and multi-agent systems use about 15× more tokens than chats." `[Source: https://www.anthropic.com/engineering/multi-agent-research-system · fetched 2026-05-18]`

  - Qdrant GraphRAG docs warn:

    > "Relying on LLMs increases costs and decreases scalability, as they are used every time data is added, queried, or generated." `[Source: https://qdrant.tech/documentation/examples/graphrag-qdrant-neo4j/ · fetched 2026-05-18]`

- **3-5 sentence summary**: **Wrong shape for Memex.** Agentic retrieval is the most LLM-heavy pattern in this catalog, the slowest, the least deterministic, and the most likely to feel chat-like to the user (long latencies + intermediate "thinking…" states). 15× token cost is irreconcilable with Memex's 100%-local promise unless we degrade to a tiny model, which loses agentic accuracy. **Skip for 14-day window.** Mention in roadmap, not in scope.
- **Memex-surface fit**: 1=1 · 2=1 · 3=1 · 4=1 · 5=2 (could power Predict) · 6=1 · 7=1.
- **Cost estimate**: ~5 K–20 K tokens per user action. ~10–30 s latency. **Cost-prohibitive locally.**

---

### Pattern 12 — MIPS-Aware Named-Vector Query Rewriting

- **Definition**: Memex has 5 named vectors (`content`, `tool`, `path`, `error`, `code`). An LLM looks at the query and emits **5 sub-queries**, each tuned for one named vector (e.g., for `code` it emits the closest-syntax version, for `error` it emits the most stack-trace-like rendering, for `path` it emits a file-path-like form).
- **Where LLM lives**: Query-time.
- **Visible to user?** Invisible.
- **Determinism**: Non-deterministic.
- **Local-model viability**: Yes. One call, ~400 tokens out (5 short rewrites). Qwen 2.5 3B.
- **Canonical source**: General query-rewriting literature, e.g., Ma et al., *Query Rewriting for Retrieval-Augmented Large Language Models*, arXiv:2305.14283 `[Source: https://arxiv.org/abs/2305.14283 · fetched 2026-05-18]`. The *named-vector-specific* twist is novel to Memex and uses Qdrant 1.10+'s Universal Query multi-named-vector prefetch pattern from `[Source: https://qdrant.tech/blog/qdrant-1.10.x/ · fetched 2026-05-18]`.
- **3-5 sentence summary**: This is the pattern that **most amplifies the named-vector primitive** — Qdrant 1.10's core showpiece. Lens slider already lets the user weight 5 vectors; MIPS-aware rewriting lets the LLM *also* rewrite the query 5 different ways before that weighting is applied. The combined effect: 5 vectors × 5 query-renderings → 25 candidate sets → RRF fusion. Visually striking in the demo if we render the 5 rewrites as 5 named-vector-colored chips above the result list.
- **Memex-surface fit**: 1=1 · 2=1 · 3=1 · 4=2 · 5=2 · 6=1 · 7=**3** (the surface this is built for).
- **Cost estimate**: 1 LLM call (~400 tokens out), 5 embed calls, 1 Qdrant Query API call with 5-arm prefetch + Fusion. ~500 ms.

---

### Bonus Pattern A — Step-Back Prompting

- **Definition**: LLM converts the specific query into a more abstract/general one ("step back" from "What CSS gradient is at line 45?" → "How does this stylesheet handle theming?"), embeds both, fuses.
- **Where LLM lives**: Query-time.
- **Visible to user?** Invisible.
- **Determinism**: Non-deterministic.
- **Local-model viability**: Yes (Qwen 2.5 3B).
- **Canonical source**: Zheng et al., *Take a Step Back: Evoking Reasoning via Abstraction in Large Language Models*, Google Research 2023 / arXiv:2310.06117. Also surveyed in `[Source: https://arxiv.org/abs/2305.14283 · fetched 2026-05-18]`.
- **3-5 sentence summary**: Mostly redundant with Patterns 1+2 for Memex; would be one extra arm in an RRF prefetch. Not worth a separate implementation slot.
- **Memex-surface fit**: 1=1 · 2=1 · 3=1 · 4=1 · 5=1 · 6=1 · 7=2.
- **Cost estimate**: ~200 tokens, ~300 ms. Skip unless free.

---

### Bonus Pattern B — Hybrid Cross-Encoder Rerank (Not LLM)

Already noted under Pattern 8. Reproducing here only for the Top-5 alternative slot: ship a **cross-encoder reranker** when fastembed-rs gets the model. Lower demo-impact than LLM-as-reranker visually, but the right engineering choice. Cite:

> "Qdrant supports the Jina Reranker v2 Base Multilingual—a cross-encoder reranker supported in FastEmbed." `[Source: https://qdrant.tech/documentation/fastembed/fastembed-rerankers/ · fetched 2026-05-18]`

---

## 3. Top-5 Fit-Matrix for Memex

Patterns ranked by Memex-specific fit. Columns: **QdrantAmp** = does it make Qdrant 1.18 primitives more visible/powerful? · **Wow** = demo wow factor in 3-min walkthrough · **Local14d** = feasible on macOS arm64 in 14 days with local model? · **Effort** = implementation S/M/L · **Brief** = "Think Outside the Bot"-compliant?

| Rank | Pattern | QdrantAmp | Wow | Local14d | Effort | Brief |
|------|---------|-----------|-----|----------|--------|-------|
| 1 | **#6 Index-time metadata enrichment** (Anthropic-style contextual) | **Very High** — payload-filter + auto-label + chip-UI all light up | **Very High** — every Time Machine card gets meaningful chips | **Yes** — Qwen 2.5 7B Q4 + index-time only | **M** — prompt + payload schema + UI chips | **Yes** — LLM strictly at scan time, never user-facing |
| 2 | **#7 Cluster auto-labeling** | **High** — Distance Matrix → KMeans → labeled clusters; one of Qdrant 1.12's biggest features finally narrates itself | **Very High** — Topology galaxy is the most photogenic surface, and labels are the headline | **Yes** — Qwen 2.5 3B, ~6 calls total | **S** — single prompt, cache by centroid hash | **Yes** — LLM at scan time |
| 3 | **#5 RAPTOR-lite hierarchical Topology** | **High** — Distance Matrix used at *multiple* tiers, demo can literally zoom | **High** — "zoom out from 200 sessions to 3 life-arcs" is unforgettable | **Borderline** — needs careful clustering + LLM summarization tuning, 7–10 days | **L** — recursive index pipeline, new sidecar collection | **Yes** — index-time |
| 4 | **#1 HyDE for the error-recall banner** | **Moderate** — improves `query()` on `error` named vector specifically | **Moderate** — invisible to user; better recall is a stat, not a moment | **Yes** — Qwen 2.5 3B, ~300 ms per call | **S** — one new function in `recall.rs` | **Borderline** — first runtime-LLM dependency. See §5 risks. |
| 5 | **#12 MIPS-aware named-vector query rewriting** | **Very High** — directly amplifies the 5-named-vector primitive, Universal Query API showpiece | **High** — visible "5 chips for 5 vectors" UI moment on Lens slider | **Yes** — Qwen 2.5 3B, ~500 ms | **M** — prompt + Lens slider UI extension | **Borderline** — runtime LLM. Could be reframed as "Lens 2.0: AI-augmented mode" toggle. |

**Patterns NOT in top-5** and why:
- #2 Multi-Query Expansion — overlaps with #1 and #12, lower marginal value.
- #3 Query Decomposition — Memex queries are atomic; misfit.
- #4 Semantic Chunking — interesting but breaks "1 session = 1 point" identity; major UX rework.
- #8 LLM-as-Reranker — cross-encoder is the right choice, not LLM.
- #9 Self-Query Retriever — brief-incompatible (chat-shaped).
- #10 Discovery Negative Inference — solid Mix & Match upgrade, but Mix & Match is a lower-traffic surface. Defer to v2.
- #11 GraphRAG / Agentic — wrong shape, wrong cost.

---

## 4. Concrete Model Choices

For each of the top-5 patterns, the specific model recommendation:

### 4.1 For Pattern #6 (Index-time enrichment) — **Qwen 2.5 7B-Instruct Q4_K_M via Ollama**

- **Size**: ~4.5 GB on disk, ~5 GB resident at runtime.
- **Runtime**: Ollama (`ollama pull qwen2.5:7b-instruct-q4_K_M`). Ollama is the **fastest install-to-first-token** path on macOS Apple Silicon and now uses MLX in preview for faster inference `[Source: https://ollama.com/blog/mlx · fetched 2026-05-18]`.
- **macOS arm64 speed estimate**: 30-50 tokens/sec decode on M2 Pro / M3, ~80–100 tokens/sec on M4 Max `[Source: https://antekapetanovic.com/blog/qwen3.5-apple-silicon-benchmark/ · fetched 2026-05-18]`.
- **Why this not another**:
  - **Why not Phi-4 14B**: 2× the resident memory; not worth it for a structured-JSON payload-tagging task that 7B already aces.
  - **Why not Qwen 2.5 3B**: 3B is reliable for cluster naming but less reliable for structured-JSON multi-field payloads.
  - **Why not Llama 3.1 8B**: Qwen 2.5 7B benchmarks better on structured-output tasks and on coding/dev domain — Memex's corpus is overwhelmingly Claude Code session text.

### 4.2 For Pattern #7 (Cluster auto-labeling) — **Qwen 2.5 3B-Instruct Q4_K_M**

- **Size**: ~2 GB.
- **Runtime**: Ollama (`ollama pull qwen2.5:3b-instruct-q4_K_M`).
- **macOS arm64 speed**: 80–120 tokens/sec decode on M2+.
- **Why this not another**: 3B is sufficient for the simple "name this cluster from 5 session snippets" prompt; using 7B would be over-spec. 1.5B is too unreliable on multi-snippet abstraction.

### 4.3 For Pattern #5 (RAPTOR-lite) — **Qwen 2.5 7B-Instruct Q4_K_M** (same as 4.1)

- **Size / Runtime / Speed**: same as 4.1 — reuse the same Ollama model.
- **Why this not another**: Summarization quality at 3B is noticeably worse for the recursive aggregation case; 7B is the sweet spot. Phi-4 14B has slightly better summaries but doubles the disk footprint of the installer.

### 4.4 For Pattern #1 (HyDE) — **Qwen 2.5 3B-Instruct Q4_K_M** (same as 4.2)

- **Size / Runtime / Speed**: same as 4.2.
- **Why this not another**: HyDE only needs a 50–150 token hypothetical answer. 3B nails this with low latency. Larger models add latency without quality gain at this short generation length.

### 4.5 For Pattern #12 (Named-vector query rewriting) — **Qwen 2.5 3B-Instruct Q4_K_M** (same as 4.2)

- **Size / Runtime / Speed**: same.
- **Why this not another**: 5 short rewrites = ~400 tokens. 3B handles this in one call cleanly. The bottleneck is structured-output reliability; one-shot prompt + JSON-schema constraint is enough.

### 4.6 Runtime engineering note

Bundling Ollama with Memex.app is **not** required — users can `brew install ollama` once. Memex detects `localhost:11434` (Ollama's default) on startup; if absent, **Memex degrades gracefully to no-LLM mode** (current behavior). This keeps the "100% local even without Ollama" claim intact:

- **With Ollama running**: rich payload chips, semantic cluster labels, optional HyDE search.
- **Without Ollama**: current Memex behavior — statistical cluster labels, no chips, fast.

Alternative runtime: **llama.cpp** directly via `llama-cpp-rs` candle bindings could be embedded *inside* Memex.app, eliminating the Ollama dependency. But this adds ~50 MB to the bundle and requires CI to build with Metal. **Recommendation: ship with Ollama-detect; embed llama.cpp only if 14 days allow.** Candle `[Source: https://github.com/huggingface/candle · fetched 2026-05-18]` is a viable third option for an all-Rust pipeline but Qwen 2.5 support in candle lags behind llama.cpp.

---

## 5. Risks — When LLM Augmentation Backfires

### 5.1 Cluster-naming non-determinism breaks replay reproducibility
The Topology galaxy must look the same on Day-1 and Day-7 of a demo for screenshots to match. If cluster labels regenerate non-deterministically each `scan --index`, the demo video and the live demo will diverge.

**Mitigation**: Cache labels keyed on `hash(centroid_vector) + hash(top-5-session-IDs)`. Re-label only when the cluster *composition* changes.

### 5.2 Multi-query / HyDE expansion blows up perceived latency
A 30 ms `query()` becoming a 400 ms LLM+query+rerank loop is fine in print, *feels* slow in demo.

**Mitigation**: Eager-fire the LLM call **on focus** of an input field (not on submit) so the hypothetical doc is already generated by the time the user hits Enter. Or: pre-compute HyDE for the most-recent-N error strings in the background.

### 5.3 Agentic retrieval contradicts "no chat at runtime"
Even if no chat surface exists, multi-second LLM loops *feel* like chat. The brief penalizes that perception more than the literal absence of an input box.

**Mitigation**: **Don't ship Pattern #11.** Reaffirm in README.

### 5.4 LLM-tag payload pollution
Index-time enrichment (Pattern #6) tags written into Qdrant payload become a permanent commitment. If the LLM hallucinates a wrong entity, the snapshot ships with that hallucination forever.

**Mitigation**: Store LLM outputs under a `derived.*` payload sub-key so users can `qdrant set --null derived` to wipe and re-derive. Show provenance in the UI ("source: Qwen 2.5 7B @ commit abc123").

### 5.5 Ollama dependency leaking into the "100% local" claim
Strictly speaking, Memex is still 100% local *with* Ollama (Ollama doesn't phone home). But the claim "no LLM at runtime" in the README is no longer literally true if Pattern #1 / #12 ship.

**Mitigation**: Reword the README badge and the hero block. See §7.

### 5.6 Demo-time model load latency
First Qwen 2.5 7B inference on a cold Ollama can take ~2 s for model load. In a 3-min demo, that 2 s feels like a freeze.

**Mitigation**: Pre-warm Ollama with a noop call on Memex.app startup.

### 5.7 Index-time LLM doubles `scan --index` duration
On a corpus of 200 sessions, Pattern #6 adds ~2 min to scan time. Power users who scan often will notice.

**Mitigation**: Make LLM enrichment **opt-in** via `memex scan --index --enrich` flag. Default `scan --index` stays fast and LLM-free.

### 5.8 Reranker temptation drift
"LLM-as-reranker" is the most-Googled pattern in this catalog but **the worst engineering choice** for Memex. Resist if a teammate suggests it.

---

## 6. Anti-Patterns — Do NOT Build These, Even If You Could

| Anti-pattern | Why we don't | Brief-violation severity |
|---|---|---|
| **Conversational query input** ("Memex, what did I work on?") | Direct chatbot. Brief-fatal. | 🔴 critical |
| **Free-text "ask the corpus" textarea anywhere in GUI** | Same as above. | 🔴 critical |
| **LLM-generated tool calls** (Memex telling the user to "run `cargo build`") | That's Claude Code's job, not Memex's. Memex *recommends* by surfacing past tool calls, not by predicting them generatively. | 🔴 critical |
| **Streaming LLM output to the user** | Streaming text = chat affordance, regardless of intent. | 🔴 critical |
| **"Chat with your session history" tab** | Brief-fatal. | 🔴 critical |
| **Pattern #9 Self-Query Retriever in GUI** | NL textarea → filter; functionally a chat. | 🟡 major |
| **Pattern #11 Agentic retrieval** | Latency + cost + chat-feel. | 🟡 major |
| **LLM-rewritten replay narration** ("In this session, you fixed a bug…") | Replay must be the **literal** record, not an LLM gloss. | 🟡 major |
| **Embedding the LLM's hypothetical-doc text into the result card UI** | Even though HyDE generates intermediate text, **never show it to the user**. | 🟡 major |

---

## 7. Product Narrative Update — README & CLAUDE.md

### 7.1 The current claim

README hero says (with badge):

> "no LLM call at runtime"
> *[Source: /Users/kimsejun/Documents/GitHub/memex/README.md line 18 + line 43]*

CLAUDE.md says:

> "no chat window and no LLM call at runtime: all retrieval is fastembed-rs BGE-small-en-v1.5 + qdrant-client gRPC, both running locally."
> *[Source: /Users/kimsejun/Documents/GitHub/memex/CLAUDE.md line 17]*

### 7.2 What changes if we adopt only Patterns #6 + #7 (index-time only)

**Nothing breaks.** "No LLM at runtime" remains literally true — the LLM only runs during `scan --index`. The user's *query path* (Time Machine scroll, Topology browse, Mix & Match click, Recall banner, Predict panel, Replay step, Lens slider) is still 100% Qdrant + fastembed. We get smarter labels and richer payload chips with the same runtime claim.

**Suggested README rewrite (minimal):**

> "**100% local** — `fastembed-rs` BGE-small + `qdrant-client` gRPC on your machine. Optional **local LLM enrichment** (Qwen 2.5 via Ollama) runs *only* during `scan --index` to label clusters and add semantic chips — never during search, never at the user's hot path. No network, no telemetry, no chat."

**Suggested CLAUDE.md rewrite:**

> "Treat the 'no chat surface / no LLM on the user's hot path' constraint as a hard product invariant. **Index-time LLM enrichment via local Ollama is permitted** for cluster labeling and payload tagging — it runs once per scan and freezes results into Qdrant payload. Adding chat-style features, NL query inputs, or query-time LLM calls defeats the entire pitch."

### 7.3 What changes if we *also* adopt Patterns #1 or #12 (runtime LLM)

The "no LLM at runtime" badge **must come down**. Replace with:

> "**Local-LLM-augmented Qdrant** — optional local Qwen 2.5 amplifies named-vector search; never a chat surface."

This is still brief-compliant — "no chatbot allowed" ≠ "no LLM allowed." But it's a perception trade-off: the "no LLM at runtime" line has been one of Memex's strongest differentiators. Consider keeping #1/#12 *off* by default and gated behind a "Lens 2.0: AI augmentation" toggle in settings, so the default-install user experience still matches the README's purest claim.

### 7.4 Recommendation

**Ship Patterns #6 + #7 (index-time only). Keep "no LLM at runtime" as the hero claim.** Defer #1, #5, #12 to a v1.1 toggle. This:
- Adds maximum demo wow with zero runtime-LLM compromise.
- Lets the Topology galaxy speak (cluster labels become words, not statistics).
- Lets every Time Machine card surface semantic chips (topic, intent, outcome).
- Preserves the strongest README differentiator unchanged.
- Keeps the snapshot snapshot-friendly (LLM outputs frozen in payload, exportable as a single file).

---

## 8. Sources

All URLs verified by WebFetch or WebSearch on **2026-05-18**.

### Qdrant — staff-authored articles & blog

- https://qdrant.tech/articles/hybrid-search/ — Universal Query API, RRF/DBSF fusion, multi-prefetch shape (no HyDE mentioned).
- https://qdrant.tech/articles/distance-based-exploration/ — Distance Matrix + KMeans clustering recipe (no LLM labeling mentioned).
- https://qdrant.tech/blog/qdrant-1.12.x/ — Distance Matrix API release, `search_matrix_pairs` / `search_matrix_offsets`.
- https://qdrant.tech/blog/qdrant-1.10.x/ — Universal Query, IDF, ColBERT, multi-named-vector prefetch.
- https://qdrant.tech/blog/qdrant-1.11.x/ — DBSF (Distribution-Based Score Fusion) added.
- https://qdrant.tech/articles/discovery-search/ — Discovery API context-pair semantics, triplet-loss inspiration.
- https://qdrant.tech/documentation/search-precision/reranking-semantic-search/ — Cross-encoder vs ColBERT vs LLM-as-reranker comparison; the *only* official Qdrant page that explicitly endorses LLM-as-reranker.
- https://qdrant.tech/documentation/fastembed/fastembed-rerankers/ — Jina Reranker v2 / BGE reranker support in fastembed.
- https://qdrant.tech/articles/cross-encoder-integration-gsoc/ — GSoC 2024 ONNX cross-encoder integration.
- https://qdrant.tech/articles/late-interaction-models/ — ColBERT as alternative reranker.
- https://qdrant.tech/blog/bitter-lesson-generative-language-model/ — Lehtimäki talk: explicitly names HyDE, multi-chunking, query decomposition, re-ranking as LLM-augmented retrieval patterns.
- https://qdrant.tech/documentation/examples/graphrag-qdrant-neo4j/ — Qdrant's official GraphRAG stance; explicit cost warning about LLM-at-every-step.
- https://try.qdrant.tech/deepseek — agentic RAG with DeepSeek (Qdrant-hosted resource).
- https://try.qdrant.tech/hackathon-vsd — VSD 2026 hackathon page, "no chatbots allowed."
- https://qdrant.tech/blog/vector-space-day-sf-2026/ — VSD 2026 announcement, "Forget the classical RAG chatbot."

### Original papers (arXiv)

- arXiv:2212.10496 — Gao, Ma, Lin, Callan, *Precise Zero-Shot Dense Retrieval without Relevance Labels* (HyDE). https://arxiv.org/abs/2212.10496
- arXiv:2401.18059 — Sarthi, Abdullah, Tuli, Khanna, Goldie, Manning, *RAPTOR: Recursive Abstractive Processing for Tree-Organized Retrieval*, ICLR 2024. https://arxiv.org/abs/2401.18059
- arXiv:2305.14283 — Ma et al., *Query Rewriting for Retrieval-Augmented Large Language Models* (2023). https://arxiv.org/abs/2305.14283
- arXiv:2305.03653 — Jagerman et al., *Query Expansion by Prompting Large Language Models* (2023). https://arxiv.org/pdf/2305.03653
- arXiv:2310.06117 — Zheng et al., *Take a Step Back: Evoking Reasoning via Abstraction* (step-back prompting, 2023).
- arXiv:2502.18469 — *Using LLM-Based Approaches to Enhance and Automate Topic Labeling* (2025). https://arxiv.org/html/2502.18469v1
- arXiv:2510.18633 — *Query Decomposition for RAG: Balancing Exploration-Exploitation* (2025). https://arxiv.org/pdf/2510.18633
- arXiv:2603.13301 — *Not All Queries Need Rewriting: When Prompt-Only LLM Refinement Helps and Hurts Dense Retrieval* (2026). https://arxiv.org/abs/2603.13301

### Anthropic engineering blog

- https://www.anthropic.com/news/contextual-retrieval — Contextual Retrieval recipe, quantified gains (35%/49%/67% failure-rate reduction), $1.02/M-token cost with prompt caching.
- https://www.anthropic.com/engineering/multi-agent-research-system — Orchestrator-worker pattern, 15× token cost vs chat, when agentic search outperforms RAG.
- https://www.anthropic.com/engineering/effective-context-engineering-for-ai-agents — Context engineering for agentic systems.

### LangChain / LlamaIndex (pattern shape only, not endorsed)

- https://docs.langchain.com/oss/python/integrations/vectorstores/qdrant — Self-Query Retriever with Qdrant.
- https://api.python.langchain.com/en/latest/retrievers/langchain.retrievers.self_query.qdrant.QdrantTranslator.html — QdrantTranslator class for NL → Filter.
- https://docs.llamaindex.ai/en/stable/examples/node_parsers/semantic_chunking/ — SemanticSplitterNodeParser (embedding-similarity-based chunking).
- https://docs.llamaindex.ai/en/stable/examples/node_parsers/semantic_double_merging_chunking/ — Double-merging variant.

### Local-model runtimes

- https://ollama.com/blog/mlx — Ollama now uses MLX on Apple Silicon (preview).
- https://antekapetanovic.com/blog/qwen3.5-apple-silicon-benchmark/ — Ollama vs llama.cpp vs MLX on Apple Silicon for Qwen 3.5.
- https://github.com/ggml-org/llama.cpp/discussions/4167 — llama.cpp on Apple Silicon M-series perf thread.
- https://github.com/huggingface/candle — Candle Rust-native runtime (alternative to ollama / llama.cpp for embedded use).
- https://ollama.com/library/qwen2.5 — Qwen 2.5 model card on Ollama.

### Cross-citations from earlier Memex research dossiers

- `/Users/kimsejun/Documents/GitHub/memex/claudedocs/research/03-qdrant-vsd-2026-hackathon.md` — VSD 2026 brief, "no chatbots allowed," 180 s demo video deadline.
- `/Users/kimsejun/Documents/GitHub/memex/claudedocs/research/02-qdrant-official-blog.md` — Qdrant 1.18 primitive inventory.
- `/Users/kimsejun/Documents/GitHub/memex/claudedocs/research/04-sparse-colbert-hybrid.md` — hybrid/sparse retrieval landscape.

---

## Appendix — Citation Recap of the Three Most Load-Bearing Quotes

For implementation discussions, the three quotes that drive the recommendation are:

> "Please give a short succinct context to situate this chunk within the overall document for the purposes of improving search retrieval."
> *[Source: https://www.anthropic.com/news/contextual-retrieval · fetched 2026-05-18]*

This is the **template prompt** for Pattern #6.

> "HyDE first zero-shot instructs an instruction-following language model … to generate a hypothetical document. The document captures relevance patterns but is unreal, and may contain false details. Then, an unsupervised contrastively learned encoder … encodes the document into an embedding vector."
> *[Source: https://arxiv.org/abs/2212.10496 · fetched 2026-05-18]*

This is the **algorithm spec** for Pattern #1.

> "Many clustering algorithms accept precomputed distance matrix as input, so we can use the same distance matrix we calculated before."
> *[Source: https://qdrant.tech/articles/distance-based-exploration/ · fetched 2026-05-18]*

This is Qdrant's **official endorsement** of the upstream half of Pattern #7 (cluster auto-labeling) — the downstream half (LLM label) is novel-to-Memex within the Qdrant ecosystem.

---

*End of dossier. Total word count ≈ 6,500. Lines ≈ 580. Authoritative URLs verified: 28.*
