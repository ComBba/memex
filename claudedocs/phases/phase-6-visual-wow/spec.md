# Phase 6 — Visual WOW Integration · SDD

**Phase ID**: P6
**Targets**: WOW-1, WOW-2, WOW-3, WOW-4, WOW-5 (5 prototype → production surface)
**Owner**: B5 Min-jae + A3 Liana
**Dependency**: P1-P5 backend ready
**Day**: D-9 ~ D-5
**Cross-phase invariants**: 6 모두 (60fps target, No-LLM, WebGL particle 금지)

---

## 1. WOW-1 · Time Machine 2.0 — Heat Trail

### 1.1 Behavior contract

기존 Time Machine stack (`src/main.js` 기존 코드)에 다음 추가:
1. 카드 hover 시 5개 가장 가까운 카드까지 SVG bezier trail
2. trail은 similarity score로 색조 가변 (>0.7 purple, 0.5-0.7 cyan, <0.5 yellow)
3. 카드 모션 60fps (`transition: transform 420ms cubic-bezier(.22,.61,.36,1)`)
4. heuristic chip (KD-01 대체): ai_title / project_name / duration / tool 분포 / outcome (resolved/unresolved)

### 1.2 Frontend integration point

- `src/main.js` — Time Machine render loop에 hover handler 추가
- `src/index.html` — SVG overlay element 추가
- `src/styles.css` — heat trail motion · chip styles

### 1.3 IPC dependency

- `list_sessions` (KB-05 order_by 옵션) — P4
- new: `top_k_neighbors(session_id, k=5)` — Topology API 재활용

### 1.4 Acceptance criteria

- [ ] **AC-6.1.1** (owner: B5 Min-jae) hover handler가 5-neighbor query 후 SVG 렌더
- [ ] **AC-6.1.2** (owner: B5 Min-jae) 60fps 유지 (requestAnimationFrame 측정)
- [ ] **AC-6.1.3** (owner: A3 Liana) chip이 enrich.rs 5 필드 표시
- [ ] **AC-6.1.4** (owner: B5 Min-jae) trail은 `@media (prefers-reduced-motion)`에서 disable

---

## 2. WOW-2 · Topology Galaxy — Gateway Nodes

### 2.1 Behavior contract

기존 Topology (3d-force-graph)에 추가:
1. Cross-project bridge edge → 보라색 + 펄스 애니메이션
2. Gap insight bubble → 두 cluster centroid 사이 floating
3. Cluster 라벨: enrich.rs `ai_title` 빈도 1위 + tool 분포 ("code+shell · Bash×1350 · 「Tauri build」")

### 2.2 Acceptance criteria

- [ ] **AC-6.2.1** (owner: B5 Min-jae) bridge edge `linkColor` 함수로 색조 변화
- [ ] **AC-6.2.2** (owner: B5 Min-jae) `linkDirectionalParticles=2` built-in 사용 (WebGL particle 추가 금지)
- [ ] **AC-6.2.3** (owner: A3 Liana) Gap bubble은 HTML overlay (raycast로 위치)
- [ ] **AC-6.2.4** (owner: B5 Min-jae) 60fps 유지 at N=10k nodes (성능 budget)

---

## 3. WOW-3 · Lens Live Contribution Bars

### 3.1 Behavior contract

P2의 ScoreBreakdown을 시각화:
1. 검색 결과 카드 옆 6 named vector stacked bar
2. transition: flex-basis 320ms cubic-bezier(.22,.61,.36,1)
3. FormulaQuery breakdown chip: content·recency·errors boost 명시

### 3.2 Acceptance criteria

- [ ] **AC-6.3.1** (owner: A3 Liana) 검색 결과마다 stacked bar
- [ ] **AC-6.3.2** (owner: A3 Liana) score breakdown chip 표시
- [ ] **AC-6.3.3** (owner: B5 Min-jae) 결과 변경 시 bar transition fire
- [ ] **AC-6.3.4** (owner: B5 Min-jae) bar는 CSS만 (WebGL/Canvas 금지)

---

## 4. WOW-4 · Predict Cinematic Panel

### 4.1 Behavior contract

기존 Predict 패널 (`src/main.js`):
1. 4×3 thumbnail grid (neighbor 8 세션)
2. 각 thumbnail에 KICK 호명: tool name + arg + 비율 + heuristic outcome
3. 클릭 시 View Transition API로 cinematic zoom → Replay
4. Chrome (Tauri 2 webview)에서 native 지원, Safari 시 fallback fade

### 4.2 Acceptance criteria

- [ ] **AC-6.4.1** (owner: B5 Min-jae) `document.startViewTransition` 사용 (`if (document.startViewTransition)`)
- [ ] **AC-6.4.2** (owner: B5 Min-jae) view-transition-name CSS 변수
- [ ] **AC-6.4.3** (owner: A3 Liana) thumbnail에 enrich outcome 캡션
- [ ] **AC-6.4.4** (owner: B5 Min-jae) fallback (no API): 240ms opacity fade

---

## 5. WOW-5 · Discovery 3D Hyperplane Splash

### 5.1 Behavior contract

기존 Mix & Match를 3D hyperplane 시각화로 진화:
1. positive/negative anchor를 3D space에 배치 (vendored three.js 또는 vanilla canvas — prototype과 동일 기술 사용)
2. hyperplane = plane geometry, normal = mean(pos) - mean(neg)
3. 결과 카드는 plane positive 쪽에서 emit
4. KB-03 (P4) context pairs 사용

### 5.2 Acceptance criteria

- [ ] **AC-6.5.1** (owner: B5 Min-jae) 3D hyperplane 렌더 (canvas 2D 또는 three.js)
- [ ] **AC-6.5.2** (owner: B1 Hiroshi) KB-03 IPC 호출 (mix_match with pairs)
- [ ] **AC-6.5.3** (owner: A3 Liana) emit animation (cubic-bezier)
- [ ] **AC-6.5.4** (owner: B5 Min-jae) 60fps with N=20 result cards
- [ ] **AC-6.5.5** (owner: A3 Liana) Relevance Feedback 👍/👎 (KA-04 P4) 통합

---

## 6. Phase 6 종합 acceptance

| ID | 항목 |
|----|------|
| P6-DONE-1 | 5개 WOW가 production `src/main.js`에 통합 |
| P6-DONE-2 | 60fps target 5/5 통과 (`requestAnimationFrame` budget) |
| P6-DONE-3 | accessibility: `aria-*`, `role`, focus management 추가 |
| P6-DONE-4 | `@media (prefers-reduced-motion)` 5/5 적용 |
| P6-DONE-5 | tests.md 모두 통과 |

---

## 7. Risk

| Risk | Mitigation |
|------|-----------|
| 60fps 미달 (Topology N=10k + pulse) | WebGL particle 금지 (B5 invariant). throttle on perf budget |
| View Transition API Safari 미지원 | fallback `transition: opacity 240ms` |
| ai_title 없는 세션의 chip 빈 라벨 | project_name + first-user-turn 첫 명사구 (enrich topic 로직) |
| WOW-5 hyperplane 회전 시 멀미 | smooth easing + 회전 속도 cap |

---

## 8. Out-of-scope

- Sound design (P7)
- Camera moves / chapter cards (P7 영상 production)
- LLM chip 자연어 라벨 (v3.2 SKIP)
