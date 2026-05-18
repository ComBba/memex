# 03 — Frontend × Tauri Integration

Read-only analysis of the Memex desktop-app frontend: `src/index.html`, `src/main.js` (1524 LOC), `src/styles.css` (1579 LOC), the vendored `3d-force-graph.min.js`, and the IPC contract surface exposed by `src-tauri/src/commands.rs` (503 LOC) and registered in `src-tauri/src/lib.rs`.

---

## 1. Snapshot

Memex's frontend is **plain vanilla HTML/CSS/JS, ESM module, zero build step, zero framework, zero TypeScript**. It is hosted inside a Tauri 2 webview that serves the directory directly (`tauri.conf.json:7`, `"frontendDist": "../src"`) and exposes the IPC bridge as a global because `withGlobalTauri: true` (`tauri.conf.json:10`). The only declared JS dependency is `@tauri-apps/cli` for the dev/build pipeline (`package.json:9-11`); no runtime dependency is installed via npm.

Why this stance — citing repo authorship rather than inferring:

- CLAUDE.md:102 — *"`src/{index.html, main.js, styles.css}` — vanilla HTML/CSS/JS, no framework"* — explicit invariant.
- CLAUDE.md:120 — *"`src/index.html` + `src/main.js` (1524 LOC) + `src/styles.css` is plain vanilla — no bundler, no TS, no framework. `tauri.conf.json` sets `withGlobalTauri: true` so the bridge is at `window.__TAURI__.core.invoke`."*
- README.md:369 — listed in the stack: *"`vanilla HTML/CSS/JS` · `Tauri 2 webview` · `3d-force-graph` (Three.js) for topology · CSS 3D `translateZ` for the Time Machine layered stack"*.

The two heavyweight dependencies are vendored rather than fetched: `src/vendor/3d-force-graph.min.js` (706 907 bytes, includes Three.js bundle) and is loaded with a non-module `<script>` tag at `src/index.html:151`. The app's own logic is loaded as `<script type="module">` at `src/index.html:152`.

**The seven surfaces this single-page frontend must host** (README.md:95–109):

| # | Surface | Where it lives |
|---|---|---|
| 1 | Time Machine layered stack | `<section class="results">` — `index.html:55-59` |
| 2 | Topology galaxy | `<dialog id="topology-modal">` — `index.html:66-83` |
| 3 | Mix & Match | `<dialog id="mix-modal">` — `index.html:85-107` |
| 4 | Proactive recall | `<div id="recall-banner">` — `index.html:134-144` |
| 5 | Predict next-action | injected into the inspector by `main.js` — see `renderInspector` at `main.js:643-654` (slot `#prediction-panel`) |
| 6 | Replay engine | `<dialog id="replay-modal">` — `index.html:109-132` |
| 7 | Lens slider | `<div id="lens-sliders">` in the left sidebar — `index.html:39` |

There is also a small **plus**: snapshot export trigger on the topbar (`index.html:27`). Snapshot **import** has a backend command but **no UI** (see §4).

Why no framework is the right call here, in one line: the app has exactly one route, one window, no shared component reuse across surfaces, and ships as a single binary — adding React/Vue would inflate the .app bundle by ~50× more JS than the entire current frontend without unlocking anything the surfaces actually need.

---

## 2. DOM structure

`src/index.html` is short (154 LOC) and the regions map cleanly onto surfaces. Lines below refer to `src/index.html`.

```
<header class="topbar">                                  (10-30)
  ├─ topbar-left:   brand + status pill                  (11-14)
  ├─ topbar-search: ⌘K search input (the only text input on the global chrome)  (15-23)
  └─ topbar-actions: Topology / Mix / Snapshot / Re-index buttons   (24-29)

<main class="layout">                                    (32-64)
  ├─ <aside class="sidebar">                             (33-53)
  │   ├─ "Lens" title + slider container #lens-sliders   (34-40)   → surface 7
  │   └─ "Recall" title + textarea + "Recall fixes" btn  (42-52)   → manual recall (companion to surface 4)
  │
  ├─ <section class="results" id="results">              (55-59)   → surface 1 (Time Machine)
  │                                                                  also reused for surface 7 hit cards
  │
  └─ <aside class="inspector" id="inspector">            (61-63)   → surface 5 host (predict panel)
                                                                     + read-only payload kv view

<dialog id="topology-modal">                             (66-83)   → surface 2
  ├─ #topology-canvas (3d-force-graph mount)              75
  ├─ #topology-stats   (project/session/bridge counters)  77-78
  └─ #topology-legend  (project rows + gap cards)         80

<dialog id="mix-modal">                                  (85-107)  → surface 3
  ├─ #mix-positive dropzone                              97-99
  ├─ #mix-negative dropzone                              101-103
  └─ #mix-results list                                   105

<dialog id="replay-modal" class="replay-modal">          (109-132) → surface 6
  ├─ controls: ⏮ ▶ ⏭ + speed select                     113-122
  ├─ #replay-list  (turn picker, scrollable)             127
  └─ #replay-detail (turn body, terminal + diffs)        128-130

<div id="recall-banner" class="recall-banner hidden">    (134-144) → surface 4

<footer class="statusbar">                               (146-149)
  ├─ #status-text
  └─ #status-latency
```

Two things to note:

- **No empty wrapper divs** — the structure is tight; 154 lines is roughly half hand-written content and half whitespace. The author has resisted the temptation to scaffold "containers for the framework to grab."
- **The seventh surface (Lens) doesn't get its own modal** — it's a permanent sidebar surface that always biases the result panel. Architecturally it is "a control that mutates `state.weights`," and that decision shows up later in CSS as a single small `.sliders` section (`styles.css:199-241`).

---

## 3. `main.js` map

`main.js` is 1524 LOC and the structure can be read as nine logical regions. Each region's line range is from `src/main.js`.

