# Phase 4 — Advanced Retrieval · SDD

**Phase ID**: P4
**KICKs**: KB-01 (late-interaction content_late), KB-03 (Discovery true context pairs), KB-04 (ACORN strict filter), KB-05 (Order-by query), KA-03 (group-by query), KA-04 (RelevanceFeedbackQuery)
**Owner**: B4 Priya (Search Quality) + B1 Hiroshi
**Dependency**: P2 (Query API), P3 (schema)
**Day**: D-11 ~ D-9
**Cross-phase invariants**: 6 모두

---

## 1. KB-01 · Late-Interaction MaxSim with `content_late`

### 1.1 Behavior contract

같은 BGE-small token-level 출력을 *multivector* point에 저장 (HNSW disabled). Lens prefetch에 rerank-only로 추가.

특히:
1. Indexing: 각 세션의 content 텍스트를 BGE-small에 통과시켜 *per-token* 384-d 벡터 list 추출 (현재는 pooled 1개만 사용)
2. Storage: `content_late` named vector에 multivector 저장
3. Query: query text도 token-level로 → MaxSim 비교

### 1.2 API signature

```rust
// src-tauri/src/indexer/embed.rs
pub async fn embed_token_level(
    embedder: &Embedder,
    text: &str,
) -> Result<Vec<Vec<f32>>> {
    // returns list of 384-d vectors (one per token)
}
```

### 1.3 Token noise mitigation (research/10 dissent 반영)

A/B 측정 (D-8 eval gate):
- Option A: SentencePiece 토큰 그대로
- Option B: sliding-window 32-token chunks

둘 다 dense-only 대비 nDCG@10 측정. +15% 미달 시 KB-01 revert (Marcus dissent 조건).

### 1.4 Acceptance criteria

- [ ] **AC-4.1.1** (owner: B4 Priya) `embed_token_level` 함수 존재
- [ ] **AC-4.1.2** (owner: B4 Priya) content_late multivector upsert 동작
- [ ] **AC-4.1.3** (owner: B4 Priya) Lens prefetch에 content_late 포함
- [ ] **AC-4.1.4** (owner: B4 Priya) D-8 eval: nDCG@10 +15% 이상 vs dense-only (gate)
- [ ] **AC-4.1.5** (owner: B3 Felix) HNSW disabled (m=0) — multivector는 rerank-only

---

## 2. KB-03 · Discovery API True Context Pairs

### 2.1 Behavior contract

Mix & Match의 단순 anchor → (positive, reference_negative) **페어** 구조로 진화.

### 2.2 Query JSON

```json
{
  "query": {
    "discover": {
      "target": <anchor positive vector>,
      "context": [
        { "positive": <sess_a>, "negative": <sess_b> },
        { "positive": <sess_c>, "negative": <sess_d> }
      ]
    }
  },
  "limit": 10
}
```

### 2.3 Frontend (WOW-5 협력)

사용자가 positive 카드 1개 + negative 카드 1개를 페어로 drag. 여러 페어 추가 가능.

### 2.4 Acceptance criteria

- [ ] **AC-4.2.1** (owner: B1 Hiroshi) `mix_match` 시그니처가 `Vec<ContextPair>`를 받음 (기존: Vec<positive> + Vec<negative>)
- [ ] **AC-4.2.2** (owner: A3 Liana) WOW-5 prototype의 pair UI가 production 통합 (P6)
- [ ] **AC-4.2.3** (owner: B1 Hiroshi) 단일 positive (페어 없이) 입력도 backward-compat

---

## 3. KB-04 · ACORN Filterable HNSW

### 3.1 Behavior contract

`has_errors=true`, `project_name=X` 필터를 HNSW 탐색 *중* 적용 (post-filtering 아닌).

### 3.2 Query params

```json
"params": {
  "hnsw_ef": 128,
  "quantization": { ... },
  "exact": false
}
```

`payload_index`의 `is_tenant: true` 효과로 자동 ACORN-shaped.

### 3.3 Acceptance criteria

