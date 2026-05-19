# Full-sync: ComBba/memex (fork) → sgwannabe/memex (upstream)

**Branch**: `ComBba:upstream-pr/full-sync` @ `58aa474`
**Base**: `sgwannabe/memex:main` @ `4973a91`
**Scope**: 37 commits (36 fork-only + 1 integration merge), 양쪽 모든 기능 보존

## TL;DR

ComBba/memex fork의 **모든 변경 사항**을 upstream에 반영하는 단일 통합 PR입니다. fork 36 commits + upstream의 자체 변경 14 commits을 **union semantics**로 병합했습니다. 결과적으로 이 PR이 merge되면 양쪽 모든 기능 (fork의 P1-P8 + WOW UI, upstream의 MCP/watcher/notifications/dashboard)이 한 트리에 공존합니다.

해커톤 팀 협업이라 단방향 cherry-pick이 아니라 **양쪽 작업을 합치는 통합 머지**가 됩니다.

## Why this PR

지난 13일 동안 ComBba fork에서 진행한 P1-P8 phases + UX 안정화 + 시각 WOW surfaces 작업을 upstream에 완전히 전달합니다. 동시에 upstream이 진행한 MCP integration / 자동 인덱싱 / Time Machine rail 작업도 fork 트리에 보존됩니다.

## What's in this PR — 37 commits 카테고리별

### Phase Features (P1-P8 + reviews)

| Commit | Phase | Description |
|---|---|---|
| `f55d417` | **P1 Security** | KF-01 path sandbox + KF-02 snapshot sandbox + KF-03 signed envelope + KH-01 multi-agent enum |
| `5aa5e24` | **P3 Schema** | `memex_sessions_v3` collection + TurboQuant bits2 + dual-write + `source_agent` payload |
| `5a9719d` | **P4 Retrieval** | late-interaction reranker + Discovery pairs + ACORN + order-by + group-by + RelevanceFeedback |
| `2691006` | **P5 Perf/Enrich** | KC-02/05/06 perf opts + KG-01/02 caches + `enrich.rs` + KH-01 `codex_parser` |
| `7cbc588` | **P2 Query API** | FormulaQuery + MMR + weighted RRF + BM25 sparse |
| `138da86` | **P6 WOW** | 5 visual WOW surfaces wired into production frontend (heat-trail, mix-modal, prediction grid 등) |
| `540281a` | **P7 Demo** | video script + ffmpeg helper + IMPLEMENTATION_REPORT |
| `70121fb`, `b4b205a`, `113665f` | **P8 E2E** | empirical validation across 7 surfaces + memex:// deep-links + private artifact redaction |

### Review feedback iterations

| Commit | Description |
|---|---|
| `769af65` | Apply all valid Gemini / Codex / CodeRabbit feedback (Round 1) |
| `abdf68d` | Apply 3 previously deferred items — payload module + batch upsert + recursive payload_to_json |
| `57abfc9` | Round-2: Codex P1 + 2 Gemini medium on PR #11 |

### Hotfixes & UX

| Commit | Description |
|---|---|
| `e1c075b` | fix(cli): ensure v3 collection before bulk index |
| `2b59dc9` | fix(predict): route Codex sessions through codex_parser (empirical E2E bug) |
| `e402b1f` | fix(mix): self-contained Mix & Match picker — modal no longer depends on cards behind backdrop |
| `712a128` | fix(ux): release stuck view-transition snapshot + heat-chip moves to top-center + recall banner auto-hide |
| `0f29b96` | fix(ux): heat-chip "giant purple oval" — clamp width, single-line bits, drop backdrop-filter |
| `84db1fc` | fix(ux+obs): enable WebView devtools + silence shell-stderr recall + add errors-badge tooltip |
| `deed283` | fix(ux): kill viewport-spanning purple oval — heat-trail SVG stroke explosion (root cause: weighted fusion sum > 1) |

### Docs

