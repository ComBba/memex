# Phase 3 — Schema Evolution · SDD

**Phase ID**: P3
**KICKs**: KC-01 (TurboQuant bits2), KC-01b (rescore+oversampling), KC-03 (tenant index), KC-04 (datetime native), KG-03 (schema_version + dual-write), KG-04 (conditional updates)
**Owner**: B3 Felix (Backend Lead)
**Dependency**: P1 (sec)
**Day**: D-12 ~ D-11
**Cross-phase invariants**: 6 모두

---

## 1. KG-03 · schema_version Payload Index + Dual-Write

### 1.1 Behavior contract

새 collection `memex_sessions_v3`을 생성하고 다음 dual-write:

1. **Read**: v3 우선, v3 miss 시 v2(기존) fallback
2. **Write**: v3에만 새 데이터. v2는 freeze.
3. **schema_version** payload field가 모든 point에 명시 (`u32`).
4. dual-write 기간: P3 → P5 완료 시점까지. P6에 v2 retire.

### 1.2 Collection schema (v3)

```json
{
  "name": "memex_sessions_v3",
  "vectors": {
    "content":      { "size": 384, "distance": "Cosine" },
    "tool":         { "size": 384, "distance": "Cosine" },
    "path":         { "size": 384, "distance": "Cosine" },
    "error":        { "size": 384, "distance": "Cosine" },
    "code":         { "size": 384, "distance": "Cosine" },
    "content_late": {
      "size": 384, "distance": "Cosine",
      "multivector_config": { "comparator": "MaxSim" },
      "hnsw_config": { "m": 0 }
    }
  },
  "sparse_vectors": {
    "path_sparse": { "modifier": "idf" },
    "tool_sparse": { "modifier": "idf" }
  },
  "quantization_config": {
    "turbo": { "bits": "Bits2", "always_ram": true }
  },
  "hnsw_config": { "m": 16, "ef_construct": 100, "payload_m": 16 },
  "strict_mode_config": {
    "enabled": true,
    "max_resident_memory_percent": 85
  },
  "wal_config": { "wal_capacity_mb": 32 }
}
```

### 1.3 Payload schema (v3)

```json
{
  "session_id": "df1906d2-...",
  "source_path": "~/.claude/projects/.../df1906d2-....jsonl",
  "project_name": "memex",
  "project_path": "~/projects/memex",
  "git_branch": "main",
  "ai_title": "(may be empty)",
  "claude_version": "2.1.143",
  "start_iso": "2026-05-17T09:15:18.335Z",
  "end_iso": "2026-05-17T10:48:02.000Z",
  "start_ts_dt": "2026-05-17T09:15:18.335Z",
  "end_ts_dt": "2026-05-17T10:48:02.000Z",
  "user_turns": 232,
  "assistant_turns": 403,
  "tool_count": 220,
  "has_errors": true,
  "schema_version": 3,
  "intent": "build",
  "entities": ["icons/", "Cargo.toml"],
  "outcome": "resolved",
  "arc": "debug-fix",
  "topic": "Tauri macOS bundle signing"
}
```

- `start_ts` (integer, v2)와 `start_ts_dt` (datetime, v3) 둘 다 유지 (dual-write 기간).
- `intent / entities / outcome / arc / topic` 은 P5 enrich.rs가 채움.

### 1.4 Payload indexes (v3)

```json
{
  "project_name":   { "type": "tenant", "is_tenant": true },
  "project_path":   { "type": "keyword" },
  "git_branch":     { "type": "keyword" },
  "ai_title":       { "type": "text" },
  "start_ts_dt":    { "type": "datetime" },
  "has_errors":     { "type": "bool" },
  "schema_version": { "type": "integer" },
  "intent":         { "type": "keyword" },
  "outcome":        { "type": "keyword" }
}
```

### 1.5 Migration plan

```
Day D-12:
  1. Create memex_sessions_v3 collection
  2. Background task: re-embed + re-index from JSONL source (parser.rs 그대로)
  3. enrich.rs (P5 with stub for now)가 heuristic 채움
  4. v2 = read-only

Day D-11:
  5. Verify v3 corpus count == v2 corpus count
  6. Frontend default = v3, v2 = fallback

Day D-5:
  7. (after P6 visual integration) v2 retire (drop collection)
```

### 1.6 API signature

```rust
pub async fn ensure_collection_v3(client: &QdrantClient) -> Result<()> { ... }
pub async fn migrate_v2_to_v3(client: &QdrantClient) -> Result<MigrationReport> { ... }
pub async fn dual_read<T>(
    client: &QdrantClient,
    op: impl Fn(&str) -> Future<Output = Result<T>>,
) -> Result<T> {
    // try v3, fallback v2 on NotFound
}
```

### 1.7 Acceptance criteria

