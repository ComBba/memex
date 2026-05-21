# Session Handoff · 2026-05-18

## §0 두 줄 요약

이번 세션에 Memex (Qdrant 1.18 desktop app) 의 VSD 2026 제출을 위한 **SDD→TDD phase plan v0.3**을 확정했다 — 7 phase × 24 KICK × ~116 acceptance criteria × ~303 test, 3-round self-review에서 `NO_FURTHER_CHANGES` 마커 명시, branch `feature/sota-v3.2-plan-and-mockups`에 commit + push 완료.

**다음 세션 1순위 액션**: `/goal-start` → 전문가 팀을 phase별로 spawn → **P1 Security (KF-01/02/03) 실제 구현** 시작. 단, **`Qdrant 1.18.0` 사용자 머신에 미설치** — 이게 P1-P5 실행의 진짜 ship-blocker. 사용자가 binary 설치하거나 docker로 띄워야 함.

---

## §1 진행한 작업 (시간순)

### Phase A · 코드베이스 + 외부 리서치 (10 expert agents, 1 round)
- 5 codebase analyst (backend / system / frontend / security / perf) → `claudedocs/analysis/01-05.md` (2,226 lines)
- 5 web researcher (Qdrant 공식 docs / 블로그 / VSD 2026 / sparse-ColBERT / Rust ecosystem) → `claudedocs/research/01-05.md` (2,934 lines)
- `claudedocs/INDEX.md` 통합 인덱스 작성

### Phase B · SOTA plan 5단계 진화 (사용자 비판 ↔ 수정 반복)
- v1 consensus mode (10 KICK · 6 만장일치) → user: "echo chamber"
- v2 divergence mode (19 KICK · 5 conflict · split votes 노출) → user: "검증 안 했지?"
- v3 pinnacle-first (2 추가 전문가: Qdrant 1.18 80 features + LLM augmentation patterns · 25 KICK · Cat A-G 7 카테고리)
- v3.1 (3 추가 전문가: Gemma 4 검증 / demo video wow / v3 self-audit · 28 KICK · 8 API-shape error 수정 · KC-01 over-claim 수정)
- v3.2 final (사용자 정량 비교 결정: LLM 0 vs Gemma 4 E1B vs E4B → 옵션 0 채택 · **24 KICK · LLM 완전 제거**)

### Phase C · Visual WOW 5 prototypes
- `claudedocs/prototypes/{index, wow-1, wow-2, wow-3, wow-4, wow-5}.html` — standalone clickable mockup
- WOW-1 Time Machine Heat Trail (CSS 3D + SVG bezier)
- WOW-2 Topology Galaxy (vanilla canvas, 4 cluster + 3 bridge + 1 gap)
- WOW-3 Lens FormulaQuery Contribution Bars (6 slider, flex-basis 320ms transition)
- WOW-4 Predict Cinematic (4×3 thumbnail + View Transition API)
- WOW-5 Discovery 3D Hyperplane Splash (vanilla canvas plane geometry)
- 외부 의존 0, 60fps target

### Phase D · 브랜치 생성 + commit + push
- `git checkout -b feature/sota-v3.2-plan-and-mockups`
- commit `e774b39`: claudedocs/ + CLAUDE.md (29 files +15,261 lines)
- `git push origin feature/sota-v3.2-plan-and-mockups`
- main 대비 0 production code changes (src/, src-tauri/, Cargo.toml 등 0 수정)

### Phase E · SDD→TDD phase plan (3-round self-review)
- `/goal-start` → `/goal` with condition (turn cap 40)
- `claudedocs/phases/PHASES.md` — 7 phase 그룹핑 + G1-G7 acceptance gates + dependency graph + 14-day mapping
- 7 phase × `{spec.md, tests.md}` — SDD AC + TDD tests
- `VERSIONS.md` v0.1 → v0.2 → v0.3 changelog
- `review-1.md` SDD audit (9 findings → v0.2)
- `review-2.md` TDD audit (11 findings → v0.3)
- `review-3.md` cross-cutting audit (**0 findings · NO_FURTHER_CHANGES** ✓)
- commit `e8ddf41` (19 files +3,185 lines) + push

---

## §2 현재 상태

### Git branches

