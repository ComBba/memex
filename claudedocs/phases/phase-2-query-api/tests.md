# Phase 2 — Query API Core · TDD

**Phase**: P2
**Test framework**: Rust `cargo test`, Tauri integration (`tests/lens_integration.rs`)

---

## 1. Unit tests · KA-01 FormulaQuery

### Test file: `src-tauri/src/indexer/search.rs` (`#[cfg(test)] mod lens_tests`)

| Test | Setup | Expected | AC |
|------|-------|---------|----|
| `t_lens_all_weights_positive` | weights {content:1, tool:1.5, path:1.5, error:.8, code:.6} | 6 prefetch, formula 6 terms | AC-2.1.1 |
| `t_lens_zero_weight_skipped` | weights {content:1, tool:0, ...} | tool prefetch 생략 | AC-2.1.5 |
| `t_lens_all_zero_err` | weights = 0.0 across | Err("no active lens") | AC-2.1.4 |
| `t_lens_empty_query_err` | query_text = "" | Err("empty query") | AC-2.1.1 |
| `t_lens_recency_30day_decay` | start_ts = now - 30d | recency_factor ≈ exp(-1) ≈ 0.368 | AC-2.1.3 |
| `t_lens_recency_today` | start_ts = now | recency_factor = 1.0 | AC-2.1.3 |
| `t_lens_recency_negative_ts_edge` | start_ts = 0 (epoch) | recency_factor 양수 (no panic) | AC-2.1.3 |
| `t_lens_has_errors_boost` | result with has_errors=true | breakdown.has_errors_boost == 0.2 | AC-2.1.4 |
| `t_lens_no_errors_no_boost` | result with has_errors=false | breakdown.has_errors_boost == 0 | AC-2.1.4 |
| `t_lens_score_breakdown_sums` | per_vector + recency*?? + boost | breakdown.final_score == 정확한 계산 | AC-2.1.2 |
| `t_lens_limit_clamp` | limit = 500 | actual limit = 100 | AC-2.1.5 |

---

## 2. Unit tests · KA-02 MMR

| Test | Setup | Expected | AC |
|------|-------|---------|----|
| `t_mmr_diversity_zero_same_ranking` | diversity=0, fake response | order == non-MMR order | AC-2.2.3 |
| `t_mmr_diversity_max_spread` | diversity=1.0 | same-project clusters max 1 in top-N | AC-2.2.1 |
| `t_mmr_default_value` | LensWeights::default() | diversity == 0.4 | AC-2.2.2 |

---

## 3. Unit tests · KA-05 weighted RRF

| Test | Setup | Expected | AC |
|------|-------|---------|----|
| `t_rrf_weights_order_matches_prefetch` | 6 prefetch, 6 weights | query JSON weights[] in prefetch order | AC-2.3.1 |
| `t_rrf_zero_weight_skips_prefetch` | weights[2] = 0 | only 5 prefetch in query | AC-2.3.2 |
| `t_rrf_fusion_keyword_present` | any query | "fusion": "rrf" always | AC-2.3.1 |

---

## 4. Unit tests · KB-02 BM25 sparse

| Test | Setup | Expected | AC |
|------|-------|---------|----|
| `t_sparse_query_uses_word_tokenizer` | query text "edit auth.js" | JSON contains `"tokenizer": "word"` | AC-2.4.2 |
| `t_sparse_query_for_path` | `using: "path_sparse"` prefetch | text = query_text | AC-2.4.2 |
| `t_sparse_query_for_tool` | `using: "tool_sparse"` prefetch | text = query_text | AC-2.4.2 |
| `t_collection_schema_has_sparse` | check schema config (P3 협력) | path_sparse, tool_sparse 존재 | AC-2.4.1 |

---

## 5. Property tests

| Property | Statement |
|---------|-----------|
| `prop_score_monotonic` | weight ↑ → 해당 vector의 contribution ↑ |
| `prop_no_panic_on_empty_results` | corpus empty → Ok(vec![]) (not Err, not panic) |
| `prop_recency_in_zero_one` | exp_decay 결과는 항상 [0, 1] |
| `prop_breakdown_sum_matches_score` | sum(per_vector) + recency_factor + has_errors_boost == final_score (within ε) |

---

## 6. Integration tests · IPC + Qdrant roundtrip

### Test file: `src-tauri/tests/lens_integration.rs`

Requires Qdrant running on `localhost:6334`. CI runs with docker container.

| Test | Action | Expected |
|------|--------|----------|
| `it_lens_search_returns_results` | Tauri `invoke('lens_search', ...)` | Ok with 20 results max |
| `it_lens_with_sparse_path` | weights {path: 2.0, others: 0} | top result has matching path tokens |
| `it_lens_breakdown_per_vector` | 4 non-zero weights | breakdown has 4 entries |
| `it_lens_filter_has_errors` | corpus has both error/non-error sessions | error vector prefetch only filters has_errors=true |
| `it_lens_recency_visible_in_ranking` | 동일 score 2 세션, 다른 start_ts | newer가 위 |
| `it_lens_offline_qdrant_graceful` | Qdrant 중지 후 lens_search | Err with retry hint |
| `it_lens_score_within_reasonable_range` | typical query | scores between 0.0 and ~5.0 |

---

## 7. Regression tests

| Surface | Check | 기대 |
|---------|-------|------|
| `lens_search` 외부 API 시그니처 | commands.rs 인터페이스 | unchanged (LensWeights는 backward-compatible new fields) |
| Topology · Mix & Match · Predict · Recall | 각자 lens_search 사용 안 함 | 영향 0 |
| WOW-3 prototype (mockup) | prototype HTML 동작 | unchanged (fake data, prototype 이라 영향 0) |

---

## 8. Performance tests

| Test | 측정 | 기대 |
|------|------|------|
| `perf_lens_round_trips` | wireshark/grpcurl 로 query 호출 수 | 1 (이전: 5+) |
| `perf_lens_latency_p95` | 80-세션 corpus, 100 query | p95 < 200ms (cold), < 50ms (warm) |
| `perf_lens_payload_size` | IPC payload size | < 10 KB / response |

---

## 9. Test → AC mapping 요약

| AC | Tests |
|----|-------|
| AC-2.1.1 ~ AC-2.1.6 | 11 unit + 7 integration + 4 perf |
| AC-2.2.1 ~ AC-2.2.3 | 3 unit |
| AC-2.3.1 ~ AC-2.3.2 | 3 unit |
| AC-2.4.1 ~ AC-2.4.3 | 4 unit + 2 integration (P3과 협력) |

**총 ~34 test case**. spec.md AC와 1:1 mapping.

---

## 10. Test fixture requirements

`src-tauri/tests/fixtures/lens/`:
- `corpus-mixed.jsonl` — 다양한 path, tool, has_errors 조합 8 세션
- `query-cases.json` — 표준 query 5개와 기대 ranking
- `score-formula-golden.json` — given input → expected per_vector/recency/boost/final 매핑

기존 fixture (`01_minimal.jsonl` ~ `05_mixed.jsonl`) 재사용.
