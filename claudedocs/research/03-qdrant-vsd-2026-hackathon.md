# Qdrant Vector Space Day 2026 — "Think Outside the Bot"
## Hackathon Research Report for Memex

**Prepared**: 2026-05-18 (T-14 days from submission deadline)
**Subject project**: `ComBba/memex` — single-binary Tauri 2 desktop app, indexes Claude Code session JSONL into local Qdrant 1.18, exposes seven non-chat surfaces over five named vectors.
**Research scope**: Verify hackathon alignment, surface gaps, rank pre-submission actions.

---

## TL;DR (3 bullets)

- **Official VSD 2026 page found?** Yes — the hackathon details live on the dedicated submission page at `https://try.qdrant.tech/hackathon-vsd`, and the announcement blog at `https://qdrant.tech/blog/vector-space-day-sf-2026/`. The conference itself is `https://qdrant.tech/vector-space-day-sf-26/` (talks/agenda, no hackathon detail).
- **Most urgent submission gap**: A **public demo video (≤3 min)** is mandatory and is **not present** in the Memex repo as of 2026-05-18. The 2026 rules also lifted the limit from 60 s (2025) to **180 s** — Memex needs a script that exercises five primitives across seven surfaces in that window.
- **Memex's likely fit verdict**: **Strong on brief / Adequate on submission readiness**. The product is unusually well-aligned with every published criterion (no chat surface, no LLM at runtime, five named vectors, four advanced primitives, recommendation surfaces), but the submission packet (demo video, README submission checklist, polished landing) needs the next two weeks of work.

---

## 1. The "Think Outside the Bot" brief

### Verbatim prompt

The 2026 hackathon page opens with:

> "Push the boundaries of vector search… no chatbots allowed!"
> *[Source: https://try.qdrant.tech/hackathon-vsd · fetched 2026-05-18]*

Expanded in the announcement blog:

> "a global, virtual hackathon challenging devs to reimagine what's possible with vector search. Forget the classical RAG chatbot!"
> *[Source: https://qdrant.tech/blog/vector-space-day-sf-2026/ · fetched 2026-05-18]*

Encouraged directions, verbatim:

> "Explore multi-modal applications, intelligent recommendations, and advanced vector search that go far beyond conversational interfaces."
> *[Source: https://qdrant.tech/blog/vector-space-day-sf-2026/ · fetched 2026-05-18]*

Explicit "don't":

> "Submissions that are only chatbots are not allowed."
> *[Source: https://try.qdrant.tech/hackathon-vsd · fetched 2026-05-18]*

And the LLM-permissive clarifier (carried over from 2025):

> "LLMs can be used in the project, but pure chatbot UIs are not allowed."
> *[Source: https://try.qdrant.tech/hackathon-2025 · fetched 2026-05-18]*

### Memex README verification

The Memex `README.md` (line 40) quotes:

> **"Think Outside the Bot."** *"Forget the classical RAG chatbot."*

**Verdict**: Both fragments are reproduced **verbatim** from the official 2026 announcement. ✅ Accurate.

The README's stated deadline of **2026-06-01** (line 411) is **also accurate** per the official rules — see Section 6.

---

## 2. Rules & eligibility

| Item | Official wording / value | Source |
|---|---|---|
| **Submission window** | "Now through June 1, 2026" — deadline **11:59 PM Pacific Time, Monday, June 1, 2026** | `https://try.qdrant.tech/hackathon-vsd` · fetched 2026-05-18 |
| **Team size** | **1 – 4** members; solo or group | `https://try.qdrant.tech/hackathon-vsd` · fetched 2026-05-18 |
| **Minimum age** | **18+** | `https://try.qdrant.tech/hackathon-vsd` · fetched 2026-05-18 |
| **Geography** | Global participation; must comply with local laws | `https://try.qdrant.tech/hackathon-vsd` · fetched 2026-05-18 |
| **Ineligible** | Qdrant employees, directors, officers, immediate families | `https://try.qdrant.tech/hackathon-vsd` · fetched 2026-05-18 |
| **Required tech** | "Qdrant Vector Database must be a material part of the project" | `https://try.qdrant.tech/hackathon-vsd` · fetched 2026-05-18 |
| **Originality** | All code must be created **during the hackathon period** — previous projects prohibited; violation = automatic disqualification | `https://try.qdrant.tech/hackathon-vsd` · fetched 2026-05-18 |
| **IP** | Participant retains ownership. Grants Qdrant a "non-exclusive, worldwide, royalty-free, sublicensable, transferable license… for marketing, promotional, educational and other business purposes, in perpetuity." | `https://try.qdrant.tech/hackathon-vsd` · fetched 2026-05-18 |
| **Warranties** | Submission must be "original work (or used with lawful permission)" and must not infringe third-party rights | `https://try.qdrant.tech/hackathon-vsd` · fetched 2026-05-18 |
| **License of repo** | **Not mandated** — public OR private GitHub repo is acceptable | `https://try.qdrant.tech/hackathon-vsd` · fetched 2026-05-18 |
| **Submission form** | `https://forms.gle/YDQ2TDUi8MqS9Vx28` | `https://try.qdrant.tech/hackathon-vsd` · fetched 2026-05-18 |

### Memex implications (rules)

- ✅ **Team size**: Solo is permitted. No action needed.
- ✅ **Tech material part**: Qdrant 1.18 is literally the only data plane. Strong.
- ⚠️ **"All code created during the hackathon period"**: This rule is **the single biggest interpretive risk**. The hackathon page does not publish a discrete start date — it says "Now through June 1, 2026." The 2025 edition framed it the same way (deadline-driven, not start-gated). Memex must be ready to argue, in the README and submission form, that the project was developed for VSD 2026 and within the announced submission window. The Memex repo `createdAt` is **2026-05-18T04:25:09Z** on `ComBba/memex` — that comfortably falls inside any reasonable interpretation of the submission window (the announcement blog is dated April 21, 2026). **Be ready to provide a one-line origin statement.**
- ✅ **Repo can be private**: But making it public maximises judge convenience. Memex is already public Apache-2.0.
- ⚠️ **IP grant**: Apache-2.0 already grants commercial use; the hackathon's additional non-exclusive license is compatible.
- ⚠️ **Submission form URL** is a Google Form — the *one* deliverable not yet exercised.

---

## 3. Judging criteria

### Verbatim criteria (2026)

> Submissions are evaluated on **"technical functionality, creativity, innovative uses of technology, and overall usability."**
> *[Source: https://try.qdrant.tech/hackathon-vsd · fetched 2026-05-18]*

**No numeric weights are published.** The 2025 edition used an almost-identical wording: *"functionality, originality, and user experience"* `[Source: https://try.qdrant.tech/hackathon-2025 · fetched 2026-05-18]`. The 2026 phrasing adds "innovative uses of technology" — a clear nudge that Qdrant-primitive depth matters as a separate axis from "creativity."

### Mapping to Memex surfaces

| Criterion (2026 verbatim) | Memex evidence | Fit rating |
|---|---|---|
| **Technical functionality** | 1,581-LOC `indexer.rs` exercises five named vectors (content/tool/path/error/code), Discovery API (mix), Distance Matrix → MST topology, Recommend API (predict), payload-filtered HNSW (recall). Verified end-to-end on 79 sessions / 17,938 tool calls per README line 411. | **Strong** |
| **Creativity** | Seven non-chat surfaces — Time Machine, Topology galaxy, Mix & Match, Proactive Recall, Predict Next Actions, Replay, Lens slider. The premise ("indexer for your own AI session history") is on-brief because it directly substitutes for a chat affordance. | **Strong** |
| **Innovative uses of technology** | (a) 5 named vectors per point with deliberate per-vector source extraction rules. (b) Weighted-multi-vector "Lens" search implemented as N parallel queries + Rust-side weighted combine (intentionally not RRF, with rationale documented). (c) Snapshot export/import. (d) `uuid_v5(session_id)` for idempotent re-indexing. (e) Petgraph MST over Distance Matrix output. | **Strong** |
| **Overall usability** | Single binary = GUI + CLI; `.dmg` bundle for macOS; Topology rendered with 3d-force-graph; lazy `AppState` so app opens instantly even before Qdrant is up. **Risk**: no signing/notarization (deferred), no Linux/Windows builds, requires user to start Qdrant. | **Adequate — could be Strong with a polished 3-min walkthrough video.** |

---

## 4. Submission deliverables — checklist

| # | Deliverable | Official rule | Memex status (2026-05-18) | Gap |
|---|---|---|---|---|
| 1 | **GitHub repository** (public or private) | `https://try.qdrant.tech/hackathon-vsd` | ✅ `https://github.com/ComBba/memex` — public, Apache-2.0 | None |
| 2 | **README.md** with project description, installation, dependencies | `https://try.qdrant.tech/hackathon-vsd` | ✅ Present, with shield, prompt quote, install steps, end-to-end run | **Add a "Submission" section** that names the hackathon and lists every primitive used, and **fix the `homepageUrl`** which currently points to the stale `sgwannabe/memex` link |
| 3 | **Demo video** — max **3 minutes**, hosted on Loom / YouTube / Dropbox or similar | `https://try.qdrant.tech/hackathon-vsd` | ❌ Not present | **HIGHEST priority** |
| 4 | **Basic code comments** for documentation | `https://try.qdrant.tech/hackathon-vsd` | ✅ `indexer.rs` and `parser.rs` are well-commented; `docs/architecture.md` is comprehensive | Light pass on `lib.rs` / `commands.rs` won't hurt |
| 5 | **Submission form filled** at `https://forms.gle/YDQ2TDUi8MqS9Vx28` | `https://try.qdrant.tech/hackathon-vsd` | ❌ Not done | Must be done before 2026-06-01 23:59 PT |

*[All "Official rule" cells: Source `https://try.qdrant.tech/hackathon-vsd` · fetched 2026-05-18]*

### 2025 → 2026 change to note

The 2025 demo video was capped at **1 minute**. The 2026 cap is **3 minutes**. That is materially more room — long enough to walk all seven surfaces once. Use it.

---

## 5. Prizes & recognition

| Tier | Amount | Source |
|---|---|---|
| 🥇 1st place | **$5,000 USD** | `https://try.qdrant.tech/hackathon-vsd` · fetched 2026-05-18 |
| 🥈 2nd place | **$3,000 USD** | `https://try.qdrant.tech/hackathon-vsd` · fetched 2026-05-18 |
| 🥉 3rd place | **$2,000 USD** | `https://try.qdrant.tech/hackathon-vsd` · fetched 2026-05-18 |
| 🏅 Best-in-Category / sponsor prizes | **Vary** — referenced as available; specific sponsor categories not yet enumerated on the 2026 page | `https://try.qdrant.tech/hackathon-vsd` · fetched 2026-05-18 |
| **Total pool** | **"Over $10K"** | `https://try.qdrant.tech/hackathon-vsd` · fetched 2026-05-18 |

For context, the 2025 sponsor categories were Mistral, CrewAI, Superlinked, TwelveLabs `[Source: https://qdrant.tech/blog/vector-space-hackathon-winners-2025/ · fetched 2026-05-18]`. The 2026 sponsor list is not yet published on the hackathon page; **assume similar categories** but do not name them in the submission.

### Recognition channel

Winners announced **live at Vector Space Day 2026, San Francisco** at the closing remarks (4:40 PM PT, June 11, 2026 at The Midway, 900 Marin St) `[Source: https://luma.com/vsd-sf · fetched 2026-05-18]`. Past pattern: winners get a dedicated blog post (see 2025 recap at `https://qdrant.tech/blog/vector-space-hackathon-winners-2025/`).

---

## 6. Timeline

| Date | Event | Source |
|---|---|---|
| **2026-04-21** | Hackathon announcement blog published | `https://qdrant.tech/blog/vector-space-day-sf-2026/` · fetched 2026-05-18 |
| 2026-05-06 | Speaker CFP deadline (**not the hackathon deadline** — easy to confuse) | `https://qdrant.tech/blog/vector-space-day-sf-2026/` · fetched 2026-05-18 |
| 2026-05-11 | Early-bird ticket pricing cutoff | `https://qdrant.tech/blog/vector-space-day-sf-2026/` · fetched 2026-05-18 |
| **2026-05-18** | **Today** — T-14 days | env |
| 2026-05-26 | "TurboQuant" webinar (Qdrant-hosted) | `https://qdrant.tech/blog/vector-space-day-sf-2026/` · fetched 2026-05-18 |
| **🔴 2026-06-01 23:59 PT** | **Hackathon submission deadline** | `https://try.qdrant.tech/hackathon-vsd` · fetched 2026-05-18 |
| **2026-06-11** | **Vector Space Day SF — winners announced** at closing remarks (4:40 PM PT, The Midway) | `https://luma.com/vsd-sf` · fetched 2026-05-18 |

**Memex's stated deadline (README line 411)**: 2026-06-01 — **matches official.** ✅

---

## 7. Past VSD winners — pattern study

### VSD 2024 — does not exist

The 2025 Berlin event was the **inaugural Vector Space Day**, with ~400 attendees. No VSD 2024 was held.
`[Source: https://qdrant.tech/blog/vector-space-day-2025-recap/ · fetched 2026-05-18]`

### VSD 2025 winners (Berlin, 2025-09-26) — reference cohort

Eleven projects total received recognition. The pattern matters because it's the only precedent we have for what the judges actually rewarded.
`[All entries: Source https://qdrant.tech/blog/vector-space-hackathon-winners-2025/ · fetched 2026-05-18]`

#### 🥇 1st — **Vector Vintage** ($5,000 + $1,000 Neo4j bonus)

- Creator: Benedict Counsell — `https://github.com/kanungle/vector-vintage-public`
- What: E-commerce browsing as an explorable 3D world; product categories become terrain features.
- Qdrant primitives: Similarity search (single-vector).
- Stack: Mistral embeddings → Qdrant → Neo4j curation → UMAP → React Three Fiber.
- Why won: **Creative spatial visualization** combined with multi-layer architectural depth.

#### 🥈 2nd — **RoboBank** ($3,000)

- Creator: Sanya Kapoor — `https://github.com/kanungle/RoboBank`
- What: Trajectory memory for robots — banks sensor-action sequences as vectors, suggests safety-aware reflexes via NN retrieval.
- Qdrant primitives: Real-time similarity search with payload-attached safety labels.
- Why won: Novel robotics domain + practical safety mechanism.

#### 🥉 3rd — **Spatio-Temporal NPCs** ($2,000)

- Creator: cortexandcode — `https://github.com/inventwithdean/spatio-temporal-npcs`
- What: Video-game NPCs whose behavior evolves from CLIP image memories + text event memories + location tags.
- Qdrant primitives: Multi-modal (CLIP + text) vectors, payload filtering on location.
- Why won: Sophisticated memory architecture, **runs locally on ~3 GB VRAM with 1–2 s latency**.

#### Sponsor prizes

| Project | Sponsor | Hook |
|---|---|---|
| **ReMap** (TatankAm Explorers) — `https://github.com/kanungle/remap` | CrewAI | Route-aware event discovery: hybrid semantic + geospatial + temporal filtering |
| **CosmicTwin** (Inferno) — `https://github.com/kanungle/cosmic-twin` | Mistral | Personality-quiz match to a "home planet" via vector similarity |
| **Bachata Vibes** (Erick Rea) — `https://github.com/kanungle/bachata_vibes` | Superlinked | AI choreographer matching music features to dance clips |
| **Qlassroom** (YHHA) — `https://github.com/kanungle/qlassroom` | TwelveLabs | Multi-modal classroom assistant (speech + video + slides + docs) |

#### Honorable mentions

- **Quant Memory Palace** (The Mondays): 3D navigable knowledge universe.
- **OmniVault** (Lone Qdrantic Agent): Privacy-first local-file + web indexer with Chrome extension.
- **Drawland**: Kids' drawing/storytelling playground using vector retrieval.

### Patterns the judges rewarded in 2025

1. **Spatial / 3D visualization** appeared in 3 of 4 honored entries (Vector Vintage, Quant Memory Palace, and CosmicTwin all use React Three Fiber). **Memex's Topology galaxy is on this pattern.** ✅
2. **Memory-as-substrate** framing (RoboBank, Spatio-Temporal NPCs, OmniVault). **Memex's pitch is literally "your own AI session memory." ✅**
3. **Local / edge-friendly deployment** (Spatio-Temporal NPCs at 3 GB VRAM, OmniVault privacy-first). **Memex runs entirely on-device with no LLM at runtime. ✅**
4. **Multi-modal** vectors (Qlassroom, Spatio-Temporal NPCs). Memex is single-modal text — minor gap, but compensated by five named vectors.
5. **Recommendation-shaped UIs** (CosmicTwin, ReMap, Bachata Vibes). **Memex's Mix & Match + Proactive Recall + Predict Next Actions are all recommendation-shaped. ✅**

**Pattern read**: Memex sits in the sweet spot of 4 of the 5 winning patterns. The missing one — multi-modal — is not a published criterion in 2026.

---

## 8. Competing entries surfaced publicly (as of 2026-05-18)

GitHub search across `topic:`, full-text repo search, and code-grep for "Qdrant Hackathon 2026" and "Think Outside the Bot Hackathon 2026". **Be neutral — these are peers, not opponents to disparage.**

### A. Cardinal — `yoyobeverage/cardinal`

- Description: "Vector-native yield discovery for crypto investors. Qdrant Hackathon 2026."
- Created: 2026-05-14 · Updated: 2026-05-15
- Stack: Python + TypeScript + Docker; live deploy at `https://cardinal-qdrant.vercel.app`
- Qdrant primitives used (per README): **6 named vectors** (`narrative`, `risk`, `yield_source`, `correlation`, `tax_treatment`, `composability`), **Recommend API with positive + negative anchors**, **Universal Query with prefetch + RRF fusion**, **payload-filtered HNSW**, plus a 5th (multi-distance — uses cosine / euclidean / dot in the same collection).
- Shape: Not a chatbot. LLM (Gemini) used twice per session only — input parsing + final narration. Selection itself is vector-arithmetic.
- **Strategic note**: Cardinal's "keep the LLM out of the selection seat" thesis is **structurally parallel to Memex's "no LLM at runtime."** Both projects are likely on-brief in the same way — this is the most direct conceptual neighbor.
- Source: `https://github.com/yoyobeverage/cardinal` · fetched 2026-05-18

### B. Unime — `emulaokar1/Unime`

- Description: "Anime recommendation system for Qdrant Think Outside The Bot Hackathon 2026."
- Created: 2026-05-09 · MIT
- Status at fetch: README is one line, repo is empty of substantive code. Likely placeholder.
- Shape: Recommendation (on-brief if delivered).
- Source: `https://github.com/emulaokar1/Unime` · fetched 2026-05-18

### C. AEGIS — `roy-shivam164/aegis`

- Description (README): "Adaptive Evolving General Intelligence System… built on top of Hermes Agent… with Qdrant as the semantic memory backbone. Built for the Qdrant 'Think Outside the Bot' Hackathon 2026."
- Created: 2026-05-05
- Architecture: Three "brains" (SkillForge, MirrorSelf, ShadowReader) over Qdrant memory; explicit JARVIS framing.
- Qdrant primitives: Semantic memory (single-vector retrieval emphasis from README).
- Shape: Agentic — borderline. The README emphasises proactive reminders and personality profiling rather than chat Q&A; whether it dodges the "only chatbot" trap depends on the final UI.
- Source: `https://github.com/roy-shivam164/aegis` · fetched 2026-05-18

### D. Other 2026 references found

The following repos mention "Qdrant Hackathon 2026" in their READMEs/SUBMISSION docs but were created for **other** 2026 hackathons (Convolve 4.0, Vectors In Orbit, ETHSilesia, etc.) — i.e. **not** VSD's Think Outside the Bot:

- `Harsh-BH/convolve-4.0` — Convolve 4.0
- `Makrembz/FinFind` — "Vectors In Orbit"
- `kunwarsahil/RESPOND---AI-System` — Convolve 4.0 MAS Track
- `pbathuri/legal-document-intelligence` — ETHSilesia adjacent
- `HalDariusz/AI-TAURON` — ETHSilesia Hackathon 2026

These are **not direct competition** for VSD 2026 (different events), but several are vector-search-shaped and might be cross-listed.

**Source**: GitHub repo search via `gh search repos`, fetched 2026-05-18.

### Visibility caveat

The official submission flow is a **Google Form, not Devpost** — so there is **no public leaderboard**. The repos above are only those whose authors chose to disclose the hackathon affiliation in a README or repo description. Expect a long tail of submissions invisible to GitHub search.

---

## 9. Memex alignment audit

| Brief requirement / criterion | Memex evidence | Verdict |
|---|---|---|
| **"No chatbots" (hard rule)** | No chat surface; no text-input "ask a question" affordance. Seven non-chat surfaces: Time Machine, Topology galaxy, Mix & Match, Proactive Recall, Predict Next Actions, Replay, Lens slider. `[CLAUDE.md line 17, README line 415]` | **Strong** |
| **"No LLM at runtime"** (self-imposed; aligned with brief spirit but stricter) | Only fastembed-rs BGE-small + qdrant-client gRPC at runtime. `[CLAUDE.md line 17]` | **Strong (over-delivers)** |
| **"Multi-modal applications"** (encouraged) | Single-modal text only. Text is decomposed into 5 semantic streams (content/tool/path/error/code) — defensible as "advanced single-modal" but not "multi-modal." | **Gap** — addressable by reframing in README and demo as "five semantic streams of one modality" rather than chasing a real image/audio vector pre-deadline |
| **"Intelligent recommendations"** (encouraged) | Mix & Match (Qdrant Discovery), Proactive Recall (payload-filtered NN), Predict Next Actions (Qdrant Recommend API). Three of seven surfaces are recommendation-shaped. | **Strong** |
| **"Advanced vector search"** (encouraged) | Five named vectors per point; weighted Lens combine across all five; Distance Matrix → MST topology (rare primitive); Discovery API with target anchor; idempotent re-index via uuid_v5. | **Strong** |
| **"Qdrant as material part"** (required) | Qdrant 1.18 is the entire data plane. There is no second storage layer. | **Strong** |
| **Technical functionality** (judging axis) | Verified on 79 sessions / 17,938 tool calls; CLI + GUI both pass through same indexer. `[README line 411]` | **Strong** |
| **Creativity** (judging axis) | "Index your own AI history" is an unusual angle in a vector-search hackathon dominated by e-commerce / recommendation / memory-for-agents pitches. | **Strong** |
| **Innovative uses of technology** (judging axis) | 5-vector decomposition with documented per-vector source extraction; intentional non-use of RRF in favor of debuggable weighted-combine; petgraph MST over Distance Matrix. | **Strong** |
| **Overall usability** (judging axis) | Single-binary GUI+CLI; .dmg bundle; lazy AppState for instant cold start; macOS Full Disk Access flow is documented. **Risk**: macOS-only; requires user to start Qdrant; no signing. | **Adequate** |
| **Public GitHub repo** | `https://github.com/ComBba/memex` is public, Apache-2.0. ✅ Note: repo `homepageUrl` still points to stale `sgwannabe/memex` (cosmetic). | **Strong with one fix** |
| **README with install/run** | Present; quick-start, CLI surface, architecture, deferred items all documented. | **Strong** |
| **Demo video ≤ 3 min** | **Missing.** | **Missing — highest priority** |
| **Submission form** | Not yet filed. | **Missing — must do before 2026-06-01 23:59 PT** |
| **Originality (all code in window)** | Repo `ComBba/memex` created 2026-05-18; announcement was 2026-04-21. Comfortably within window. Add a one-line origin statement to README to pre-empt any question. | **Adequate — add origin line** |

---

## 10. Pre-submission action items (ranked by impact-per-hour, max 7)

> All items respect the invariant: **no chat surface, no LLM at runtime.** Per CLAUDE.md line 19, adding chat-style features defeats the entire pitch.

### 🔴 1. Record the 3-minute demo video — **8 h** — impact: **decisive**

- **Rule satisfied**: Mandatory deliverable — "Demo video of no more than 3 minutes" `[Source: https://try.qdrant.tech/hackathon-vsd · fetched 2026-05-18]`.
- **Why decisive**: Submission is rejected outright without it. Of the 11 honored 2025 entries, every single one had a video; the winner (Vector Vintage) led with visual immersion.
- **Concrete script** (180 s budget):
  - 0:00–0:15 — Title card + prompt quote ("Forget the classical RAG chatbot."). Position Memex as **"Time Machine for AI session JSONL."**
  - 0:15–0:35 — Time Machine + Topology galaxy (the spatial / 3D moment — this is the 2025-winner pattern).
  - 0:35–0:55 — Lens slider (5 named vectors visualized as live weight chips).
  - 0:55–1:25 — Mix & Match → Proactive Recall → Predict Next Actions (three recommendation primitives in sequence).
  - 1:25–1:50 — Replay (the "no LLM at runtime" payoff — show on-demand JSONL re-parse).
  - 1:50–2:30 — Architecture screen: 5 named vectors, payload indexes, Discovery + Distance Matrix + Recommend APIs labelled.
  - 2:30–3:00 — Tagline + repo URL + submission line. Hard stop at 3:00.
- **Code changes**: None. Pure recording.
- **Tooling**: macOS QuickTime + Cmd-Shift-5 for screen recording; upload to YouTube (unlisted) since both YouTube and Loom are explicitly accepted.

### 🔴 2. Fix `homepageUrl` and stale `sgwannabe/memex` references — **0.5 h** — impact: **medium**

- **Rule satisfied**: README clarity (judging axis: usability).
- **Where**: `gh repo edit ComBba/memex --homepage https://github.com/ComBba/memex` (and the README footer line 466 that still says `sgwannabe/memex`).
- **Per CLAUDE.md**: Stale `sgwannabe/memex` references in README are *acknowledged historical artifacts* (CLAUDE.md line 10 — "무시한다 / ignore"). For the hackathon submission specifically, the footer link is a judge-facing item and should be corrected. Confirm with user before mass edit.
- **Code changes**: `README.md` line 466; `gh repo edit` for homepageUrl.

### 🟡 3. Add a "Submission" section to README — **1.5 h** — impact: **medium-high**

- **Rule satisfied**: README must describe the project (judging axis: technical functionality + clarity).
- **Concrete content** to add near the top of README:
  - One-line origin statement: *"Built for Qdrant Vector Space Day 2026 between [announcement date 2026-04-21] and 2026-06-01."*
  - A 5-row table mapping each of the 5 Qdrant primitives used → file + function + surface:
    - Named vectors (5) → `indexer.rs` → all surfaces
    - Discovery API → `indexer.rs::mix_match` → Mix & Match
    - Recommend API → `indexer.rs::predict_next_actions` → Predict
    - Distance Matrix → `indexer.rs::topology` → Topology galaxy
    - Payload-filtered HNSW → `indexer.rs::recall` → Proactive Recall
  - Demo video link (once recorded).
- **Code changes**: `README.md` only.

### 🟡 4. Record a 30-second silent loop GIF/MP4 for the README hero — **2 h** — impact: **medium**

- **Rule satisfied**: Overall usability (judging axis). Auto-playing visuals on a public README dominate judge first impressions; 3 of 4 2025 honorable mentions had hero GIFs.
- **Content**: Topology galaxy rotating + Lens slider dragging. No narration, no chat input visible (reinforces "Think Outside the Bot").
- **Code changes**: `README.md` hero block to embed the GIF.

### 🟡 5. Write a brief blog post / Gist about the architecture — **3 h** — impact: **medium**

- **Rule satisfied**: Not strictly required by 2026 rules, but the 2025 winners blog `[Source: https://qdrant.tech/blog/vector-space-hackathon-winners-2025/ · fetched 2026-05-18]` shows that Qdrant amplifies entries that already have shareable narrative content. A Gist or `claudedocs/` blog draft also pre-arms the submission form's "project description" field.
- **Concrete content**: 800–1,200 words covering (a) why no chat / no LLM, (b) the 5-vector decomposition, (c) Distance-Matrix-to-MST trick, (d) idempotent uuid_v5 indexing.
- **Code changes**: New file in `claudedocs/` or `docs/blog-vsd-2026.md`.

### 🟢 6. Smoke-test the .dmg on a fresh macOS user account — **2 h** — impact: **medium**

- **Rule satisfied**: Overall usability (judging axis). Risk reduction.
- **Why**: Tauri bundles intermittently misbehave on first launch from Finder (Gatekeeper, Full Disk Access prompt, HOME-CWD path). A judge who tries to run the .dmg and hits a quarantine error is a lost vote.
- **Code changes**: Only if a defect surfaces — `main.rs` HOME-CWD logic is the most likely culprit per CLAUDE.md line 42.

### 🟢 7. File the submission form on or before 2026-05-30 — **0.5 h** — impact: **decisive (it's the actual submission)**

- **Rule satisfied**: Mandatory at `https://forms.gle/YDQ2TDUi8MqS9Vx28` `[Source: https://try.qdrant.tech/hackathon-vsd · fetched 2026-05-18]`.
- **Why -2 days, not -0**: Two-day buffer for any "your video link is broken" follow-up email. Hackathon organizers are sometimes generous with corrections inside the window; they are never generous after.
- **Required inputs** to have ready: project name, repo URL, demo video URL, team members, one-paragraph description, list of Qdrant features used.
- **Code changes**: None.

**Total estimated effort**: ~17.5 hours over 14 days — single-developer-doable.

---

## 11. Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| **No demo video by 2026-06-01 23:59 PT** | High if untreated | **Disqualification** | Item #1, this week |
| **Submission form not filed** | Medium | **Disqualification** | Item #7, by 2026-05-30 |
| **"Code created before hackathon window" challenge** | Low — repo created 2026-05-18, after the 2026-04-21 announcement | Disqualification if pressed | Add origin statement to README (Item #3); be prepared to point at the commit graph |
| **Repo deleted / made private before judging** | Very low | Disqualification | Do not modify visibility; do not force-push history |
| **Demo video > 3 minutes** | Easy to overshoot | Disqualification per rule wording "no more than 3 minutes" | Hard-cut at 2:55 in the script |
| **"Only a chatbot" reinterpretation** | Effectively zero — Memex has no chat surface | Disqualification | Already mitigated by product design |
| **License mismatch with IP grant** | Low — Apache-2.0 is compatible with the requested non-exclusive license | Eligibility issue | None needed |
| **macOS-only delivery** | Medium — not a rule violation, but limits judge environments | Lower usability score | Mention Linux/Windows as roadmap; ensure judges can read code and watch video even if they can't run binary |
| **Sponsor side-prize sponsors not yet published** | Certain (as of 2026-05-18) | Cannot tailor submission to a sponsor angle | Don't try; build for the main brief and let sponsors find you |
| **CLAUDE.md remote-discipline violation** | Avoidable | Could push to wrong fork (`sgwannabe/memex`) | All git operations stay on `ComBba/memex` per CLAUDE.md line 7 |
| **Confusing the speaker CFP deadline (2026-05-06) with the hackathon deadline (2026-06-01)** | The README is already correct; do not let any new copy regress | None directly | Cross-check any new date strings |

---

## Sources

All sources fetched 2026-05-18 from the working environment.

### Primary (official Qdrant)

1. **Hackathon submission page (2026)** — `https://try.qdrant.tech/hackathon-vsd`
   - Authoritative for: deadlines, prizes, judging criteria, deliverables, team size, IP terms, prohibitions.
2. **Announcement blog (2026)** — `https://qdrant.tech/blog/vector-space-day-sf-2026/`
   - Authoritative for: hackathon prompt verbatim, encouraged directions, dates.
3. **Conference page (2026)** — `https://qdrant.tech/vector-space-day-sf-26/`
   - Authoritative for: agenda tracks, attendance details. (No hackathon-specific detail on this page.)
4. **Luma event page (2026)** — `https://luma.com/vsd-sf`
   - Authoritative for: closing remarks time (winner announcement), location.

### Reference (prior year)

5. **Hackathon submission page (2025)** — `https://try.qdrant.tech/hackathon-2025`
   - Used to disambiguate carried-over rule wording (e.g. "LLMs can be used in the project, but pure chatbot UIs are not allowed").
6. **2025 winners blog** — `https://qdrant.tech/blog/vector-space-hackathon-winners-2025/`
   - Authoritative for: 2025 winner cohort, patterns, sponsor categories.
7. **2025 recap blog** — `https://qdrant.tech/blog/vector-space-day-2025-recap/`
   - Authoritative for: confirmation that 2025 was the inaugural VSD (no VSD 2024).
8. **2025 announcement blog** — `https://qdrant.tech/blog/vector-space-day-2025/`
9. **2025 speaker lineup blog** — `https://qdrant.tech/blog/vector-space-day-lineup-2025/`

### Peer projects (2026 cohort, neutral reference)

10. `https://github.com/yoyobeverage/cardinal` — Cardinal
11. `https://github.com/emulaokar1/Unime` — Unime
12. `https://github.com/roy-shivam164/aegis` — AEGIS

### Memex project (self-reference)

13. `https://github.com/ComBba/memex` — Memex repo (the authoritative remote per CLAUDE.md line 7)
14. `/Users/kimsejun/Documents/GitHub/memex/CLAUDE.md` — Project invariants
15. `/Users/kimsejun/Documents/GitHub/memex/README.md` — Project description and hackathon claims

### Tools

- Search via `gh search repos`, `gh search code`, `gh api repos/.../readme`
- Web fetch via WebFetch tool against the URLs listed above
- All citations carry inline `[Source: <URL> · fetched 2026-05-18]` markers in the body