| Branch | Latest commit | vs main | Pushed |
|--------|---------------|---------|--------|
| `main` | `a987952` (5 days ago — docs landing page) | — | ✓ |
| `feature/sota-v3.2-plan-and-mockups` | `e8ddf41` (today — phases plan v0.3) | **+2 commits · +48 files · +18,446 lines** | ✓ origin |

### Live URLs

- Repo: `https://github.com/ComBba/memex`
- Branch on GitHub: `https://github.com/ComBba/memex/tree/feature/sota-v3.2-plan-and-mockups`
- Compare main↔branch: `https://github.com/ComBba/memex/compare/main...feature/sota-v3.2-plan-and-mockups`
- No PR opened yet (브랜치만 push)
- Landing page: `https://sgwannabe.github.io/memex/` (main의 단일 index.html, 핸드오프 시점에 미변경)

### Build / metrics

- production code: **0 변경** (src/, src-tauri/, Cargo.toml, package.json, tauri.conf.json, README.md, docs/)
- artifacts: 47 files in `claudedocs/` (planning + research + prototypes)
- last-known build status: 미검증 (main commit `a987952` 기준은 정상이었지만 이번 세션에 검증 안 함)
- plan coverage: 24/24 KICK mapped to 7 phases, ~116 AC ↔ ~303 tests, 6 SKIP marked

### 환경 상태

| Tool | 버전 | 비고 |
|------|------|------|
| Node | 24.15.0 | ✓ (README 22+ 요건 충족) |
| pnpm | 10.27.0 | ✓ |
| npm | 11.12.1 | ✓ |
| cargo | 1.93.0 | ✓ (README 1.88+ 요건 충족) |
| rustc | 1.93.0 | ✓ |
| python3 | 3.12.4 | ✓ |
| **Qdrant** | **미설치** | ❌ **P1-P5 실행 차단** |
| fastembed cache | 비어있음 | 첫 indexing에 130MB 다운로드 필요 |
| Ollama | 미설치 | OK — v3.2 no-LLM |
| `gh` | 설치됨 (open PR 0) | ✓ |

---

## §3 다음 세션에서 할 수 있는 것

### 즉시 가능 (사용자 추가 입력 없이 진행)

- `claudedocs/phases/PHASES.md` 읽고 phase별 작업 순서 파악
- `claudedocs/phases/phase-1-security/{spec,tests}.md` 읽고 P1 코드 작성 시작 — Rust 단독 작업 (Qdrant 불필요)
  - `validate_session_path` 함수 (`src-tauri/src/indexer/sec.rs` 신규)
  - `SnapshotSandbox` struct
  - `SignedEnvelope` struct (sign/verify)
  - `cargo test --manifest-path src-tauri/Cargo.toml` 로 unit test 실행 가능
- main 대비 PR 작성 (`gh pr create --draft` — push는 끝났음, draft만)
- `claudedocs/prototypes/index.html` Chrome에서 클릭해 5 WOW 동작 재확인

### 사용자 입력 필요 (AskUserQuestion으로 받아야 함)

이번 세션 사용자 요청 "필요한 것은 사전에 모두 askuserquestion을 이용하여 모두 결정"에 따라 다음 결정을 *시작 전*에 받아야 함:

1. **Qdrant 1.18.0 설치 방법**: docker (`docker run -d -p 6333:6333 -p 6334:6334 qdrant/qdrant:v1.18.0`) vs 로컬 binary (`.qdrant/` 다운로드) vs 사용자 직접 처리
2. **구현 진행 순서**: PHASES.md의 P1 → P3 → P4 → P5 → P6 → P7 (critical path) 그대로 vs 사용자 우선순위
3. **PR 전략**: 새 PR 매 phase마다 vs 단일 PR (`feature/sota-v3.2-plan-and-mockups`에 누적)
4. **실제 코드 변경 base**: main commit `a987952` (현 main) vs feature branch에서 시작
5. **데모 영상 production scope**: 코드까지만 vs 영상 녹화 자동화 시도 vs 사용자 직접 (D-7~)
6. **24 KICK 완수 범위**: 모두 (SHIP+COND+TOGGLE) vs SHIP 19개만 vs 사용자 결정 KICK
7. **반복 종료 조건**: 사용자가 "시간/턴 제한 없다"고 명시 — 어디까지 끝까지? 모든 phase의 G3-G7 (build pass · unit pass · integration smoke · regression · doc) 통과? 또는 D-day 도달?
8. **전문가 에이전트 spawn 정책**: 매 phase마다 specialist agent dispatch vs 한 명이 끝까지 vs phase-lead만 specialist 후 worker는 일반
9. **AGENTS.md 처리**: untracked 파일 (사용자가 작성?) — branch에 포함할지