- [ ] **AC-4.3.1** (owner: B4 Priya) `recall` 호출 시 filter가 HNSW 통과 중 적용 (Qdrant 1.16+ 자동)
- [ ] **AC-4.3.2** (owner: B4 Priya) recall@10 +20% 측정 (filtered query)
- [ ] **AC-4.3.3** (owner: B3 Felix) hnsw_ef tuning 가능 (기본 128)

---

## 4. KB-05 · Order-by Query

### 4.1 Behavior contract

`list_sessions` (Time Machine stack)에서 정렬 옵션:
- `start_ts_dt desc` (기본 — 최근 우선)
- `tool_count desc`
- `start_ts_dt asc` (오래된 순)
- `has_errors desc` (에러 우선)

### 4.2 Query

```json
{
  "order_by": { "key": "start_ts_dt", "direction": "desc" },
  "limit": 80,
  "with_payload": true
}
```

### 4.3 Acceptance criteria

- [ ] **AC-4.4.1** (owner: B1 Hiroshi) `list_sessions(order_by: OrderBy)` 시그니처
- [ ] **AC-4.4.2** (owner: A3 Liana) Time Machine UI에 정렬 드롭다운

---

## 5. KA-03 · Group-by Query

### 5.1 Behavior contract

`lens_search` 결과를 `project_name`으로 그룹핑. 각 프로젝트 top-3.

### 5.2 Query

```json
{
  "query": { ... formula ... },
  "group_by": "project_name",
  "group_size": 3,
  "limit": 8,
  "with_payload": true
}
```

### 5.3 Acceptance criteria

- [ ] **AC-4.5.1** (owner: B1 Hiroshi) `lens_search`의 옵션 파라미터 `group_by: Option<String>`
- [ ] **AC-4.5.2** (owner: B4 Priya) result struct에 그룹 정보 포함

---

## 6. KA-04 · RelevanceFeedbackQuery (1.17)

### 6.1 Behavior contract

Mix & Match 카드에 👍/👎 → server에 feedback 전송 → anchor 재계산 → 즉시 재정렬.

### 6.2 Query

```json
{
  "query": {
    "relevance_feedback": {
      "positive": [<id1>, <id2>],
      "negative": [<id3>],
      "previous_query": { ... }
    }
  }
}
```

### 6.3 Frontend

WOW-5에서 결과 카드에 👍/👎 버튼. 클릭 즉시 새 query.

### 6.4 Acceptance criteria

- [ ] **AC-4.6.1** (owner: B1 Hiroshi) `relevance_feedback` IPC command 존재
- [ ] **AC-4.6.2** (owner: A3 Liana) UI 결과 카드에 👍/👎
- [ ] **AC-4.6.3** (owner: B4 Priya) feedback 결정론 (같은 input → 같은 result)
- [ ] **AC-4.6.4** (owner: A2 Marcus) **D-9 dissent gate**: Daichi storyboard에 끼우지 못하면 backend only (UI 0)

---

## 7. Phase 4 종합 acceptance

| ID | 항목 |
|----|------|
| P4-DONE-1 | KB-01 eval gate 통과 OR revert 결정 (D-8) |
| P4-DONE-2 | KB-03 context pairs IPC + WOW-5 spec 통합 |
| P4-DONE-3 | KB-04 ACORN 측정 보고 |
| P4-DONE-4 | KB-05 order_by 4가지 옵션 동작 |
| P4-DONE-5 | KA-03 group_by 결과 그룹화 |
| P4-DONE-6 | KA-04 relevance feedback backend (UI는 P6) |
| P4-DONE-7 | tests.md 모두 통과 |

---

## 8. Risk

| Risk | Mitigation |
|------|-----------|
| KB-01 nDCG 무효 | D-8 eval gate · revert 절차 명시 |
| KB-03 페어 dragging UX 복잡 | P6에서 1 페어 default, 추가 페어 progressive disclosure |
| KA-04 RelevanceFeedback이 1.17 GA 미확인 | research/06 cite 재확인 + Qdrant Discord ask |
| group_by + RRF 호환성 | Qdrant 문서 cite + spike PR |

---

## 9. Out-of-scope

- Pattern #1 HyDE (v1.1 toggle, v3.2 SKIP)
- Pattern #12 MIPS rewriting (v1.1 toggle, v3.2 SKIP)
