# Phase 2 — Query API Core · SDD

**Phase ID**: P2
**KICKs**: KA-01 (FormulaQuery), KA-02 (MMR on NearestQuery), KA-05 (weighted RRF), KB-02 (BM25 sparse on path+tool)
**Owner**: B1 Hiroshi (Principal) + B4 Priya (Search Quality)
**Dependency**: P3 schema (sparse vectors + schema_version)
**Day**: D-13~D-12
**Cross-phase invariants**: 6개 모두 적용 (특히 No-LLM, Result<T,String> IPC)

---

## 1. KA-01 · FormulaQuery — server-side lens scoring

### 1.1 Behavior contract

`lens_search(query: String, weights: HashMap<String, f32>)` is replaced:

- **현재**: 5 parallel cosine queries → Rust client-side weighted sum (`indexer.rs:539-615`)
- **목표**: Single Qdrant 1.18 Query API call with `prefetch` + `formula` score expression

특히:
1. prefetch에 dense + sparse + multivec(P4) prefetch 모두 포함 (현 phase는 dense + sparse 우선)
2. score formula = weighted sum + recency exp_decay + has_errors boost
3. round-trips: 5 → **1**

### 1.2 API signature

```rust
// src-tauri/src/indexer/search.rs
pub async fn lens_search(
    client: &QdrantClient,
    embedder: &Embedder,
    query_text: &str,
    weights: &LensWeights,
    limit: u64,
) -> anyhow::Result<Vec<LensResult>> {
    // build prefetch[] from weights (skip zero-weight vectors)
    // build score formula with sum + mult + exp_decay
    // single query_points() call
}

pub struct LensWeights {
    pub content: f32,
    pub tool: f32,
    pub path: f32,       // sparse (BM25 via Qdrant server-side)
    pub error: f32,
    pub code: f32,
    pub content_late: Option<f32>,  // P4 추가, 옵션
}

pub struct LensResult {
    pub session_id: String,
    pub score: f32,
    pub score_breakdown: ScoreBreakdown,  // for WOW-3
    pub payload: SessionPayload,
}

pub struct ScoreBreakdown {
    pub per_vector: HashMap<String, f32>,   // content: 0.42, path: 0.34, ...
    pub recency_factor: f32,                // exp_decay value
    pub has_errors_boost: f32,              // 0.0 or +0.2
    pub final_score: f32,
}
```

### 1.3 Data structure · Qdrant query JSON

```json
{
  "prefetch": [
    { "using": "content",      "query": [..dense 384..], "limit": 50 },
    { "using": "tool",         "query": [..dense 384..], "limit": 50 },
    { "using": "path_sparse",  "query": { "indices": [...], "values": [...] }, "limit": 50 },
    { "using": "tool_sparse",  "query": { "indices": [...], "values": [...] }, "limit": 50 },
    { "using": "error",        "query": [..dense 384..], "limit": 50,
      "filter": { "must": [{ "key": "has_errors", "match": { "value": true } }] } },
    { "using": "code",         "query": [..dense 384..], "limit": 50 }
  ],
  "query": {
    "formula": {
      "sum": [
        { "mult": [{ "var": "$score.content" }, 1.0] },
        { "mult": [{ "var": "$score.tool" }, 1.5] },
        { "mult": [{ "var": "$score.path_sparse" }, 1.5] },
        { "mult": [{ "var": "$score.error" }, 0.8] },
        { "mult": [{ "var": "$score.code" }, 0.6] },
        { "exp_decay": { "x": { "var": "payload.start_ts" }, "scale": 2592000 } },
        { "value": 0.2, "filter": { "key": "has_errors", "match": { "value": true } } }
      ]
    }
  },
  "limit": 20,
  "with_payload": true
}
```

> **v0.2 보강**: `$score.<vector_name>`은 prefetch 각각의 score를 참조. `payload.<field>`는 payload field 참조. 1.14+ FormulaQuery spec.

### 1.4 Edge cases

| 케이스 | 동작 |
|------|------|
| 모든 weight = 0 | Err("no active lens") |
| query_text 빈 문자열 | Err("empty query") |
| Qdrant offline | Err with retry hint (lazy AppState re-init) |
| sparse vector 없는 collection (P3 미완) | dense-only fallback (sparse prefetch 생략) |
| limit > 100 | clamp to 100 |
| has_errors filter no match (corpus 무에러) | error prefetch 결과 empty, formula는 정상 |

### 1.5 Acceptance criteria

- [ ] **AC-2.1.1** (owner: B1 Hiroshi) `lens_search` 가 단일 `query_points()` 호출
- [ ] **AC-2.1.2** (owner: B4 Priya) `ScoreBreakdown.per_vector` 가 각 prefetch의 raw score 보존 (WOW-3 시각화)
- [ ] **AC-2.1.3** (owner: B4 Priya) recency decay 30-day half-life (`scale: 2592000` = 30d in seconds)
- [ ] **AC-2.1.4** (owner: B4 Priya) has_errors boost 정확히 `+0.2`
- [ ] **AC-2.1.5** (owner: B1 Hiroshi) sparse fallback (P3 미완 시 dense-only graceful)
- [ ] **AC-2.1.6** (owner: A3 Liana) frontend `main.js` 가 `score_breakdown`을 WOW-3 bar로 렌더

