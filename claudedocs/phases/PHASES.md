# Memex × Qdrant 1.18 — Phase Plan (top-level)

**Base**: `claudedocs/sota-plan-v3.html` (v3.2 final · 24 KICK · no-LLM)
**Branch**: `feature/sota-v3.2-plan-and-mockups`
**Methodology**: SDD (Spec-Driven Design) → TDD (Test-Driven Development)
**Deadline**: 2026-06-01 (VSD 2026 submission)
**Version**: v0.3 (see `VERSIONS.md`)

---

## Phase ↔ KICK mapping (v3.2의 24 KICK → 7 phase)

| Phase | KICKs | Why this grouping | Dep |
|-------|-------|-------------------|-----|
| **P1 Security Hardening** | KF-01, KF-02, KF-03 | Ship-blocker. critic Aisha hold권. demo machine 외부 노출 전 fix 필수. | none |
| **P2 Query API Core** | KA-01, KA-02, KA-05, KB-02 | Demo wow의 핵심 frame 1:10 (before/after split) · 1:22 (contribution bars). FormulaQuery 한 줄이 5 round-trip을 1로 압축. | P3 schema |
| **P3 Schema Evolution** | KC-01, KC-01b, KC-03, KC-04, KG-03, KG-04 | 새 collection `memex_sessions_v3`. dual-write로 무중단. TurboQuant + tenant idx는 모든 후속 phase의 토대. | P1 |
| **P4 Advanced Retrieval** | KB-01, KB-03, KB-04, KB-05, KA-03, KA-04 | Late-interaction + Discovery pairs + ACORN + group-by + RelevanceFB. demo frame 1:32 · 2:08 · 2:22. | P2, P3 |
| **P5 Performance & Enrichment** | KC-02, KC-05, KC-06, KG-01, KG-02, enrich.rs (heuristic) | 60fps · cold-scan 가속 · OOM 방어 · 캐시 · LLM-free payload 채움. | P3 |
| **P6 Visual WOW Integration** | WOW-1, WOW-2, WOW-3, WOW-4, WOW-5 | 5 prototype을 실제 surface에 통합. heat trail · gateway · contribution bars · cinematic · hyperplane. | P1-P5 backend ready |
| **P7 Demo Production** | (영상 · README footer · DMG · 제출) | 3-min climax-staged 영상 (12s silence at 1:42). Google Form. clean-machine test. | P6 |

**KICK 미배치**: 없음. 모든 24 KICK이 P1-P5에 1:1 매핑.

---

## Cross-phase invariants (전 phase에 강제 적용)

1. **No-LLM at runtime** — v3.2 결정. heuristic enrichment만.
2. **100% local · zero outbound** — fastembed cache 다운로드(첫 launch) 외엔 network 0.
3. **uuid_v5(session_id) idempotent** — 모든 phase에서 point id 안정.
4. **schema_version payload index** — P3에서 도입되면 모든 후속 query/upsert에 명시.
5. **Tauri IPC contract `Result<T, String>`** — commands.rs 시그니처 유지.
6. **git remote = `https://github.com/ComBba/memex`** — CLAUDE.md absolute rule.

---

## Dependency graph

```
P1 Security ──┐
              ├─→ P3 Schema ──┐
P2 Query API ─┘               ├─→ P4 Advanced Retrieval ─┐
                              │                           ├─→ P6 Visual WOW ─→ P7 Demo
                              └─→ P5 Perf & Enrichment ──┘
```

- **Critical path**: P1 → P3 → P4 → P6 → P7 (5 phase serial)
- **P2** parallel to P3 (서로 다른 collection-level vs query-level 변경)
- **P5** parallel to P4 (대부분 별도 모듈)

---

## SDD → TDD per phase

각 phase는 두 문서를 가진다:

- **`spec.md`** (SDD: Spec-Driven Design)
  - 행동 계약 (behavior contract)
  - API/UI 시그니처 (Rust function · Tauri command · JS handler)
  - 자료 구조 (Qdrant schema · payload field · vector config)
  - Edge case 명세 (failure mode · graceful degrade)
  - Acceptance criteria (이 phase가 "끝났다"의 정의)

