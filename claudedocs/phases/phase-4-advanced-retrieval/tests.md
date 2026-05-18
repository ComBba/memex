# Phase 4 — Advanced Retrieval · TDD

**Phase**: P4

---

## 1. Unit tests · KB-01 late-interaction

| Test | Setup | Expected | AC |
|------|-------|---------|----|
| `t_embed_token_level_returns_list` | text "hello world" | Vec<Vec<f32>>, len ≥ 2 | AC-4.1.1 |
| `t_embed_token_level_dim_correct` | any text | each vec.len() == 384 | AC-4.1.1 |
| `t_embed_token_level_deterministic` | same text × 2 | identical output | AC-4.1.1 |
| `t_content_late_upsert_format` | session with multivec | proto Vectors::Multi variant | AC-4.1.2 |
| `t_content_late_hnsw_disabled` | collection config | content_late.hnsw_config.m == 0 | AC-4.1.5 |
| `t_lens_prefetch_includes_content_late` | LensWeights with content_late=0.7 | prefetch[].using includes "content_late" | AC-4.1.3 |
| `t_lens_prefetch_skips_content_late_zero` | content_late=0 | prefetch omits content_late | AC-4.1.3 |

### Eval gate test (D-8)

`tests/lens_ndcg_eval.rs`:

| Test | Setup | Expected | AC |
|------|-------|---------|----|
| `eval_ndcg_dense_only_baseline` | corpus + 20 labeled queries | baseline nDCG@10 | — |
| `eval_ndcg_with_content_late` | same corpus + 20 queries | nDCG ≥ baseline × 1.15 | AC-4.1.4 |
| `eval_ndcg_token_chunk_variants` | A: raw tokens, B: 32-tok chunks | report both, choose better | (Marcus dissent) |

---

## 2. Unit tests · KB-03 Discovery context pairs

| Test | Setup | Expected | AC |
|------|-------|---------|----|
| `t_context_pair_query_format` | 2 pairs | JSON has context array of 2 objects | AC-4.2.1 |
| `t_single_anchor_backward_compat` | positive only, no pairs | works (legacy mode) | AC-4.2.3 |
| `t_empty_pairs_uses_target_only` | target + context=[] | valid discover query | AC-4.2.1 |
| `t_pairs_include_session_ids` | (sess_a, sess_b) pair | retrieves points by id | AC-4.2.1 |

---

## 3. Unit tests · KB-04 ACORN

| Test | Setup | Expected | AC |
|------|-------|---------|----|
| `t_recall_filter_in_hnsw_params` | recall query | params.hnsw_ef set | AC-4.3.3 |
| `t_recall_has_errors_filter_intact` | filter has_errors=true | unchanged proto structure | AC-4.3.1 |

### Eval test

| Test | Setup | Expected | AC |
|------|-------|---------|----|
| `eval_recall_filtered_acorn` | corpus with mixed has_errors | recall@10 ≥ post-filter recall × 1.20 | AC-4.3.2 |

---

## 4. Unit tests · KB-05 Order-by

| Test | Setup | Expected | AC |
|------|-------|---------|----|
| `t_list_sessions_order_default` | call without order_by | start_ts_dt desc | AC-4.4.1 |
| `t_list_sessions_order_by_tool_count` | order_by=tool_count desc | results sorted by tool_count | AC-4.4.1 |
| `t_list_sessions_order_by_errors` | order_by=has_errors desc | has_errors=true sessions first | AC-4.4.1 |
| `t_list_sessions_order_oldest_first` | order_by=start_ts_dt asc | oldest first | AC-4.4.1 |

---

## 5. Unit tests · KA-03 Group-by

| Test | Setup | Expected | AC |
|------|-------|---------|----|
| `t_lens_with_group_by_project` | lens_search(group_by=project_name, group_size=3) | results grouped, each group ≤ 3 | AC-4.5.1 |
| `t_lens_without_group_by` | omit group_by | flat results | AC-4.5.1 |
| `t_group_result_struct_has_group_id` | grouped response | per-result group_id field | AC-4.5.2 |

---

## 6. Unit tests · KA-04 RelevanceFeedback

| Test | Setup | Expected | AC |
|------|-------|---------|----|
| `t_relevance_feedback_command_exists` | check Tauri commands list | `relevance_feedback` registered | AC-4.6.1 |
| `t_relevance_feedback_query_format` | feedback {pos:[1,2], neg:[3]} | JSON has relevance_feedback obj | AC-4.6.1 |
| `t_relevance_feedback_determinism` | same input × 2 | identical results | AC-4.6.3 |
| `t_relevance_feedback_empty_pos` | only negative | Err or graceful empty | (edge) |

---

## 7. Property tests

| Property | Statement |
|---------|-----------|
| `prop_late_max_sim_bounded` | MaxSim score ∈ [-1, 1] |
| `prop_discover_pairs_commutative` | (a,b) pair는 (b,a) pair와 다른 결과 (negative direction matters) |
| `prop_group_by_size_limit` | 모든 group의 결과 ≤ group_size |
| `prop_order_by_strict_monotonic` | result[i].key ≥ result[i+1].key (descending) |

---

## 8. Integration tests

| Test | Setup | Action | Expected |
|------|-------|--------|----------|
| `it_late_max_sim_returns_results` | corpus indexed with content_late | lens_search with content_late=1.0 | non-empty results |
| `it_discover_pairs_filters_corpus` | (pos, neg) pair | discover query | result excludes points similar to neg |
| `it_acorn_recall_perf` | timing benchmark | recall with filter | latency vs post-filter recorded |
| `it_order_by_with_filter` | order + filter combined | query | both applied |
| `it_group_by_with_formula` | group + KA-01 formula | query | results grouped with scoring intact |
| `it_relevance_feedback_iteration` | initial query → 👍/👎 → re-query | iteration | rankings shifted |

---

## 9. Regression tests

| Surface | Check |
|---------|-------|
| Time Machine `list_sessions` | order_by default 동일 (descending by start_ts) |
| Topology | unchanged |
| Lens (P2) | group_by 미지정 시 동일 동작 |
| Mix & Match | single anchor 호환 (context_pair=[]) |
| Predict · Recall · Replay | unchanged |

---

## 10. Test → AC mapping

| AC | Tests |
|----|-------|
| AC-4.1.1 ~ AC-4.1.5 | 7 unit + 3 eval + 1 integration |
| AC-4.2.1 ~ AC-4.2.3 | 4 unit + 1 integration |
| AC-4.3.1 ~ AC-4.3.3 | 2 unit + 1 eval + 1 integration |
| AC-4.4.1 ~ AC-4.4.2 | 4 unit + 1 integration |
| AC-4.5.1 ~ AC-4.5.2 | 3 unit + 1 integration |
| AC-4.6.1 ~ AC-4.6.4 | 4 unit + 1 integration |

**총 ~33 test case**.
