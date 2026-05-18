# Review Round 2 — TDD Completeness + Edge Cases

**Reviewer role**: TDD specialist (independent critic, distinct from R1)
**Scope**: v0.2 draft (R1 보강 반영 후)
**Date**: 2026-05-18
**Outcome**: 11 findings → 보강 명령. v0.3 작성으로 이동.

---

## 1. Methodology

Each phase's `tests.md` audited against:
- **A. Unit test completeness** for every AC
- **B. Property tests** for invariants
- **C. Integration test** roundtrip coverage
- **D. Regression test** for unchanged surfaces
- **E. Edge case** (numeric stability, empty inputs, race conditions)
- **F. Performance test** existence for SLA-bound features
- **G. Test → AC mapping** completeness

---

## 2. Findings

### F2.1 · P2 tests missing numeric stability for exp_decay

**File**: `phase-2-query-api/tests.md` §1
**Issue**: `t_lens_recency_30day_decay` (now-30d) 있음. 하지만 **timestamp = 0 (epoch) edge** 누락 — exp_decay 수식에서 huge negative exponent → overflow 위험.
**Severity**: Medium
**Action**: Edge test 추가.
**v0.3 변경**: §1에 `t_lens_recency_negative_ts_edge` 추가 (start_ts=0 → finite positive recency_factor, no NaN/Inf).

### F2.2 · P3 tests dual-read fallback로 backward query 부재

**File**: `phase-3-schema-evolution/tests.md` §4
**Issue**: dual-read v3 hit / v3 miss → v2 hit 테스트 있음. 하지만 **v3 query 중 v2 데이터 동시 접근 (혼합)** 테스트 없음.
**Severity**: Medium (migration 도중 race)
**Action**: backward query test 추가.
**v0.3 변경**: §4에 `t_dual_read_v3_partial_v2_complete` (v3에 일부만 migrated, 나머지는 v2) test 추가.

### F2.3 · P4 tests RelevanceFeedback 결정론 검증 부족

**File**: `phase-4-advanced-retrieval/tests.md` §6
**Issue**: `t_relevance_feedback_determinism` 있음. 하지만 *seed* 통제 없음 — Qdrant 내부 ranking에 random tiebreak이 있으면 비결정적.
**Severity**: Medium (Replay reproducibility)
**Action**: 명시적 seed + determinism strong assertion.
**v0.3 변경**: §6에 `t_relevance_feedback_strict_determinism` (5 runs all identical) 추가.

### F2.4 · P5 tests Topology cache invalidation 미세 케이스

**File**: `phase-5-perf-enrichment/tests.md` §5
**Issue**: `t_cache_hit_skips_scan`, `t_cache_miss_on_mtime_change` 있음. 하지만 **mtime 동일하지만 파일 내용 변경된 경우** (사용자가 vi로 빠른 수정) — Aisha critic의 dissent 항목.
**Severity**: Medium
**Action**: 추가 edge.
**v0.3 변경**: §5에 `t_cache_priority_mtime_only` (mtime이 cache key, content는 무관 — 의도된 동작이라는 명시) + 사용자 경고 노트.

### F2.5 · P6 tests 60fps target의 frame timing budget 부재

**File**: `phase-6-visual-wow/tests.md` §6
**Issue**: `perf_topology_60fps_at_n10k` 있음. 하지만 측정 단위가 fps avg. **개별 frame이 16.67ms 초과 (frame drop)** 검출 없음.
**Severity**: Medium (UI smoothness)
**Action**: per-frame budget assertion.
**v0.3 변경**: §6 perf 테스트에 "no frame exceeds 16.67ms" assertion 추가. Playwright/devtools performance API 사용.

### F2.6 · P7 tests README diff 검증이 too loose

**File**: `phase-7-demo-production/tests.md` §3
**Issue**: `t_readme_hero_unchanged` grep으로 line 존재 확인. 하지만 hero **순서**나 **중간 삽입** 검출 안 됨.
**Severity**: Low
**Action**: line number + hash 검증.
**v0.3 변경**: §3에 `t_readme_byte_for_byte_unchanged_except_status` (whole-file sha256 비교 except Status 섹션) 추가.

### F2.7 · 모든 phase spec.md acceptance criterion에 owner 누락