| Region | Lines | Primary responsibility | Top 1–2 risks |
|---|---|---|---|
| **Header / constants / state** | 1–42 | ESM-style header, pulls `invoke` off `window.__TAURI__.core` (`:5`), declares `LENSES` (`:7`), and the global `state` (`:9-39`) — including the recall cache TTL/cap (`:41-42`) | Single global mutable `state` — accidental aliasing is easy. No `Object.freeze` on the LENSES constant. |
| **Init / DOMContentLoaded** | 44–56 | Boot sequence: build lens sliders → attach all event listeners → `loadInitialStack()` → `pollUntilReady()` → `startRecallPolling()` | If `loadInitialStack` fails silently (catch-and-warn at `:68-70`), user is stuck staring at an empty `<section class="results">` until `pollUntilReady` resolves. The catch happens but no UI fallback. |
| **Time Machine stack** | 58–220 | `loadInitialStack` (`:62-71`) loads first 60 sessions via `list_sessions`; `attachStackEvents` (`:73-104`) wires ↑↓/Enter + wheel; `advanceStack` (`:106-113`) and `renderStack` (`:115-203`) own the 3D card render; `enterSearchMode`/`enterStackMode` (`:205-220`) flip mode | Wheel handler accumulates `deltaY` (`:89-103`) but never bounds the accumulator if the user fires a long touchpad inertial swipe — could trigger a burst of `advanceStack(1)` calls. `renderStack` blows away `.card`/`.empty`/`.stack-card` and rewrites all visible cards on every keypress (`:118, :129`) — re-paint on each ↑↓ press at 60 sessions is fine; at 600+ would smell. |
| **Bootstrap poller** | 222–262 | `pollUntilReady` (`:222-262`) hits `collection_info` every 500 ms for up to 240 tries (2 min) and shows distinct hints for "Qdrant not reachable" vs "BGE-small loading" vs generic warmup | The retry uses chained `setTimeout` recursion — fine, but on success it kicks `schedulePrediction(state.selected)` (`:240`), creating a subtle ordering dependency: if `loadInitialStack` arrived first, the prediction now fires for whichever session the stack auto-selected. |
| **Top-bar wiring + Search/Lens** | 264–369 | `attachEvents` (`:264-294`) hooks every topbar/sidebar button + ⌘K shortcut + dialog `[data-close]` + topology modal `close` event for WebGL teardown; `onSearchInput`/`runLensSearch` (`:296-328`) with generation-counter race protection (`:308, :319, :325`); `buildLensSliders`/`resetLens` (`:330-369`) | `runLensSearch` correctly drops stale responses by `queryGen`, but the search-input listener is debounced 200 ms (`:266`); two interactions in close succession can still send two parallel `lens_search` invokes — the race protection means only the latest renders, but the wasted IPC is unavoidable without abort. (Tauri doesn't currently propagate AbortSignal.) |
| **Results / inspector / prediction** | 371–655 | `renderResults` (`:371-417`) renders search hit cards; `selectSession` (`:419-448`) loads payload via `get_session`; `schedulePrediction` (`:452-461`) debounces 220 ms; `loadPredictions`/`renderPredictionPanel` (`:489-610`); `renderInspector` (`:612-655`) | Prediction panel re-creates inline (`:519-525`) if the inspector hasn't been rendered yet, then the next inspector render replaces the whole inspector HTML — both work fine but the "create then get clobbered" sequence is fragile. If `schedulePrediction` ever shipped panels asynchronously after inspector renders, the panel would survive; today the inspector render writes `<section id="prediction-panel">` (`:648`) again. |
| **Recall (manual)** | 657–671 | `onRecall` wires the sidebar textarea → `invoke("recall")` → `renderResults` | Reuses `renderResults` (search hit cards) for recall fixes — UI doesn't distinguish "this is a recall, not a search". User has to read the status bar. |
| **Topology renderer** | 673–1071 | Largest region. `onTopology` (`:673-684`) opens the dialog; `renderTopology3D` (`:699-949`) builds the d3 force graph (data shape at `:755-775`, scene config `:777-816`, stats `:819-833`, legend rows + gap cards `:837-948`); `setHighlight`/`focusCluster`/`focusClusters` (`:955-994`); `dim`/`topologyTooltip`/`projectClusterForce` helpers (`:996-1058`); `projectColor` palette (`:1060-1071`) | One hardcoded fallback for missing globals: `typeof window.ForceGraph3D !== "function"` (`:712`). If the vendor script fails to load (CSP, file:// quirks), users see "3D engine failed to load." Dispose is best-effort (`:690-697`) — relies on `_destructor` from 3d-force-graph being stable across upgrades. |
| **Mix & Match** | 1073–1145 | `openMixModal` (`:1073-1077`), drop-zone CRUD (`addToMix`/`removeFromMix`/`renderMixDropzones` at `:1079-1115`), `runMix` invokes `mix_match` (`:1117-1145`) | "Dropzone" is misnamed — only click-add via the per-card `+ pos`/`− neg` buttons works; there is no actual HTML5 drag-and-drop wiring. Cards display ID prefixes only (`:1105`) — no way to tell which session is which after adding several. |
| **Snapshot / Refresh** | 1147–1176 | `onSnapshot` (`:1147-1160`) uses `window.prompt` for the path; `onRefresh` (`:1162-1176`) invokes `refresh_index` and updates collection counter | `window.prompt` is the documented deferred workaround (CLAUDE.md:137). On macOS this is a webview-native prompt — minimally accessible but ugly. |
| **Utilities** | 1178–1198 | `setStatus`, `setLatency`, `debounce`, `escapeHtml` | `escapeHtml` is correct (handles `&<>"'`) — and used consistently when interpolating user/JSONL data into `innerHTML`. |
| **Replay engine** | 1200–1448 | `attachReplayEvents`/`openReplay` (`:1204-1252`), `renderReplayList` (`:1254-1284`), `renderReplayTurn` (`:1286-1320`), per-tool renderers (`renderToolCall` `:1322-1396`, `renderStrayResult` `:1398-1404`), playback control (`stepReplay`/`toggleReplayPlay`/`startReplayTimer`/`stopReplayTimer`/`updatePlayButton`/`updateProgress` `:1406-1448`) | `renderReplayTurn` is called on every tick and rewrites the entire `#replay-detail.innerHTML` (`:1301-1310`). At 4× speed (250 ms) that's 4 full innerHTML rewrites/sec. See §10 perf smells. |
| **Proactive recall poller** | 1450–1524 | `startRecallPolling` (`:1456-1461`), `pollRecall` (`:1463-1482`), `recallCached` (TTL+LRU at `:1486-1499`), `showRecallBanner` (`:1501-1506`), `attachRecallBannerEvents` (`:1508-1524`) | Only the **first** recent error per tick yields a banner (`:1478`) — if there are two distinct ongoing failing sessions, the second is invisible until the user dismisses. Acceptable for the demo but worth flagging. |

A note on cohesion: the file is organized by surface, with separator-comment headers (`:58`, `:463`, `:1200`, `:1450`) that match the regions in the table above. That's the closest thing this file has to "module boundaries" — and it is, for a 1524-LOC vanilla JS file, surprisingly readable. The risk is the global `state` object (`:9-39`): nine subsystems mutate it, with no convention guarding which keys "belong" to which region.

---

## 4. Tauri IPC surface — frontend ↔ backend mapping

The frontend's `invoke()` call sites and the backend `#[tauri::command]` definitions are now compared 1:1. All 13 backend commands are registered in `src-tauri/src/lib.rs:63-77` via `tauri::generate_handler!`. All 11 invoke names actually used by `main.js`:

| invoke name (frontend site) | Backend `#[tauri::command]` site | Match | Notes |
|---|---|---|---|
| `list_sessions` (`main.js:64`) | `commands.rs:363` | ✅ | Args: `{ limit: 60 }` — pure parser walk, no Qdrant dependency. |
| `collection_info` (`main.js:230`) | `commands.rs:258` | ✅ | No args. Used by the boot poller. |
| `lens_search` (`main.js:312`) | `commands.rs:90` | ✅ | Args: `{ query, weights, limit: 20 }`. Race-guarded by `queryGen`. |
| `get_session` (`main.js:430`) | `commands.rs:153` | ✅ | Args: `{ sessionId }`. Returns the Qdrant payload as JSON map. |
| `predict_next_actions` (`main.js:496`) | `commands.rs:225` | ✅ | Args: `{ sessionId, lastNTurns: 3, horizon: 3, neighbors: 8 }`. Tauri auto-camelCase ↔ snake_case mapping in effect. |
| `recall` (`main.js:665`, `main.js:1492`) | `commands.rs:140` | ✅ | Args: `{ errorText, limit }`. Called from both manual "Recall fixes" button and the 12-s poller (cached). |
| `topology` (`main.js:679`) | `commands.rs:119` | ✅ | Args: `{ sample: 80, perPoint: 6 }`. Backend defaults `per_point` to 5 (`commands.rs:132`) — frontend explicitly overrides to 6. |
| `mix_match` (`main.js:1126`) | `commands.rs:106` | ✅ | Args: `{ positive, negative, limit: 10 }`. |
| `snapshot_export` (`main.js:1155`) | `commands.rs:247` | ✅ | Args: `{ path }` from `window.prompt`. |
| `refresh_index` (`main.js:1165`) | `commands.rs:279` | ✅ | No args. |
| `get_session_turns` (`main.js:1239`) | `commands.rs:174` | ✅ | Args: `{ sessionId }`. Returns a parsed `Session` — Replay engine consumer. |
| `tail_recent_errors` (`main.js:1465`) | `commands.rs:408` | ✅ | Args: `{ sinceSeconds: 90 }`. Frontend passes 90 s; backend defaults to 60 s (`commands.rs:416`). |

**Orphaned commands** (registered but unused by frontend):

- `snapshot_import` — defined at `commands.rs:252`, registered at `lib.rs:71`, but `grep` returns zero hits in `main.js`/`index.html`. This is the missing inverse of the export button. It is reachable via the CLI (`memex snapshot import`), but the desktop app cannot import a `.snapshot`. The Topbar's `#btn-snapshot` only exports (`main.js:1147-1160`). This is consistent with the README's "Snapshot export/import" framing (README.md:111) but only export ships in the GUI today.

**Stub callers** — none. Every frontend `invoke()` lands on a real registered command, and every argument key matches the backend's destructured parameter name (with Tauri's standard camelCase ↔ snake_case conversion). Notably the frontend uses **camelCase** consistently in argument names (e.g., `sessionId`, `errorText`, `sinceSeconds`, `lastNTurns`, `perPoint`) and the backend uses **snake_case** in the Rust signatures — Tauri's argument serializer handles the bridge transparently.

**One latent contract drift**:

- `predict_next_actions` (`main.js:497-501`) passes `{ sessionId, lastNTurns, horizon, neighbors }`. The backend declares `last_n_turns: Option<usize>` (`commands.rs:228`). Tauri's camelCase auto-mapping handles `lastNTurns → last_n_turns` correctly today, but this is the kind of field most likely to silently desync if either side renames without updating the other. Worth a unit test stub.

---

## 5. CSS architecture

`src/styles.css` is **1579 LOC, no preprocessor, no PostCSS, no utility framework**. It is hand-authored, organized by surface, and uses ASCII section banners (e.g., `styles.css:58`, `:120`, `:164`, `:337`, `:406`, `:872`, `:890`, `:1127`, `:1340`, `:1506`, `:1565`) as the structural skeleton.

**Section map** (line ranges, source `src/styles.css`):

| Lines | Section |
|---|---|
| 1–56 | Design tokens (`:root`) + global reset |
| 58–118 | Topbar |
| 120–163 | Button system (`.btn`, `.btn.primary`, `.btn.ghost`, `.btn.xs`) |
| 164–241 | Main 3-column layout + sidebar + Lens sliders |
| 244–335 | Search-result `.card` |
| 337–404 | Inspector (kvs + raw payload `<details>`) |
| 406–793 | Modal shell + Topology body (canvas + legend + gap cards) |
| 794–871 | Mix & Match modal interior |
| 872–889 | Statusbar |
| 890–1126 | Time Machine 3D stack (perspective + `data-layer` keyed transforms) |
| 1127–1339 | Replay engine (turn rows, terminal, diff blocks, tool blocks) |
| 1340–1505 | Predict panel + shimmer keyframes |
| 1506–1564 | Recall banner |
| 1565–1579 | Custom scrollbar |

**Design tokens** (`styles.css:3-33`):

```
--bg-window       #1c1c1e        (Apple deep dark)
--bg-surface      #2c2c2e
--bg-surface-hi   #3a3a3c
--bg-glass        rgba(255,255,255,0.06)
--bg-glass-hi     rgba(255,255,255,0.10)
--bg-input        rgba(0,0,0,0.35)

--border          rgba(255,255,255,0.10)
--border-strong   rgba(255,255,255,0.18)

--text-primary    rgba(255,255,255,0.96)
--text-secondary  rgba(255,255,255,0.62)
--text-muted      rgba(255,255,255,0.40)
--text-faint      rgba(255,255,255,0.22)

--accent          #0a84ff   (system blue)
--accent-soft     rgba(10,132,255,0.18)
--accent-glow     rgba(10,132,255,0.40)

--green/--yellow/--orange/--red — Apple semantic palette

--sans            SF Pro / Pretendard / Inter fallback chain
--mono            SF Mono / JetBrains Mono
```

A `color-scheme: dark` declaration (`:32`) ensures form controls match.

**Naming approach** — *not* BEM, *not* utility-first. It's flat-CSS-by-surface: `.stack-card`, `.stack-counter`, `.replay-row`, `.replay-detail`, `.prediction-card`, `.prediction-rank`, `.legend-row`, `.legend-mini`, `.gap-card`, `.gap-head`. Each surface gets a 1–2-word prefix and components nest by descendant. Specificity stays low (no `!important` anywhere, confirmed by the section grep), and the only attribute selectors are the `data-layer` keys for the stack (`:960, :966, :973, :980, :987, :993, :999, :1008`).

**Verdict on organization** — for 1579 LOC, this is **disciplined** rather than sprawling. The section banners are reliable, tokens are centralized, and there is no dead "legacy" zone. Two minor observations: (1) `.modal` (`:408`) and `.modal-sub` (`:794`) live in different sections — they should sit together. (2) `.empty` (`:253`) is used by both the search-empty placeholder and the inspector empty placeholder — fine, but it does mean the visual treatment of "nothing here" is identical across surfaces, which is actually a deliberate calm choice.

**Surface-by-surface stylesheets** (deepest CSS lives in):

1. Time Machine — `:890-1126` (~236 LOC, the heaviest single surface, owns the perspective transforms)
2. Replay — `:1127-1339` (~213 LOC, terminal + diff styles)
3. Topology — `:406-793` (~387 LOC including modal + legend + gap)
4. Predict panel — `:1340-1505` (~165 LOC including shimmer)
5. Lens sliders — `:199-241` (~43 LOC, surprisingly small for surface 7)

---

## 6. Visual / spatial surfaces — what does the heavy lifting

### Time Machine layered stack — pure CSS 3D + DOM diff

The render loop lives entirely in `main.js:115-203` (`renderStack`). It produces **at most ~9 stack cards** in the DOM at once: `start = max(0, focus - 1)` to `end = min(total, focus + 7)` (`main.js:131-133`). Each card carries `data-layer = i - state.stackFocus` (`:140`).

The 3D effect comes from CSS alone:

- `.results { perspective: 1200px; perspective-origin: 50% 28% }` — `styles.css:894-901`
- Per-layer `transform: translate3d(0, -Δy, -Δz) rotateX(Δdeg)` and graduated `opacity` + `filter: blur(...)` — `styles.css:960-1004` (layers 0…6) and `:1008-1014` (layer -1, the "just left the focal plane" effect).
- `transition: transform 0.42s cubic-bezier(0.2, 0.8, 0.2, 1)` (`styles.css:932`) makes ↑↓ navigation animate the transforms smoothly without JS keyframes.

There is **no requestAnimationFrame loop and no canvas** for this surface — it is the cheapest of the three, and the most idiomatically-CSS. Wheel input is bound directly on `#results` with `passive: false` (`main.js:90, :102`) so the page doesn't scroll while the stack receives wheel deltas.

### Topology galaxy — 3d-force-graph (Three.js) WebGL

Render loop owned by the **vendored 3d-force-graph** at `src/vendor/3d-force-graph.min.js` (707 KB, includes its own copy of Three.js). The mount + configuration code lives at `main.js:699-816` inside `renderTopology3D`. Key calls:

- `window.ForceGraph3D({...})(mount)` — `:778-781`, instantiates the renderer in the `<div id="topology-canvas">`.
- `.graphData(graphData)`, `.nodeRelSize`, `.nodeVal`, `.nodeColor`, `.nodeLabel`, `.linkColor`, `.linkOpacity`, `.linkWidth`, `.linkDirectionalParticles`, `.onNodeClick`, `.onNodeHover` — `:782-807`.
- Custom d3 force `projectClusterForce` (`main.js:1026-1058`) pulls same-project nodes toward their centroid each tick, complementing the per-link distance force at `:810`.

The "render loop" is the library's own internal `requestAnimationFrame` — Memex code only writes the scene once per opening and registers callbacks. Disposal is explicit: `topologyGraph._destructor()` is called both on modal close (`main.js:291-293`) and at the top of every re-open (`disposeTopology` at `:690-697`) — important because WebGL contexts on macOS Tahoe leak GPU memory aggressively.

The library is loaded as a non-module script tag (`index.html:151`) before the ESM `main.js`, so `window.ForceGraph3D` is reliably present by the time `renderTopology3D` runs — and the code defensively checks anyway (`main.js:712-715`).

### Replay engine — raw DOM diff via `innerHTML` rewrite per turn

Render loop is `renderReplayTurn(i)` at `main.js:1286-1320`. It is called:

- on initial open (`:1247`)
- on `stepReplay` ⏮/⏭ (`:1410`)
- on each list-row click (`:1276`)
- on every interval tick during playback (`:1430`)

It builds an HTML string for the entire `#replay-detail` (`:1301-1310`) and assigns it via `.innerHTML`. The list `#replay-list` is rendered once per session-open (`renderReplayList` at `:1254-1284`). Per-tool rendering switches on `tc.name` in `renderToolCall` (`main.js:1322-1396`) — eight cases (Bash, Edit/MultiEdit, Write, Read, WebFetch/WebSearch, Task/Agent, default).

This is **DOM-rewrite-on-tick**, not virtual-DOM-diff. At 1× (`speedMs = 2000`) it's fine; at 8× (`speedMs = 250`) and a 600-turn session, it's four full subtree replacements per second. See §10.

---

## 7. Proactive recall poller

Code lives at `main.js:1450-1524`. The pump is the simplest possible setInterval:

```js
const RECALL_POLL_MS = 12_000;   // main.js:1454

async function startRecallPolling() {           // main.js:1456-1461
  try { await pollRecall(); } catch {}
  setInterval(pollRecall, RECALL_POLL_MS);
}
```

`pollRecall` (`main.js:1463-1482`) does exactly the sequence described in CLAUDE.md:114:

1. `invoke("tail_recent_errors", { sinceSeconds: 90 })` — `:1465`.
2. Iterate the returned `RecentError[]` newest-first; skip any with a key in `state.recall.dismissedKeys` (`:1469-1470`).
3. For the first not-dismissed event: `await recallCached(ev.error_text, 3)` — `:1471`.
4. Filter the hits to drop the still-failing session itself (`:1473`).
5. Show one banner and `return` (`:1475-1478`).

**Dedup behavior** — exists at two layers:

- Per-error dismissal (`state.recall.dismissedKeys`, a `Set` keyed by `${session_id}::${error_text.slice(0,80)}`) — `:1469`, plus dismissal handlers at `:1509-1512` and the "open replay" handler also adds to the set at `:1519`. So once you click "Dismiss" or "Open replay" on an error, that exact (session, error-prefix) pair never re-banners.
- Per-error-text cache (`state.recall.cache`, a `Map` keyed by raw `error_text` with TTL `RECALL_CACHE_TTL_MS = 60_000`) — `:1486-1499`. Within 60 s, re-encountering the same error text returns the cached `hits` array instead of re-invoking `recall`. Cache is bounded at 50 entries (`:1494`) with FIFO eviction (`:1495-1497`).

**Back-off behavior** — none. The interval is fixed at 12 000 ms regardless of failure or success. If Qdrant is down, `pollRecall` swallows the error (`:1479-1481`) and waits the same 12 s. There is no exponential back-off, no jitter, and no pausing while the window is hidden (no `document.visibilitychange` hook). For a desktop app that may sit minimized for hours, this is a deliberate tradeoff — the 12 s tick is a flat constant, and the backend's mtime cache (`commands.rs:387-396, :437-457`) keeps each tick cheap (<10 ms after warmup, per the docstring at `commands.rs:406`).

**Subtle behaviors worth flagging**:

- The first `await pollRecall()` (`main.js:1458`) runs **before** Qdrant might be ready (`pollUntilReady` runs in parallel — see `:54-55`). Its `try/catch` (`:1457-1459`) makes that benign; the user just doesn't get a banner on the very first 12 s window of a cold start.
- Banner replacement: the code shows only one banner at a time (`return` at `:1478`) and the `state.recall.lastBannerError` is overwritten each call (`:1476`). If the previous banner is still visible and a different error fires, the displayed text is silently rewritten — the user might be staring at one project's error and look up to find a different project named.

---

## 8. Predict Next Actions panel

The Predict surface is the newest one (per the README "NEW" badge at line 107) and is rendered into the inspector pane, not its own modal.

**Trigger flow**:

1. `selectSession(sessionId)` runs whenever a card is clicked or the stack focus changes (`main.js:419-448`). It always calls `schedulePrediction(sessionId)` (`:435, :443`).
2. `schedulePrediction` (`main.js:452-461`) renders a shimmer placeholder immediately (`:456`), then debounces 220 ms before firing `loadPredictions(sessionId)` (`:457-460`). This means rapid ↑↓ navigation through the stack does not generate N parallel prediction invocations — only the session the user *lingers on* triggers the round-trip.
3. `loadPredictions` (`main.js:489-511`) invokes `predict_next_actions` with `{ sessionId, lastNTurns: 3, horizon: 3, neighbors: 8 }` (`:496-501`) and is **selection-guarded** — if `state.selected !== sessionId` by the time the response arrives, the response is dropped (`:502, :505`).
4. `renderPredictionPanel(ctx)` (`main.js:513-610`) writes the panel.

**Payload shape consumed** (extracted from the render code):

```ts
ctx = {
  loading?:    boolean,          // synthetic, not from backend
  error?:      string,           // synthetic, not from backend
  source_session_id?: string,
  neighbors_searched: number,
  neighbors_used: number,
  predictions: PredictionItem[]
}

PredictionItem = {
  rank: number,                  // 1-indexed
  tool_name: string,             // "Bash" | "Edit" | "MultiEdit" | "Write" | ...
  frequency: number,             // 0..1, displayed as ${freqPct}% of times
  confidence: number,            // 0..1, displayed as sim ${confPct}%
  example_input_summary: string, // monospaced one-liner preview
  from_session_id: string,
  from_session_project: string,
  from_turn_index: number,
}
```

This is consumed at: `main.js:563-564` (counters), `:567-588` (per-card render), `:585` ("Jump to replay" button data attrs).

**Jump to replay wiring** — `main.js:590-607`:

```js
card.querySelector("button[data-replay-id]").addEventListener("click", async (e) => {
  e.stopPropagation();
  const sid = e.target.dataset.replayId;        // from p.from_session_id
  const turn = parseInt(e.target.dataset.replayTurn, 10);  // from p.from_turn_index
  await openReplay(sid, { project_name: p.from_session_project });
  setTimeout(() => {
    if (state.replay.sessionId === sid) {
      state.replay.cursor = Math.min(turn, state.replay.turns.length - 1);
      renderReplayTurn(state.replay.cursor);
    }
  }, 600);
});
```

The 600 ms `setTimeout` is the only fragile part: the code assumes `get_session_turns` (called inside `openReplay` at `main.js:1239`) returns within that window. On a cold cache or a giant session this might not hold, and the user would land at `cursor = 0` instead of the intended turn. A cleaner fix would await `openReplay` *and then* check `state.replay.turns.length`, but the current `openReplay` doesn't return until the turns load — so technically the `await` already does that. The `setTimeout` looks redundant. (The defensive `state.replay.sessionId === sid` guard at `:599` would protect against a user mashing replay buttons during the 600 ms.)

**Empty / error states**:

- `loading: true` → shimmer panel (`:526-538`).
- `error: <msg>` → muted empty card with escaped error text (`:540-547`).
- `predictions: []` → "No close-enough past sessions to project a next action from" (`:550-558`).
- Has predictions → list with `prediction-rank` badge, tool name + icon, `frequency × similarity` percent meta, `example_input_summary` block, source attribution + Jump button, and a CSS-only `prediction-freq-bar` (`:588`).

Worth noting the panel sits **above** the inspector's k/v rows (`main.js:648`, comment at `:642-643`), so it's always visible without scrolling — that placement was the explicit goal of the most recent design change (see commit `a638ad4` per session header).

---

## 9. Accessibility audit (lightweight)

This frontend is dark-themed, mouse-and-keyboard-first, and **not screen-reader-friendly out of the box**. Concrete gaps follow.

**ARIA / semantic labeling** — `grep` for `aria-`, `role=`, `tabindex`, `alt=` against `src/index.html` and `src/main.js` returns **zero hits**. That means:

- The Time Machine has no `role="listbox"` and no `aria-activedescendant` on the focused stack card.
- The Topology canvas (`<div id="topology-canvas">`) is just a div with WebGL inside — no `role="application"`, no `aria-label`, no keyboard alternative listing.
- The Recall banner (`#recall-banner`) has no `role="status"` or `aria-live="polite"` — a screen reader user would never know a banner appeared.
- The search input has `placeholder` but no `<label>` and no `aria-label`. The placeholder text *contains* the keybinding hint (`⌘K`), which is fine visually but invisible to assistive tech (`index.html:19`).
- Dialog open/close — `<dialog>` elements (`index.html:66, 85, 109`) have built-in semantics, which is good. But the close-button selector `[data-close]` (`main.js:283-287`) attaches a generic handler; no `aria-label="Close"` is set on those buttons (their textContent is "Close" which is okay).
- Tool-call blocks in Replay (`main.js:1322-1396`) render diff hunks as `<pre>` — fine for sighted reading, but the `+`/`-` prefixes are bare characters in the `<pre>` (`:1347-1348`); a screen reader will read them as "minus" / "plus" rather than "removed line" / "added line."

**Keyboard navigation in the Time Machine** — partial:

- ↑/← go older, ↓/→ go newer (`main.js:77-82`). Enter opens the focused session (`:83-86`).
- Focus management is **manual**: the focused stack card has `data-layer="0"` and gets the `.selected` class via `selectSession` (`main.js:422-424`). But the `.stack-card` is not a `<button>` and has no `tabindex`, so it never receives DOM focus. A user tabbing through the chrome can't reach the stack — they have to start using arrows on the body, which works because the listener is on `document` (`main.js:74`).
- The keydown listener early-returns for INPUT/TEXTAREA (`main.js:76`) — good. But it never accounts for the topology modal or replay modal being open, so ↑↓ can still mutate `state.stackFocus` while a modal is in front. Cards beneath the modal don't visually re-render until the modal closes (because the stack isn't repainted while it's mode is "search" or behind a dialog), so users probably don't notice — but it is a latent foot-gun.

**Focus management on modal close** — none. After `<dialog>.close()`, focus returns to the body, not to the trigger button (no `dialog.returnValue` flow). On macOS Tahoe + VoiceOver this is the most jarring gap.

**Reduced motion** — `grep` for `prefers-reduced-motion` returns zero hits across `main.js` and `styles.css`. The Time Machine's 0.42 s ease-out card transitions (`styles.css:932`), the shimmer keyframes (`styles.css:1501-1504`), and the recall banner transform/opacity transition (`styles.css:1514`) ignore the OS preference. For users with vestibular-motion sensitivity, the Time Machine in particular is intense.

**Color contrast spot-check** — the design tokens (`styles.css:14-22`) lean on white-with-alpha against `#1c1c1e`:

- `--text-primary` (rgba(255,255,255,0.96)) on `--bg-window` (#1c1c1e) — effectively `#f5f5f5` on `#1c1c1e`, contrast ratio ≈ 14.5:1. Excellent.
- `--text-secondary` (rgba 0.62) on same bg — ≈ 7.5:1. AAA.
- `--text-muted` (rgba 0.40) on same bg — ≈ 4.0:1. **Below WCAG AA 4.5:1 for normal text**, but fine for 14 px and above as "muted" hints. The k/v key cells (`.kv .k`) and the `meta` chips in stack cards use this token (`styles.css:373` and similar) — they sit just under the threshold.
- `--text-faint` (rgba 0.22) on bg — ≈ 1.9:1. Sub-WCAG; thankfully only used for the most decorative copy.
- `.vec-chip` text on its background (semi-transparent), and the gold cross-project edge labels in the topology tooltip (`main.js:1014`) — these were not spot-checked, but the tooltip background `rgba(20,20,22,0.94)` is very near-black, so contrast there is fine.

**Concrete accessibility gaps** (the short list):

1. No `aria-live` on `#recall-banner` (`index.html:134`) — assistive tech will never announce it.
2. No `<label>` or `aria-label` on `#search-input` (`index.html:16-22`).
3. No `<label>` on the `#recall-input` textarea (`index.html:47-51`) — just a heading nearby.
4. Lens slider range inputs (`main.js:336-358`) get a visible `slider-value` span (`:339`) but the `<input type="range">` has no `aria-valuenow`/`aria-label` and the label-span isn't tied to it via `for`.
5. Replay row buttons (`main.js:1258-1283`) are `<button>` — good — but the role indicator (`role-user`/`role-assistant`/`role-system`) is communicated only by color (`.role-user`, `.role-assistant`, `.role-system` at `styles.css:1199-1213`) and a single character ("U"/"A"/"S"). A screen reader user gets "U" with no expansion.
6. No focus restoration after `dialog.close()`.
7. No `prefers-reduced-motion` honoring.
8. The Topology surface is keyboard-unreachable (no Tab into the WebGL canvas, no list alternative). A user with no pointer device cannot operate surface 2 at all.

None of these are showstoppers for the VSD 2026 demo, but they're the kind of polish that would make a follow-up release feel mature.

---

## 10. Performance smells

**Time Machine** — fine. `renderStack` (`main.js:115-203`) only renders a window of at most 9 cards (`:131-133`). Each ↑↓ keypress repaints the whole window; with 9 elements that's negligible. CSS transitions handle the actual animation work on the GPU compositor (the `transform: translate3d` triggers a layer; `will-change: transform, opacity` at `styles.css:938` opts in explicitly).

**Search-driven Lens results** — fine. `renderResults` (`main.js:371-417`) repaints at most 20 cards (`limit: 20` at `:316`). Slider mutations are change-debounced 150 ms (`:351-355`) so dragging doesn't fire one IPC per pixel.

**Topology** — bounded. Default `sample: 80, perPoint: 6` (`main.js:679`) caps nodes at 80 and edges at ~480 pre-MST. The 3d-force-graph runs at 60 fps for those numbers on M-series silicon. The risk vector is the per-link `linkDirectionalParticles((l) => l.cross ? 2 : 0)` (`main.js:798`) — each cross-project edge spawns 2 particle sprites that loop forever; with hundreds of bridges (which the current corpus won't hit) it would accumulate. Disposal is correct (`:690-697`).

**Replay engine — the actual concern**:

- `renderReplayTurn` rewrites the entire `#replay-detail.innerHTML` per call (`main.js:1301-1310`). At 8× speed (`speedMs = 250`), the autoplay timer (`:1422-1431`) calls it every 250 ms. Each rewrite serializes the entire turn (possibly several KB of escaped Bash output capped at 1200 chars per block — `:1338`).
- Per turn, every `tool_call` and `tool_result` is re-rendered (`:1292-1299`). A turn with 5 tool calls produces 5 nested templates. Each `escapeHtml` (`main.js:1194-1198`) runs a regex over the content (up to ~1200 chars per block × multiple blocks).
- Scroll-into-view happens after the innerHTML replace (`:1316-1317`). That's a forced layout per tick.
- **At 600 turns played at 8× the user does ~150 s of playback** — sustained DOM churn. Not catastrophic, but the worst-case smell. A simple mitigation is to render once and just toggle which `.turn` is visible via a `.active` class — but that requires pre-building the entire `#replay-detail`, which uses more memory upfront. For an MVP this is the right trade.
- A second risk: `selectedRow.scrollIntoView({ block: "nearest" })` on every tick (`main.js:1317`). On Tauri/webkit it's cheap, but it does force the panel to lay out before scrolling.
- The replay-list is rendered once (`:1254-1284`) and never repainted, which is good — but it builds a `<button>` per turn with no virtualization. A 600-turn session = 600 buttons in the DOM. That's still under "I'm comfortable with it" but is the second axis on which Replay won't scale gracefully.

**Topology — large arrays**:

- The `graphData.nodes` / `links` arrays are rebuilt from the backend response on every `renderTopology3D` call (`main.js:756-775`). On every re-open we map all 80 nodes + ~150 edges into objects with `projectColor`, `dim`, `Math.sqrt` etc. — once-per-open is fine. Not a hot loop.
- The legend-row builder runs `legendEl.appendChild` per project (`main.js:843-883`). At up to ~30 projects this is trivial.

**Recall poller** — well-mitigated:

- The 12 s interval is gentler than the inner cache eviction (60 s TTL), so the cache reliably absorbs repeated errors.
- The backend has its own mtime-keyed file-level cache (`commands.rs:387-396, :440-449`), so a tick with no file modifications costs ~5–10 ms total.

**Things that look like smells but aren't**:

- `runLensSearch` (`main.js:307-328`) bumps `state.queryGen` and drops stale responses (`:319, :325`). This is correct race-handling for a debounced input.
- `schedulePrediction` (`main.js:452-461`) clearTimeouts and re-schedules on every selection change. The selection-guard in `loadPredictions` (`:502`) double-protects against late responses.

**Debounce / throttle inventory** — present and correctly placed:

- Search input: 200 ms debounce (`main.js:266`).
- Lens slider change: 150 ms debounce (`main.js:351-355`).
- Prediction load: 220 ms debounce via `schedulePrediction` (`main.js:457`).
- Wheel input on stack: not debounced but threshold-gated (`Math.abs(wheelAccum) > 60` at `main.js:96`).
- Recall poll: 12 000 ms fixed interval; no jitter, no backoff (`main.js:1454, :1460`).

---

## 11. No-LLM / no-chat invariant audit

This is the hard constraint (CLAUDE.md:17, CLAUDE.md:19, CLAUDE.md:141-143, README.md:43). The frontend is clean.

**Keyword scan**:

```
grep -niE "openai|anthropic|chat\b|completion|gpt|llm|claude\.ai/api|
          api\.openai|api\.anthropic|generate.*text|complete.*chat"
  src/main.js src/index.html src/styles.css
→ (no output)
```

Zero hits across all three frontend source files. Confirmed.

**Network egress scan**:

```
grep -niE "fetch\(|XMLHttpRequest|WebSocket\(|EventSource\("  src/main.js
→ (no output)
```

The frontend never makes a direct HTTP/WS call. Every interaction with the outside world goes through `window.__TAURI__.core.invoke(<name>)` (single import at `main.js:5`), which is in-process IPC to the Rust backend. The backend in turn talks only to localhost Qdrant (gRPC 6334, HTTP 6333) per CLAUDE.md:25 — there is no LLM endpoint anywhere in the stack.

**UI affordance check**:

- No `<input>` or `<textarea>` whose semantics are "ask a question." The only two text inputs are:
  - `#search-input` (`index.html:17-22`) — placeholder *"Search your past sessions ⌘K"*, wired to `lens_search`, returns ranked session hits, **not** a generative response.
  - `#recall-input` (`index.html:47-51`) — placeholder *"Paste an error message…"*, wired to `recall`, returns ranked past sessions whose `error` vector neighbors the pasted text.
- No "submit a prompt" / "send" / "generate" affordance.
- No streaming-token UI (`grep` for `stream-token`, `chunk`, `delta` returns nothing relevant).
- No conversation thread component.

**Invariant verdict**: clean. The frontend honors the "Think Outside the Bot" constraint with zero leakage. Every surface offers a *spatial*, *recommendation*, or *replay* affordance — never a generative one.

---

## 12. Top 5 frontend improvements (non-urgent, no-framework-friendly)

All of these can land without introducing a build step, a bundler, a framework, or violating the no-LLM invariant.

### 1. Wire a native file picker for snapshot import/export

`onSnapshot` (`main.js:1147-1160`) uses `window.prompt()`, and the inverse — snapshot **import** — has no UI at all despite the backend command being live (`commands.rs:252`). CLAUDE.md:137 lists "native file picker for snapshots" as a deferred item that requires `tauri-plugin-dialog`. This is a ~30-line change: add the plugin to `Cargo.toml`, register it, and replace the prompt with `dialog.save({ defaultPath: ... })` and `dialog.open({ ... })`. No frontend framework needed; the plugin exposes a global on `window.__TAURI__.dialog`. Also: surface "Import snapshot" as a second topbar button or a hidden submenu of `#btn-snapshot`.

### 2. Add ARIA + reduced-motion respect

The accessibility gaps in §9 are all small textual additions:

- `aria-live="polite"` on `#recall-banner` (`index.html:134`).
- `aria-label` on `#search-input`, `#recall-input`, every range input in the Lens panel, and every `[data-close]` button.
- `role="application"` + an `aria-describedby` fallback on `#topology-canvas` pointing to a hidden `<ul>` listing the top N nodes, so keyboard users have a way in.
- `@media (prefers-reduced-motion: reduce) { .stack-card { transition: none; } ... }` in `styles.css`. One block, ~10 LOC, kills the 0.42 s ease-outs and the shimmer for users who asked for it.
- Restore focus to the trigger button on `dialog.close()` — three event handlers (one per dialog), trivially scoped.

Nothing here requires more than `Element.setAttribute` calls. Zero framework cost.

### 3. Virtualize the Replay turn-list and avoid full innerHTML rewrites

Replay is the one surface that demonstrably won't scale past today's corpus (~80 sessions averaging tens of turns). Two cheap wins:

- Pre-render the `#replay-list` button list (already done at `main.js:1254-1284`) **and** keep it under ~100 visible at a time via a windowed scroll handler. Even a naive "render rows i±50 around cursor" is enough.
- Instead of rewriting `#replay-detail.innerHTML` per tick (`main.js:1301-1310`), render each turn into a hidden `<section data-turn-index="i">` once when `openReplay` parses the session, and switch a `.active` class on tick. This keeps the DOM mass higher but eliminates the per-tick string serialization + escapeHtml passes. Same `escapeHtml`, same template literals — just executed once per turn instead of once per playback frame.

The second is ~50 LOC of refactor in `openReplay`/`renderReplayTurn` and changes nothing user-visible.

### 4. Add a tiny test harness for the IPC contract

The most likely silent break in this codebase is a backend rename desynchronizing from a frontend camelCase argument. There is no test today for `predict_next_actions`'s `lastNTurns` ↔ `last_n_turns` mapping (§4). A 20-line vitest-or-node-test file isn't worth pulling in a test runner for, but the same file can be a plain `npm test` script that:

- Parses `main.js` (string-grep is fine) for every `invoke("<name>", { ... })` and extracts the argument keys.
- Parses `commands.rs` for every `#[tauri::command]` and its parameter list.
- Asserts the camelCase-to-snake-case mapping holds 1:1 for every command-key pair.

That's a ~80-LOC node script. No deps. Caught by `cargo test` adjacency would be ideal, but a separate npm script is enough.

### 5. Lift the global `state` into module scope with named accessors

Today `main.js:9-39` declares one mutable `state` object and nine subsystems mutate fields directly. This works but makes it easy to typo a key (`state.stackfocus` vs `state.stackFocus`) and get silent failures. A 30-line refactor:

- Move each subsystem's state into a separately-named module-scope object: `const stackState = { ... }`, `const replayState = { ... }`, `const recallState = { ... }`.
- Replace `state.stack`, `state.stackFocus`, `state.mode` with `stackState.items`, `stackState.focus`, `stackState.mode`.
- Each subsystem's helpers (e.g., `advanceStack`, `renderStack`) only touch its own state object.

This is still vanilla ESM, no build step, no framework — but it gives each surface a coherent boundary and reads better at 1500 LOC. The change is mechanical and could be done in one commit per subsystem.

---

## Appendix A — File-level metric snapshot

```
src/index.html                154 LOC
src/main.js                  1524 LOC
src/styles.css               1579 LOC
src/vendor/3d-force-graph.min.js   706 907 bytes
src/assets/{javascript,tauri}.svg  (icons, not used in chrome)
package.json                   12 LOC (only @tauri-apps/cli)

src-tauri/src/commands.rs     503 LOC
                              13 registered commands
                              1  orphan vs frontend (snapshot_import)
                              0  stub callers
```

## Appendix B — Full IPC roster

The 13 registered Tauri commands (lib.rs:63-77) in registration order, with frontend usage:

| Command | Used by frontend? | Caller(s) |
|---|---|---|
| `lens_search` | ✅ | `main.js:312` (runLensSearch) |
| `mix_match` | ✅ | `main.js:1126` (runMix) |
| `topology` | ✅ | `main.js:679` (onTopology) |
| `recall` | ✅ | `main.js:665` (onRecall, manual), `main.js:1492` (recallCached, poller) |
| `get_session` | ✅ | `main.js:430` (selectSession) |
| `get_session_turns` | ✅ | `main.js:1239` (openReplay) |
| `snapshot_export` | ✅ | `main.js:1155` (onSnapshot) |
| `snapshot_import` | ❌ orphan | — no GUI surface |
| `collection_info` | ✅ | `main.js:230` (pollUntilReady) |
| `refresh_index` | ✅ | `main.js:1165` (onRefresh) |
| `tail_recent_errors` | ✅ | `main.js:1465` (pollRecall) |
| `list_sessions` | ✅ | `main.js:64` (loadInitialStack) |
| `predict_next_actions` | ✅ | `main.js:496` (loadPredictions) |
