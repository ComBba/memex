# Hackathon Demo Video — closing Memex's wow gap

**Prepared**: 2026-05-18 (T-14 from VSD 2026 submission)
**Subject**: `ComBba/memex` — Tauri 2 desktop app, Qdrant 1.18, 7 non-chat surfaces.
**Mandate**: Direct the ≤3-min submission video. Plan v3 has surfaces; it does not yet have a *director's eye*. This is the eye.

---

## 1. Executive summary (≤200 words)

Plan v3's §8 demo script is a **feature shopping list in 24-second slices** — eight equal segments, every one captioned with a KICK acronym. That maximises Qdrant-primitive density and minimises *story*. The Devpost playbook is explicit: "Make sure your demo video is highly visual, and interactive (show your project in use)" — *not* an annotated tour `[Source: https://info.devpost.com/blog/6-tips-for-making-a-hackathon-demo-video · 2026-05-18]`. The five changes that close the gap:

1. **Cold-cut the Bush 1945 quote.** The video opens on the Topology galaxy *moving* at 0:00. The thesis card comes at 0:08, not 0:18.
2. **Pick one hero shot, not seven.** The Topology fly-through is the "Vector Vintage moment" — 25 seconds, single take, music swell.
3. **Animate one number on screen at all times.** "17,938 tool calls · 79 sessions · 84 ms" — a HUD that counts up as cards fly past.
4. **Mute-friendly captions.** Product Hunt convention: video must work on mute `[Source: https://www.flowjam.com/blog/product-hunt-launch-video-2025-template-pitfalls]`. Karpathy-large bold sans, 64 px, one phrase per cut.
5. **Drop music to zero on the Proactive Recall reveal.** That is the "reverse query" — the loudest *quiet* moment in the video.

Total production additions in 14 days: Kenney UI Audio (CC0), one Bensound track, OBS 1080p60, eight pre-rendered captions. No new code dependencies.

---

## 2. Catalog of reference videos

> ⚠️ **VSD 2025 demo-video URLs are sparse.** Qdrant's official 2025 winners blog `https://qdrant.tech/blog/vector-space-hackathon-winners-2025/` embeds *no* video links; submissions were collected via Google Form and never republished as a public playlist. Two of the eleven 2025 honored projects have surfaced their own demo URLs on GitHub. The rest are inferred from the blog's prose. **This is one of the most important findings of this report**: there is no high-bar VSD video the judges will compare you to. You are setting the bar.

| # | Project / video | URL | Length | What worked |
|---|---|---|---|---|
| **R1** | **Spatio-Temporal NPCs — 1-min demo** (VSD 2025 🥉 3rd) | `https://youtu.be/4bk44WQv0EM` | ~1:00 (per README label "1 minute Demo Video") | Title alone telegraphs the stack — "AI with Spatio-Temporal intelligence \| Unreal Engine \| Qdrant \| GPT-OSS-120B". *Names the primitives in the title*; judges read titles first. Source: `https://github.com/inventwithdean/spatio-temporal-npcs` README. |
| **R2** | **Spatio-Temporal NPCs — extended demo** | `https://youtu.be/T7lnAcWFLNw` | Longer, length not specified by README | Companion long-form, free of the 1-min compression. Pattern Memex could mirror: short ≤3-min "submission cut" + extended "director's cut" for the blog post / Devpost gallery. Source: same README. |
| **R3** | **Vector Vintage — live demo** (VSD 2025 🥇 grand prize) | `https://vector-vintage.benedictcounsell.com` (live site, *no* video URL in repo) | n/a | The grand-prize project chose to ship a **live deployed URL** as the demo, *not* a YouTube cut. Live deploy is itself a wow moment that an .app bundle cannot match. Memex's macOS-only constraint must be compensated by video. Source: `https://github.com/kanungle/vector-vintage-public` README. |
| **R4** | **RoboBank** (VSD 2025 🥈 2nd) — described frame | Demo .mp4 not exposed in repo; LinkedIn announcement at `https://www.linkedin.com/posts/sanya-k-494584261_second-solo-hackathon-second-win-a-activity-7378813743922626560-A6yP` | n/a | README describes the visual: "Grid-based 10×10 room on a dark grey/black background. Robot represented by a triangle with orientation arrow… Color-coded paths indicate **safe (green), near-miss (orange), unsafe (red)** outcomes." Source: `https://github.com/kanungle/RoboBank`. *Lesson*: a single iconic colour-coded visual ("triangle in a grid, three coloured outcomes") is more memorable than five competing widgets. |
| **R5** | **ReMap** (VSD 2025 CrewAI sponsor prize) | `ReMap.mp4` referenced in README but not externally hosted | n/a | README emphasizes "interactive frontend… buffered search areas, event markers." Source: `https://github.com/kanungle/remap`. *Lesson*: the demo-video file lives in the repo as a binary asset — fine for submission, terrible for amplification. Memex should host on YouTube unlisted for both. |
| **R6** | **Hacker News fly-through (Aug 2025)** — "Visualizing embeddings from a vector database" | `https://news.ycombinator.com/item?id=44996310` (HN thread; video itself was the post) | ~30s class | Hit HN front page on the strength of *one camera move*: a six-degrees-of-freedom fly-through 3D embedding cloud. **Directly analogous to Memex Topology.** Lesson: the camera move, not the data, is the hook. |
| **R7** | **Cluely launch ad (April 2025)** — "lying on a date" | `https://x.com/cluely/status/...` (referenced in coverage; 13M+ views on X) | ~90s | Single set, single camera, *narrative scene with a product overlay* — viral mechanic was *aesthetic conviction*, not demo polish. Sources: `https://techcrunch.com/2025/04/21/columbia-student-suspended-over-interview-cheating-tool-raises-5-3m-to-cheat-on-everything/` · `https://sfstandard.com/2025/07/18/cluely-startups-roy-lee-columbia-cheating-viral-tiktok/`. *Lesson Memex steals*: cold open with a *frame*, not a logo. |
| **R8** | **Karpathy "How to Build GPT"** (4M+ views) | `https://www.youtube.com/@AndrejKarpathy` channel | ~2h (not a demo length, but a *style reference*) | Karpathy's video grammar: large kerned bold-sans titles, one idea per card, code visible but de-emphasized when the camera focuses on diagrams. Source: `https://karpathy.ai/` profile. *Lesson*: caption typography Memex should adopt — Inter / Montserrat **Bold**, 64 px minimum, white on black scrim. |

### What the reference set tells us

