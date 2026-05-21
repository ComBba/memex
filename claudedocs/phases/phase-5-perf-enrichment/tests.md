# Phase 5 — Performance & Enrichment · TDD

**Phase**: P5

---

## 1. Unit tests · enrich.rs

### Test file: `src-tauri/src/indexer/enrich.rs`

| Test | Setup | Expected | AC |
|------|-------|---------|----|
| `t_intent_build_from_bash_dominant` | tool_counts {Bash:100, Edit:10} | intent == "build" | AC-5.0.2 |
| `t_intent_impl_from_edit_dominant` | tool_counts {Edit:100, Bash:10} | intent == "impl" | AC-5.0.2 |
| `t_intent_debug_from_read_dominant` | tool_counts {Read:100, Edit:10} | intent == "debug" | AC-5.0.2 |
| `t_intent_mixed_no_dominant` | tool_counts {Bash:30, Edit:30, Read:30} | intent == "mixed" | AC-5.0.2 |
| `t_intent_empty_counts` | tool_counts {} | intent == "mixed" | (edge) |
| `t_entities_extract_file_paths` | turn with `path/to/file.rs` in tool_calls | entities contains "path/to/file.rs" | AC-5.0.2 |
| `t_entities_dedupe` | same path twice | entities has unique values | AC-5.0.2 |
| `t_entities_top10_limit` | 50 unique paths | entities.len() == 10 | AC-5.0.2 |
| `t_outcome_resolved_from_substring` | last turn "fix is in" | outcome == "resolved" | AC-5.0.2 |
| `t_outcome_unresolved_from_error` | last turn "error: ENOENT" | outcome == "unresolved" | AC-5.0.2 |
| `t_outcome_partial_neither` | last turn neutral | outcome == "partial" | AC-5.0.2 |
| `t_outcome_bash_exit_zero_resolved` | last Bash exit_code=0 | outcome includes "resolved" hint | AC-5.0.2 |
| `t_arc_fix_pattern` | Read,Read,Edit,Bash | arc == "fix" | AC-5.0.2 |
| `t_arc_debug_fix_pattern` | Read,Bash(fail),Bash(fail),Edit | arc == "debug-fix" | AC-5.0.2 |
| `t_arc_impl_pattern` | Edit,Edit,Edit | arc == "impl" | AC-5.0.2 |
| `t_arc_explore_pattern` | Read,Read,Read | arc == "explore" | AC-5.0.2 |
| `t_topic_uses_ai_title_when_present` | session.ai_title = "X" | topic == "X" | AC-5.0.2 |
| `t_topic_fallback_first_user_turn` | ai_title None, first turn "Fix the bundle icons" | topic includes "bundle icons" or similar | AC-5.0.2 |
| `t_enrich_determinism` | call enrich × 2 with same input | identical output | AC-5.0.2 |
| `t_enrich_no_external_calls` | check imports | no reqwest, no spawn outside in-process | AC-5.0.1 |

---

## 2. Unit tests · KC-05 spawn_blocking + Semaphore

| Test | Setup | Expected | AC |
|------|-------|---------|----|
| `t_embedder_uses_spawn_blocking` | embed_batch call trace | tokio::task::spawn_blocking entered | AC-5.1.1 |
| `t_semaphore_caps_concurrency` | parallel embed_batch × 10 | max in-flight == cpus/2 | AC-5.1.1 |
| `t_batch_chunks_32` | bulk_index 80 sessions | embedder called 3 times (32+32+16) | AC-5.1.2 |
| `t_batch_size_partial_last` | 33 sessions | 2 batches (32+1) | AC-5.1.2 |

---

## 3. Unit tests · KC-02 per-vector HNSW

| Test | Setup | Expected | AC |
|------|-------|---------|----|
| `t_content_hnsw_m_24` | collection config | content.hnsw_config.m == 24 | AC-5.2.1 |
| `t_tool_hnsw_m_12` | collection config | tool.hnsw_config.m == 12 | AC-5.2.1 |
| `t_content_late_hnsw_m_0` | collection config | content_late.hnsw_config.m == 0 (rerank-only) | AC-5.2.1, AC-4.1.5 |

---

## 4. Unit tests · KC-06 strict mode