- [ ] **AC-3.1.1** (owner: B3 Felix) `ensure_collection_v3` 멱등 (재실행 안전)
- [ ] **AC-3.1.2** (owner: B3 Felix) Migration script가 v2 corpus 전체 → v3 (count 일치)
- [ ] **AC-3.1.3** (owner: B3 Felix) Dual-read가 v3 우선, miss시 v2 fallback (transparent to caller)
- [ ] **AC-3.1.4** (owner: B3 Felix) schema_version=3 payload index 존재, query에서 filterable
- [ ] **AC-3.1.5** (owner: B2 Aisha) D-3 schema lock 이후 v3 추가 변경 금지 (gate)

---

## 2. KC-01 + KC-01b · TurboQuant bits2 + Rescore

### 2.1 Behavior contract

**KC-01**: TurboQuant `bits2`로 vector storage 16× 압축.
**KC-01b**: query time `rescore=true` + `oversampling=2.0`로 recall 회복.

**중요 (research/10 over-claim 방지)**:
- `"quantization_config": { "turbo": { "bits": "Bits2" } }` ← 정확한 1.18 syntax
- ❌ Binary Quantization (1.15-era `"binary"`)과 혼동 금지

### 2.2 Query params (모든 search 시)

```json
"params": {
  "quantization": {
    "ignore": false,
    "rescore": true,
    "oversampling": 2.0
  }
}
```

### 2.3 Acceptance criteria

- [ ] **AC-3.2.1** (owner: B3 Felix) collection config의 `"turbo"` key + `Bits2` enum
- [ ] **AC-3.2.2** (owner: B3 Felix) 모든 `lens_search` / `recall` / `predict_next_actions` 호출에 rescore + oversampling 자동 적용
- [ ] **AC-3.2.3** (owner: B4 Priya) recall@10 (KC-01b 없이 KC-01만)과 (KC-01+KC-01b)의 차이 측정 — eval gate에서 보고
- [ ] **AC-3.2.4** (owner: B3 Felix) storage size 측정: v2 (raw) vs v3 (turbo bits2 + rescore) — 16× 압축 검증

---

## 3. KC-03 · Tenant Index per `project_name`

### 3.1 Behavior contract

`project_name`을 tenant key로 → per-project HNSW 그래프. 같은-프로젝트 query가 다른 프로젝트 scan 안 함.

### 3.2 Config

```json
"payload_index": {
  "project_name": { "type": "tenant", "is_tenant": true }
}
```

### 3.3 Acceptance criteria

- [ ] **AC-3.3.1** (owner: B3 Felix) tenant index 생성 확인 (`collection_info` 호출)
- [ ] **AC-3.3.2** (owner: B4 Priya) `project_name="memex"` filter query가 다른 project 데이터 미접근 (server logs 확인)

---

## 4. KC-04 · Datetime Native Index on `start_ts`

### 4.1 Behavior contract

`start_ts` (int) → `start_ts_dt` (datetime ISO 8601) 필드 추가. P5 enrich에서 채움.
KA-01 FormulaQuery의 `exp_decay`가 datetime 직접 참조.

### 4.2 Acceptance criteria

- [ ] **AC-3.4.1** (owner: B3 Felix) `start_ts_dt` payload index 존재
- [ ] **AC-3.4.2** (owner: B4 Priya) datetime range query 동작 (예: 지난 7일)

---

## 5. KG-04 · Conditional Updates + update_mode::Update

### 5.1 Behavior contract

Embedding staleness worker가 `schema_version < 3` point만 재처리:

```rust
client.update_payload(
    &collection,
    &points,
    payload,
    Some(UpdateMode::Update),
    Some(condition!({ "schema_version": { "lt": 3 } })),
).await
```

### 5.2 Acceptance criteria

- [ ] **AC-3.5.1** (owner: B3 Felix) `update_payload` 호출에 `UpdateMode::Update` 명시
- [ ] **AC-3.5.2** (owner: B3 Felix) condition 미적용 시 모든 point가 변경됨 — 적용 시 schema_version<3만 (test로 검증)

---

## 6. Phase 3 종합 acceptance

| ID | 항목 |
|----|------|
| P3-DONE-1 | `memex_sessions_v3` collection 존재, 6 dense + 1 multivec + 2 sparse + TurboQuant bits2 + tenant idx + datetime idx |
| P3-DONE-2 | Migration v2→v3 1회 완료, count 일치 |
| P3-DONE-3 | Dual-read 동작 (v3 우선) |
| P3-DONE-4 | 모든 query에 rescore+oversampling 자동 |
| P3-DONE-5 | `tests.md` 모든 통과 |

---

## 7. Risk

| Risk | Mitigation |
|------|-----------|
| TurboQuant 정확도 손실 | KC-01b rescore 페어링 (필수). 측정으로 검증 |
| v3 collection 재생성 중 데모 데이터 손실 | v2 freeze + read-only fallback |
| Migration이 D-12 안에 안 끝남 | background task로 분리, frontend는 자연 degrade |
| datetime 형식 호환성 (ISO 8601 vs timestamp) | 양쪽 다 저장 |

---

## 8. Out-of-scope

- v2 collection 즉시 drop (P5 안정화 후로 미룸)
- Migration UI (개발자만, CLI로 수동 trigger)
- Per-vector HNSW tuning (P5 KC-02)