- **`tests.md`** (TDD: Test-Driven Development)
  - Unit tests (개별 함수 · 결정론 · property)
  - Integration tests (Tauri IPC end-to-end · Qdrant round-trip)
  - Regression tests (기존 동작 보호 · main 호환)
  - 각 test case는 spec.md의 acceptance criterion에 1:1 mapping
  - **선작성 원칙**: tests.md의 모든 case를 spec 확정 직후 작성, 구현 전에 실패해야 함

---

## Acceptance gate (모든 phase 공통)

각 phase는 다음 gate를 모두 통과해야 다음 phase 진입:

| Gate | Check |
|------|-------|
| G1 Spec lock | `spec.md`의 모든 acceptance criterion에 "owner" 명시 |
| G2 Tests preceding | `tests.md`의 모든 case가 작성된 후에만 코드 작성 |
| G3 Local build | `cargo build --release --manifest-path src-tauri/Cargo.toml` 성공 |
| G4 Unit pass | `cargo test --manifest-path src-tauri/Cargo.toml` (해당 phase 추가분) 통과 |
| G5 Integration smoke | 해당 phase의 새 surface가 `npm run tauri dev`에서 동작 확인 |
| G6 Regression | 기존 7 surface가 break 없음 (수동 smoke check list) |
| G7 Doc update | spec.md · tests.md commit (코드 commit과 분리) |

---

## v3.2 KICK 안에 SKIP된 항목 (재확인)

v3.2가 명시적으로 제외한 것 — phase plan에도 포함 안 함:

- ❌ KD-01 (Pattern #6 LLM) · KD-02 (cluster LLM) — LLM 제거 결정
- ❌ KE-01 HyDE · KE-02 MIPS rewrite — runtime LLM toggle 보류
- ❌ BM42 sparse (still experimental in research/06)
- ❌ 공격적 strict-mode without rescore (KC-01b가 대체)

heuristic enrichment (P5의 `enrich.rs`)가 LLM-시리즈를 대체.

---

## 진행 표시

| Phase | Spec | Tests | Implementation | Validation |
|-------|------|-------|----------------|------------|
| P1 | ✅ v0.3 | ✅ v0.3 | ⏸ pending (사용자 작업 영역) | — |
| P2 | ✅ v0.3 | ✅ v0.3 | ⏸ pending | — |
| P3 | ✅ v0.3 | ✅ v0.3 | ⏸ pending | — |
| P4 | ✅ v0.3 | ✅ v0.3 | ⏸ pending | — |
| P5 | ✅ v0.3 | ✅ v0.3 | ⏸ pending | — |
| P6 | ✅ v0.3 | ✅ v0.3 | ⏸ pending | — |
| P7 | ✅ v0.3 | ✅ v0.3 | ⏸ pending | — |

> ⏸ = phase plan은 확정. 실제 코드 구현은 사용자 시간/세션 관리 영역 (D-14 → D-0).

---

## 14-day 매핑 (v3.2 §6 Roadmap과의 일치)

| Day | Phase 작업 |
|-----|------------|
| D-14 | P1 시작 + 완료 (SEC fix) |
| D-13 | P2 시작 (FormulaQuery spike) + P3 시작 (TurboQuant) |
| D-12 | P2 merge, P3 진행 |
| D-11 | P3 완료 + P5 enrich.rs 구현 |
| D-10 | P4 시작 (KA-03/KB-05/KC-05) |
| D-9 | P4 진행 (KB-01 A/B + KA-04) |
| D-8 | P4 완료 + P6 시작 (WOW-1/3) |
| D-7 | P6 진행 (WOW-2/4) · P7 영상 draft 1 |
| D-6 | P6 완료 (WOW-5 · cinematic) |
| D-5 | P7 진행 (KF-03 · landing) |
| D-4 | FEATURE FREEZE · 회귀 |
| D-3 | schema lock · bugfix |
| D-2 | P7 (Google Form draft · DMG test) |
| D-1 | dry-run · 영상 final |
| D-0 | SUBMIT |