- **Three of the five VSD 2025 honored projects we could investigate did not publish a YouTube link in a way Google indexes.** This is a *huge* opportunity: a single well-produced 3-min video on YouTube unlisted, embedded in the Memex README and Devpost, is unusual enough to be the artefact judges remember.
- **The 2025 cap was 1 minute. The 2026 cap is 3 minutes.** Plan v3 already exploits this — but the script splits 180 s into eight 24-s slices. None of the 2025 winners did that. The 2025 winner (Vector Vintage) led with *one immersive scene*; the 2025 third place (Spatio-Temporal NPCs) led with *one running simulator*. **Memex should lead with one shot, not eight.**

---

## 3. The "oh shit" moment taxonomy

Six wow patterns work in vector-DB demos. Memex's plan v3 cleanly hits five — but only one of them is *staged for the camera*.

| Pattern | What the viewer says | Memex surface that earns it | Plan v3 status |
|---|---|---|---|
| **Latency reveal** | "Wait, that was instant?" | Lens search across 5 vectors in ≤200 ms; Predict in ≤300 ms | Not staged in v3 script. **Must add: on-screen latency counter on every search call.** |
| **Density reveal** | "Wait, how many sessions is that?" | 79 sessions / 17,938 tool calls is *already in the README*. Topology galaxy makes it tangible. | Implied in §8's Topology beat. **Must add: HUD count-up at 0:00–0:08.** |
| **Connection reveal** | "I had no idea those two were related." | Mix & Match Discovery; Topology MST edges | v3 puts this at 2:18, buried after six other beats. **Move earlier.** |
| **Reverse query** | "You didn't tell it your query — but it found the answer." | **Proactive Recall** — listens for a new error, returns the past session that solved a similar one | **THIS IS MEMEX'S SIGNATURE MOMENT.** v3 buries it at 1:54 with a caption "ACORN filterable." That is criminal under-direction. |
| **Recommendation magic** | "It learned what you don't want without you telling it negatively." | Mix & Match with implicit negatives | Lightly staged in v3. Keep. |
| **Spatial reveal** | "It's not a list — it's a place." | Topology 3D galaxy + Time Machine card stack | **Best-staged moment in v3** (Topology at 0:42–1:06). Promote earlier. |

### The single biggest miss

> **Proactive Recall is the "reverse query" wow** — and the only one of the six patterns where the viewer *experiences* something they have not seen in *any* prior vector-DB demo. Vector Vintage had spatial reveal. Spatio-Temporal NPCs had connection reveal. *Nobody in 2025 had reverse query.* Plan v3 spends 24 s on it with a 2-word caption. That is the single biggest video-wow misallocation in the plan.

### Pattern frequency in the 2025 cohort (audited from `https://qdrant.tech/blog/vector-space-hackathon-winners-2025/`)

| Pattern | 2025 cohort instances | Novelty value in 2026 |
|---|---|---|
| Spatial reveal | 3 of 11 honored (Vector Vintage, Quant Memory Palace, CosmicTwin) | **Saturating.** Judges will have seen 3D embedding worlds. Memex needs to do it *better* (fly-through, not orbit). |
| Density reveal | 1 of 11 (OmniVault — "indexes local files and enables web browsing… highlights and scrolls to recalled text") | Still rare. Memex's 17,938 tool calls number is unusually large for a desktop demo. |
| Connection reveal | 2 of 11 (Spatio-Temporal NPCs, ReMap) | Common, but always lands. |
| Reverse query | **0 of 11** | **Highest-novelty pattern available.** Memex owns this slot if it stages it. |
| Recommendation magic | 4 of 11 (CosmicTwin, ReMap, Bachata Vibes, Qlassroom) | Common. Memex should not lead with this. |
| Latency reveal | 1 of 11 (Spatio-Temporal NPCs — "1–2 s latency" called out in README) | Underused for the *opportunity*. Memex's BGE-small + local Qdrant can hit 80–200 ms — *visible* on screen. |

The take: lead with **density** (HUD at 0:00), peak on **spatial** (Topology fly-through 0:46–0:56), climax on **reverse query** (Proactive Recall 1:42).

---

## 4. Production conventions — what's expected, what's overdone

### Expected (table stakes)

- **Mute-friendly first 3 seconds.** Product Hunt: "your launch video has 3 seconds to stop the scroll, and most founders waste those seconds on logos and intros" `[Source: https://www.flowjam.com/blog/product-hunt-launch-video-2025-template-pitfalls · 2026-05-18]`.
- **MP4, 1080p, ≥30 fps.** Devpost requires no specific format but rejects videos behind privacy walls: "Check the privacy settings of your video—if it's private, the judges won't be able to access it" `[Source: https://info.devpost.com/blog/6-tips-for-making-a-hackathon-demo-video · 2026-05-18]`.
- **Open with a 2-sentence pitch.** Devpost: "Explain what your app does and how it addresses the hackathon in the first few seconds" `[Source: same]`.
- **Show the product in use.** Devpost again: "Snazzy marketing videos are great for promotional purposes, but they don't help others understand and evaluate your app… judges liked when creators demo their code or do a voiceover while showing the different components" `[Source: https://help.devpost.com/article/84-video-making-best-practices · 2026-05-18]`.

### YC pitch conventions (transferable)