### 검증 가능한 종료 상태 (suggest /goal condition)

다음 세션에서 사용자가 `/goal-start`로 만들 수 있는 condition (참고):

```
P1-P7 모든 phase의 실제 구현 + 검증 완료. 검증:
  cargo build --release 성공
  cargo test 모두 통과 (~303 test 중 unit/integration 통과)
  npm run tauri build 성공 (DMG 생성)
  claudedocs/phases/*/spec.md 의 모든 AC가 코드/test로 충족
  feature/sota-v3.2-plan-and-mockups 브랜치에 모든 변경 commit + push
  PR 생성 (draft 또는 ready), main과 머지 가능 (no conflict)
  최종 보고서 claudedocs/IMPLEMENTATION_REPORT.md 작성
제약:
  README.md / CLAUDE.md / docs/* 0 수정 (v3.2 결정)
  ComBba/memex remote만
  No-LLM at runtime (Ollama 의존 0)
```

---

## §4 할 수 없는 것 (외부 변수)

| 항목 | 이유 |
|------|------|
| **사용자 머신에 Qdrant 설치** | 외부 binary 실행 권한 + 디스크 / 사용자 보안 결정 |
| **DMG clean-machine 테스트** | 별도 macOS VM/머신 필요 (사용자 환경) |
| **Google Form 제출** | 사용자 계정 / 외부 사이트 인증 |
| **YouTube unlisted 업로드** | 사용자 Google 계정 |
| **데모 영상 녹화 + DaVinci 편집** | 60fps 화면 캡처 + post-production은 사용자 손 |
| **Apple Developer 코드 서명** | 사용자 인증서 (v3.2 out-of-scope) |
| **FDA (Full Disk Access) grant** | macOS 시스템 dialog (사용자 클릭) |
| **외부 5명 blind test** (CONFLICT 02 해결) | 실제 사람 |
| **Qdrant Discord에서 API syntax 확인** | 외부 커뮤니티 |

---

## §5 추가로 필요한 것

### 사용자 확인 필요 (next session 시작 시 AskUserQuestion으로)

- [ ] Qdrant 설치 방법 (위 §3 항목 1)
- [ ] 구현 우선순위 + 종료 조건 (§3 항목 2, 6, 7)
- [ ] PR 전략 (§3 항목 3)
- [ ] AGENTS.md 처리 (§3 항목 9)
- [ ] 전문가 spawn 정책 (§3 항목 8)
- [ ] 데모 영상 production scope (§3 항목 5)

### 환경 점검 (사용자 머신)

- [ ] Qdrant 1.18.0 binary 또는 Docker 이미지
- [ ] 인터넷 연결 (fastembed 130MB 첫 다운로드)
- [ ] `~/.claude/projects/` 에 sample sessions (Memex 인덱싱 대상)
- [ ] macOS Full Disk Access 권한 (`Memex.app`에 grant)
- [ ] 디스크 여유 (현재 indexed 0, 80 sessions 가정 시 < 200 MB)

### 알아둘 사실

- v3.2 결정: README 변경 *불필요* (현재 narrative와 plan 100% 일치)
- v3.2 결정: Cat D (LLM index-time) · Cat E (LLM runtime) 모두 SKIP
- KC-01 over-claim 주의: TurboQuant `bits2` ≠ Binary 2-bit (research/10 finding)
- 14-day 일정: D-14 (오늘 시작) → D-0 (2026-06-01 23:59 UTC submit)

---

## §6 다음 세션 시작 프롬프트

복사해서 다음 세션 입력창에 붙여넣기:

