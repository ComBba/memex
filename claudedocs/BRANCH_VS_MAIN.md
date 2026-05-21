# Branch vs Main — `feature/sota-v3.2-plan-and-mockups`

**Branch**: `feature/sota-v3.2-plan-and-mockups`
**Parent**: `main` (commit `a987952` — `docs: public landing page + README — add Predict + landing link`)
**Author**: 2026-05-18
**Remote**: `https://github.com/ComBba/memex` (CLAUDE.md absolute rule)

## 비교 방법

### A. GitHub PR (push 후, draft 권장)
```bash
git push origin feature/sota-v3.2-plan-and-mockups
gh pr create --base main --head feature/sota-v3.2-plan-and-mockups --draft \
  --title "Plan v3.2 + Visual WOW prototypes (no merge target)" \
  --body "Plan-only branch. v3.2 보고서 + 5 visual prototypes. main 빌드/실행 영향 0."
```
PR URL은 만들어진 후 명시. Draft 상태라 머지 압박 없음.

### B. GitHub compare URL (push만, PR 없음)
```bash
git push origin feature/sota-v3.2-plan-and-mockups
open "https://github.com/ComBba/memex/compare/main...feature/sota-v3.2-plan-and-mockups"
```

### C. 로컬 only (push 불필요)
```bash
git diff main..feature/sota-v3.2-plan-and-mockups --stat
git log main..feature/sota-v3.2-plan-and-mockups --oneline
gitk main feature/sota-v3.2-plan-and-mockups   # GUI
```

## 본 브랜치가 main에 더하는 것

```
claudedocs/
├── INDEX.md                          # 10편 사전조사 인덱스
├── BRANCH_VS_MAIN.md                 # 이 파일
├── sota-plan.html                    # = v2 (별칭)
├── sota-plan-v1.html                 # consensus mode (10 KICK)
├── sota-plan-v2.html                 # divergence mode (19 KICK)
├── sota-plan-v3.html                 # = v3.2 final (24 KICK, no-LLM)
├── sota-plan-v3.1.html               # backup (28 KICK, Gemma 4)
├── analysis/
│   ├── 01-backend-rust-architecture.md
│   ├── 02-system-architecture-decisions.md
│   ├── 03-frontend-tauri-integration.md
│   ├── 04-security-audit.md
│   └── 05-performance-analysis.md
├── research/
│   ├── 01-qdrant-official-docs-2026.md
│   ├── 02-qdrant-official-blog.md
│   ├── 03-qdrant-vsd-2026-hackathon.md
│   ├── 04-sparse-colbert-hybrid.md
│   ├── 05-rust-tauri-fastembed-ecosystem.md
│   ├── 06-qdrant-1.18-feature-pinnacle.md
│   ├── 07-llm-augmentation-no-chat.md
│   ├── 08-gemini-gemma-2026-line.md
│   ├── 09-hackathon-demo-video-wow.md
│   └── 10-qdrant-pinnacle-v3-coverage-review.md
└── prototypes/                       # 본 브랜치에서 신규
    ├── index.html                    # 5개 prototype 진입점
    ├── wow-1-time-machine-heat-trail.html
    ├── wow-2-topology-galaxy.html
    ├── wow-3-lens-contribution-bars.html
    ├── wow-4-predict-cinematic.html
    └── wow-5-discovery-splash.html

CLAUDE.md                             # 어제 작성된 코드베이스 가이드
```

## 본 브랜치가 main에서 **건드리지 않은** 것

- `src/` (프론트엔드 소스) — 0 변경
- `src-tauri/` (Rust 소스) — 0 변경
- `Cargo.toml`, `package.json`, `tauri.conf.json` — 0 변경
- `README.md` — 0 변경 (v3.2 결정: narrative 변경 불필요)
- `docs/architecture.md`, `docs/qdrant-features.md` — 0 변경
- 어떤 production 코드도 0 변경

→ **main에서 fast-forward 가능, conflict 위험 0, 빌드 영향 0**.

## prototypes 동작 검증

각 prototype은 standalone HTML. 외부 의존 0 (CDN 없음, vanilla canvas/CSS).

| Prototype | 어떻게 열어보나 | 검증 포인트 |
|-----------|----------------|------------|
| index.html | `open claudedocs/prototypes/index.html` | 5개 링크 카드 모두 가시 |
| WOW-1 | 클릭 후 카드 hover | 가장 가까운 5 세션에 SVG bezier trail · 60fps · ↑↓ scrub |
| WOW-2 | 드래그로 회전, 휠로 zoom | 4 cluster · 3 bridge edge (pulse) · 1 gap insight · 60fps |
| WOW-3 | 6 slider 조작 | bar가 320ms transition으로 재구성 · ranking 변경 · FormulaQuery breakdown chip |
| WOW-4 | thumbnail 클릭 | View Transition API로 cinematic zoom (Chromium) · fallback fade (Safari) |
| WOW-5 | 캔버스 드래그 + regenerate 버튼 | 3D hyperplane + positive(green)/negative(red)/emerged(cyan) · plane normal 화살표 |

## v3.2 plan과 prototypes 매핑

각 prototype은 v3.2 KICK의 **시각 검증용**. 실제 구현(Rust + Qdrant 연결)은 main에 incremental PR로 들어갈 예정 (사용자 시간/세션 관리).

| Prototype | 검증하는 KICK | 실제 main 구현 예상 위치 |
|-----------|--------------|------------------------|
| WOW-1 | KG-01 Topology insights cache · 기존 Time Machine + heuristic chip | `src/main.js` (기존) · `src-tauri/src/indexer/enrich.rs` (NEW) |
| WOW-2 | 기존 Topology + KD-02-free 자동 라벨 (heuristic) | `src/main.js` 기존 force-graph + enrich.rs |
| WOW-3 | **KA-01 FormulaQuery + KA-05 weighted RRF + KB-02 BM25 sparse** | `src-tauri/src/indexer/search.rs` (NEW) · `src/main.js` lens slider |
| WOW-4 | KG-02 Predict LRU + view-transition | `src-tauri/src/indexer/predict.rs` (NEW) · `src/main.js` 패널 |
| WOW-5 | **KB-03 Discovery true context pairs** | `src-tauri/src/indexer/discover.rs` (NEW) · `src/main.js` Mix&Match |

## main과의 diff 통계 (commit 후)

```bash
# 본 브랜치 commit 후 실행:
git diff main..feature/sota-v3.2-plan-and-mockups --stat
git diff main..feature/sota-v3.2-plan-and-mockups --shortstat
```

예상 결과: 0 changed (production), ~25 new files (claudedocs/), 0 deleted.

## 다음 단계 (사용자 결정 대기)

1. **이 브랜치 그대로 두기** — 로컬에서 검토, push 없이
2. **GitHub에 push** — `git push origin feature/sota-v3.2-plan-and-mockups` (사용자 명시 승인 후)
3. **Draft PR 열기** — push 후 `gh pr create --draft`
4. **main에 실제 코드 변경 시작** — v3.2 KICK 하나씩 incremental PR로 (별도 작업)

git push는 사용자 명시 승인 받기 전엔 실행하지 않음 (CLAUDE.md absolute rule).
