# Phase 3 — Schema Evolution · TDD

**Phase**: P3
**Requires**: Local Qdrant for integration tests

---

## 1. Unit tests · Schema creation

### Test file: `src-tauri/src/indexer/schema.rs` (`#[cfg(test)] mod schema_tests`)

| Test | Setup | Expected | AC |
|------|-------|---------|----|
| `t_collection_config_has_all_dense` | build CollectionConfig | 5 dense vectors (content, tool, path, error, code) | AC-3.1.1 |
| `t_collection_config_has_multivec` | build CollectionConfig | content_late with MaxSim + m=0 | AC-3.1.1 |
| `t_collection_config_has_sparse` | build CollectionConfig | path_sparse, tool_sparse with modifier idf | AC-3.1.1 |
| `t_collection_config_has_turbo_bits2` | build CollectionConfig | quantization_config.turbo.bits == Bits2 | AC-3.2.1 |
| `t_collection_config_strict_mode` | build CollectionConfig | strict_mode.max_resident_memory_percent == 85 | (P5 협력) |
| `t_payload_index_tenant_present` | build PayloadIndex | project_name is_tenant=true | AC-3.3.1 |
| `t_payload_index_datetime_present` | build PayloadIndex | start_ts_dt type=datetime | AC-3.4.1 |
| `t_payload_index_schema_version` | build PayloadIndex | schema_version type=integer | AC-3.1.4 |

---

## 2. Unit tests · Payload schema

| Test | Setup | Expected |
|------|-------|---------|
| `t_payload_v3_serializes_schema_version` | SessionPayload v3 | JSON has `"schema_version": 3` |
| `t_payload_v3_includes_enrich_fields` | new payload | intent, entities, outcome, arc, topic 필드 존재 (값은 enrich.rs가 채움 — null 허용) |
| `t_payload_v3_includes_dt_field` | new payload | start_ts_dt is ISO 8601 string |
| `t_payload_v2_compat_read` | v2 payload (no schema_version) | read에서 schema_version=2로 추론 |

---

## 3. Unit tests · TurboQuant + Rescore

| Test | Setup | Expected | AC |
|------|-------|---------|----|
| `t_quantization_query_params_default` | build search params | rescore=true, oversampling=2.0 | AC-3.2.2 |
| `t_quantization_ignore_false_default` | default | ignore=false | AC-3.2.2 |
| `t_quantization_applies_to_all_searches` | lens_search, recall, predict 빌더 | 모두 동일한 quantization params 포함 | AC-3.2.2 |

---

## 4. Unit tests · Dual-write / Dual-read

### Test file: `src-tauri/src/indexer/crud.rs`

| Test | Setup | Expected | AC |
|------|-------|---------|----|
| `t_dual_read_v3_hit` | v3 has point | result from v3 (not v2) | AC-3.1.3 |
| `t_dual_read_v3_miss_v2_hit` | v3 empty, v2 has point | result from v2 (fallback) | AC-3.1.3 |
| `t_dual_read_both_miss` | both empty | Err(NotFound) | AC-3.1.3 |
| `t_dual_write_only_v3` | upsert | v2 unchanged, v3 has new point | AC-3.1.2 |

---

## 5. Unit tests · Conditional updates (KG-04)

| Test | Setup | Expected | AC |
|------|-------|---------|----|
| `t_update_payload_with_update_mode` | upsert call | mode=Update in proto | AC-3.5.1 |
| `t_conditional_update_filters_by_schema_version` | mix of v2/v3 points | only schema_version<3 affected | AC-3.5.2 |

---

## 6. Integration tests · Qdrant roundtrip

Test file: `src-tauri/tests/schema_integration.rs`. Requires Qdrant running.

