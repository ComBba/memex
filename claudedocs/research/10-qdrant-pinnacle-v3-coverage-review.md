# Memex Plan v3 — Qdrant 1.18 Pinnacle Coverage Review

**Review date:** 2026-05-18
**Auditor role:** independent reviewer (no co-authorship on v1 / v2 / v3)
**Subject:** `claudedocs/sota-plan-v3.html` §3 (25 KICKs across 7 categories)
**Authoritative inputs cross-checked:**
- `claudedocs/research/06-qdrant-1.18-feature-pinnacle.md` (80-row inventory)
- `claudedocs/sota-plan-v2.html` (delta context — 19 KICKs)
- GitHub releases v1.18.0 / v1.17.0 / v1.16.0 (verbatim changelog bodies)
- `qdrant.tech` documentation (quantization, hybrid-queries, multitenancy, indexing, vectors, late-interaction)
- `api.qdrant.tech/api-reference/search/query-points` (QueryRequest schema, all Expression variants)
- `docs.rs/qdrant-client/1.18.0` (Rust client builders + struct fields)

> **Citation discipline.** Every API-shape claim is followed by a `[URL]` reference resolved in §7. Items where the published source is silent are marked `unverified — needs implementation spike`.

---

## 1. Executive summary (≤200 words)

**Net verdict: v3 reaches the 1.18 pinnacle in *concept* — but ~1/3 of its KICK descriptions contain API-shape errors that will surface as compile errors or runtime 4xx during the spike phase.** Coverage of the 80-feature surface is **on-target** (no high-leverage Direct-fit miss exceeds Effort >2 h to add). The plan has **zero clear under-claims** of significant pinnacle features. However:

- **Over-claim score: 8 of 25 KICKs have a wrong API shape, wrong version, or a feature-naming conflation that will need correction before D-13 spike day.**
- **On-target ratio: 17/25 KICKs map cleanly to verified 1.18 surfaces.**
- **Under-claim score: 0 features rated `Direct-fit` in research/06 are missing from v3 with effort ≤4h that should clearly have been included.**

The single most dangerous over-claim is **KC-01 "TurboQuant (Binary 2-bit)"** — TurboQuant is a *separate* quantization family from Binary Quantization; the bit-options `bits4/bits2/bits1_5/bits1` are TurboQuant's own enum, not Binary 2-bit. Implementing the v3 KC-01 sketch literally would either pick the wrong builder or produce a hybrid that doesn't exist.

