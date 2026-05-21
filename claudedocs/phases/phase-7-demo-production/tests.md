# Phase 7 — Demo Production · TDD

**Phase**: P7
**Note**: Most production work (video editing, form submission) is manual. Tests focus on automated checks + manual checklists.

---

## 1. Automated tests · Build artifacts

### Test file: `scripts/p7_validate.sh` or CI

| Test | Action | Expected | AC |
|------|--------|----------|----|
| `t_dmg_built` | check `src-tauri/target/release/bundle/macos/Memex.app` exists | true | AC-7.3.1 |
| `t_dmg_size_reasonable` | size of .dmg file | 10-30 MB range | AC-7.3.1 |
| `t_app_runs_on_macos11` | macOS 11.0 VM check | launches | AC-7.3.1 |

---

## 2. Manual checklists

### 2.1 Demo video review checklist

D-7 (draft 1), D-4 (draft 2), D-1 (final):

- [ ] M-7.1.1 Length 180s ±2s
- [ ] M-7.1.2 Cold open Bush 1945 quote typed-out 0:00–0:08
- [ ] M-7.1.3 Time Machine fly-in 0:08–0:18
- [ ] M-7.1.4 Chip close-up 0:18 with KD-free heuristic
- [ ] M-7.1.5 Number count-up "80 sessions" 0:28
- [ ] M-7.1.6 Topology cluster auto-label 0:40 (enrich.rs 5필드)
- [ ] M-7.1.7 Bridge + gap reveal 0:55
- [ ] M-7.1.8 Lens before/after split 1:10
- [ ] M-7.1.9 Contribution bar live 1:22
- [ ] M-7.1.10 Predict cinematic 1:32
- [ ] M-7.1.11 **CLIMAX silence starts at 1:42 ±0.5s**
- [ ] M-7.1.12 **Silence duration ≥ 12s** (verified: 0 audio peaks in [1:42, 1:58])
- [ ] M-7.1.13 Recall banner 1:54 in silence
- [ ] M-7.1.14 Music returns 2:00
- [ ] M-7.1.15 Discovery hyperplane 2:08
- [ ] M-7.1.16 Relevance Feedback 2:22
- [ ] M-7.1.17 Outro 2:48–3:00 (Apache-2.0 + Qdrant logo + github URL)

### 2.2 DMG clean-machine smoke checklist

- [ ] M-7.3.1 Fresh macOS 11+ VM (no Memex installed)
- [ ] M-7.3.2 Qdrant binary downloaded + running on :6333/:6334
- [ ] M-7.3.3 Memex.dmg drag-mount → /Applications
- [ ] M-7.3.4 First launch shows Full Disk Access prompt
- [ ] M-7.3.5 After FDA grant, app opens with empty Time Machine
- [ ] M-7.3.6 CLI: `memex scan --index` downloads fastembed (130MB) once
- [ ] M-7.3.7 Re-launch: Time Machine populates
- [ ] M-7.3.8 ⌘+T: Topology renders
- [ ] M-7.3.9 ⌘K + query: Lens responds
- [ ] M-7.3.10 Mix & Match: Discovery works
- [ ] M-7.3.11 Predict panel: thumbnail grid
- [ ] M-7.3.12 Proactive Recall: 12s poll triggers
- [ ] M-7.3.13 Replay: card click opens turn-by-turn
- [ ] M-7.3.14 Network inspector during all: 0 outbound after fastembed download
- [ ] M-7.3.15 No Ollama installed: all 7 surfaces still work

### 2.3 Google Form pre-submission checklist (D-2)

- [ ] M-7.4.1 Project name: "Memex"
- [ ] M-7.4.2 Repo URL: `https://github.com/ComBba/memex` (public)
- [ ] M-7.4.3 Video URL: YouTube unlisted (link copied + tested in incognito)
- [ ] M-7.4.4 License: Apache-2.0 verified
- [ ] M-7.4.5 Team size: 1
- [ ] M-7.4.6 One-line description: 280 chars or less
- [ ] M-7.4.7 Features list: all Cat A/B/C/F/G + 5 WOW
- [ ] M-7.4.8 Brief alignment: thesis statement
- [ ] M-7.4.9 Contact email confirmed
- [ ] M-7.4.10 Form preview reviewed before submit

### 2.4 Final submission checklist (D-0)

- [ ] M-7.4.11 Submit time < 2026-06-01 23:59 UTC
- [ ] M-7.4.12 Confirmation email received
- [ ] M-7.4.13 Confirmation screenshot saved to `claudedocs/submission/confirm.png`

---

## 3. README/CLAUDE.md/docs/* invariant tests

| Test | Method | Expected | AC |
|------|--------|----------|----|
| `t_readme_hero_unchanged` | `git diff main README.md \| grep "No chat box. No LLM"` | match (line preserved) | AC-7.2.1 |
| `t_claude_md_invariant_unchanged` | `git diff main CLAUDE.md \| grep "no LLM at runtime"` | match | AC-7.2.2 |
| `t_docs_architecture_unchanged` | `git diff main docs/architecture.md` | empty (no diff) | AC-7.2.2 |
| `t_docs_qdrant_features_unchanged` | `git diff main docs/qdrant-features.md` | empty | AC-7.2.2 |
| `t_readme_status_addition` | `git diff main README.md` | only Status section +2 lines | AC-7.2.3 |

---

## 4. Landing page checks

| Test | Method | Expected | AC |
|------|--------|----------|----|
| `t_landing_loads` | open `index.html` | no JS errors | AC-7.5.1 |
| `t_landing_has_bush_quote` | grep `index.html` for "1945" | match | AC-7.5.3 |
| `t_landing_has_qdrant_logo` | grep `index.html` for "qdrant" | match | AC-7.5.2 |
| `t_landing_has_repo_link` | grep `index.html` for "github.com/ComBba/memex" | match | AC-7.5.2 |

---

## 5. Network audit (final pre-submission)

```bash
# Run with Memex running:
sudo lsof -i -P | grep Memex
# Expected: only localhost:6334 (Qdrant gRPC) + huggingface.co (first launch only)
```

| Test | Setup | Expected |
|------|-------|----------|
| `audit_runtime_network` | post-first-launch, idle | only localhost connections |
| `audit_no_telemetry` | inspect outgoing | no 3rd party domains |

---

## 6. Test → AC mapping

| AC | Tests |
|----|-------|
| AC-7.1.1 ~ AC-7.1.6 | 17 manual checklist items |
| AC-7.2.1 ~ AC-7.2.3 | 5 automated diff tests |
| AC-7.3.1 ~ AC-7.3.4 | 15 manual smoke items + 3 automated |
| AC-7.4.1 ~ AC-7.4.4 | 13 form checklist items |
| AC-7.5.1 ~ AC-7.5.3 | 4 automated tests |

**총 ~35 manual + ~12 automated**.

---

## 7. Sign-off ceremony (D-1 final)

D-1 23:00 UTC, A1 Eunha 주재:

```
[ ] M-7.1.* (영상 17 items): A4 Daichi
[ ] M-7.3.* (DMG 15 items): B3 Felix
[ ] M-7.4.* (form 13 items): A5 Sara
[ ] AC-7.2.* (docs invariant): A1 Eunha
[ ] AC-7.5.* (landing): A3 Liana

Sign-off threshold: 100% of M-7.1.*, M-7.3.*, M-7.4.* + all automated
Then: A5 Sara submits at D-0 09:00 UTC (15h margin to deadline)
```
