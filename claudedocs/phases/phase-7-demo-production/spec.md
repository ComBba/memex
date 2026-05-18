# Phase 7 — Demo Production · SDD

**Phase ID**: P7
**Targets**: 3-min climax-staged demo video + Google Form 제출 + DMG clean-machine test + landing page polish
**Owner**: A4 Daichi (video) + A5 Sara (compliance) + A1 Eunha (narrative)
**Dependency**: P6 visual ready
**Day**: D-7 ~ D-0
**Cross-phase invariants**: README narrative 변경 0, ComBba/memex만, no-LLM

---

## 1. Demo Video — 3-min climax-staged

### 1.1 Behavior contract (production)

기준: research/09 (claudedocs/research/09-hackathon-demo-video-wow.md)의 18-shot timeline.

| Time | Frame · action | Audio | Caption / KICK |
|------|----------------|-------|----------------|
| 0:00 | Bush 1945 잡지 페이지 fade-in. 한 줄 인용 typed-out | silence | cold open |
| 0:08 | Memex app launch. Time Machine stack fly-in | music drops in (mid pulse) | "Time Machine" |
| 0:18 | enrich.rs chip이 카드에 즉시 채워지는 close-up | chime SFX (Kenney) | KD-01-free · chips heuristic |
| 0:28 | Number animation: "80 sessions · 17,938 tool calls" count-up | music swell | "density reveal" |
| 0:40 | ⌘+T → camera zoom into Topology galaxy. cluster auto-labels (enrich.rs) | music pulse continues | KD-02-free · cluster naming |
| 0:55 | Bridge edge + gap bubble: "redesign ↔ yc · sim 0.97 · no bridge" | music · subtle | "connection reveal" |
| 1:10 | ⌘K + "edit auth.js" 타이핑. BEFORE/AFTER split-screen: 좌측 lens v1 (5 round-trips · 320ms), 우측 v3 FormulaQuery (1 call · 45ms) | tick sound | KA-01 · FormulaQuery |
| 1:22 | contribution bar 살아 움직임 + breakdown chip | music · subtle | KA-01 · breakdown |
| 1:32 | Predict 4×3 thumbnail grid · cinematic zoom → Replay | chime | KG-02 LRU · KD-01-free outcome |
| **1:42** | **⚡ CLIMAX · cargo build 실패 → Memex 카메라 천천히 pan. 음악 완전 fade-out. screen silent for 12s** | **SILENCE (960 frames at 60fps)** | **⚡ reverse query** |
| **1:54** | Recall banner slides in: "I've seen this before · sim 0.93 · ACORN-filtered" | soft banner SFX | KB-04 ACORN |
| 2:00 | 음악 복귀 (slower tempo). Mix & Match로 컷 | music returns soft | "recovery" |
| 2:08 | positive 2 카드 · negative 1 카드 drag → 3D hyperplane 등장 | music · build | KB-03 · Discovery |
| 2:22 | 결과 카드 👍 → anchor 재계산 → 결과 즉시 재정렬 | chime | KA-04 · Relevance FB |
| 2:35 | 전체 surface montage (6장 빠른 컷) | music · climb | "breadth" |
| 2:48 | Title card: "Memex · spatial memory · Qdrant 1.18 pinnacle" | music · resolve | "thesis" |
| 2:56 | github.com/ComBba/memex · Apache-2.0 · Qdrant logo · "Think Outside the Bot ✓" | final chime | outro |

총 18 shot. 정확히 180s. 1:42-1:58 silence frame = 960 frames at 60fps.

### 1.2 Production stack

- **Music**: Bensound CC-attribution 1트랙 ("Slow Motion" 또는 "Cinematic Documentary")
- **SFX**: Kenney UI Audio (CC0) — click · hover · chime · save · banner · outro (6 hit)
- **Capture**: OBS 60fps screen recording
- **Editor**: DaVinci Resolve
- **Caption**: Karpathy-style 큰 kerning
- **App audio**: 0 (모든 sound는 post)

### 1.3 Acceptance criteria

- [ ] **AC-7.1.1** (owner: A4 Daichi) 영상 길이 = 180s ±2s
- [ ] **AC-7.1.2** (owner: A4 Daichi) 1:42-1:58 12초 silence 정확히
- [ ] **AC-7.1.3** (owner: A4 Daichi) 모든 18 shot 등장
- [ ] **AC-7.1.4** (owner: A4 Daichi) 60fps capture, post에서 24fps export 허용
- [ ] **AC-7.1.5** (owner: A5 Sara) Outro에 Apache-2.0 license + Qdrant logo + github URL
- [ ] **AC-7.1.6** (owner: A1 Eunha) 외부 5명 blind test → "이게 챗봇이 아닌 걸 즉시 이해" 평균 응답 시간

---

## 2. README · CLAUDE.md · docs/* 변경