| Test | Setup | Expected | AC |
|------|-------|---------|----|
| `t_strict_mode_enabled` | collection config | strict_mode.enabled == true | AC-5.3.1 |
| `t_max_resident_memory_85` | collection config | max_resident_memory_percent == 85 | AC-5.3.1 |
| `t_max_query_limit_100` | collection config | max_query_limit == 100 | AC-5.3.1 |

---

## 5. Unit tests · KG-01 Topology cache

| Test | Setup | Expected | AC |
|------|-------|---------|----|
| `t_cache_hit_skips_scan` | first call computes, second call same mtime | scan_dir 호출 1회 | AC-5.4.1 |
| `t_cache_miss_on_mtime_change` | first call, mtime changed | scan_dir 재호출 | AC-5.4.1 |
| `t_cache_per_root_separate` | 2 different roots | 2 entries | (edge) |

---

## 6. Unit tests · KG-02 Predict LRU

| Test | Setup | Expected | AC |
|------|-------|---------|----|
| `t_lru_hit_skips_parse` | parse session_a, parse session_a again | parse_session 호출 1회 | AC-5.5.1 |
| `t_lru_evicts_at_64` | parse 65 different sessions | first session evicted | AC-5.5.1 |
| `t_lru_invalidates_on_mtime` | parse session_a, mtime change, parse again | parse_session 재호출 | AC-5.5.1 |

---

## 7. Property tests

| Property | Statement |
|---------|-----------|
| `prop_enrich_pure` | 같은 input → 같은 output (5 필드 모두) |
| `prop_intent_in_valid_set` | intent ∈ {"build","impl","debug","mixed"} 항상 |
| `prop_outcome_in_valid_set` | outcome ∈ {"resolved","unresolved","partial"} |
| `prop_entities_bounded` | entities.len() ≤ 10 |
| `prop_lru_capacity_bounded` | cache 크기 ≤ 64 항상 |

---

## 8. Integration tests

| Test | Setup | Action | Expected |
|------|-------|--------|----------|
| `it_scan_index_with_enrich` | 80 sessions | bulk_index | all v3 points have 5 enrich fields populated |
| `it_topology_cold_then_warm` | call topology × 2 with same corpus | second is fast | < 5% of first call latency |
| `it_predict_warm_50ms` | predict on session × repeat | warm latency | < 50ms |
| `it_bulk_index_perf_3_4x` | N=80 cold scan | wall-clock | 3-4× faster than pre-batch baseline |
| `it_no_ui_freeze_during_indexing` | start indexing + IPC poll | poll responds < 16ms | AC-5.1.3 |

---

## 9. Regression tests

| Surface | Check |
|---------|-------|
| Time Machine | unchanged |
| Topology | result identical (cache hit produces same data as miss) |
| Lens (P2) | unchanged response shape |
| Mix & Match · Recall | unchanged |
| Predict | result identical (LRU is pure cache) |
| Replay | unchanged |

---

## 10. Performance benchmark tests

### Test file: `src-tauri/benches/perf.rs` (criterion crate)

| Benchmark | Setup | Target |
|-----------|-------|--------|
| `bench_bulk_index_n80` | 80 sessions | < 60s |
| `bench_topology_n80` | warm | < 200ms |
| `bench_topology_n10k` | warm | < 200ms (확장 검증) |
| `bench_predict_warm` | LRU hit | < 50ms |
| `bench_predict_cold` | LRU miss | < 900ms |
| `bench_lens_search_p95` | 100 queries | p95 < 200ms |

---

## 11. Test → AC mapping

| AC | Tests |
|----|-------|
| AC-5.0.1 ~ AC-5.0.4 | 20 unit + 5 prop + 1 integration |
| AC-5.1.1 ~ AC-5.1.4 | 4 unit + 1 integration + 1 bench |
| AC-5.2.1 ~ AC-5.2.2 | 3 unit + 1 bench |
| AC-5.3.1 ~ AC-5.3.2 | 3 unit (integration is OOM-simulated) |
| AC-5.4.1 ~ AC-5.4.2 | 3 unit + 1 integration + 2 bench |
| AC-5.5.1 ~ AC-5.5.2 | 3 unit + 1 integration + 2 bench |

**총 ~50 test case + 6 benchmark**.