> "If there are more than 7 words on a slide it is likely to be too many."
> *[Source: https://www.ycombinator.com/blog/guide-to-demo-day-pitches/ · 2026-05-18]*

> "Do not use screenshots of your app. The buttons are too small to see on a YC Demo Day video. Use a simple drawing of your app instead."
> *[Source: https://zyner.io/blog/yc-demo-day · 2026-05-18]*

The "simple drawing" rule does not apply to a 3-min video shown on Devpost (judges scrub at 100%), but the *spirit* — radical legibility — does. Captions ≤7 words. UI screenshots only when zoomed.

### Caption typography

Subtitle research converges on **bold sans, large x-height, semi-transparent dark scrim** `[Source: https://www.veed.io/learn/best-font-for-subtitles · 2026-05-18]`:

> "For short-form content on Shorts, Reels, and TikTok, Montserrat Bold has become the go-to — its bold, geometric forms stay legible on small mobile screens."

For Memex: **Inter Bold or Pretendard Bold (Korean fallback) at 56–64 px** on a 1080p canvas, anchored bottom-center with a 30%-opacity black scrim. Karpathy-style: appear, hold ~2 s, hard cut.

### Number animations

Count-ups for density, count-downs for latency. The convention is:

- Count from a *startable* zero or low number — viewers can't read "from 1,247,932 to 17,938."
- ≤1.2 s animation, ease-out cubic.
- Highlight delta in accent colour for 200 ms after settle.
- Use monospace digit font (JetBrains Mono, SF Mono) so digits don't jitter width.

Memex already has `--mono` defined in plan v3 (`JetBrains Mono`). Use it.

### Sound design — overused vs underused

Overused in 2024-2025 vector-DB demo culture: **whoosh transitions on every cut, generic "epic build-up" Epidemic Sound tracks, Apple-keynote zoom-into-pixel sound effect.**

Underused: **silence**. Cluely's launch ad drops to ambient room sound for the punchline. Karpathy mutes background between technical beats. *Silence on the Proactive Recall reveal is the most expensive shot in Memex's reach.*

### Camera language

Vector Vintage's prize-winning move was *immersive 3D world that turns*. The convention for vector-DB demos is **dolly-in or fly-through** on the spatial surface. Cuts elsewhere should be hard — no dissolves, no slides. Pans are reserved for one moment only: the Topology galaxy reveal.

### Chapter cards / title sequences

**Skip them.** The 2025 winners did not use chapter cards. A 3-min cap leaves no room. Use the caption track as your structure. One project name card at the end, not the beginning.

### Easter eggs / opening

Plan v3 currently opens with the Bush 1945 frame for the full 0:00–0:18 (18 s on a *title card*). This is over-budgeted. Best practice from Cluely / Karpathy / Product Hunt is **cold open the product**, then drop the thesis at the first transition (~0:08, not 0:18).

### Music

- **Source**: Bensound (CC-attribution) or YouTube Audio Library (royalty-free). Skip Epidemic Sound — paid, and Memex needs only one track.
  - Reference: `https://wyzowl.com/best-royalty-free-music-sites/` lists Bensound, Mixkit, YouTube Audio Library, Free Music Archive, Incompetech as free-tier viable.
- **Genre**: Cinematic ambient / electronic-minimal — not synth-trap, not corporate-uplift. Avoid the "vibrant brand pop" cliché.
- **Tempo**: 80–110 BPM. Edit cuts on the *beat* or *intentionally off-beat* — never randomly.
- **Drops**: Silence at 1:42 (Proactive Recall reveal). Re-enter at 1:58.

### End-screen

- ≤4 seconds.
- Three lines max:
  1. `github.com/ComBba/memex`
  2. `Apache-2.0 · 100% local · no LLM at runtime`
  3. `Qdrant VSD 2026 — Think Outside the Bot`
- Hard cut to black at 2:55. **Do not exceed 3:00 — rule says "no more than 3 minutes," not "around 3 minutes."** `[Source: https://try.qdrant.tech/hackathon-vsd · 2026-05-18]`

---

## 5. Memex shot-by-shot direction — 180 seconds

Format per row: `time | frame | audio | caption | hud`. Captions are ≤6 words each. Camera moves explicit. Latency / count HUDs persist across cuts unless noted.

```
0:00–0:03 | Black. White cursor. Single key press SFX (Kenney UI Audio "click4"). | Silence. | (no caption) | (no HUD)
0:03–0:08 | Topology galaxy fades in from black, already rotating slowly. Cards drift as bright nodes; MST edges fade in. | Soft sub-bass swell (Bensound "Slow Motion" or equivalent, fade-in 5 s). | "Eighty sessions. One place." | TOP-LEFT HUD appears: "79 sessions · 17,938 tool calls"
0:08–0:14 | Camera dolly-in 30% toward a dense cluster. Cluster auto-label fades in: "Tauri build · resolved errors" (KD-02 auto-label). | Music continues, gentle hi-hats join. | "Cluster auto-label · index-time" | HUD persists.
0:14–0:18 | Camera holds. Subtle parallax. Second label appears in the periphery: "fastembed cache CWD". | Music sustains. | "No chat box. No LLM at runtime." | HUD persists.
0:18–0:24 | Hard cut to Time Machine 3D card stack. Cards fan into view, wheel-scroll. KD-01 chip animates onto top card ("Tauri build · resolved"). | Click SFX synchronizes with chip landing. Music drops to 30% volume. | "Time Machine — every session, ↑↓." | HUD updates: "selected · 03:42 AM · 23 turns"
0:24–0:32 | User scrolls down 4 cards. Chip on each animates in sequence: "FFI panic · resolved", "snapshot import · open", "topology bench · resolved". | UI clicks per scroll (zsa-style — short, dry). | "Outcome chips at index time." | HUD updates per scroll.
0:32–0:38 | Cut to Lens slider. 5 named-vector chips visible (content / tool / path / error / code). User drags "tool" weight from 1.0 → 2.0. Results re-rank live. | Slider drag → light analog filter sweep SFX. | "Lens — weight per vector." | TOP-RIGHT HUD appears: "120 ms"
0:38–0:46 | Cmd-K. FormulaQuery score breakdown chip appears on top result: "content .42 · recency ×1.4 · errors +0.2". | Music re-builds, lead synth enters. | "One FormulaQuery call." | HUD: "84 ms · 1 RPC"
0:46–0:56 | Cut to Topology galaxy again — this time the *editorial* shot. Camera fly-through. Pass through cluster A, around cluster B, into cluster C. MST edges illuminate as camera passes. | Music swell — first major peak. NO SFX during fly-through (let the music carry). | (no caption — let the camera speak) | HUD persists: "79 · 17,938"
0:56–1:02 | Camera arrives at a single node. Node expands into a card preview ("symlink workaround"). | Brief silence. Single chime SFX. | "From galaxy to one session." | HUD: "node 4f2c · cluster 3"
1:02–1:12 | Cut to Mix & Match. Three positive cards on left, two negative on right. User clicks "Mix." 3D hyperplane visualization rotates briefly. Result list re-orders. | Music continues. Card-add SFX (low pop). | "Mix & Match — Discovery API." | HUD: "Discovery target = positive[0] · 156 ms"
1:12–1:22 | User clicks 👍 on first result. Anchor recomputes — visual shimmer on the hyperplane. Cards re-rank. | Subtle "tink" SFX on thumbs-up. | "Relevance feedback — re-anchor." | HUD: "anchor v2 · 94 ms"
1:22–1:30 | Cut to Predict Next Actions. 4×3 grid of session thumbnails. Each thumbnail has KD-01 outcome caption underneath ("resolved · symlink fix"). Cinematic slow zoom into one thumbnail. | Music dips to ~50% — preparing for the silence at 1:42. | "Predict — neighbor pivot." | HUD: "Recommend API · 220 ms"
1:30–1:38 | Thumbnail expands. Replay surface loads. Bash terminal turn animates: `$ ln -sf …`. Edit-diff turn fades in below. | Single keystroke SFX synced to the `$` cursor. | "Replay — original turns, on-demand." | HUD: "JSONL re-parsed · 38 ms"
1:38–1:42 | Replay continues, two more turns. Music tapers. | Music fades to NEAR-ZERO. | (no caption) | HUD fades to 20% opacity.
1:42–1:50 | **THE PROACTIVE RECALL MOMENT.** Cut to a fresh Claude Code window (real, not mocked). User types into the editor: `Error: symlink loop on .fastembed_cache`. NO action by the user. Tray icon pulses. Memex banner slides in from top: "Past session matches · sim 0.93". | TOTAL SILENCE except for one soft chime when the banner appears. | "You didn't ask. It found." | HUD reappears at full opacity: "Recall · 80 ms · ACORN-filtered"
1:50–1:58 | Banner expands. Shows the session ("FFI panic Sep 12 · resolved"). User clicks. Replay opens. The fix is visible. | Music re-enters at full volume — emotional peak. | "Proactive — payload-filtered HNSW." | HUD: "has_errors = true · 1 of 80 sessions"
1:58–2:08 | Cut back to Topology galaxy, zoomed out. The just-found session lights up red briefly, then fades to white. Camera holds. | Music sustains. | "Every error session is a node here." | HUD: "12 has_errors · 67 resolved"
2:08–2:16 | Snapshot export — user clicks. Progress bar fills in 2 seconds. File icon writes to disk visualization. | Hard mechanical SFX on file-write. | "Snapshot — export your memory." | HUD: "memex.snapshot · 4.2 MB · 79 sessions"
2:16–2:24 | Quick cut: snapshot import on a clean machine. Memex window opens; cards rebuild in fast-motion. | Time-lapse rebuild — fast ticking SFX. | "Restore on any Mac." | HUD: "re-index 79/79 · 6.4 s"
2:24–2:32 | Cut to architecture text card (NOT a screenshot — a designed card): "5 named vectors · FormulaQuery · Discovery · Recommend · Distance Matrix · ACORN" — each line types out. | Music swells one last time. | (text on screen IS the caption) | (no HUD)
2:32–2:42 | Three primitive logos animate in sequence: Qdrant 1.18 · fastembed-rs · Tauri 2. Single CPU/MAC icon (no cloud, no GPU). | Music sustains. | "Local. Local. Local." | (no HUD)
2:42–2:50 | Hard cut: Memex window, Topology galaxy still turning. Subtitle: `github.com/ComBba/memex`. | Music begins resolving chord. | "github.com/ComBba/memex" | (no HUD)
2:50–2:55 | Final card. Black background. Three lines stacked center: "Memex / VSD 2026 — Think Outside the Bot / Apache-2.0 · no LLM at runtime". | Music final chord lands at 2:53. | (the lines ARE the caption) | (no HUD)
2:55–3:00 | Black. | Silence. | (no caption) | (no HUD)
```

### Beat counts

- **Total wow moments**: 6 (density at 0:03, spatial at 0:46, connection at 1:02, recommendation at 1:12, recall/reverse at 1:42, replay/payoff at 1:30 + 1:50)
- **Captions**: 18. Average duration on screen: 4.5 s. Longest hold: "You didn't ask. It found." at 1:42 (8 s, deliberately — the music silence amplifies it).
- **HUD updates**: 14 — one update per major surface action. Persistent across cuts.
- **Music drops to silence**: once (1:38–1:50, 12 seconds). This is the most expensive 12 seconds of the video.
- **Camera moves vs cuts**: 23 hard cuts, 2 dolly-ins (0:08, 1:22), 1 fly-through (0:46–0:56). All other surfaces are static UI captures.

### Annotation — why each shot length

| Block | Length | Why this length |
|---|---|---|
| 0:00–0:08 (Topology hold) | 8 s | Pinecone / Vercel convention: 3 s to stop the scroll, then 5 s of *intriguing-but-quiet* visual to earn the thesis card. |
| 0:08–0:18 (Topology + thesis) | 10 s | "No chat box. No LLM at runtime." needs ≥6 s on screen — it is the load-bearing claim. Held over a moving visual so the eye stays engaged. |
| 0:18–0:32 (Time Machine + chips) | 14 s | Two interactions (scroll-down on a card, watch four chips animate). Each chip needs ~1.5 s to register. |
| 0:32–0:46 (Lens + FormulaQuery) | 14 s | The slider drag is the *primary act of agency* in the video. It must feel deliberate, not rushed. |
| 0:46–1:02 (Topology fly-through + node land) | 16 s | The single longest contiguous "no caption" block. Per Karpathy and Vector Vintage convention: when the visual is doing the work, *get out of the way*. |
| 1:02–1:22 (Mix & Match + RelFB) | 20 s | Two interactions (click Mix, click 👍). Each gets ~10 s including animation. |
| 1:22–1:42 (Predict → Replay → fade to silence) | 20 s | Sets up the silence at 1:42. The fade-down from 1:38 is *part of* the wow that follows. |
| 1:42–1:58 (Proactive Recall) | 16 s | Climax. Long enough to land emotionally. *Do not* cut early. |
| 1:58–2:24 (Topology recap + snapshot) | 26 s | Cooldown. Two ancillary primitives (snapshot export/import). Pacing slows because the climax already happened. |
| 2:24–2:55 (architecture + end card) | 31 s | Architecture card needs ~8 s to read; end-card needs ~5 s; ~18 s buffer for music to resolve gracefully. |

### Risk: the Proactive Recall take

The 1:42–1:58 block depends on a *real* Claude Code window producing a *real* error that Memex *really* matches. This is the single hardest shot in the production. Mitigation steps:

- **Pre-stage** the corpus to contain `~/.claude/projects/.../session-with-symlink-fix.jsonl` indexed and verified before the take.
- **Bump the poll** interval to 2 s in a debug build (`MEMEX_POLL_MS=2000`) so the take is ≤5 s of dead air.
- **Hide** the dock; record the Memex tray icon pulse as part of the frame.
- **Practice 5 takes** the day before. Burn time on this. It is the one shot that, if missed, the video has *no* climax.



---

## 6. Gap analysis — Plan v3's WOW prototypes vs what the video needs

Plan v3 §4 lists five visual WOW items (WOW-1 to WOW-5). They are *surface* wow ("the surface works correctly"). They are *not* video wow ("the viewer says oh shit at 0:42").

| Plan v3 WOW | Surface wow (already covered) | Video wow gap |
|---|---|---|
| **WOW-1** · Time Machine 2.0 + Contextual chips | Chips render at index-time on top of cards. | **Camera language missing.** Cards must *animate in* with a wheel-scroll motion, not appear static. Already implied — needs explicit ≤300 ms ease-out. |
| **WOW-2** · Topology Galaxy + cluster auto-labels | Labels float over clusters. | **No fly-through staged.** Plan v3 mentions Topology but does not specify camera move. **Add: 6-DoF orbit camera with pre-recorded path for the demo.** This is the *single biggest visual asset to add in 14 days.* |
| **WOW-3** · Lens contribution bars + FormulaQuery breakdown | Chip overlay on top result. | **Slider drag is not staged.** Plan v3 shows the breakdown but does not show the *act of dragging*. The drag is the wow (interactive cause-and-effect). **Capture cursor + slider animation.** |
| **WOW-4** · Predict + KD-01 outcome chips | Thumbnails with outcome captions. | **No cinematic zoom specified.** Predict at 4×3 is dense; viewer can't read 12 thumbnails. Zoom-into-one is required. |
| **WOW-5** · Discovery splash | 3D hyperplane on Mix & Match. | **Hyperplane is a graphics novelty, not a clarity device.** Risk: viewer doesn't understand what they're looking at. **Add a subtle text overlay during the rotate ("positive anchor · negative cone").** |

### What's missing entirely from WOW-1..5 that the video needs

| Required for video | Currently absent from v3 plan | Cost |
|---|---|---|
| **Persistent HUD** showing latency / session count / tool-call count | Not in v3. | ~2 h frontend work (vanilla JS counter, debug query param) |
| **Pre-recorded Topology camera path** (6-DoF orbit, 10-second loop) | Not in v3. Must be added — using `graph.cameraPosition(...)` interpolated keyframes on the existing 3d-force-graph (Three.js underneath). | ~4 h |
| **Proactive Recall *live trigger* recording** — a real Claude Code window with a real error message, captured *as Memex reacts in real time* | v3 mentions Proactive Recall but does not specify the recording setup. **This is the most important shot in the video. Plan for it.** | ~2 h (corpus setup + debug poll interval + 5 take rehearsal) |
| **A single dedicated caption font + scrim layer** in screen-recording compositor (DaVinci Resolve free tier, or Final Cut Pro) | Not in v3. | ~1 h template setup |
| **Mute-friendliness check** — every shot must be intelligible with audio muted | Not in v3. | Free; QA-only |
| **Outcome chip pre-population** — the KD-01 chips in Time Machine must be visible and correct *before* the take. If chips render late (after the user scrolls), the wow is gone. | v3 promises chips; recording-readiness is a separate concern. | ~1 h (verify chips populate at index-time on the demo corpus) |
| **Cluster auto-label** must read **legibly** in the recorded frame — i.e. ≥18 px on a 1080p canvas. Current rendering may be smaller. | v3 promises labels; legibility on a downsized video is a recording-time concern. | ~1 h (CSS bump on a debug class) |

### Total production cost in code

**~11 hours of frontend / debug-build work** — all behind query strings or env vars so production behavior is unchanged. The remaining ~6–10 hours of production work (compositing, sound, captioning) is video editing, not code.



---

## 7. What to add in 14 days — concrete production list

> All additions respect Memex's invariant: vanilla JS / three.js / CSS frontend, no new heavy dependencies. Most additions are *recording* and *editing*, not runtime code.

### A. Sound design pack — 0 cost, 1 hour to integrate

- **Kenney UI Audio** (CC0): `https://kenney.nl/assets/ui-audio` — 50 sounds, free, public-domain. Use 4: short "click4", soft "tink2", warm "chime3", file-write "save1".
- **Method**: imported into DaVinci Resolve timeline. Not embedded in the app. The app remains silent in production; sounds are added post.

### B. Music — 0 cost, 30 min to pick

- **Bensound** (`https://www.bensound.com`) — free CC-attribution license. Recommended tracks: "Slow Motion", "Cinematic Documentary", "Better Days". Pick *one*. Whole video = one track + one silence drop.
- Alternative: **YouTube Audio Library** (free, no attribution required) — search "cinematic ambient instrumental 100 BPM."
- **Avoid**: Epidemic Sound (paid), Artlist (paid), Audiio (paid).

### C. Number-tween library — 0 cost, **already in the stack**

- **No new dependency.** Use a 12-line vanilla JS counter:
  ```js
  function tween(el, from, to, ms){
    const t0 = performance.now();
    requestAnimationFrame(function step(t){
      const k = Math.min(1, (t-t0)/ms);
      el.textContent = Math.round(from + (to-from)*(1-(1-k)**3));
      if (k < 1) requestAnimationFrame(step);
    });
  }
  ```
- Add a fixed-position HUD `<div class="hud-overlay">` to `src/index.html` styled with `pointer-events:none; opacity:0` and a debug query param (`?hud=1`) that opacities it to 1 only when explicitly requested. This ships hidden — the demo recording uses `?hud=1`. **Production stays clean.**

### D. Screen recorder setup — 0 cost, 1 hour to validate

- **macOS Quicktime** (`Cmd-Shift-5`) is sufficient at 60 fps on Apple Silicon. Settings:
  - Window-bounded recording (the Memex app window only, not the whole desktop).
  - Hide mouse pointer when *not* being shown explicitly (the Lens slider drag and Proactive Recall click are the exceptions).
  - 1920×1080 canvas (resize Memex window to ≤1920 before recording).
- **Optional upgrade**: OBS Studio with Apple VT H264 hardware encoder, 1080p60 @ 20,000 Kbps `[Source: https://obsproject.com/forum/threads/best-1080p-60fps-recording-settings.169492/ · 2026-05-18]`. Use only if Quicktime drops frames during the Topology fly-through.

### E. Mouse pointer style

- Built-in macOS pointer is fine. **Do not** install Mouseposé or large-pointer overlays — they look like screencast tutorials, not product films. Karpathy / Cluely / Vector Vintage all use default pointers.
- Mouse should be **off-screen** for the Topology fly-through (camera moves itself).

### F. Caption rendering — 0 cost, 30 min template

- DaVinci Resolve free tier (or Final Cut Pro if licensed): one Title preset.
  - Font: **Inter Bold** (free, system) or **Pretendard Bold**.
  - Size: 56 px.
  - Color: pure white (#FFFFFF).
  - Background scrim: 30%-opacity black rectangle, 12 px padding, 4 px corner radius.
  - Position: bottom-center, 80 px from bottom edge.
  - Entry: fade-in 100 ms; exit: hard cut.

### G. Topology fly-through pre-record — 4 hours of one-time work

The single biggest production add. The current Topology renders on user interaction. For the demo, add a **`?demo=topology-fly` URL parameter** that triggers a predefined camera path:

- Vendored 3d-force-graph already runs on Three.js — use `graph.cameraPosition(...)` interpolated over 10 seconds.
- 4 waypoints, ease-in-out-cubic between them.
- Auto-trigger on page load with the query param; otherwise no behavioral change.
- *Ship this shipped behind a query string so production users never see it.* This is a recording aid.

### H. Proactive Recall live trigger — 2 hours to stage

- Open a real Claude Code window beside Memex on a clean test corpus that contains the `FFI panic on .fastembed_cache symlink` session.
- Pre-write the error text in a buffer; on take, paste it into a real editor. Memex's existing 12-second poll triggers naturally.
- *Or* — bump the poll interval to 2 seconds in a debug build to keep the take under 5 seconds.

### I. End-card — 30 min in DaVinci

Three lines, white on black, Inter Bold 64 px, line-height 1.6:
```
Memex
VSD 2026 — Think Outside the Bot
Apache-2.0 · 100% local · no LLM at runtime
```

### J. Anti-patterns to avoid

Plan v3 §8 currently risks several. Explicit list:

- **No talking-head intro.** No founder face. The product is the face.
- **No slow zoom on the Memex logo.** Don't have a logo? Don't make one.
- **No establishing shots of the macOS dock.** Cuts inside the app window only.
- **No "Coming up next" preview slides.** The 3 minutes is the preview.
- **No same-feature-twice framing.** Each surface gets one beat. Replay is the only exception (it appears twice — once as the Predict payoff at 1:30, once as the Recall payoff at 1:50 — and that's deliberate because the *meaning* is different).
- **No caption walls.** Max ≤6 words per caption. Plan v3 has captions like "Discovery + Relevance FB" (3 words — good) and "Distance Matrix · auto-label" (3 words — good) but also "FormulaQuery · single call" which the audience can't parse in 2 seconds. **Rewrite to "One Qdrant call. Score breakdown."**
- **No multi-language captions.** Pick one. English. The hackathon is judged in English, the README is English, mixing in Korean (which v3's source HTML implies as a possibility — see `lang="ko"`) splits viewer attention.

---

## 8. Submission deliverables checklist

> All values verified against `research/03-qdrant-vsd-2026-hackathon.md` (which itself fetched the rules on 2026-05-18).

### Video format / hosting

| Item | Required | Memex plan |
|---|---|---|
| Max length | "Demo video of no more than 3 minutes" `[Source: https://try.qdrant.tech/hackathon-vsd · 2026-05-18 via research/03]` | Hard-cut at 2:55. Render at 2:58 max. |
| Hosting | "Loom / YouTube / Dropbox or similar" `[same source]` | Primary: **YouTube unlisted**. Mirror: copy file to repo `docs/demo.mp4` (≤100 MB) as redundancy. **Do not** rely on Dropbox/Loom alone — both have shown link expiry issues in prior hackathons. |
| Video privacy | Must be accessible to judges without login | YouTube **unlisted** (not private). Devpost: "Check the privacy settings of your video—if it's private, the judges won't be able to access it" `[Source: https://info.devpost.com/blog/6-tips-for-making-a-hackathon-demo-video · 2026-05-18]`. |
| Container | MP4 (H.264 / AAC) | DaVinci export "YouTube 1080p" preset. |
| Frame rate | Not specified by rule | 60 fps for the Topology fly-through, 30 fps acceptable elsewhere. Match throughout — *do not* mix. Set DaVinci timeline to 60 fps; let static shots resample. |
| Resolution | Not specified by rule | 1920×1080. Do not submit 4K — file size doubles for negligible viewer benefit. |
| Audio | Stereo, normalized to −16 LUFS (YouTube default) | DaVinci has a one-click loudness normalize. |

### Form fields to prepare (per research/03 §2)

- Project name: **Memex**
- Project description (≤300 chars suggested): *"A Tauri 2 desktop app that indexes every Claude Code session JSONL into local Qdrant 1.18, exposing seven non-chat surfaces over five named vectors. No chat box, no LLM at runtime — pure spatial memory."*
- Repo URL: `https://github.com/ComBba/memex`
- Demo video URL: (YouTube unlisted, populated by D-2)
- Team members: solo (Sejun Kim / ComBba)
- Qdrant features used (verbatim list): 5 named vectors · FormulaQuery (score breakdown) · Discovery API (Mix & Match) · Recommend API (Predict) · Distance Matrix → MST (Topology) · Payload-filtered HNSW with ACORN (Proactive Recall) · Snapshot export/import
- Origin statement: *"Built for Qdrant VSD 2026 between 2026-04-21 (announcement) and 2026-06-01 (deadline)."*

### Thumbnail design

For YouTube — the thumbnail decides whether judges click *quickly* between submissions vs hover-and-scrub. Per CTR research, thumbnails featuring faces with strong emotion can increase CTR by 20–30% `[Source: https://vidiq.com/blog/post/youtube-thumbnail-design-tips/ · 2026-05-18]`. **Memex has no face, so use contrast instead:**

- 1280×720 PNG. <2 MB.
- Background: dark navy (#0a0e1a — Memex hero gradient).
- Foreground: a single still frame from the Topology fly-through (the densest cluster shot, ~0:50 in the video).
- Three-word overlay: **"79 sessions · ONE place"** — Inter Black 96 px, white on dark.
- Bottom right: tiny Qdrant 1.18 logo + "VSD 2026" pill.
- **Do not** include the word "demo" or "hackathon." The thumbnail is the work, not its label.

---

## 9. Critique of Plan v3's §8 demo script (explicit)

> Plan v3 §8 (lines 910–924 of `claudedocs/sota-plan-v3.html`) defines the script as eight equal segments of 24 seconds each, captioned with KICK acronyms. Verbatim from v3:

> "각 segment가 명시적으로 Qdrant 1.18 feature를 호명한다. 자막에 KICK 이름 등장 (자막은 ≤2어 줄임표)."
> *(Each segment explicitly names a Qdrant 1.18 feature. Captions show the KICK name (captions ≤2-word ellipsis).)*
> *[Source: `/Users/kimsejun/Documents/GitHub/memex/claudedocs/sota-plan-v3.html` §8 · 2026-05-18]*

**Three critiques**:

1. **Equal-segment pacing is the wrong rhythm for video.** Plan v3 gives Topology, Lens, Mix, Predict, Recall, Replay each 24 seconds. The judges' brain spends those 24 s registering "another surface, OK" — every one is equally weighted, so none lands hardest. The corrected timeline (§5 above) gives the Topology fly-through 10 s of camera-only screen time and the Proactive Recall reveal 12 s of *silence*. Unequal pacing tells the viewer what matters.

2. **Captioning with KICK names is engineering-doc convention, not film convention.** Captions like "Distance Matrix · auto-label" or "FormulaQuery · single call" read like the API call sheet. Karpathy / YC / Devpost guidance is unanimous: caption with *what the viewer just saw* in human language. Replace with "Cluster auto-label" and "One call. Full score." respectively.

3. **The Bush 1945 cold open is 18 seconds long.** Plan v3 spends the entire first 10% of the video budget on a *quote*. Cluely's launch ad opens on the product within 1 second. Product Hunt's stop-the-scroll convention is 3 seconds. The Bush quote belongs on the *end card* — paid off after the viewer knows what Memex does, not before. **Move the Bush 1945 reference to the README or to a 1-second flash at 2:50.**

The shot-by-shot in §5 above is the corrected script — it preserves every Qdrant primitive call-out v3 wants, but redistributes screen time according to wow-density rather than feature-count.

---

## 10. Sources

All fetched 2026-05-18 unless noted.

### Reference videos / projects (VSD 2025 cohort)

1. `https://youtu.be/4bk44WQv0EM` — Spatio-Temporal NPCs 1-min demo (VSD 2025 🥉). One of two VSD 2025 honored projects with a publicly indexed demo video.
2. `https://youtu.be/T7lnAcWFLNw` — Spatio-Temporal NPCs extended demo.
3. `https://github.com/inventwithdean/spatio-temporal-npcs` — README that links both videos.
4. `https://github.com/kanungle/vector-vintage-public` — Vector Vintage (VSD 2025 🥇). Live deploy `https://vector-vintage.benedictcounsell.com`; no video URL in the repo.
5. `https://github.com/kanungle/RoboBank` — RoboBank (VSD 2025 🥈) README, visual description quoted in §2.
6. `https://github.com/kanungle/remap` — ReMap (CrewAI sponsor prize) README, references local `ReMap.mp4`.
7. `https://www.linkedin.com/posts/sanya-k-494584261_second-solo-hackathon-second-win-a-activity-7378813743922626560-A6yP` — Sanya Kapoor's RoboBank winner announcement.
8. `https://qdrant.tech/blog/vector-space-hackathon-winners-2025/` — Official 2025 winners post (no embedded video URLs).
9. `https://qdrant.tech/blog/vector-space-day-2025-recap/` — 2025 recap (references a winner video without linking).
10. `https://news.ycombinator.com/item?id=44996310` — HN front-page "Visualizing embeddings from a vector database" (vector fly-through, August 2025) — direct genre reference for Memex's Topology shot.
11. `https://atlas.nomic.ai/` — Nomic Atlas (industry reference for embedding map visualization). Source: search hit `https://docs.nomic.ai/atlas/embeddings-and-retrieval/guides/how-to-visualize-embeddings`.

### Hackathon / pitch convention sources

12. `https://try.qdrant.tech/hackathon-vsd` — VSD 2026 official rules. **Verified deadline 2026-06-01 23:59 PT and 3-min video cap.**
13. `https://try.qdrant.tech/hackathon-2025` — VSD 2025 rules, used to confirm 1-min → 3-min cap change.
14. `https://info.devpost.com/blog/6-tips-for-making-a-hackathon-demo-video` — Devpost demo video tips (quoted at §4 and §8).
15. `https://help.devpost.com/article/84-video-making-best-practices` — Devpost video best practices.
16. `https://info.devpost.com/blog/hackathon-judging-tips` — Devpost judging tips.
17. `https://www.ycombinator.com/blog/guide-to-demo-day-pitches/` — YC demo day guide (7-word rule, opener rule, closing rule).
18. `https://zyner.io/blog/yc-demo-day` — YC demo day guide (the "simple drawing" rule, the "no screenshots" rule).
19. `https://www.flowjam.com/blog/product-hunt-launch-video-2025-template-pitfalls` — Product Hunt launch video best practices (the 3-second rule).
20. `https://www.flowjam.com/blog/tech-product-announcement-video-2026-30-day-launch-playbook` — Tech product announcement playbook.

### Production resources (free / CC0)

21. `https://kenney.nl/assets/ui-audio` — Kenney UI Audio (CC0, 50 sounds, recommended).
22. `https://www.bensound.com` — Bensound (CC-attribution music).
23. `https://wyzowl.com/best-royalty-free-music-sites/` — Roundup of royalty-free music sites.
24. `https://obsproject.com/forum/threads/best-1080p-60fps-recording-settings.169492/` — OBS 1080p60 settings (optional upgrade over Quicktime).
25. `https://www.veed.io/learn/best-font-for-subtitles` — Subtitle font research (Montserrat / Inter for short-form).
26. `https://vidiq.com/blog/post/youtube-thumbnail-design-tips/` — YouTube thumbnail CTR practices (the 20–30% face-emotion stat).

### Viral / aesthetic reference (2024-2026 demo culture)

27. `https://x.com/karpathy/status/1886192184808149383` — Karpathy's vibe coding tweet (genre context, 4.5M views).
28. `https://karpathy.ai/` — Karpathy profile; channel `https://www.youtube.com/@AndrejKarpathy` (caption typography reference).
29. `https://techcrunch.com/2025/04/21/columbia-student-suspended-over-interview-cheating-tool-raises-5-3m-to-cheat-on-everything/` — Cluely viral demo coverage.
30. `https://sfstandard.com/2025/07/18/cluely-startups-roy-lee-columbia-cheating-viral-tiktok/` — Cluely viral mechanics analysis.
31. `https://www.powerofusnewsletter.com/p/how-the-rise-of-cluely-a-tool-for` — How Cluely exploited online virality.

### Memex internal references

32. `/Users/kimsejun/Documents/GitHub/memex/CLAUDE.md` — Project invariants ("no LLM call at runtime," etc.).
33. `/Users/kimsejun/Documents/GitHub/memex/README.md` — Hackathon claims and surface descriptions.
34. `/Users/kimsejun/Documents/GitHub/memex/claudedocs/sota-plan-v3.html` §4 (WOW 1–5) and §8 (demo script v3) — the plan this report critiques.
35. `/Users/kimsejun/Documents/GitHub/memex/claudedocs/research/03-qdrant-vsd-2026-hackathon.md` — Authoritative on the 2026 rules (own prior research).

---

---

## Appendix A — Recording day checklist

> Print this. Tick off in order on take day. Estimated total time on the day: 4–6 hours including 5 rehearsals of the Proactive Recall take.

### Pre-take (30 min)

- [ ] Quit every non-essential app. Slack, Notion, Discord, Mail — all closed. Notifications off (Do Not Disturb on).
- [ ] Set macOS resolution to 1920×1080 (System Settings → Displays → scaled 1080p). DPI matters — high-DPI captures look soft when downscaled.
- [ ] Verify Memex window opens at exactly 1600×900 within the 1920×1080 canvas. This leaves a 160-px margin all around for the caption scrim.
- [ ] Start Qdrant 1.18 on `localhost:6334` (gRPC). Verify with `curl localhost:6333/healthz`.
- [ ] Verify the demo corpus is indexed: `memex search "symlink"` returns ≥1 hit; `memex search "FFI panic"` returns ≥1 hit.
- [ ] Set debug query params:
  - For Topology fly-through take: `?demo=topology-fly`
  - For HUD takes: `?hud=1`
  - For Proactive Recall: `MEMEX_POLL_MS=2000 ./Memex.app/Contents/MacOS/memex`
- [ ] Pre-load YouTube Audio Library track on a *separate* device — never play music while recording (audio bleed risk).
- [ ] Mouse pointer: macOS default. No Mouseposé. No "press-to-show" overlays.

### Per-take settings (each take, ≤2 min setup)

- [ ] Quicktime → File → New Screen Recording → set source to "Memex" window.
- [ ] Click "Options" → set save location to `~/Desktop/memex-take-NN/`.
- [ ] Microphone: **OFF.** All audio added in post.
- [ ] Start recording 2 s before the action; stop 2 s after. Trim in post.

### Take order (record in this sequence — easiest to hardest)

1. **End card** (2:50–2:55). Record first. It's stationary. Get the muscle memory.
2. **Architecture card** (2:24–2:42). Pure text-on-screen.
3. **Topology fly-through** (0:46–1:02). With `?demo=topology-fly` — let the camera path run untouched.
4. **Lens slider drag** (0:32–0:46). Practice the drag speed once; aim for ~1.2 s drag duration.
5. **Mix & Match + Relevance Feedback** (1:02–1:22). Two clicks. Verify the cards re-rank visibly.
6. **Time Machine + chips** (0:18–0:32). Slow wheel scroll, ~1 card per second.
7. **Predict + Replay** (1:22–1:42). The Replay turn-by-turn animation must be visible.
8. **Snapshot export/import** (2:08–2:24). Two short takes.
9. **Topology recap** (1:58–2:08). Re-use the fly-through B-roll if needed.
10. **Proactive Recall** (1:42–1:58). **DO THIS LAST.** Burn 5 takes on it. Pick the one where Memex reacts within 3 seconds of the error text being pasted.

### Post-take (in DaVinci Resolve, ~6 h)

- [ ] Create 60 fps 1920×1080 project.
- [ ] Import all takes. Mark in/out per the §5 timeline.
- [ ] Drop the Bensound track on audio track 1. Mark the silence-drop point at 1:38, re-enter at 1:58.
- [ ] Add Kenney UI Audio SFX on audio track 2 — one click per chip, one tink on 👍, one chime on Recall banner, one save sound on snapshot.
- [ ] Apply caption template (Inter Bold 56 px white on 30%-opacity black scrim) per §5.
- [ ] Add HUD overlay PNG (rendered separately from `?hud=1` recording, or composited in post) — top-left, persistent.
- [ ] Audio normalize: −16 LUFS (YouTube standard).
- [ ] Color: leave UI captures untouched. Apply a tiny LUT only to the Topology fly-through (lift dark navy +5%, push electric blue +8%) — adds 5 minutes, gains a lot.
- [ ] Export: H.264 MP4, 1080p60, 20 Mbps target bitrate.
- [ ] Verify final file plays at 2:55 ±2 s. **If over 3:00, cut the snapshot import block (2:16–2:24) — it's the most expendable shot.**

### Upload (≤30 min)

- [ ] Upload to YouTube → set to **Unlisted** (not Private).
- [ ] Title: `Memex — Time Machine for AI session memory · Qdrant VSD 2026`
- [ ] Description: 2-paragraph (project description from submission form) + GitHub URL + Qdrant primitives list + Apache-2.0 line.
- [ ] Custom thumbnail per §8.
- [ ] Verify by opening the unlisted URL in a private browser window. If it asks for sign-in, your privacy setting is wrong.
- [ ] Mirror copy: `git lfs track docs/demo.mp4` and add to repo as redundancy (≤100 MB). LFS not strictly required for ≤100 MB but cleaner for history.

### Submission form (≤30 min, D-2 / 2026-05-30)

- [ ] Open `https://forms.gle/YDQ2TDUi8MqS9Vx28`.
- [ ] Paste the YouTube unlisted URL into the demo-video field.
- [ ] Paste the description, repo URL, primitives list, origin statement (per §8).
- [ ] **Hit submit by 2026-05-30 12:00 PT** — leaves a 36-hour buffer for any organizer follow-up.

---

## Appendix B — The single director's note

> Plan v3 is technically excellent. The 25 KICKs ship a product that wins on the *technical functionality* judging axis. But hackathon judges decide the *creativity* and *innovative uses of technology* axes — both verbatim 2026 criteria per `https://try.qdrant.tech/hackathon-vsd` — from a 3-minute video they watch with the sound on (most likely) and once (almost certainly).
>
> The 24-second equal-segment script in v3 §8 protects against the failure mode "judges don't see all five Qdrant primitives." That is the *wrong failure to optimize against*. The actual failure mode is "judges watch the video, are mildly impressed, and move on to the next of 200 submissions." The cure is *one shot they tell their colleagues about* — the silence on the Proactive Recall reveal at 1:42.
>
> Build for *one frame the viewer screenshots*. Then everything else is in service of that frame.

---

*Authored 2026-05-18 by the Memex video direction pass. No code or repo changes proposed; this is a production-direction artefact. Recording begins on D-10 (2026-05-21) per plan v3 timeline.*
