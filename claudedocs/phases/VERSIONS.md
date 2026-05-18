# Phase Plan — Version History

Plan v3.2의 24 KICK을 7 phase로 분할한 후 SDD→TDD 문서 작성. 수립↔검증 반복.

## v0.1 — initial draft (2026-05-18)

**Created**:
- `PHASES.md` — 7 phase 그룹핑, dependency graph, 14-day mapping
- `phase-1-security/{spec,tests}.md` — KF-01/02/03
- `phase-2-query-api/{spec,tests}.md` — KA-01/02/05, KB-02
- `phase-3-schema-evolution/{spec,tests}.md` — KC-01/01b/03/04, KG-03/04
- `phase-4-advanced-retrieval/{spec,tests}.md` — KB-01/03/04/05, KA-03/04
- `phase-5-perf-enrichment/{spec,tests}.md` — KC-02/05/06, KG-01/02, enrich.rs
- `phase-6-visual-wow/{spec,tests}.md` — 5 WOW frame
- `phase-7-demo-production/{spec,tests}.md` — 영상, README, 제출

**Status**: 17 새 파일. 24 KICK 매핑 완료. SDD acceptance criteria 명시. TDD case 1:1 대응.

## v0.2 — review-1 보강 (2026-05-18)

**Driver**: `review-1.md` 발견사항

**Changes**:
- P1 `spec.md` — SEC-003 path containment의 *symlink 추격* 케이스 명시 (canonicalize)
- P2 `spec.md` — FormulaQuery의 `payload field` reference 문법 명확화 (`$score` vs `payload.field`)
- P3 `spec.md` — TurboQuant의 `bits2` enum 값 정확한 Rust syntax 추가 (research/10 over-claim 방지)
- P3 `tests.md` — dual-write feature flag toggle 테스트 추가 (롤백 가능성 검증)
- P4 `spec.md` — late-interaction multivector의 `comparator: MaxSim` 명시 + HNSW disabled 강조
- P5 `spec.md` — `enrich.rs`의 heuristic 5 필드 (intent · entities · outcome · arc · topic) 입력/출력 contract 추가
- P6 `spec.md` — WOW-3 contribution bar의 `transition: flex-basis 320ms cubic-bezier(.22,.61,.36,1)` 모션 명시
- P7 `spec.md` — 영상 1:42-1:58 silence frame의 정확한 frame 수 (60fps × 16s = 960 frames)
- All phases — Cross-phase invariants 6개를 각 spec.md 머리에 명시

**Status**: 9 항목 보강. 8 파일 수정.

## v0.3 — review-2 + review-3 보강 (2026-05-18)

**Driver**: `review-2.md` 추가 검출 + `review-3.md` 최종 확인

**review-2 changes**:
- P2 `tests.md` — `recency exp_decay` formula의 numeric stability case (timestamp = 0 edge)
- P3 `tests.md` — schema_version migration의 backward query (v2 collection 동시 존재 중 v3 query) 테스트
- P4 `tests.md` — RelevanceFeedbackQuery의 anchor 재계산 결정론 (같은 feedback → 같은 결과)
- P5 `tests.md` — Topology insights cache의 mtime 우선순위 (cache hit 시 raw `scan_dir` 호출 0)
- P6 `tests.md` — WOW-2 60fps target의 frame timing budget (16.67ms 이내)
- All phase `spec.md` — "owner" 필드를 acceptance criterion마다 명시 (G1 gate)

**review-3 changes (최종)**:
- `PHASES.md` — Acceptance gate G1-G7 표 추가
- `PHASES.md` — Critical path / parallel 명시
- `PHASES.md` — KICK 미배치 항목 0 확인
- `PHASES.md` — v3.2 SKIP 항목 4개 재확인 (BM42, LLM 등)

**Status**: review-3.md에서 `NO_FURTHER_CHANGES` 마커 명시. 계획 확정.

---

## 변경 통계

| 버전 | 새 파일 | 수정 파일 | 발견 항목 |
|------|---------|-----------|-----------|
| v0.1 | 17 (PHASES + 7×spec + 7×tests + VERSIONS) | 0 | — |
| v0.2 | 1 (review-1.md) | 8 | 9 |
| v0.3 | 2 (review-2.md, review-3.md) | 11 | 11 |
| **합계** | **20** | **19** | **20** |

## 검증 sign-off

| 라운드 | Critic 역할 | 결과 |
|--------|-----------|------|
| review-1 | SDD 완전성 · KICK 미배치 · 자료구조 정확도 | 9 누락 검출 → v0.2 |
| review-2 | TDD 완전성 · edge case · property test · regression coverage | 11 누락 검출 → v0.3 |
| review-3 | 최종 확정 · cross-phase consistency · acceptance gate | **NO_FURTHER_CHANGES** ✓ |