---

## 2. KA-02 · MMR on NearestQuery — diversification

### 2.1 Behavior contract

`lens_search`의 결과에 MMR(Maximal Marginal Relevance) 적용:
- Qdrant 1.15+ `params.diversity` 옵션 사용
- 같은-프로젝트 중복 자동 분산
- 기본값 0.4 (사용자가 slider로 조정 가능 — P6)

### 2.2 API signature

```rust
pub struct LensWeights {
    // ... 기존 필드
    pub diversity: f32,  // 0.0 (no MMR) ~ 1.0 (max diversity)
}

// query JSON에 추가:
// "params": { "diversity": 0.4 }
```

### 2.3 Acceptance criteria

- [ ] **AC-2.2.1** (owner: B4 Priya) `diversity` 필드가 LensWeights에 존재
- [ ] **AC-2.2.2** (owner: B4 Priya) diversity=0.4 default · UI slider로 조정 가능
- [ ] **AC-2.2.3** (owner: B4 Priya) diversity=0일 때 기존 ranking과 동일

---

## 3. KA-05 · Weighted RRF Fusion

### 3.1 Behavior contract

prefetch 결합 방식:
- 1차 fusion: `rrf` with `weights` (KA-05)
- 2차 score: formula (KA-01)

즉 prefetch별 RRF rank 후, 그 결과에 formula 적용. Qdrant 1.17+ weighted RRF.

### 3.2 Data structure

```json
"query": {
  "fusion": "rrf",
  "weights": [1.0, 1.5, 1.5, 0.8, 0.6, 1.0]  // prefetch 순서대로
}
```

### 3.3 Acceptance criteria

- [ ] **AC-2.3.1** (owner: B1 Hiroshi) prefetch 순서와 weights 순서 매핑 결정론
- [ ] **AC-2.3.2** (owner: B1 Hiroshi) weight=0인 prefetch는 query에서 생략

---

## 4. KB-02 · Server-side BM25 Sparse on `path` + `tool`

### 4.1 Behavior contract

`path`와 `tool` named vector를 sparse vector로 보강:
- Qdrant 1.15.2+ server-side tokenization (word tokenizer)
- modifier = `idf`
- 클라이언트 임베딩 없음 — server가 토큰 직접 처리

### 4.2 Collection config (P3에서 적용)

```json
"sparse_vectors": {
  "path_sparse": {
    "modifier": "idf",
    "index": { "type": "mutable", "on_disk": false }
  },
  "tool_sparse": {
    "modifier": "idf",
    "index": { "type": "mutable", "on_disk": false }
  }
}
```

### 4.3 Upsert · query 시그니처

```rust
// upsert (P3에서 실제 수행)
let sparse_value = SparseInput::Text {
    text: session.all_paths_joined(),  // 또는 all_tools_joined()
    tokenizer: Tokenizer::Word,
};

// query
let sparse_query = SparseInput::Text {
    text: query_text,
    tokenizer: Tokenizer::Word,
};
```

### 4.4 Acceptance criteria

- [ ] **AC-2.4.1** (owner: B3 Felix) `path_sparse`, `tool_sparse`가 collection schema에 존재 (P3과 협력)
- [ ] **AC-2.4.2** (owner: B4 Priya) `lens_search` prefetch에 두 sparse 포함
- [ ] **AC-2.4.3** (owner: B4 Priya) BM25 score가 "edit auth.js" 류 쿼리에서 dense 대비 명확한 ranking 차이

---

## 5. Phase 2 종합 acceptance

| ID | 항목 |
|----|------|
| P2-DONE-1 | `lens_search` 가 단일 Query API 호출, 모든 6 named vector 지원 |
| P2-DONE-2 | `ScoreBreakdown` 가 frontend로 IPC 통과 (Result<LensResult, String>) |
| P2-DONE-3 | WOW-3 prototype (claudedocs/prototypes/wow-3-...html)이 실제 lens_search 결과로 동작 (fake 아닌 real) |
| P2-DONE-4 | 기존 lens_search 호출 사이트 (commands.rs `lens_search`) 시그니처 호환 |
| P2-DONE-5 | `tests.md`의 모든 unit + integration 통과 |

---

## 6. Risk

| Risk | Mitigation |
|------|-----------|
| FormulaQuery 문법 학습 곡선 | D-13 spike PR · qdrant-client 1.18 예제 검증 |
| `$score.<name>` reference 문법이 docs와 불일치 | research/06 cite한 URL 재확인 + Qdrant Discord ask |
| sparse vector recall 저하 (idf 부적합) | modifier 옵션 fallback (`none`) 준비 |
| Tauri IPC payload 크기 (ScoreBreakdown × 20 결과) | 평균 ~5 KB, 우려 없음 |

---

## 7. Out-of-scope (P2 안 함)

- Late-interaction multivec (P4)
- Discovery API (P4)
- Group-by query (P4)
- Order-by (P4)
- ACORN filter strict (P4)
