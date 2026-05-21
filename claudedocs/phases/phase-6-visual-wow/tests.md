# Phase 6 — Visual WOW Integration · TDD

**Phase**: P6
**Test framework**: vitest + Playwright (existing) or vanilla HTML test pages

---

## 1. Unit tests (JS) · WOW-1 Time Machine heat trail

### Test file: `src/tests/wow-1.test.js` (vitest or vanilla)

| Test | Setup | Expected | AC |
|------|-------|---------|----|
| `t_hover_triggers_neighbor_query` | hover card | invoke('top_k_neighbors', {k:5}) called | AC-6.1.1 |
| `t_trail_color_by_sim` | sim=0.8 | path stroke color #bf5af2 | AC-6.1.1 |
| `t_trail_color_mid_sim` | sim=0.55 | stroke #64d2ff | AC-6.1.1 |
| `t_trail_color_low_sim` | sim=0.4 | stroke #ffd60a | AC-6.1.1 |
| `t_chip_shows_ai_title` | session with ai_title | chip text == ai_title | AC-6.1.3 |
| `t_chip_fallback_no_ai_title` | ai_title null | chip uses heuristic topic | AC-6.1.3 |
| `t_reduced_motion_disables_trail` | media query simulated | trail not rendered | AC-6.1.4 |

---

## 2. Unit tests (JS) · WOW-2 Topology gateway

| Test | Setup | Expected | AC |
|------|-------|---------|----|
| `t_bridge_edge_color_purple` | cross-project link | linkColor returns #bf5af2 | AC-6.2.1 |
| `t_bridge_particles_count` | bridge edge | linkDirectionalParticles == 2 | AC-6.2.2 |
| `t_gap_bubble_position` | 2 cluster centroids | bubble at midpoint (raycast verified) | AC-6.2.3 |
| `t_cluster_label_includes_ai_title` | cluster with ai_titles | label contains most-frequent ai_title | AC-6.2.1 |

---

## 3. Unit tests (JS) · WOW-3 Lens contribution bars

| Test | Setup | Expected | AC |
|------|-------|---------|----|
| `t_bar_segments_match_breakdown` | breakdown {content:.4, error:.2, ...} | 6 segments, widths proportional | AC-6.3.1 |
| `t_bar_transition_320ms` | result update | computed style transition contains 320ms | AC-6.3.3 |
| `t_score_chip_shows_formula` | result with breakdown | chip text includes "recency ×" and "errors +" | AC-6.3.2 |
| `t_no_webgl_canvas_in_bar` | DOM inspection | bar element is <div>, not <canvas> | AC-6.3.4 |

---

## 4. Unit tests (JS) · WOW-4 Predict cinematic

| Test | Setup | Expected | AC |
|------|-------|---------|----|
| `t_thumbnail_grid_4x3` | 8 predictions | grid has 4 cols, ≥2 rows | AC-6.4.3 |
| `t_thumbnail_shows_outcome` | prediction with outcome | thumbnail text includes outcome string | AC-6.4.3 |
| `t_view_transition_api_used_if_supported` | mock document.startViewTransition | called on click | AC-6.4.1 |
| `t_fallback_fade_no_api` | document.startViewTransition undefined | opacity transition 240ms | AC-6.4.4 |
| `t_view_transition_name_set` | grid items | each item has --vt-name | AC-6.4.2 |

---

## 5. Unit tests (JS) · WOW-5 Discovery splash

| Test | Setup | Expected | AC |
|------|-------|---------|----|
| `t_hyperplane_normal_calculation` | (pos, neg) anchors | normal = norm(mean(pos) - mean(neg)) | AC-6.5.1 |
| `t_emit_animation_cubic_bezier` | result emerges | CSS animation uses cubic-bezier(.22,.61,.36,1) | AC-6.5.3 |
| `t_relevance_feedback_buttons` | result card | 👍 + 👎 buttons present | AC-6.5.5 |
| `t_feedback_click_calls_ipc` | 👍 click | invoke('relevance_feedback', ...) called | AC-6.5.5 |
| `t_pair_drag_calls_mix_match_with_pairs` | drag pair | invoke('mix_match', { pairs: [...] }) | AC-6.5.2 |

