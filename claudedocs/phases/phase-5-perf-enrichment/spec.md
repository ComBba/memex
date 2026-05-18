# Phase 5 — Performance & Enrichment · SDD

**Phase ID**: P5
**KICKs**: KC-02 (per-vector HNSW), KC-05 (spawn_blocking + Semaphore + batch=32), KC-06 (max_resident_memory_percent), KG-01 (Topology insights cache), KG-02 (Predict LRU)
**Plus**: `enrich.rs` heuristic module (LLM-free replacement for Cat D)
**Owner**: B3 Felix + B4 Priya
**Dependency**: P3
**Day**: D-11 ~ D-9 (parallel to P4)
**Cross-phase invariants**: 6 모두 (no-LLM 엄수)

---

## 1. `enrich.rs` Heuristic Enrichment Module (LLM-free)

### 1.1 Behavior contract

`scan --index`에서만 호출. 각 Session에 대해 다음 5개 payload field를 deterministic하게 채움:

| 필드 | 로직 |
|------|------|
| `intent` | tool 분포 다수결: Bash dominant → "build", Edit dominant → "impl", Read dominant → "debug", 혼합 → "mixed" |
| `entities` | `tool_calls`에서 file path, command name 추출 (regex), 중복 제거, 빈도순 top-10 |
| `outcome` | 마지막 assistant turn에 `resolved\|fixed\|works\|done\|success` substring 있으면 "resolved", `fail\|error\|stuck` 있으면 "unresolved", 둘 다 없으면 "partial". 마지막 Bash exit code = 0이면 "resolved" 우선 |
| `arc` | tool sequence pattern: Read*→Edit→Bash 패턴 = "fix", Read*→Bash failure*→Edit = "debug-fix", Edit+ → "impl", Read+ → "explore" |
| `topic` | `ai_title` 있으면 그대로. 없으면 project_name + first-user-turn의 첫 명사구 (정규식 추출) |

### 1.2 API signature

```rust
// src-tauri/src/indexer/enrich.rs
pub struct EnrichmentInput<'a> {
    pub session: &'a Session,
    pub turns: &'a [Turn],
}

pub struct EnrichmentOutput {
    pub intent: String,
    pub entities: Vec<String>,
    pub outcome: String,
    pub arc: String,
    pub topic: String,
}

pub fn enrich(input: EnrichmentInput) -> EnrichmentOutput {
    EnrichmentOutput {
        intent: classify_intent(&input.session.tool_counts),
        entities: extract_entities(input.turns),
        outcome: classify_outcome(input.turns),
        arc: classify_arc(input.turns),
        topic: derive_topic(input.session),
    }
}
```

### 1.3 Determinism

모든 함수는 pure function. 같은 input → 같은 output. 어떤 random/시간 의존 0. (replay reproducibility 보장)

### 1.4 Acceptance criteria

- [ ] **AC-5.0.1** (owner: B3 Felix) `enrich.rs` 모듈 분리, in-process 함수만 (외부 호출 0)
- [ ] **AC-5.0.2** (owner: B4 Priya) 5 필드 모두 deterministic
- [ ] **AC-5.0.3** (owner: B4 Priya) 80% LLM equivalent quality (Memex corpus 평가) — eval로 검증
- [ ] **AC-5.0.4** (owner: B3 Felix) enrich 출력이 P3 payload schema의 5 필드 채움

---

## 2. KC-05 · spawn_blocking + Semaphore + batch=32

### 2.1 Behavior contract

ONNX inference를 blocking pool로 옮김. Semaphore로 동시성 캡. bulk_index의 per-session loop를 cross-session batch로.

### 2.2 API signature

```rust
// src-tauri/src/indexer/embed.rs
use tokio::sync::Semaphore;
use std::sync::Arc;

pub struct Embedder {
    inner: Arc<TextEmbedding>,
    sem: Arc<Semaphore>,
}

impl Embedder {
    pub fn new(model: TextEmbedding) -> Self {
        let cpus = num_cpus::get();
        Self {
            inner: Arc::new(model),
            sem: Arc::new(Semaphore::new((cpus / 2).max(1))),
        }
    }
    pub async fn embed_batch(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
        let _permit = self.sem.acquire().await?;
        let model = self.inner.clone();
        tokio::task::spawn_blocking(move || model.embed(texts, None))
            .await?
    }
}

// bulk_index loop
const CROSS_SESSION_BATCH: usize = 32;
for chunk in sessions.chunks(CROSS_SESSION_BATCH) {
    let all_texts: Vec<String> = chunk.iter().flat_map(|s| s.extract_5_texts()).collect();
    let all_embeddings = embedder.embed_batch(all_texts).await?;
    // map back to sessions, upsert
}
```