| Commit | Description |
|---|---|
| `8509096` | docs: upstream PR plan — divergence analysis + backport recommendations |
| `d377e7a` | docs(upstream-pr): complete 7-agent analysis package |

### Merge / Integration

| Commit | Description |
|---|---|
| `58aa474` | merge: integrate upstream/main into fork main for full-sync PR (union semantics) |

## Integration approach

Conflict 발생 9 파일 (18 markers) 모두 **union semantics**로 해결:

### Backend (6 files)

| 파일 | 전략 |
|---|---|
| `Cargo.toml` | union 모든 deps: fork (`dirs`, `sha2`, `tempfile`, `tauri-plugin-deep-link`) + upstream (`tauri-plugin-notification`) |
| `Cargo.lock` | regenerate via `cargo generate-lockfile` |
| `capabilities/default.json` | union: 양쪽 plugin permissions (`deep-link:default` + `notification:default`) |
| `cli.rs` | 양쪽 subcommands 통합. upstream `cmd_mcp` + `cmd_install_mcp` BEFORE fork `cmd_scan` |
| `commands.rs` | fork `scan_root_routed`/`scan_all_roots` primary, upstream `parse_transcript_session`/`scan_transcripts_dir` 통합. fork `validate_session_path` sandbox wiring 보존 |
| `lib.rs` | 모든 `pub mod` declarations union (fork: sec/lens/schema/crud/codex_parser/enrich/payload/snapshot, upstream: mcp/watcher). 양쪽 plugins 등록. 모든 commands invoke_handler에 |

### Frontend (3 files)

| 파일 | 전략 |
|---|---|
| `index.html` | fork `#heat-trail` SVG + `#heat-chip` (WOW-1) AND upstream `#timeRail` aside — 모두 `.results` 안에서 sibling |
| `main.js` | `enterSearchMode`에서 `clearHeatTrail()` (fork) + `const root` 리팩터 (upstream) 통합. function count: fork 100 → merged 111 (+11) |
| `styles.css` | fork WOW styles + upstream `.watcher-chip` block |

### Auto-merged (conflict 없음)

- `indexer.rs`, `parser.rs`, `main.rs` — git auto-merge
- Upstream-only 파일 (clean add): `mcp.rs`, `watcher.rs`, `dashboard.html`, `dashboard.js`, `IMPL-MCP.md`, test fixtures

## How to review (in chunks)

37 commits이라 한 번에 review는 어렵습니다. 권장 순서:

1. **Merge commit `58aa474`만 먼저 확인** — conflict 해결이 의도대로 됐는지. 위 표 + `git show 58aa474` 비교.
2. **Phase 별로 commit cherry-pick review** — P1 → P3 → P4 → P5 → P2 → P6 → P7 → P8 순서 (각 자기완결 feature).
3. **UX hotfix 그룹** (`712a128` → `0f29b96` → `84db1fc` → `deed283`) — 모두 D-13 사용자 보고 기반 회귀 fix.
4. **Review feedback 그룹** (`769af65`, `abdf68d`, `57abfc9`) — Gemini/Codex/CodeRabbit 외부 리뷰 반영.

각 phase는 별도 commit이라 individual revert 가능.

## Verification

```bash
# Branch sanity
git fetch upstream
git log upstream/main..ComBba:upstream-pr/full-sync --oneline | wc -l  # → 37

# Backend
cd src-tauri
cargo check                # PASS (verified by Rust resolver agent)
cargo build --release      # 1차 빌드 진행 중 (background)
cargo test --lib           # 1차 테스트 진행 중

# Frontend
node --check src/main.js   # PASS (verified by frontend resolver agent)

# Tauri full build (10분+)
npm run tauri build
```

본 PR을 받으신 후 위 명령들을 실행해서 통합 빌드 확인 부탁드립니다.

## Caveats / known issues (사전 공개)