```text
/handon

이전 세션 핸드오프: claudedocs/2026-05-18-session-handoff.md

읽고 다음 결정 사항에 답한 뒤 진행하세요:

1. Qdrant 1.18.0 설치 — docker / 로컬 binary / 사용자 직접 (어떤 방식?)
2. 구현 진행 순서 — PHASES.md critical path (P1→P3→P4→P5→P6→P7) 그대로 / 사용자 우선순위?
3. PR 전략 — phase별 새 PR / 단일 누적 PR (feature/sota-v3.2-plan-and-mockups에)
4. 실제 코드 변경 base — main (a987952) / feature branch에서 시작?
5. 24 KICK 완수 범위 — SHIP+COND+TOGGLE 모두 / SHIP 19개만 / 사용자 결정?
6. 반복 종료 조건 — cargo build+test+tauri build 모두 통과까지 / D-day 도달까지?
7. 전문가 spawn 정책 — phase마다 specialist 매번 / 한 명이 끝까지?
8. 데모 영상 production — 코드만 / 영상 자동화 시도 / 사용자 직접 (D-7~)?
9. AGENTS.md (untracked) — branch에 포함?

D-day: 2026-06-01 23:59 UTC (D-14 = 오늘)
Branch: feature/sota-v3.2-plan-and-mockups @ e8ddf41
Goal hint: "P1-P7 실제 구현 + 검증 완료 · 0 production code change to README/CLAUDE.md/docs · ComBba/memex remote만 · no-LLM at runtime"
```

---

## §7 핵심 자산 위치 reference

### Plan 보고서 (HTML, 브라우저용)

| 파일 | 모드 | KICK | 용도 |
|------|------|------|------|
| `claudedocs/sota-plan-v1.html` | consensus | 10 | 안전한 합의 보고 |
| `claudedocs/sota-plan-v2.html` | divergence | 19 | 진짜 panel 토론 결과 |
| `claudedocs/sota-plan-v3.html` | **v3.2 final** | **24** | **확정판 (no-LLM)** |
| `claudedocs/sota-plan-v3.1.html` | (백업) | 28 | Gemma 4 plan (참고) |
| `claudedocs/sota-plan.html` | v2 별칭 | 19 | (기본 링크용) |

### Plan 보고서 (Markdown, phase 실행용)

| 파일 | 내용 |
|------|------|
| `claudedocs/phases/PHASES.md` | 7 phase × 24 KICK · G1-G7 gates · 14-day mapping |
| `claudedocs/phases/VERSIONS.md` | v0.1 → v0.3 changelog |
| `claudedocs/phases/phase-1-security/{spec,tests}.md` | KF-01/02/03 |
| `claudedocs/phases/phase-2-query-api/{spec,tests}.md` | KA-01/02/05, KB-02 |
| `claudedocs/phases/phase-3-schema-evolution/{spec,tests}.md` | KC-01/01b/03/04, KG-03/04 |
| `claudedocs/phases/phase-4-advanced-retrieval/{spec,tests}.md` | KB-01/03/04/05, KA-03/04 |
| `claudedocs/phases/phase-5-perf-enrichment/{spec,tests}.md` | KC-02/05/06, KG-01/02, enrich.rs |
| `claudedocs/phases/phase-6-visual-wow/{spec,tests}.md` | WOW-1..5 통합 |
| `claudedocs/phases/phase-7-demo-production/{spec,tests}.md` | 영상 + form + DMG |
| `claudedocs/phases/review-1.md` | SDD audit (9 findings) |
| `claudedocs/phases/review-2.md` | TDD audit (11 findings) |
| `claudedocs/phases/review-3.md` | cross-cutting · NO_FURTHER_CHANGES ✓ |

### 사전조사 (12편)

| 파일 | 분야 |
|------|------|
| `claudedocs/analysis/01-backend-rust-architecture.md` | 403줄 backend |
| `claudedocs/analysis/02-system-architecture-decisions.md` | 456줄 system |
| `claudedocs/analysis/03-frontend-tauri-integration.md` | 562줄 frontend |
| `claudedocs/analysis/04-security-audit.md` | 250줄 SEC-001..012 |
| `claudedocs/analysis/05-performance-analysis.md` | 555줄 perf |
| `claudedocs/research/01-qdrant-official-docs-2026.md` | 900줄 Qdrant 1.18 docs |
| `claudedocs/research/02-qdrant-official-blog.md` | 507줄 31 articles catalog |
| `claudedocs/research/03-qdrant-vsd-2026-hackathon.md` | 421줄 VSD brief |
| `claudedocs/research/04-sparse-colbert-hybrid.md` | 385줄 BM42/ColBERT |
| `claudedocs/research/05-rust-tauri-fastembed-ecosystem.md` | 721줄 Rust ecosystem |
| `claudedocs/research/06-qdrant-1.18-feature-pinnacle.md` | 686줄 80-feature matrix |
| `claudedocs/research/07-llm-augmentation-no-chat.md` | 537줄 12 patterns |
| `claudedocs/research/08-gemini-gemma-2026-line.md` | 508줄 Gemma 4 verified |
| `claudedocs/research/09-hackathon-demo-video-wow.md` | 539줄 climax-staged |
| `claudedocs/research/10-qdrant-pinnacle-v3-coverage-review.md` | 445줄 v3 self-audit |