### 2.1 Behavior contract

**변경 없음** (v3.2 결정). 단 다음 *예외*만 허용:

- `README.md` Status 섹션 1줄 추가 (선택 사항):
  ```
  + Plan v3.2 validated: 24 Qdrant 1.18 KICKs, no LLM dependency.
  +   See claudedocs/sota-plan-v3.html.
  ```
- 그 외 README/CLAUDE.md/docs/architecture.md/docs/qdrant-features.md: **0 수정**

### 2.2 Acceptance criteria

- [ ] **AC-7.2.1** (owner: A1 Eunha) README hero ("No LLM call at runtime") 그대로
- [ ] **AC-7.2.2** (owner: A1 Eunha) CLAUDE.md invariant 그대로
- [ ] **AC-7.2.3** (owner: A5 Sara) Status 1줄 추가는 D-1 merge

---

## 3. DMG Clean-Machine Test

### 3.1 Behavior contract

D-2에 외부 macOS 16GB MacBook (또는 VM)에서:
1. DMG 다운로드 → Drag to /Applications
2. Memex.app 첫 launch (Full Disk Access prompt)
3. Qdrant binary 별도 다운로드 + 실행
4. memex scan --index (외부 download: fastembed model 130MB)
5. 7 surface 모두 동작 확인

### 3.2 Acceptance criteria

- [ ] **AC-7.3.1** (owner: B3 Felix) DMG download → launch까지 사용자 step ≤ 5
- [ ] **AC-7.3.2** (owner: B3 Felix) fastembed 모델 download 외 outbound network 0 (네트워크 inspector 확인)
- [ ] **AC-7.3.3** (owner: B3 Felix) 7 surface 모두 동작 (수동 smoke checklist)
- [ ] **AC-7.3.4** (owner: B2 Aisha) Ollama 미설치 환경에서 100% 동작 (LLM 없음)

---

## 4. Google Form 제출

### 4.1 Behavior contract

VSD 2026 공식 form: `https://forms.gle/YDQ2TDUi8MqS9Vx28`

필수 필드:
- Project name: Memex
- Repo: `https://github.com/ComBba/memex`
- Demo video URL: YouTube unlisted
- License: Apache-2.0
- Team size: 1 (Sangguen Chang)
- One-line description: "Your AI session history as spatial memory — Qdrant 1.18 pinnacle, no chatbot"
- Qdrant features used: list of 24 KICKs by Cat
- Hackathon brief alignment: "no chatbot UX · multi-modal · recommendation"

### 4.2 Acceptance criteria

- [ ] **AC-7.4.1** (owner: A5 Sara) Form 모든 필수 필드 작성
- [ ] **AC-7.4.2** (owner: A5 Sara) Video URL public unlisted (private 아님)
- [ ] **AC-7.4.3** (owner: A5 Sara) 제출 시점 < 2026-06-01 23:59 UTC
- [ ] **AC-7.4.4** (owner: A5 Sara) 제출 confirmation screenshot 보관

---

## 5. Landing Page Polish (`index.html` 루트)

### 5.1 Behavior contract

기존 single-file `index.html` (28KB). D-1까지 자유 편집:
- Demo video embed
- Plan v3.2 link
- "Think Outside the Bot" thesis 강조
- GitHub repo CTA

### 5.2 Acceptance criteria

- [ ] **AC-7.5.1** (owner: A3 Liana) D-1까지 final design
- [ ] **AC-7.5.2** (owner: A5 Sara) Apache-2.0 + Qdrant logo footer
- [ ] **AC-7.5.3** (owner: A1 Eunha) Bush 1945 인용 cold open으로 유지

---

## 6. Phase 7 종합 acceptance

| ID | 항목 |
|----|------|
| P7-DONE-1 | 영상 180s ±2s, climax silence 12s 검증 |
| P7-DONE-2 | DMG clean-machine test passed |
| P7-DONE-3 | Google Form 제출 완료 (confirmation 보유) |
| P7-DONE-4 | Landing page Bush 인용 그대로 유지 |
| P7-DONE-5 | README/CLAUDE.md/docs/* 변경 0 (Status 1줄 추가만) |
| P7-DONE-6 | tests.md 모두 통과 (영상은 manual review) |

---

## 7. Risk

| Risk | Mitigation |
|------|-----------|
| 영상 3분 초과 | shot별 시간 budget 엄수, post trim |
| Silent frame 어색함 | 외부 review 3명 사전 검토 |
| DMG 외부 머신 미동작 | D-2 reserve, D-1 final test |
| Google Form 제출 누락 | D-2 draft, D-1 final submit, D-0 confirm |
| YouTube 업로드 실패 | Vimeo backup |

---

## 8. Out-of-scope

- 코드 서명 (post-hackathon)
- Linux/Windows 빌드 (post-hackathon)
- 자동화된 영상 generation