### 1. WebView `devtools` feature 활성화 유지
`Cargo.toml`에서 `tauri = { features = [..., "devtools"] }` 그대로. release 빌드에서 WebKit Inspector가 활성화되어 IPC 노출 위험이 있습니다. **이 통합 PR 다음으로 별도 follow-up commit에서 Cargo feature flag (`cfg(debug_assertions)` 또는 `--features devtools`)로 게이트하는 것을 권장**합니다.

### 2. `scan_all_sources` (upstream) 보존하되 `#[allow(dead_code)]`
Rust resolver agent가 upstream의 원래 two-root walker를 `#[allow(dead_code)]`로 보존했습니다. 통합된 `scan_all_roots`로 통일했지만 upstream 도구가 직접 호출할 수도 있어 일단 살려뒀습니다. 명확히 deprecated하려면 별도 commit으로 제거 가능합니다.

### 3. `refresh_index` / `list_sessions`가 legacy transcripts도 walk
단일 root path를 지정해도 `~/.claude/transcripts/`를 추가로 walk합니다 (upstream의 migration value 보존 의도). 기존 fork 동작(pure routing)과 차이. `include_legacy: bool` 인자로 게이트하면 양쪽 호환 가능.

### 4. Cargo upgrades 11건 보류
`cargo generate-lockfile`이 `notify 6→8`, `lru 0.12→0.18`, `dirs 5→6` 등 upgrade 가능을 알렸지만 본 PR에서는 적용 안 했습니다. 별도 maintenance PR로 분리 권장.

### 5. `tauri-plugin-deep-link` URL scheme 선언
fork가 `memex://` URL scheme을 deep-link plugin을 통해 등록합니다. upstream `tauri.conf.json`의 `bundle.plugins.deep-link`에 scheme이 선언되어 있는지 확인 필요. 누락 시 별도 commit으로 추가.

## 의도적으로 변경하지 않은 것

- Upstream의 `parser.rs` `parse_transcript_session` + `scan_transcripts_dir` (+283 LOC) — 완전 보존
- Upstream의 `mcp.rs` (9 tools), `watcher.rs` (auto-index daemon), `dashboard.html/js` (data archaeology + Time Machine rail) — 완전 보존
- Upstream의 macOS notification plugin (`tauri-plugin-notification`) — 완전 보존
- Upstream의 `IMPL-MCP.md`, test fixtures — 완전 보존

## 작은 PR로 나눠야 한다면

본 PR이 너무 크면 다음 3개의 **단순화된 small PR**도 이미 준비되어 있습니다 (`sgwannabe/memex#1`, `#2`, `#3`):

1. `#1` mix-modal self-contained picker (3 files, +336/-6)
2. `#2` recall stderr filter + errors tooltip (2 files, +42/-8)
3. `#3` KF-01 path sandbox (7 files, +668/-11)

본 full-sync PR을 close하고 위 3개부터 incremental하게 진행하는 것도 가능합니다. 그 경우 fork의 나머지 33 commits + upstream의 14 commits 분기는 계속 유지됩니다.

## Related artifacts

`ComBba/memex` repo의 `claudedocs/upstream-pr/` 디렉토리에 7-agent 분석 패키지 전체가 있습니다:

- `00-divergence-matrix.md` — file-level 매핑 + 의존성 그래프
- `candidates/01-mix-modal-backport.md`, `02-p1-security-backport.md` — 작은 PR 후보 분석
- `reviews/security-review.md` — OWASP per-commit verdict
- `reviews/test-plan.md` — 단위/통합/property/E2E 매트릭스
- `reviews/cicd-impact.md` — 빌드/CI/release 영향
- `reviews/maintainer-q-and-a.md` — 예상 reviewer 질문 + 사전 답변

---

해커톤 팀 협업이라 양쪽 작업을 합치는 통합 머지로 진행했습니다. 큰 PR이지만 conflict는 모두 union 보존으로 해결됐고, 각 phase commit은 individually revert 가능합니다. 검토 의견 받겠습니다.