### Visual WOW prototypes (브라우저 클릭 가능)

| 파일 | 시각화 |
|------|------|
| `claudedocs/prototypes/index.html` | 5 prototype 진입점 |
| `claudedocs/prototypes/wow-1-time-machine-heat-trail.html` | CSS 3D stack + SVG trails |
| `claudedocs/prototypes/wow-2-topology-galaxy.html` | vanilla canvas 3D galaxy |
| `claudedocs/prototypes/wow-3-lens-contribution-bars.html` | FormulaQuery breakdown sliders |
| `claudedocs/prototypes/wow-4-predict-cinematic.html` | View Transition API |
| `claudedocs/prototypes/wow-5-discovery-splash.html` | 3D hyperplane canvas |

### 기타

- `claudedocs/INDEX.md` — 사전조사 10편 통합 인덱스
- `claudedocs/BRANCH_VS_MAIN.md` — 브랜치 비교 가이드
- `CLAUDE.md` — 이번 세션 작성, 다음 Claude 가이드 (production 코드베이스 이해)

---

## §8 알려진 issue / open question

### Issue

- **Qdrant 미설치** — P1만 Rust 단독 가능, P2 이상은 Qdrant 필수. 첫 번째 사용자 액션.
- **fastembed cache 비어있음** — 첫 `memex scan --index` 시 130MB ONNX 다운로드. 인터넷 필요.
- **KC-01 TurboQuant `bits2` syntax** — research/10이 over-claim risk로 지목. qdrant-client 1.18 정확한 `TurboQuantizationBuilder` 호출 syntax는 D-13 spike PR에서 검증 필요.
- **KA-04 RelevanceFeedbackQuery 1.17 GA 여부** — research/06 cite 재확인 필요 (D-9 작업 전).
- **KB-01 nDCG eval gate** — D-8까지 +15% 미달 시 revert 정해야 함 (Marcus dissent 조건).

### Open question (사용자 결정 영역)

- AGENTS.md 정체 (untracked) — 사용자가 작성? branch에 포함할지?
- `docs/qdrant-ultimate-advancement-plan.html` (untracked) — 마찬가지 처리 결정 필요
- 사용자가 "전문가 팀을 기능별로 스폰" — phase별 lead 1명 + worker spawn? 또는 phase 전체를 single agent?
- 사용자가 "시간/턴 제한 없음" — 끝까지 = 무한 토큰 가능? 비용 cap?
- D-3 schema lock 이후 사용자가 v3.2를 변경하길 원하면? (현재 plan은 v0.3에 lock)

### Risk (v3.2 §7 Risk Register 참조)

| ID | Sev | Risk |
|----|-----|------|
| R1 | HIGH | SEC-003 path containment 미수정 시 임의 파일 노출 |
| R2 | HIGH | demo 영상 미녹화 → 자동 DQ |
| R3 | MED | FormulaQuery 문법 학습 곡선 |
| R4 | MED | KB-01 late-interaction nDCG 무효 |
| R5 | MED | TurboQuant 정확도 손실 (KC-01b 페어링 필수) |
| R6 | MED | 60fps motion 미달 |
| R7 | LOW | Qdrant 1.18 → 1.19 snapshot 호환성 |

---

## Sign-off

- 이번 세션 시작: 2026-05-18 (D-14)
- 핸드오프 작성: 2026-05-18
- Branch state: `feature/sota-v3.2-plan-and-mockups` @ `e8ddf41` pushed
- Next deadline: **D-0 = 2026-06-01 23:59 UTC** (14 일 남음)
- Auto-clear of /goal: 이전 goal은 NO_FURTHER_CHANGES 조건 충족으로 자동 정리됨
