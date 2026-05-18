# Review Round 1 — SDD Completeness Audit

**Reviewer role**: SDD specialist (independent critic, distinct from authors)
**Scope**: v0.1 draft (PHASES.md + 7×spec.md + 7×tests.md)
**Date**: 2026-05-18
**Outcome**: 9 findings → 보강 명령. v0.2 작성으로 이동.

---

## 1. Methodology

Each phase's `spec.md` audited against these axes:
- **A. 행동 계약 (behavior contract)** completeness
- **B. API/UI 시그니처** 정확성 (Rust syntax, JSON schema)
- **C. 자료구조 (data structure)** field 누락
- **D. Edge case** coverage
- **E. Acceptance criteria** owner 명시
- **F. Cross-phase consistency** (invariants, dependency)

---

## 2. Findings

### F1.1 · P1 spec.md missing symlink-chase case

**File**: `phase-1-security/spec.md` §1.4
**Issue**: Edge cases table에 internal symlink만 명시. **chained symlink** (link → link → /tmp/foo) 부재.
**Severity**: Medium (security boundary 약화 가능)
**Action**: Edge case 추가 + canonicalize 의 chain-following 명시.
**v0.2 변경**: §1.4 edge cases에 `chained-symlink` row 추가; canonicalize 호출은 std::fs::canonicalize (chain follow OK) 명시.

### F1.2 · P2 spec.md FormulaQuery `$score.<name>` 문법 모호

**File**: `phase-2-query-api/spec.md` §1.3
**Issue**: `{ "var": "$score.content" }` syntax이 Qdrant 1.14 doc과 정확히 일치하는지 cite 없음. payload field 참조도 `payload.<field>` 와 `payload.<field>.value` 차이 불명확.
**Severity**: High (구현자가 잘못된 syntax 작성 위험)
**Action**: research/06 cite 추가 + 정확한 Qdrant 1.14 spec 인용.
**v0.2 변경**: §1.3에 `[Source: qdrant.tech/documentation/concepts/hybrid-queries/#formula]` 추가, `$score.<vector_name>`은 prefetch identifier, `payload.<field>`는 raw value 참조 명시.

### F1.3 · P3 spec.md TurboQuant Bits2 enum syntax 모호

**File**: `phase-3-schema-evolution/spec.md` §2
**Issue**: `bits: Bits2` (Rust enum) vs `"bits": "Bits2"` (JSON) vs `"bits": 2` (raw int) — 어느 것이 정확한지 명시 없음. research/10이 over-claim risk로 지목한 정확한 지점.
**Severity**: Critical (구현 시 잘못된 builder 호출 → 1.15 binary quant으로 fallback)
**Action**: Rust client 1.18 `TurboQuantizationBuilder` 정확한 syntax + JSON 등가 표기.
**v0.2 변경**: §2.1에 `qdrant_client::qdrant::quantization_config::Quantization::Turbo(...)` 정확한 Rust enum path + `Bits2` proto enum 값 (= 2) 명시.

### F1.4 · P3 tests.md dual-write feature flag toggle 부재

**File**: `phase-3-schema-evolution/tests.md` §4
**Issue**: dual-read 테스트는 있으나 **dual-write feature flag를 off → on → off** 토글 시나리오 부재. 롤백 가능성 검증 부족.
**Severity**: Medium (D-3 schema lock 이전 안전 롤백 path 미검증)
**Action**: Toggle test 추가.
**v0.2 변경**: §4에 `t_dual_write_flag_off_only_v2`, `t_dual_write_flag_on_both`, `t_dual_write_flag_toggle_safe` 3 test 추가.

### F1.5 · P4 spec.md content_late multivector 설정 누락

**File**: `phase-4-advanced-retrieval/spec.md` §1
**Issue**: KB-01의 `comparator: MaxSim` 옵션 미명시. `multivector_config` 설정 누락. P3 schema와 협력 부분 불분명.
**Severity**: High (구현자가 잘못된 multivector type 사용)
**Action**: P3 schema 인용 + comparator 명시 + hnsw_config m=0 강조.
**v0.2 변경**: §1.4 acceptance criteria에 multivector_config.comparator=MaxSim, hnsw_config.m=0 명시. P3 §1.2 collection schema에서 이 두 필드 추가.