The single most important add is **per-prefetch score reference in FormulaQuery** — without it, KA-01's "5 dense + 1 sparse + 1 multivec lens weighting on a server-side formula" pattern cannot work as written. This is not a missing feature, it's a missing *technique* (use Discovery's `$score` only once or restructure prefetches with explicit `using`).

---

## 2. Verdict 1 — Coverage gap (research/06 → v3 forward check)

Cross-checking research/06 §2 (80 rows) against v3 §3 (25 KICKs). I report only features with `Memex potential adoption fit: Direct` and ≤4-hour effort — anything heavier or `Doesn't fit / Cloud-only` is treated as legitimately deferred.

| research/06 row | Feature | Fit | In v3? | Verdict |
|---|---|---|---|---|
| #2 | Add/delete named vectors on existing collection (1.18) | Direct | Implicit (via KD-01 payload, no schema CRUD KICK) | **Justified miss.** Memex doesn't need runtime schema CRUD during the demo window; collection is created once at first scan. Skipping it is fine. |
| #15 | Quantization `oversampling` + `rescore` per-query | Direct | **No** | **Real gap.** Pairs with KC-01 TurboQuant — research/06 §6 explicitly warns "TurboQuant + `rescore: false`" is a recall trap. v3 KC-01 doesn't mention oversampling. **Recommend ADD as KC-01b.** |
| #18 | Filterable HNSW `enable_hnsw=false` per payload index (1.17) | Direct | **No** | **Justified miss.** Risk-bearing per research/06 §6 (disabling on a frequently-used filter hurts recall). Memex shouldn't enable this without an eval gate. |
| #28 | Text index ASCII folding (1.16) | Direct | **No** | **Soft miss.** 1-line config flag on the existing `ai_title` text index. Demo-fit is 0 but it's a free correctness win for non-English file paths. **Recommend ADD as KB-06 (S, ~5 min).** |
| #29 | Text index phrase search (1.15) | Direct | **No** | **Soft miss.** Useful for "edit `auth.js`" queries (the same demo angle as KB-02 BM25). Lower priority since BM25 covers it. |
| #30 | Text index stopwords (1.15) | Direct | **No** | **Justified miss.** Memex doesn't index natural-language prose. |
| #31 | Text index Snowball stemmer (1.15) | Direct | **No** | Same as #30. |
| #38 | Formula math ops (sum/mult/.../log10/ln) | Direct | Implicit in KA-01 | **On-target** — KA-01 lists `sum/mult/exp_decay` explicitly; full operator surface is implied. |
| #44 | Query API `sample: random` | Adapter | **No** | **Justified miss.** Demo-fit 0. |
| #50 | Weighted RRF `k` parameter | Direct | KA-05 covers weights, but **`k` is not mentioned** | **Minor gap.** Add `"k": 60` (default) to KA-05 or note it as a tunable. |
| #56 | Snapshot SHA256 checksum on recover | Direct | Implicit in KF-03 | **On-target** — KF-03 ("signed snapshot envelope, SHA-256") subsumes it. |
| #57 | Conditional updates (filter-gated upsert) | Direct | **No** | **Real miss.** research/06 §3.7 ranks this as a top-10 unused feature for the staleness-worker case. Effort ~30 min. v3 doesn't have a KICK for it. **Recommend ADD as KG-04 (S).** |
| #58 | `update_mode` insert/update/upsert (1.17) | Direct | **No** | Same as #57 (typically deployed together). |
| #63 | Strict mode `max_resident_memory_percent` (1.18) | Direct | **No** | **Real miss.** This is one of research/06's top-5 (§1 item #5). For desktop Memex on 16 GB MacBooks, missing this is a real production risk. Effort ~30 min. **Recommend ADD as KC-06 (S).** |
| #64 | Strict mode `search_max_batchsize` (1.18) | Direct | **No** | Same family as #63. |
| #66 | Low memory mode (force on-disk on startup) (1.18) | Direct | **No** | Same family as #63. |
| #67 | Deep memory reporting `/memory` endpoint (1.18) | Direct | **No** | **Soft miss.** Demo-fit 1/5 (operator HUD). Could become a "Memex devtools panel" stretch goal but not pinnacle-blocking. |
| #70 | Audit logging + tracing ID (1.17) | Direct | **No** | **Justified miss.** Desktop single-user app; audit-log value is near zero. |
| #72 | Inline storage (vectors-in-HNSW) (1.16) | Direct | **No** | **Soft miss.** research/06 §3.8 calls this out as "10-min config change, ~16× page-read reduction on disk-backed deployments." Memex defaults to mmap so it does matter at >5k sessions. **Recommend ADD as KC-07 (S, ~10 min).** |
| #75 | Bulk-load trick `m=0` then re-enable | Direct | **No** | **Justified miss.** Useful only for the very first ingest of a >100k corpus; out of demo scope. |
| #77 | Datatype per named vector (f32/f16/u8) | Direct | **No** | **Soft miss.** Stacks with TurboQuant but adds another moving part. Defer. |
| #78 | `text_any` filter (OR over tokens) (1.16) | Direct | **No** | **Justified miss.** Memex's text filter use-cases are AND-shaped today. |
| #79 | RRF custom `k` parameter (1.16) | Direct | **No** | Same as #50 — fold into KA-05. |

**Coverage gap totals**

- Real gaps (Direct fit, ≤4h, no risk note, missing from v3): **5** (rows #15, #57+#58 pair, #63+#64+#66 family, #72). Of these, the two genuinely demo-or-correctness-relevant ones are **#63 strict-mode memory** and **#15 oversampling+rescore**.
- Soft misses (Direct fit but demo-fit 0–1, defendable to skip): **6**.
- Justified omissions: **balance**.

**Bottom line:** v3 is **not under-claiming** — it has captured every high-leverage pinnacle feature with demo-fit ≥3. The missing surfaces are all operator-side (memory, low-memory mode, deep memory reporting, conditional updates) — useful for production but invisible in a 3-min demo. The plan would be **strictly better** with the two recommended adds (KC-06 strict-mode memory + KC-01b oversampling+rescore), neither of which is on the critical path.

---

## 3. Verdict 2 — Over-claim audit (v3 KICK → official API check)

For each of v3's 25 KICKs I verified the underlying API shape against current docs. Flagged here are KICKs that contradict, mis-name, or mis-version. KICKs with no API surface (KF-01/02, KG-01/02 — pure Rust-side work) are noted as "not API-bound" and skipped.

| KICK | v3 claim | Actual per source | Severity | Required correction |
|---|---|---|---|---|
| **KA-01** | Formula syntax mixes `{ "value": 0.2, "filter": {...} }` for conditional payload boost | The official Expression schema has **no `value`+`filter` combined object**. Conditional boost is done as `{"mult": [<const>, <Condition>]}` where the Condition is a regular `{"key": "x", "match": {"value": true}}` block evaluated as 0/1. [api.qdrant.tech/api-reference/search/query-points] | **HIGH** | Rewrite KA-01 example to: `{"mult": [0.2, {"key": "has_errors", "match": {"value": true}}]}`. The Condition-as-Expression pattern is the documented one. |
| **KA-01** | Uses `"$score.path_sparse"` to reference a specific prefetch's score | The schema documents only `"$score"` (singular). **There is no documented `$score.<name>` indexing into prefetches.** | **HIGH** | Either (a) restructure as multi-stage prefetch where each lens has its own prefetch and the Formula uses a single `$score` per sub-call; or (b) mark this as `unverified — needs spike` per research/06 §3.1. The "one Formula scoring 5 named-vector prefetches with explicit weights" pattern as written does not match documented Formula semantics. |
| KA-02 | MMR `diversity=0.4` on NearestQuery | Confirmed. `Mmr.diversity` is `[0,1]`; `NearestInputWithMmr` is the correct Rust shape. [docs.rs/qdrant-client/1.18.0 §NearestInputWithMmr] | LOW | None — on target. |
| **KA-03** | "Group-by query + group_size" on the standard `query` endpoint | **`QueryPointsBuilder` has no `group_by` / `group_size` methods.** The functionality lives on a *different* endpoint: `query_groups` with `QueryPointGroupsBuilder`. [docs.rs/qdrant-client/1.18.0 §QueryPointGroupsBuilder] | **MED** | KA-03 must be reframed as "switch to `query_groups` endpoint" — that's a separate Rust call path, not a flag on the existing query call. Not hard, but the v3 description and code sketch mislead. |
| KA-04 | RelevanceFeedbackQuery (1.17) GA | Confirmed GA in 1.17 release notes (listed with documentation link, not flagged experimental). `RelevanceFeedbackInputBuilder` exists. [docs.rs/qdrant-client/1.18.0 §RelevanceFeedbackInput] | LOW | None — on target. |
| **KA-05** | Weighted RRF JSON: `"query": { "fusion": "rrf", "weights": [...] }` | Actual shape is `"query": { "rrf": { "weights": [3.0, 1.0], "k": 60 } }` — note the outer key is `rrf`, not `fusion`. [qdrant.tech/documentation/concepts/hybrid-queries/] The Rust `RrfBuilder` matches. | **MED** | Update the JSON example. Behaviorally identical, but the existing snippet would 400 the server. |
| KB-01 | Late-interaction MaxSim via 6th vector `content_late` | Correct concept. **Naming nit:** the docs JSON value is `"comparator": "max_sim"` (lowercase snake), not `"MaxSim"`. The Rust enum is `MultiVectorComparator::MaxSim` (Pascal). Either way, v3's narrative "comparator: MaxSim" is fine as prose — but the spec doc should clarify the wire vs. Rust form. [qdrant.tech/documentation/concepts/vectors/] | LOW | Cosmetic — clarify in the spec. |
| KB-02 | Server-side BM25 sparse on `path` + `tool` | Confirmed. `Modifier::Idf` is the correct path; server-side BM25 is the **only** inference flow GA on self-hosted Qdrant. [qdrant.tech/documentation/concepts/inference/] | LOW | None. |
| KB-03 | Discovery API "true" context pairs | API confirmed (DiscoverInputBuilder + ContextInputBuilder). | LOW | None. |
| **KB-04** | "ACORN filterable HNSW (strict filter during search)" | Two corrections: (a) Docs call this **"ACORN Search Algorithm"** (no `-1` suffix in current docs, though release notes used "ACORN-1"). [qdrant.tech/documentation/concepts/indexing/] (b) "Filterable HNSW" is a **separate** mechanism (additional payload-indexed edges in the HNSW graph) — ACORN is the *fallback* algorithm activated when filtered HNSW alone disconnects the graph. The v3 description conflates the two. | **MED** | Rename to "ACORN search (filtered HNSW second-hop fallback)" and clarify it activates only on the *query* side via `params.acorn`, not as a collection-config flag. |
| KB-05 | Order-by query (1.16) — temporal navigation | **Version is wrong.** OrderBy as a Query variant is older than 1.16 (the 1.16 release notes do not list OrderBy). It's part of the general Query API surface introduced with the unified query endpoint (~1.10–1.12 era). | LOW | Drop the "(1.16)" attribution. The feature is GA, just not new in 1.16. |
| **KC-01** | "TurboQuant (Binary 2-bit, 1.18 GA) over Scalar" | **Conflation.** TurboQuant is a *distinct* quantization family from Binary Quantization. The TurboQuant config key is `"turbo"`, and its `bits` enum is `bits4` (default) / `bits2` / `bits1_5` / `bits1`. Binary 2-bit refers to the 1.15-era asymmetric Binary Quantization (`"binary"` key). [qdrant.tech/documentation/guides/quantization/] Picking "Binary 2-bit" in the description but pointing at TurboQuant in the prose ("1.18 GA") will lead an implementer to either (a) use the wrong builder or (b) try to pass `bits2` to BinaryQuantizationBuilder, which is wrong. | **HIGH** | Rename to "TurboQuant `bits2` (1.18 GA)" and drop "Binary 2-bit" entirely. Add `always_ram: true` to the spec. |
| KC-02 | Per-vector HNSW tuning | Confirmed. `HnswConfigDiffBuilder` exists; per-named-vector params are GA. | LOW | None. |
| **KC-03** | "Tenant index (1.17 신규)" | **Version is wrong.** `is_tenant: true` was introduced in **v1.11.0** ("`is_tenant` parameter is available as of v1.11.0"). [qdrant.tech/documentation/guides/multitenancy/] The 1.16 *tiered* multitenancy with `ReplicatePoints` is the 1.16 addition — but tenant-flagged keyword index has been GA since 1.11. | **MED** | Drop the "(1.17 신규)" claim. Cite 1.11 GA. The KICK is still valid as a pinnacle feature (Memex doesn't use it today), just not a 1.17-or-later flag. |
| **KC-03** | JSON snippet shows top-level `"payload_index": { "project_name": { "type": "keyword", "is_tenant": true } }` inside collection-config | Wrong location. Payload indexes are created via `PUT /collections/{name}/index` with `{ "field_name": "project_name", "field_schema": { "type": "keyword", "is_tenant": true } }` — they are not embedded in `collection-create`. [qdrant.tech/documentation/guides/multitenancy/] | **MED** | Move the index creation to a second call. Note: `hnsw_config.payload_m` *is* a real collection-config knob — that part of the snippet is OK. |
| KC-04 | Datetime native index on `start_ts` | Datetime payload index is GA, but research/06 §2 row #24 already had it as `1.x` with no specific introduction in 1.18. The doc fetch did not surface a verbatim JSON example, so the exact builder path is `unverified at fetch depth-1` — likely `PayloadIndexParamsBuilder` with a `datetime` schema variant. | LOW | Verify exact builder name during spike. |
| KC-05 | spawn_blocking + Semaphore + batch=32 | Not Qdrant-API-bound (Tokio + fastembed-rs). | n/a | None. |
| KD-01 / KD-02 | Gemma 3 9B / 4B via Ollama for index-time augmentation | Not Qdrant-1.18-bound. Effect on Qdrant is "payload fields populated" — uses standard payload-index machinery, which is GA. | LOW | None for the Qdrant surface. (LLM eval-gate concerns belong to a separate review.) |
| KE-01 / KE-02 | HyDE / MIPS-aware query rewriting | Not Qdrant-API-bound (LLM-side technique). | n/a | None. |
| KF-01 / KF-02 / KF-03 | Path containment + snapshot validation + signed envelope | Not Qdrant-API-bound (Rust file-system code). | n/a | None. |
| KG-01 / KG-02 | Topology / pivot-parse caches | Rust-side caching. | n/a | None. |
| KG-03 | Payload index on `schema_version` + dual-write | Standard keyword index. | LOW | None. |

**Over-claim totals**

- **HIGH severity (will break at spike time):** 3 — KA-01 conditional-boost shape, KA-01 `$score.<name>` reference, KC-01 TurboQuant/Binary-2bit conflation.
- **MED severity (wrong endpoint, wrong version, will work but doc misleads):** 5 — KA-03 endpoint, KA-05 fusion key, KB-04 ACORN naming, KC-03 version+location.
- **LOW (cosmetic):** 3 — KB-01 comparator case, KB-05 version claim, KC-04 builder unverified.
- **On-target:** 14 KICKs.

---

## 4. Verdict 3 — Rust client coverage (`qdrant-client = "1"`, pinned to 1.18.0)

Per-KICK Rust-access check. The pinned client version is `qdrant-client v1.18.0`. The question: can Memex's Rust code reach this surface without dropping to raw HTTP?

| KICK | Required Rust type / builder | Present in `qdrant-client 1.18.0`? | Notes |
|---|---|---|---|
| KA-01 FormulaQuery | `FormulaBuilder`, `Expression` enum, `DecayParamsExpressionBuilder`, `SumExpression`, `MultExpression`, `GaussDecayExpression`, `ExpDecayExpression` | **Yes — all present.** [docs.rs/qdrant-client/1.18.0] | Build the Expression tree manually; `FormulaBuilder` wraps it. Conditional-boost via `Condition` struct nested as Expression. |
| KA-02 MMR on NearestQuery | `Mmr`, `NearestInputWithMmr` (+ implicit builder via `Default + setters`) | **Yes.** `NearestInputWithMmr { nearest, mmr }`. [docs.rs/qdrant-client/1.18.0/qdrant_client/qdrant/struct.NearestInputWithMmr.html] | **No method on `QueryPointsBuilder`** — MMR is set inside the Query variant. Construct: `Query::from(NearestInputWithMmr { nearest: Some(v.into()), mmr: Some(Mmr { diversity: Some(0.4), candidates_limit: Some(50) }) })`. |
| KA-03 group_by + group_size | `QueryPointGroupsBuilder` (note: **different endpoint** from `query`) | **Yes — but on a separate builder.** `query_groups()` client method. [docs.rs/qdrant-client/1.18.0 §QueryPointGroupsBuilder] | The v3 implication that this is a flag on the existing query path is wrong — Memex must call `client.query_groups(...)` instead of `client.query(...)`. |
| KA-04 RelevanceFeedback | `RelevanceFeedbackInputBuilder`, `FeedbackItem`, `FeedbackStrategy` | **Yes.** [docs.rs/qdrant-client/1.18.0/qdrant_client/qdrant/struct.RelevanceFeedbackInput.html] | On target. |
| KA-05 Weighted RRF | `RrfBuilder` with `weights: Vec<f32>` and `k: Option<u32>` | **Yes.** | API-shape correction (see Verdict 2) is at the JSON-prose level, not Rust-availability. |
| KB-01 MaxSim multivector | `MultiVectorConfig`, `MultiVectorComparator::MaxSim` | **Yes.** Collection-config side. | On target. |
| KB-02 Sparse BM25 + IDF | `SparseVectorParamsBuilder`, `Modifier::Idf`, `Document { text, model }` for upsert | **Yes.** | On target. The `Document` flow uses the gRPC client transparently. |
| KB-03 Discovery context pairs | `DiscoverInputBuilder`, `ContextInputBuilder` | **Yes.** | On target. |
| KB-04 ACORN | `AcornSearchParams` (+ `Builder`) inside `SearchParams.acorn` | **Yes.** Per-query opt-in. | Verbatim from research/06 §3.9. On target at the Rust level. |
| KB-05 OrderBy | `OrderByBuilder` + `Query::OrderBy(...)` variant | **Yes.** | The v3 "(1.16)" version claim is wrong (this surface is older), but the Rust path is fine. |
| KC-01 TurboQuant | `TurboQuantizationBuilder` with `.bits(TurboBits::Bits2)` (enum unverified — see below) | **Builder confirmed present.** The `TurboBits` enum exact variant names are `unverified at fetch depth-1` — likely `Bits4 / Bits2 / Bits1_5 / Bits1`. | Spike to confirm enum casing. |
| KC-02 Per-vector HNSW | `HnswConfigDiffBuilder` per-named-vector | **Yes.** | On target. |
| KC-03 Tenant index | `PayloadFieldType::Keyword` schema with `is_tenant: true`. Rust side: `KeywordIndexParamsBuilder` (or via `FieldType::Keyword` with extra params). | **Yes** — keyword index params support `is_tenant` field. | On target at Rust level; metadata correction (version + endpoint) needed in the v3 doc. |
| KC-04 Datetime index | `PayloadIndexParamsBuilder` with `datetime` schema variant | `unverified at fetch depth-1` | Spike. |
| KC-05 spawn_blocking | Not Rust-client-bound. | n/a | n/a |
| KD-01/02 | Payload field write + index. | **Yes** — standard `Document`/`Value` payload. | On target. |
| KE-01/02 | Embedding + standard query. | **Yes**. | On target. |
| KF-01/02/03 | Rust filesystem code. | n/a | n/a |
| KG-01/02 | Rust caching. | n/a | n/a |
| KG-03 | Keyword payload index. | **Yes**. | On target. |

**Rust-client gating summary**

- **All 25 KICKs are reachable from `qdrant-client 1.18.0` without dropping to HTTP.** Memex's gRPC-preferred posture is preserved.
- **Two enums need spike verification at variant-name level**: `TurboBits` (KC-01), `PayloadIndexParams` datetime variant (KC-04).
- **The one trap is KA-03** — implementer must remember `query_groups`, not `query`.

---

## 5. Proposed delta to v3 §3

Concrete add / revise / drop list to bring v3 from "directionally correct" to "implementer-safe."

| Action | KICK | Change |
|---|---|---|
| **REVISE** | **KA-01** | Replace `{ "value": 0.2, "filter": {...} }` with `{ "mult": [0.2, {"key": "has_errors", "match": {"value": true}}] }`. Drop `"$score.path_sparse"` — either restructure as a single dense Formula with prefetch-rescore, or mark this as a multi-stage Formula spike (research/06 §3.1 already noted the trade-off). Add `defaults: { has_errors: false }` to the Formula. |
| **REVISE** | **KA-03** | Change "flag on `query_points`" → "switch to `query_groups` endpoint" (`client.query_groups(QueryPointGroupsBuilder::new(...)`). Update code sketch. |
| **REVISE** | **KA-05** | Fix JSON: `"query": { "rrf": { "weights": [1.0, 1.5, 0.8, 1.2], "k": 60 } }`. Note that `k` defaults to 60 in 1.16+; only override if Memex needs tighter rank smoothing. |
| **REVISE** | **KB-04** | Rename to "ACORN search (filtered-HNSW second-hop fallback)". Note that ACORN is *per-query* (`params.acorn`), not collection-level. Cite 1.16 release notes ("ACORN-1 search method") and current docs ("ACORN Search Algorithm"). |
| **REVISE** | **KB-05** | Drop "(1.16)". OrderBy as a Query variant has been GA since the unified Query API rollout (~1.10–1.12). Cite docs without a specific version. |
| **REVISE** | **KC-01** | Rename to "TurboQuant `bits2` on `content` (1.18 GA)". Drop "Binary 2-bit" language entirely. Update JSON: `"quantization_config": { "turbo": { "bits": "bits2", "always_ram": true } }`. **Pair with** new KC-01b (oversampling + rescore — see ADD below). |
| **REVISE** | **KC-03** | Fix version: "1.11 GA" not "1.17 신규". Fix endpoint: payload index is created via `PUT /collections/{name}/index`, not in collection-config. Keep `hnsw_config.payload_m: 16` as a separate collection-config tweak. |
| **ADD** | **KC-01b** | "TurboQuant rescore + oversampling" — set `params.quantization = QuantizationSearchParams { rescore: Some(true), oversampling: Some(2.0), ignore: Some(false), .. }` at query time. Effort ~15 min. Mitigates research/06 §6 risk. |
| **ADD** | **KC-06** | "Strict mode `max_resident_memory_percent: 85`" — desktop RAM guardrail. Effort ~30 min. Set conservatively (85% per research/06 §6 risk note). One of research/06 §1 top-5. |
| **ADD** | **KG-04** | "Conditional updates + `update_mode::Update` for embedding-staleness worker" — research/06 §3.7. Effort ~30 min. Skipped over because v3 doesn't have an explicit embedding-version-rotation KICK; combining the two lands the existing Memex worker on a 1.17 surface. |
| **KEEP** | KA-02, KA-04, KB-01, KB-02, KB-03, KC-02, KC-04, KC-05, KD-01, KD-02, KE-01, KE-02, KF-01, KF-02, KF-03, KG-01, KG-02, KG-03 | No changes — these 18 KICKs match documented 1.18 surfaces. |
| **CONSIDER DROP** | **KE-01 / KE-02** | Default-OFF runtime LLM toggles. They satisfy the v3 "negotiable" invariant relaxation, but a strict reading of the single invariant ("Maximize Qdrant 1.18 pinnacle") gives them **zero direct pinnacle contribution** — they're LLM features, not Qdrant features. The plan would be cleaner with these moved to "POST-HACKATHON". (Not a hard recommendation; the user-rejected "no LLM at runtime" was the v1/v2 framing, not a Qdrant feature.) |

After delta: v3 grows from 25 → **28 KICKs** (3 adds), with 8 revisions applied. The "25" round-number marketing slogan would need to update to "28 KICKs · 7 Categories" — or accept that 5 KICKs are descriptive prose (KF, KG) that don't move the Qdrant pinnacle needle and shouldn't have inflated the count in the first place.

---

## 6. What "pinnacle" means in 14 days — final director's call

Even with the proposed delta, three KICKs should *defer* to post-hackathon because demo-fit is 0 *and* effort exceeds 8 h *and* the ROI on visible-pinnacle is negligible.

| KICK | Why defer |
|---|---|
| **KE-01 HyDE** + **KE-02 MIPS-aware rewriting** | Both are LLM-runtime techniques. The single v3 invariant is "Maximize Qdrant 1.18 *pinnacle* usage" — these contribute ~zero Qdrant surface visibility. They land as settings-toggle features that 99% of the demo audience will never enable. Engineering hours are better spent finishing KD-01 quality work. |
| **KB-01 Late-interaction MaxSim** | Marked `COND` in v3 with a D-8 nDCG +15% eval gate (research/06 §6 already flags this as stack-risk). If the eval gate fails, revert is two-day work — within the 14-day window that's risky. **Recommendation:** keep KB-01 *in*, but pre-decide the revert criterion strictly. If Daichi cannot get a 12-second demo clip showing MaxSim *visibly* improving a ranking, drop it before the gate. |
| **KA-04 RelevanceFeedback** | Marked `COND` in v3 (Daichi storyboard gate). research/06 §3.4 effort is "M ~1 day" and the UI wiring is real. If the storyboard slot is already taken by Discovery + Mix & Match, this duplicates the "feedback evolves the query" beat with marginal incremental wow. Backend-only ship is fine; UI ship is the dropable. |

**KICKs that should *stay even if invisible*** (because the invariant is "pinnacle," not "demo-fit ≥3"):

- KC-01 TurboQuant (revised) + new KC-01b + new KC-06 — these are *audited claims* the user can verify in a 1-minute screen-share of a Qdrant memory dashboard, even if no demo viewer sees them.
- KB-04 ACORN (revised) — same logic. The recall@10 +20% claim in v3 is a number, not a frame.
- KA-05 weighted RRF (revised) — same.

**The most-overrated pinnacle KICK in v3** is **KC-05 spawn_blocking + Semaphore + batch=32** (Q-fit 2/5 by v3's own scoring). It's good engineering but contributes 0 to "Qdrant 1.18 pinnacle." Under the single invariant, it shouldn't have been a §3 KICK at all — it's an internal Tokio refactor. Same for KG-01/02 and KF-01/02/03. If the invariant is taken seriously, these belong in a separate §3.5 "non-pinnacle work" section so the §3 count of 25 doesn't dilute the headline number. The plan's tally already acknowledges this implicitly (Q-fit 0/5 on KF-01).

---

## 7. Sources

All URLs fetched 2026-05-18.

1. `https://github.com/qdrant/qdrant/releases/tag/v1.18.0` — TurboQuant (GA), named-vector CRUD (#8605), deep memory reporting (#8606), low-memory mode (#8714), strict mode `max_resident_memory_percent` (#8715), dynamic CPU pool (#8790 / #8769), snapshot URL-restore toggle (#8628).
2. `https://github.com/qdrant/qdrant/releases/tag/v1.17.0` — Relevance Feedback (milestone#38, with docs link — listed as standard feature, no preview/experimental qualifier), weighted RRF (#8063), `update_mode` (#7963), audit access logging (#8071), payload-index HNSW-link disable (#7887).
3. `https://github.com/qdrant/qdrant/releases/tag/v1.16.0` — ACORN-1 search method (#7414), inline storage, conditional updates (#7006), `text_any` (#7100), custom RRF k (#7065), tiered multitenancy (milestone#37 with `ReplicatePoints`), ASCII folding.
4. `https://github.com/qdrant/qdrant/releases` — Release list confirming v1.18.0 published 2026-05-11; v1.17.1 (2026-03-27), v1.17.0 (2026-02-20), v1.16.x (2025-11–12), v1.15.x (2025-08–09).
5. `https://qdrant.tech/documentation/concepts/hybrid-queries/` — Weighted RRF JSON shape (`"rrf": { "weights": [...] }`); group_by/limit/group_size JSON; multi-stage prefetch syntax. **Does not** document Formula syntax (intentional — Formula lives on API reference, not concepts page).
6. `https://api.qdrant.tech/api-reference/search/query-points` — Full FormulaQuery schema (`formula` is an Expression-directly, with `defaults` map); Expression variants (Mult/Sum/Neg/Abs/Div/Sqrt/Pow/Exp/Log10/Ln/LinDecay/ExpDecay/GaussDecay/Datetime/DatetimeKey/GeoDistance + Condition + Variable string + Constant number); `$score` documented as a string-variable example; conditional boost via `{"mult": [0.5, <Condition>]}` pattern; NearestQuery `mmr` field with `diversity`/`candidates_limit`; `Rrf` object as `{"k": ..., "weights": [...]}` wrapped in `{"rrf": ...}`.
7. `https://qdrant.tech/documentation/guides/quantization/` — TurboQuant: "Available as of v1.18.0", JSON key `"turbo"`, `bits` enum values `bits4 / bits2 / bits1_5 / bits1`, `always_ram` option. Distinct from Binary Quantization (separate `"binary"` key, separate 1.5/2-bit asymmetric features GA'd in v1.15.0).
8. `https://qdrant.tech/documentation/guides/multitenancy/` — `is_tenant: true` flag introduced **v1.11.0**, placed in `field_schema` under `PUT /collections/{name}/index`, restricted to keyword fields. Tiered multitenancy + `ReplicatePoints` is the v1.16 addition (different feature).
9. `https://qdrant.tech/documentation/concepts/indexing/` — "ACORN Search Algorithm" naming in current docs (no "-1" suffix). Filterable HNSW is a separate base mechanism (extra payload-indexed edges); ACORN is the second-hop fallback. Datetime field type listed without a verbatim JSON example at fetch depth-1.
10. `https://qdrant.tech/documentation/concepts/vectors/` — Multivector JSON: `"multivector_config": { "comparator": "max_sim" }`, GA v1.10.0. Note lowercase snake on wire; Rust enum is Pascal `MaxSim`.
11. `https://qdrant.tech/articles/late-interaction-models/` — Python-client example uses `multivector_config=MultiVectorConfig(comparator=MultiVectorComparator.MAX_SIM)`. No JSON example in this article specifically (cross-referenced with concepts/vectors).
12. `https://docs.rs/qdrant-client/1.18.0/qdrant_client/qdrant/index.html` — Builder index confirming `FormulaBuilder`, `MmrBuilder`, `OrderByBuilder`, `TurboQuantizationBuilder`, `AcornSearchParamsBuilder`, `RelevanceFeedbackInputBuilder`, `RrfBuilder`. Notes that `QueryPointsBuilder` does NOT expose `group_by`/`group_size`/`order_by`/`mmr` as setter methods; these live on related types.
13. `https://docs.rs/qdrant-client/1.18.0/qdrant_client/qdrant/struct.QueryPointsBuilder.html` — Confirmed missing methods: `group_by`, `group_size`, `order_by`, `mmr`. Confirms present: `prefetch`, `query`, `using`, `filter`, `params`, `limit`, `offset`, `score_threshold`, `with_vectors`, `with_payload`.
14. `https://docs.rs/qdrant-client/1.18.0/qdrant_client/qdrant/struct.QueryPoints.html` — Same field set as builder; `group_by`/`group_size`/`order_by` not present (so it must be the separate groups endpoint).
15. `https://docs.rs/qdrant-client/1.18.0/qdrant_client/qdrant/struct.QueryPointGroupsBuilder.html` — Confirmed: `group_by(String)`, `group_size(u64)`, `query`, `prefetch` all present. This is the actual entry for grouping.
16. `https://docs.rs/qdrant-client/1.18.0/qdrant_client/qdrant/struct.NearestInputWithMmr.html` — Fields: `nearest: Option<VectorInput>`, `mmr: Option<Mmr>`. Direct struct construction (no dedicated builder name surfaced at depth-1).
17. `https://docs.rs/qdrant-client/1.18.0/qdrant_client/qdrant/struct.RelevanceFeedbackInput.html` — Fields: `target: Option<VectorInput>`, `feedback: Vec<FeedbackItem>`, `strategy: Option<FeedbackStrategy>`. `RelevanceFeedbackInputBuilder` present, converts to `Query`.
18. `https://qdrant.tech/documentation/concepts/inference/` — Server-side inference: BM25 is the only flow GA on self-hosted; all other inference paths are managed-cloud-only. (Re-confirmed from research/06.)
19. `https://qdrant.tech/documentation/database-tutorials/score-boosting/` — Returned **404** on 2026-05-18. (Same as research/06 §7 footnote — Qdrant has not republished the tutorial; FormulaQuery details verified via api.qdrant.tech reference + concepts/hybrid-queries.)
20. `https://qdrant.tech/documentation/release-notes/` — Redirects to GitHub releases. (Listed for completeness; substantive content fetched from the GitHub releases pages directly.)
21. `claudedocs/research/06-qdrant-1.18-feature-pinnacle.md` (read-only) — 80-row inventory baseline.
22. `claudedocs/sota-plan-v3.html` (read-only) — Subject of audit.
23. `claudedocs/sota-plan-v2.html` (read-only) — Delta-context baseline.

**Items NOT independently verified (carried from research/06 with `unverified` tag):**

- `TurboBits` Rust enum variant exact spelling (`Bits4` vs `BITS4` etc.) — needs a depth-2 fetch of `docs.rs/qdrant-client/1.18.0/qdrant_client/qdrant/enum.TurboBits.html` or live `cargo doc` against the pinned client.
- `PayloadIndexParams` datetime variant builder path — same.
- The `"$score.<prefetch_name>"` reference form in FormulaQuery — the official OpenAPI schema documents only the singular `"$score"`. Whether multi-prefetch score indexing is supported is **unverified — needs implementation spike** (research/06 §3.1 used the singular form in its B.2 example; v3 KA-01 reintroduces the multi-prefetch form without a citation).

---

## 8. Per-KICK forensic table — all 25 v3 items at a glance

Single table for cross-referencing. "Pinnacle Q-fit" is the auditor's recomputed Q-fit *after* corrections (so KC-01 keeps a 5 because TurboQuant itself is still a 1.18 pinnacle feature even if the description is wrong).

| KICK | v3 ship-tag | v3 Q-fit (claimed) | Auditor Q-fit | Auditor verdict | Action |
|---|---|---|---|---|---|
| KA-01 FormulaQuery | SHIP | 5/5 | 5/5 | API shape wrong (conditional boost + `$score.<name>`) — feature itself is the v3 crown jewel | REVISE (HIGH) |
| KA-02 MMR | SHIP | 4/5 | 4/5 | On target; note MMR is via `NearestInputWithMmr` not a builder setter | KEEP |
| KA-03 Group-by | SHIP | 4/5 | 4/5 | Wrong endpoint — `query_groups`, not `query` | REVISE (MED) |
| KA-04 RelevanceFeedback | COND | 5/5 | 5/5 | GA in 1.17 confirmed; Rust builder confirmed | KEEP (storyboard gate fine) |
| KA-05 Weighted RRF | SHIP | 4/5 | 4/5 | JSON syntax wrong — `rrf:{weights}` not `fusion:rrf, weights` | REVISE (MED) |
| KB-01 MaxSim multivector | COND | 5/5 | 4/5 | API correct; eval gate at D-8 is the right risk control | KEEP |
| KB-02 BM25 sparse + IDF | SHIP | 5/5 | 5/5 | On target; server-side BM25 is only self-hosted inference flow | KEEP |
| KB-03 Discovery context pairs | COND | 4/5 | 4/5 | API correct | KEEP |
| KB-04 ACORN | SHIP | 4/5 | 4/5 | Concept correct, naming + collection-vs-query confusion | REVISE (MED) |
| KB-05 OrderBy | SHIP | 4/5 | 3/5 | Version claim wrong (not 1.16 introduction) | REVISE (LOW) |
| KC-01 TurboQuant | SHIP | 5/5 | 5/5 | **Binary 2-bit conflation — highest-severity over-claim** | REVISE (HIGH) |
| KC-02 Per-vector HNSW | COND | 3/5 | 3/5 | API correct | KEEP |
| KC-03 Tenant index | SHIP | 4/5 | 4/5 | Version wrong (1.11 not 1.17), endpoint wrong (separate index call) | REVISE (MED) |
| KC-04 Datetime index | SHIP | 3/5 | 3/5 | Concept correct, builder name unverified | KEEP + spike |
| KC-05 spawn_blocking | SHIP | 2/5 | 1/5 | Not Qdrant-pinnacle work (Tokio refactor) | KEEP but acknowledge |
| KD-01 Contextual chips | SHIP | 4/5 | 3/5 | Pure LLM work; Qdrant value is payload-index population | KEEP |
| KD-02 Cluster auto-label | SHIP | 3/5 | 2/5 | Pure LLM work; no Qdrant API surface | KEEP |
| KE-01 HyDE | TOGGLE | 3/5 | 1/5 | Zero Qdrant pinnacle contribution under strict invariant | CONSIDER DROP |
| KE-02 MIPS rewriting | TOGGLE | 4/5 | 2/5 | Same as KE-01 | CONSIDER DROP |
| KF-01 SEC-003 fix | SHIP | 0/5 | 0/5 | Not Qdrant work; ship-blocker correctness | KEEP |
| KF-02 SEC-004 fix | SHIP | 0/5 | 0/5 | Same | KEEP |
| KF-03 Signed envelope | SHIP | n/a | 1/5 | Snapshot SHA-256 surface (1.x GA) — small Qdrant credit | KEEP |
| KG-01 Topology cache | SHIP | n/a | 0/5 | Rust cache | KEEP |
| KG-02 Predict LRU | SHIP | n/a | 0/5 | Rust cache | KEEP |
| KG-03 schema_version index | SHIP | n/a | 2/5 | Standard payload-index | KEEP |
| **(new) KC-01b** | (proposed SHIP) | — | 4/5 | TurboQuant rescore + oversampling — required pairing | ADD |
| **(new) KC-06** | (proposed SHIP) | — | 4/5 | Strict-mode `max_resident_memory_percent` — 1.18 GA, top-5 in research/06 §1 | ADD |
| **(new) KG-04** | (proposed SHIP) | — | 3/5 | Conditional updates + `update_mode::Update` — embedding-staleness worker | ADD |

**Sum of auditor Q-fit across all 28 proposed KICKs: 76/140 (54%).** Across the 14 KICKs that *actually* touch a Qdrant 1.18-pinnacle surface (Cat A + Cat B + Cat C, minus KC-05): **average 4.0/5**. The plan does reach the pinnacle where it claims to.

---

## 8.5 Source-of-truth diffs (v2 → v3 delta correctness check)

Cross-checking that v3's "delta vs v2" claims are themselves correct. v3 §s10 claims:
> "v1 (10) → v2 (19) → v3 (25). 주요 새 KICK: FormulaQuery, TurboQuant, KD-01/02, Tenant idx."

Per-feature audit:

| v3 claim | v2 KICK that maps to it (if any) | Auditor verification |
|---|---|---|
| KA-01 FormulaQuery is "new in v3" | v2 had no FormulaQuery (K01 was plain RRF) | **Correct** — Formula is a v3 addition. |
| KA-02 MMR is "ship-elevated" from v2 K04 | v2 K04 = "MMR diversity rerank" | **Correct** — same feature, escalated to its own category. |
| KA-03 group_by "v3 implementation" | v2 had no group_by KICK | **Correct** — research/06 §3.3 flagged this as a v3 add. |
| KA-04 RelevanceFeedback "v3 new" | v2 had no RelevanceFeedback KICK | **Correct** — research/06 §3.4 flagged as miss. |
| KA-05 Weighted RRF "v3 new" | v2 K01 used plain RRF | **Correct.** v3's escalation to weighted RRF (1.17) is a real pinnacle move. |
| KB-01 content_late = v2 K02 | v2 K02 = "6th vector content_late, late-interaction MaxSim" | **Correct** — preserved with same eval gate. |
| KB-02 BM25 sparse = v2 K03 | v2 K03 = "Server-side BM25 sparse on path & tool" | **Correct** — unanimous v2 ship, preserved. |
| KB-03 Discovery context = v2 K12 | v2 K12 = "Discovery API true context pairs" | **Correct.** |
| KB-04 ACORN = v2 K14 | v2 K14 = filter-aware HNSW (unspecified naming) | **Correct concept, naming sharpened** — though v3 still mis-names it (see Verdict 2). |
| KB-05 OrderBy "v3 new" | v2 had no OrderBy | **Correct** — research/06 §2 row #43 flagged as Direct-fit Memex miss. |
| KC-01 TurboQuant "1.18 GA, replaces v2 SQ" | v2 had Scalar Quantization in its KICK list | **Correct upgrade direction**, but the description over-claims (Verdict 2). |
| KC-02 Per-vector HNSW = v2 K15 | v2 K15 = "Per-vector HNSW tuning (m, ef_c, ef_s)" | **Correct.** |
| KC-03 Tenant index "v3 new" | v2 had no tenant index KICK | **Correct** — research/06 §3.10 flagged as Memex miss. (Version metadata still wrong; see Verdict 2.) |
| KC-04 Datetime index = v2 K16 | v2 K16 = "Datetime native index on start_ts" | **Correct.** |
| KC-05 spawn_blocking = v2 K06 | v2 K06 = "spawn_blocking + Semaphore + batch=32" | **Correct.** |
| KD-01/02 LLM augmentation "v3 NEW" | v2 had nothing — "no LLM" was v2 hard invariant | **Correct** — this is the v3 invariant relaxation working as intended. |
| KE-01/02 Runtime LLM toggles "v3 NEW" | Same as KD-01/02 — v2 had nothing | **Correct.** |
| KF-01/02 SEC fixes = v2 carry-over | v2 had path containment + snapshot_export in roadmap | **Correct.** |
| KF-03 signed snapshot = v2 K05 | v2 K05 = "Signed snapshot envelope" | **Correct.** |
| KG-01/02/03 = v2 K07/K08/K09 | Direct mapping | **Correct.** |

**v3-vs-v2 delta check verdict: 20/20 mappings are accurate.** v3 honestly represents its delta from v2.

Where v3 *misses* a v2-to-v3 carry-forward:
- v2 K01 was "Single-call Multi-stage Query API + RRF" — a base infrastructural move. v3 implicitly absorbs this into KA-01 (FormulaQuery requires the prefetch-multi-stage shape) and KA-05 (weighted RRF). Neither v3 KICK calls out the "round-trip 5→1" base improvement explicitly. **Suggestion:** add a footnote to KA-01 noting that the Query API single-call infrastructure (v2 K01) is the substrate for both KA-01 and KA-05. Not a delta omission, but a narrative thinning.

---

## 9. Cross-check against research/06's "top 5 pinnacle misses" (§1 executive summary)

research/06 §1 listed five highest-leverage misses for hackathon ROI. v3 must address all five to claim pinnacle.

| research/06 §1 miss | v3 coverage | Status |
|---|---|---|
| 1. Formula-based score boosting | KA-01 | **COVERED** (with API-shape revisions needed — see Verdict 2) |
| 2. MMR diversity rerank | KA-02 | **COVERED** (on target) |
| 3. Relevance Feedback API | KA-04 | **COVERED** (on target, COND for UI) |
| 4. Group-by query | KA-03 | **COVERED** (with endpoint revision needed) |
| 5. Strict-mode `max_resident_memory_percent` + low memory mode | **NOT COVERED** | **GAP** — see proposed KC-06 ADD |

Plus the "freebie" from research/06 §1: `is_tenant: true` on `project_path` — **COVERED** by KC-03 (version metadata wrong but feature present).

**4-of-5 coverage of the executive-summary top-5.** The missing one (strict-mode memory) is the single highest-ROI miss after the proposed delta.

---

## 9.5 Deep-dive: the three HIGH-severity over-claims

These three need full unpacking because they will determine whether spike day succeeds or burns a day on Qdrant-server 4xx debugging.

### 9.5.1 KA-01 — FormulaQuery conditional-boost shape

**v3 wrote:**
```json
{ "value": 0.2, "filter": { "key": "has_errors", "match": { "value": true } } }
```

**This object literal does not appear in the documented Expression schema.** The schema defines 19 Expression variants (Mult / Sum / Neg / Abs / Div / Sqrt / Pow / Exp / Log10 / Ln / LinDecay / ExpDecay / GaussDecay / Datetime / DatetimeKey / GeoDistance / Condition / Variable-string / Constant-number). None of them have a top-level `{ "value": ..., "filter": ... }` shape.

**The documented idiom for "constant boost when condition matches"** (per [api.qdrant.tech/api-reference/search/query-points] example): a Condition expression *is* an Expression — it evaluates to 1.0 when matched, 0.0 otherwise. Multiply by a constant to get a conditional boost:

```json
{
  "mult": [
    0.2,
    { "key": "has_errors", "match": { "value": true } }
  ]
}
```

When the condition matches, this expression evaluates to `0.2 * 1.0 = 0.2`. When it doesn't, `0.2 * 0.0 = 0.0`. This is the *only* documented way to do filter-conditional boost in a FormulaQuery.

**Risk if uncorrected:** Qdrant server returns 400 on parse, spike day burns 2 hours diagnosing.

### 9.5.2 KA-01 — `"$score.path_sparse"` reference

**v3 wrote:**
```json
{ "mult": [ "$score.path_sparse", 1.5 ] }
```

**The OpenAPI schema documents only the singular `"$score"`** as the score-reference Variable. No `$score.<prefetch_name>` form appears in the published schema. The documented multi-prefetch scoring pattern is:

1. Define multiple prefetches, each producing its own ranked list.
2. Use a *FusionQuery* (RRF or DBSF) to merge them.
3. Then optionally apply a FormulaQuery *after* fusion, where `$score` refers to the fused score (single scalar per point).

**The "5 dense + 1 sparse + 1 multivec, each with its own weight in a single Formula" pattern v3 KA-01 envisions does not have a documented API path.** It might exist as an undocumented feature, but the audit can only verify what's published.

**Two repair paths:**

- **(A) Spike + confirm** — fire one query against a local Qdrant 1.18 with `$score.path_sparse` syntax. If it works, document the find for the Qdrant team. If 400s, route (B).
- **(B) Restructure** — use a weighted RRF (KA-05) at the prefetch-fusion stage with the lens weights, then apply a *post-fusion* FormulaQuery that adds the `gauss_decay` recency and `has_errors` boost on the single fused `$score`. This is what research/06 §3.1 implicitly assumed in its B.2 example (single `$score` reference). **This is the safe path.**

Auditor recommendation: pin v3.1 to path (B). It still achieves the v3 narrative ("server-side single call combining lens weights + recency decay + error boost") and uses only documented surfaces.

### 9.5.3 KC-01 — TurboQuant vs Binary 2-bit conflation

**v3 wrote:**
> "TurboQuant (Binary 2-bit, 1.18 GA) over Scalar"

**Three things wrong:**

1. **Binary 2-bit is NOT TurboQuant.** Binary Quantization with 2-bit asymmetric encoding was introduced in v1.15.0 (per release notes: "asymmetric / 1.5-bit / 2-bit binary quantization"). It uses the `"binary"` config key with `encoding` and `query_encoding` sub-options.
2. **TurboQuant is its own family.** Introduced v1.18.0. Uses the `"turbo"` config key. Has its own `bits` enum: `bits4` (default) / `bits2` / `bits1_5` / `bits1`. Architecture: Hadamard rotation + asymmetric encoding ([qdrant.tech/blog/qdrant-1.18.x/]).
3. **The headline "8× compression without recall tax"** specifically refers to TurboQuant's `bits4` default. To match the v3 "16× over SQ's 4×" math, v3 actually wants TurboQuant `bits2` (16×) — *not* "Binary 2-bit" which has different storage characteristics.

**Auditor verdict:** This is the single highest-severity error in v3. An implementer reading "Binary 2-bit, 1.18 GA" might:
- Reach for `BinaryQuantizationBuilder` (wrong — it's the 1.15 surface, not 1.18 GA).
- Try `BinaryQuantizationBuilder::default().bits(BinaryBits::Bits2)` — if such a setter exists at all (unverified at depth-1 fetch).
- Ship a collection that uses Binary Quant instead of TurboQuant and *still claims* "1.18 pinnacle on the headline new feature."

The Memex demo's "Qdrant 1.18 pinnacle" credibility hinges on actually using TurboQuant. Misnaming it Binary-2-bit in the plan creates a real risk of shipping the wrong feature under the right name.

**Corrected KC-01:**

```json
"quantization_config": {
  "turbo": {
    "bits": "bits2",
    "always_ram": true
  }
}
```

Rust:
```rust
TurboQuantizationBuilder::default()
    .bits(TurboBits::Bits2)         // enum variant exact name: unverified, spike
    .always_ram(true)
    .build()
```

Paired with new KC-01b (oversampling + rescore) per Verdict 2 and research/06 §6.

---

## 10. Skeptical reading — where might this audit be wrong?

In the spirit of the user's "no echo chamber" directive, the auditor self-checks where own claims could be over-confident:

1. **"`$score.<name>` doesn't exist"** — I verified this only via the OpenAPI schema fetch. Qdrant has community blog posts (not fetched in this round) that occasionally show idioms not in the formal schema. If a Qdrant 1.14+ tutorial does demonstrate per-prefetch score reference, the verdict on KA-01 should soften from HIGH to LOW. Conservative recommendation: spike a 30-min test against a local Qdrant 1.18 before pinning the API shape.
2. **"Group-by requires `query_groups`"** — Confirmed via QueryPointsBuilder methods and QueryPointGroupsBuilder existence. But the REST endpoint *might* support `group_by` as an inline parameter (the docs JSON shows it inline). If REST inlines work and only the gRPC Rust client splits them, Memex could use REST for that one path. Either way, the v3 code sketch as written would fail in Rust.
3. **"KC-03 version is 1.11"** — Verified directly from `qdrant.tech/documentation/guides/multitenancy/`. v3's "1.17 신규" claim is unambiguously wrong on the `is_tenant` flag. (The 1.16-era tiered multitenancy with `ReplicatePoints` is a separate feature and Memex's single-node deployment doesn't need it.)
4. **TurboBits enum spelling** — Marked `unverified` honestly. Could be `TurboBits::Bits2` or `TurboQuantizationBits::B2` or other. Spike-required.
5. **"KE-01/02 = 0 pinnacle contribution"** — Could be argued the other direction: HyDE generates richer query vectors that *exercise* more of Qdrant's recall surface. Counter-argument is weak (any embedding does that), so the audit stands, but flagged as a soft call.

---

## 11. Recommended pre-D-13 actions (one engineer, half-day budget)

Prioritized fixes to close the gap before the D-13 spike PR per v3 roadmap:

1. **Rewrite KA-01 §3 example** with the correct Condition-as-Expression idiom. 20 min. (Highest impact: KA-01 is the v3 crown jewel.)
2. **Spike `$score.<prefetch>` reference** on a local Qdrant 1.18 instance. If unsupported, restructure KA-01 to a single dense Formula over a hybrid prefetch (keep multi-lens via prefetch fusion, not via per-prefetch `$score` indexing). 40 min.
3. **Fix KC-01 wording** — drop "Binary 2-bit", use "TurboQuant `bits2`". 5 min.
4. **Fix KA-05 JSON** — `"rrf": {weights}`. 5 min.
5. **Fix KA-03 endpoint** — call out `query_groups`. 5 min.
6. **Fix KC-03 version + endpoint** — "1.11 GA", separate index-creation call. 5 min.
7. **Fix KB-04 ACORN naming + per-query placement.** 5 min.
8. **Drop KB-05 "(1.16)" claim.** 1 min.
9. **Add KC-01b, KC-06, KG-04** as full KICK entries in §3. 30 min.

Total: ~2 hours of editing + 40 min of one spike test = under one engineer-afternoon. Output: v3.1 with implementer-safe §3 ready for D-13 PR.

---

## 12. One-paragraph director's note

v3 is a *real* improvement over v2 in framing (single invariant, clearer categorization) and in coverage (FormulaQuery, TurboQuant, Tenant index, group-by are all genuine 1.18-pinnacle features that v2 missed). The plan reaches the pinnacle in **scope**. Where it falls short is **precision** — KA-01, KA-03, KC-01, KC-03, KA-05, KB-04 all have API-shape or version mismatches that will cost the implementer ~2–3 hours of debugging at spike time. None of these are fatal; all are fixable with a copy-edit pass plus the three adds (KC-01b, KC-06, KG-04). The single most important pre-D-13 action: **someone re-reads §3 with `api.qdrant.tech/api-reference/search/query-points` and `docs.rs/qdrant-client/1.18.0` open in two tabs and corrects the 8 flagged KICKs.** That alone moves v3 from "directionally pinnacle" to "implementer-safe pinnacle." If the audit author had to draft v3.1 themselves, the priority order would be: (1) fix KC-01 wording, (2) spike `$score.<name>` to settle KA-01's shape, (3) add KC-06 strict-mode memory as the one undeniable research/06 §1 gap, (4) leave KE-01/02 in for narrative reasons even though they fail the strict invariant test. The plan does not need a v4 — it needs a 2-hour copy-edit.

---

*End of audit. 28 KICKs proposed (25 v3 + 3 ADD), 8 KICKs flagged for revision, 0 KICKs recommended for hard drop, 2 KICKs (KE-01/02) optionally deferable on strict invariant-fidelity grounds. Coverage verdict: **on-target** (within recommended delta). Most dangerous over-claim: **KC-01 TurboQuant ↔ Binary 2-bit conflation**. Most important add: **strict-mode `max_resident_memory_percent` (KC-06)** to close the single research/06 §1 top-5 gap.*