| Test | Setup | Action | Expected |
|------|-------|--------|----------|
| `it_ensure_v3_idempotent` | empty Qdrant | ensure_collection_v3 × 2 | 두 번째 호출 noop, no Err |
| `it_v3_upsert_and_search` | fresh v3 collection | upsert 5 sessions, lens_search | results match |
| `it_migrate_v2_to_v3` | v2 has 10 sessions | migrate_v2_to_v3 | v3 count == 10 |
| `it_tenant_index_isolation` | 2 projects × 10 sessions each | search with project_name filter | only that project's points scored |
| `it_turboquant_storage_saving` | indexed corpus | collection size | v3 size ≈ 1/16 of raw vectors |
| `it_dual_read_fallback` | v2 only has session #1, v3 has nothing | get session #1 | success via v2 |
| `it_dual_read_v3_priority` | both have session #1 with different timestamps | get session #1 | v3 timestamp returned |
| `it_datetime_range_query` | corpus with various start_ts | filter "last 7 days" | only recent points |
| `it_strict_mode_blocks_oom` | (simulate near-OOM) | upsert | bounded error (no crash) |

---

## 7. Migration tests

| Test | Setup | Action | Expected |
|------|-------|--------|----------|
| `mig_full_corpus_count_match` | v2 has N sessions | migrate | v3 has N sessions |
| `mig_payload_enrich_fields_null` | v2 → v3 (no enrich yet) | post-migrate query | intent/entities/etc = null |
| `mig_schema_version_marker` | migrated points | query schema_version | == 3 |
| `mig_repeatable` | migrate × 2 | second migration | noop (no duplicates) |
| `mig_v2_unaffected` | migrate | check v2 | unchanged count + content |

---

## 8. Property tests

| Property | Statement |
|----------|-----------|
| `prop_quantization_recall_with_rescore` | rescore=true 일 때 top-K recall ≥ raw - 5% (KC-01b의 가치) |
| `prop_dual_read_consistent` | 같은 session_id 두 번 read → 같은 결과 |
| `prop_upsert_idempotent` | 같은 session 두 번 upsert → 단일 point (uuid_v5) |

---

## 9. Regression tests

| Surface | Check |
|---------|-------|
| `list_sessions` | v2 + v3 합쳐서 전체 list 반환 |
| `topology` | v2 only 시점에 동작, v3 migrate 후도 동작 |
| `lens_search` | sparse vector 없는 v2에서 graceful fallback (P2와 협력) |
| `recall` | error vector 그대로 동작 |
| `predict_next_actions` | source_path 가져오기 그대로 동작 |
| `snapshot_export` / `snapshot_import` | v3 collection의 snapshot 가능 |

---

## 10. Performance tests

| Test | Measurement | Expected |
|------|-------------|----------|
| `perf_migration_80_sessions` | wall-clock time | < 60s on M3 |
| `perf_lens_v3_vs_v2` | p95 query latency | v3 within 1.5× of v2 (rescore 비용 감안) |
| `perf_storage_v3` | disk size | < 200 MB at N=10k (KC-01 bits2) |
| `perf_tenant_query_scan` | server logs (scanned points) | per-project query는 해당 tenant만 |

---

## 11. Test → AC mapping

| AC | Tests |
|----|-------|
| AC-3.1.1 ~ AC-3.1.5 | 8 unit + 5 integration + 1 prop |
| AC-3.2.1 ~ AC-3.2.4 | 3 unit + 3 integration + 1 prop |
| AC-3.3.1 ~ AC-3.3.2 | 1 unit + 2 integration |
| AC-3.4.1 ~ AC-3.4.2 | 1 unit + 1 integration |
| AC-3.5.1 ~ AC-3.5.2 | 2 unit |

**총 ~50 test case** (가장 큰 phase — schema 변경의 광범위 영향).

---

## 12. Test fixture requirements

`src-tauri/tests/fixtures/schema/`:
- `v2-sample.jsonl` (10 세션) — 기존 schema
- `v3-sample.json` — 새 schema payload 예시
- `migration-expected.json` — 예상 migration output

기존 fixture 재사용 + 위 3개 추가.