### 2.3 Acceptance criteria

- [ ] **AC-5.1.1** (owner: B3 Felix) Embedder가 spawn_blocking + Semaphore 사용
- [ ] **AC-5.1.2** (owner: B3 Felix) bulk_index가 cross-session batch=32
- [ ] **AC-5.1.3** (owner: B5 Min-jae) indexing 중 UI 멈춤 0 (60fps 유지)
- [ ] **AC-5.1.4** (owner: B4 Priya) cold scan 3-4× 가속 측정

---

## 3. KC-02 · Per-vector HNSW Tuning

### 3.1 Behavior contract

벡터별 HNSW config:
- `content`: m=24, ef_construct=200 (고차원 의미)
- `tool`, `path`: m=12, ef_construct=64 (짧고 노이지)
- `error`: m=16, ef_construct=100
- `code`: m=20, ef_construct=150
- `content_late`: m=0 (HNSW disabled, rerank-only)

### 3.2 Acceptance criteria

- [ ] **AC-5.2.1** (owner: B3 Felix) 각 named vector가 다른 HNSW config (P3 schema 협력)
- [ ] **AC-5.2.2** (owner: B4 Priya) cold query latency -15% 측정

---

## 4. KC-06 · Strict Mode max_resident_memory_percent

### 4.1 Behavior contract

P3 schema의 strict_mode_config:
```json
"strict_mode_config": {
  "enabled": true,
  "max_resident_memory_percent": 85,
  "max_query_limit": 100
}
```

### 4.2 Acceptance criteria

- [ ] **AC-5.3.1** (owner: B3 Felix) strict_mode 활성화
- [ ] **AC-5.3.2** (owner: B2 Aisha) 16GB MacBook에서 mid-demo OOM 0

---

## 5. KG-01 · Topology Insights Cache

### 5.1 Behavior contract

`compute_insights`를 `(root_path, max_mtime)`로 memoize.

```rust
struct InsightsCache {
    cache: Mutex<HashMap<(PathBuf, SystemTime), Insights>>,
}

impl InsightsCache {
    pub fn get_or_compute(&self, root: PathBuf, mtime: SystemTime, f: impl Fn() -> Insights) -> Insights {
        let key = (root, mtime);
        let mut c = self.cache.lock();
        c.entry(key).or_insert_with(f).clone()
    }
}
```

### 5.2 Acceptance criteria

- [ ] **AC-5.4.1** (owner: B3 Felix) cache hit 시 raw `scan_dir` 호출 0
- [ ] **AC-5.4.2** (owner: B4 Priya) Topology N=10k에서 17min → 200ms 측정

---

## 6. KG-02 · Predict Pivot-Parse LRU

### 6.1 Behavior contract

`predict_next_actions`가 neighbor 8 session을 재파싱. LRU 64-entry, mtime-keyed.

```rust
use lru::LruCache;

struct ParseLruCache {
    inner: Mutex<LruCache<(PathBuf, SystemTime), Arc<Session>>>,
}
```

### 6.2 Acceptance criteria

- [ ] **AC-5.5.1** (owner: B3 Felix) LRU 64-entry
- [ ] **AC-5.5.2** (owner: B4 Priya) per-prediction 300-900ms → ~50ms warm 측정

---

## 7. Phase 5 종합 acceptance

| ID | 항목 |
|----|------|
| P5-DONE-1 | enrich.rs 5 필드 모두 동작, 80% quality |
| P5-DONE-2 | bulk_index cold scan 3-4× 가속 |
| P5-DONE-3 | Topology N=10k 200ms 도달 |
| P5-DONE-4 | Predict warm 50ms 도달 |
| P5-DONE-5 | strict_mode + per-vector HNSW 활성화 |
| P5-DONE-6 | tests.md 모두 통과 |

---

## 8. Risk

| Risk | Mitigation |
|------|-----------|
| enrich heuristic 품질 부족 (80% 미달) | regex/패턴 fine-tune · ai_title 우선 사용 |
| Semaphore가 너무 작아 throughput 낮음 | num_cpus/2 안정선, 측정 후 조정 |
| LRU cache invalidation 버그 (Aisha dissent) | mtime 변경 즉시 invalidate · 단위 test |
| strict_mode가 정상 upsert 차단 | max_resident_memory_percent=85 (안전 마진) |

---

## 9. Out-of-scope

- ONNX Runtime CoreML EP (이전 plan에서 빠짐, 추후 검토)
- Multi-thread fastembed (fastembed-rs 5.x 단일 inference)