### F1.6 · P5 spec.md enrich.rs 입력/출력 contract 불완전

**File**: `phase-5-perf-enrichment/spec.md` §1
**Issue**: enrich 함수의 5 필드 로직은 명시. 하지만 **각 함수의 input/output type 정확한 signature** 누락. 빈 turns, ai_title 없는 케이스 등 edge case 미명시.
**Severity**: Medium (heuristic 품질 측정 못 함)
**Action**: signature + edge case 추가.
**v0.2 변경**: §1.2에 `classify_intent(&HashMap<String, u64>) -> String` 등 5 함수 signature 명시. 빈 input · 모든 turn assistant 등 edge case.

### F1.7 · P6 spec.md motion timing 단순

**File**: `phase-6-visual-wow/spec.md` §3
**Issue**: WOW-3 contribution bar `transition: flex-basis 320ms` 만 적힘. **easing curve** (`cubic-bezier(.22,.61,.36,1)`) 누락 — 다른 phase의 motion과 일관성 부족.
**Severity**: Low (UX inconsistency)
**Action**: cubic-bezier 통일.
**v0.2 변경**: §3.1에 `cubic-bezier(.22,.61,.36,1)` 명시. 전체 phase에 motion token system: 240ms (small), 320ms (medium), 420ms (large), 모두 동일 cubic-bezier.

### F1.8 · P7 spec.md 1:42 silence frame 정확한 frame 수 부재

**File**: `phase-7-demo-production/spec.md` §1
**Issue**: "12 full seconds" 만 명시. 60fps capture × 16s = 960 frames의 정확한 수 계산 없음 → editor가 frame-perfect alignment 검증 못 함.
**Severity**: Medium (영상 production 정확도)
**Action**: frame 단위 명시.
**v0.2 변경**: §1.1 acceptance criteria + AC-7.1.2에 "1:42-1:58 = exactly 960 frames at 60fps (or 384 frames at 24fps post-export)" 명시.

### F1.9 · Cross-phase invariants 6개가 각 spec.md 머리에 없음

**Files**: 모든 phase/spec.md
**Issue**: PHASES.md에는 invariant 6개 명시. 각 phase/spec.md에 "Cross-phase invariants 적용"만 한 줄. **어떤 invariant가 이 phase에서 특히 중요한지** 명시 부족.
**Severity**: Medium (구현 중 invariant 위반 위험)
**Action**: 각 spec.md 머리에 invariant 6개 + 강조 1-2개.
**v0.2 변경**: 모든 phase/spec.md head section에 "Cross-phase invariants: 6 모두 적용 + 강조 N개" 명시.

---

## 3. Summary table

| Finding | File | Severity | Status in v0.2 |
|---------|------|----------|----------------|
| F1.1 chained symlink | P1 spec §1.4 | Medium | ✅ added |
| F1.2 FormulaQuery syntax | P2 spec §1.3 | High | ✅ cite added |
| F1.3 TurboQuant Bits2 | P3 spec §2 | Critical | ✅ Rust syntax explicit |
| F1.4 dual-write toggle | P3 tests §4 | Medium | ✅ 3 tests added |
| F1.5 multivector config | P4 spec §1 + P3 schema | High | ✅ comparator/m=0 명시 |
| F1.6 enrich.rs contract | P5 spec §1 | Medium | ✅ signatures + edge cases |
| F1.7 motion easing | P6 spec §3 | Low | ✅ cubic-bezier 통일 |
| F1.8 frame count | P7 spec §1 | Medium | ✅ 960 frames |
| F1.9 cross-phase invariants | all spec.md head | Medium | ✅ head sections |

---

## 4. Verdict

- ❌ v0.1 not yet plan-complete
- ✅ 9 findings all actionable
- 🔄 → v0.2 보강 후 review-2 dispatch