**Files**: 모든 phase/spec.md (P1, P5는 owner 있음, 일부 누락)
**Issue**: 일부 AC에 `(owner: ...)` 명시 없음. PHASES.md §6의 G1 gate 위반.
**Severity**: Medium
**Action**: 빠진 owner 채움.
**v0.3 변경**: 모든 phase/spec.md의 100% AC에 owner 명시 — A1 Eunha, A2 Marcus, A3 Liana, A4 Daichi, A5 Sara, B1 Hiroshi, B2 Aisha, B3 Felix, B4 Priya, B5 Min-jae 10명 중 1명 또는 다수.

### F2.8 · Property test "no panic on empty corpus" 부재 (P2)

**File**: `phase-2-query-api/tests.md` §5
**Issue**: `prop_no_panic_on_empty_results` 있음. 하지만 **corpus 자체가 empty** (0 points) → search 시 비정상 종료 가능.
**Severity**: Medium
**Action**: Empty corpus property 추가.
**v0.3 변경**: §5에 `prop_no_panic_empty_collection` (corpus 0 points → Ok(vec![]) not Err) 추가.

### F2.9 · P4 tests Discovery context pair commutative 모호

**File**: `phase-4-advanced-retrieval/tests.md` §7
**Issue**: `prop_discover_pairs_commutative` statement: "(a,b) pair는 (b,a) pair와 다른 결과 (negative direction matters)". 그런데 spec의 (positive, negative) 순서 swap 결과를 정확히 명시 안 함.
**Severity**: Low (semantic clarity)
**Action**: Property를 더 sharp하게.
**v0.3 변경**: §7 property를 `prop_discover_pairs_swap_changes_ranking` (구체적: swap 시 top-1이 변경됨)로.

### F2.10 · P5 enrich.rs test에 i18n / 비-영어 input 부재

**File**: `phase-5-perf-enrichment/tests.md` §1
**Issue**: `t_outcome_resolved_from_substring` 등은 영어 substring만. Memex corpus에는 한국어 세션도 있을 수 있음 ("해결됨", "고쳤어").
**Severity**: Low (실제 Memex corpus의 영어 비율 높을 가능성)
**Action**: 결정 — i18n 지원할지 (현 spec에 없음) or 영어 우선 명시.
**v0.3 변경**: §1에 `t_outcome_non_english_partial` (비영어 input → "partial" fallback) 추가. Spec에 "영어 substring 우선, 다국어는 post-MVP" 명시.

### F2.11 · 모든 phase tests.md에서 "코드 변경 없이 통과 가능한 test" 누락

**Files**: 모든 phase/tests.md
**Issue**: Test가 모두 *코드 변경 후* 통과. **선작성 원칙**: TDD는 test 작성 → run → fail → 코드 작성 → run → pass. fail step에서 fail message가 명확한지 검증 없음.
**Severity**: Medium (TDD 원칙)
**Action**: 각 phase에 "1st-fail expected message" 명시.
**v0.3 변경**: 각 tests.md 머리에 "Test-first protocol: 새 test는 (a) red state로 commit, (b) 코드 추가로 green, (c) refactor. Step (a)에서 'function X not found' 같은 명확한 fail message 기대" 1 paragraph 추가.

---

## 3. Summary table

| Finding | File | Severity | Status in v0.3 |
|---------|------|----------|----------------|
| F2.1 exp_decay numeric stability | P2 tests §1 | Medium | ✅ edge test |
| F2.2 v3 partial migration mid-query | P3 tests §4 | Medium | ✅ test added |
| F2.3 RelevanceFB strict determinism | P4 tests §6 | Medium | ✅ 5-run test |
| F2.4 cache mtime-only priority | P5 tests §5 | Medium | ✅ priority test + note |
| F2.5 frame timing budget | P6 tests §6 | Medium | ✅ per-frame assertion |
| F2.6 README diff strictness | P7 tests §3 | Low | ✅ sha256 compare |
| F2.7 AC owner missing | all spec.md | Medium | ✅ 100% owners |
| F2.8 empty corpus property | P2 tests §5 | Medium | ✅ test added |
| F2.9 Discovery swap property | P4 tests §7 | Low | ✅ sharper statement |
| F2.10 i18n outcome | P5 tests §1 | Low | ✅ non-eng fallback test |
| F2.11 test-first protocol | all tests.md | Medium | ✅ red-green-refactor noted |

---

## 4. Verdict

- ❌ v0.2 still not plan-complete
- ✅ 11 findings all actionable
- 🔄 → v0.3 보강 후 review-3 dispatch (최종)