---

## 6. Performance tests (E2E via Playwright)

| Test | Setup | Expected | AC |
|------|-------|---------|----|
| `perf_time_machine_60fps` | hover ×10 over cards | fps ≥ 58 | AC-6.1.2 |
| `perf_topology_60fps_at_n10k` | corpus N=10k | fps ≥ 55 | AC-6.2.4 |
| `perf_lens_bar_transition_smooth` | 5 weight slides | no frame drops > 16.67ms | AC-6.3.3 |
| `perf_predict_view_transition` | click thumbnail | transition completes < 400ms | AC-6.4.1 |
| `perf_discovery_60fps_n20_cards` | 20 result cards emerging | fps ≥ 55 | AC-6.5.4 |

---

## 7. Accessibility tests

| Test | Surface | Check | AC |
|------|---------|-------|----|
| `a11y_aria_labels` | all 5 WOW | aria-label on interactive elements | P6-DONE-3 |
| `a11y_keyboard_nav` | Time Machine | ↑↓ Enter Escape work | P6-DONE-3 |
| `a11y_focus_visible` | all buttons | :focus-visible style | P6-DONE-3 |
| `a11y_contrast_aa` | colors | WCAG AA contrast ≥ 4.5:1 | P6-DONE-3 |
| `a11y_reduced_motion_5_surfaces` | media query | 5 surfaces all respect | P6-DONE-4 |

---

## 8. Integration tests · IPC contract verification

| Test | Action | Expected |
|------|--------|----------|
| `it_lens_breakdown_to_bars` | lens_search returns breakdown | bar segments match exactly |
| `it_predict_to_thumbnails` | predict_next_actions response | 4×3 grid renders all entries |
| `it_mix_match_pairs_roundtrip` | drag pair → mix_match | results emit in WOW-5 |
| `it_relevance_feedback_iteration` | click 👍 → re-query → result shift | rankings change |

---

## 9. Regression tests

| Surface | Check |
|---------|-------|
| Existing keyboard shortcuts (⌘K, ⌘T, Esc) | unchanged |
| Existing Time Machine ↑↓ scroll | unchanged |
| Search input ⌘K behavior | unchanged (lens_search 진입점) |
| Snapshot Export/Import buttons | unchanged |
| Re-index button | unchanged |
| Window resize behavior | unchanged |

---

## 10. No-LLM verification (cross-phase invariant)

```bash
# Run in CI:
grep -nrE "openai|anthropic|chat|completion|gpt|llm|ollama|gemma|qwen" src/ 2>/dev/null
# Expected: 0 matches (the prototypes/ folder may have history strings — ignore)
```

| Test | Expected |
|------|----------|
| `nolm_grep_main_js` | 0 LLM keyword in `src/main.js` |
| `nolm_grep_index_html` | 0 LLM keyword in `src/index.html` |
| `nolm_no_fetch_to_localhost_11434` | `src/main.js` has no `fetch('http://localhost:11434...')` |

---

## 11. Test → AC mapping

| AC | Tests |
|----|-------|
| AC-6.1.1 ~ AC-6.1.4 | 7 unit + 1 perf |
| AC-6.2.1 ~ AC-6.2.4 | 4 unit + 1 perf |
| AC-6.3.1 ~ AC-6.3.4 | 4 unit + 1 perf |
| AC-6.4.1 ~ AC-6.4.4 | 5 unit + 1 perf |
| AC-6.5.1 ~ AC-6.5.5 | 5 unit + 1 perf |
| P6-DONE-3 (a11y) | 4 a11y tests |
| P6-DONE-4 (reduced motion) | 1 a11y test |

**총 ~35 test case + 5 perf + 5 a11y**.
