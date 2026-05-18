<div align="center">

# Memex

### Time Machine for your Claude Code session history.

*Search, replay, and learn from every AI coding session you've ever run — all on your laptop, powered by Qdrant.*

<p>
  <a href="LICENSE"><img alt="License" src="https://img.shields.io/badge/license-Apache_2.0-blue.svg?style=flat-square"></a>
  <a href="https://tauri.app"><img alt="Tauri" src="https://img.shields.io/badge/Tauri-2.x-24c8db?style=flat-square&logo=tauri&logoColor=white"></a>
  <a href="https://qdrant.tech"><img alt="Qdrant" src="https://img.shields.io/badge/Qdrant-1.18-dc382d?style=flat-square&logo=qdrant&logoColor=white"></a>
  <a href="https://www.rust-lang.org"><img alt="Rust" src="https://img.shields.io/badge/Rust-1.88-orange?style=flat-square&logo=rust&logoColor=white"></a>
  <a href="#install--run"><img alt="macOS" src="https://img.shields.io/badge/macOS-11%2B_(Apple_Silicon)-000000?style=flat-square&logo=apple&logoColor=white"></a>
  <br>
  <img alt="100% local" src="https://img.shields.io/badge/privacy-100%25_local-30d158?style=flat-square">
  <img alt="No telemetry" src="https://img.shields.io/badge/telemetry-none-30d158?style=flat-square">
  <img alt="Hackathon MVP" src="https://img.shields.io/badge/status-Qdrant_VSD_2026_MVP-yellow?style=flat-square">
</p>

<p>
<a href="#-what-you-can-ask-memex"><b>Use cases</b></a> ·
<a href="#-five-qdrant-unique-features"><b>Features</b></a> ·
<a href="#-quick-start"><b>Quick start</b></a> ·
<a href="#-cli-reference"><b>CLI</b></a> ·
<a href="#-architecture"><b>Architecture</b></a> ·
<a href="#-status--roadmap"><b>Status</b></a>
</p>

</div>

---

## 🧠 Why Memex?

Every Claude Code session you've ever run is sitting on your laptop right now:

```
~/.claude/projects/<encoded-cwd>/<session-uuid>.jsonl
```

Inside each `.jsonl` is your *entire* conversation — every prompt, every tool call, every diff, every output, every error. **Months of personal engineering memory, perfectly preserved, but practically unreachable.**

| Without Memex | With Memex |
|---|---|
| 🔍 You vaguely remember "I fixed that Qdrant connection thing back in March" — you can't find it. | ⌘K → `Qdrant connection` → top hit takes you straight to the session. |
| 🔁 You hit the same `WAL Kind(WouldBlock)` error you already debugged last month. | A banner slides in: *"I've seen this — open the past session that solved it."* |
| 📁 `project-marketing-v2`, `-v3`, `-v4` — were they actually different work, or did you redo the same thing? | Topology view shows them as one tight cluster, with cluster auto-labels. |
| ⏯ You want to *re-watch* yourself fix a tricky bug, turn by turn. | Replay any session at 1× / 2× / 4× / 8× with Bash terminals, Edit diffs, and tool visualizations rendered inline. |
| 🌐 You stitch results from cloud-hosted, telemetry-bearing services. | Everything — parsing, embedding, similarity search, replay — happens **on your machine**. Nothing leaves. |

> Memex turns your `.jsonl` pile into a **queryable, replayable, time-machine UI** powered by local Qdrant + FastEmbed.

---

## 🪟 Demo

> **Place placeholders here for the demo video + key screenshots.** Update once recorded.

| Time Machine stack | Topology galaxy | Replay engine |
|:---:|:---:|:---:|
| _Layered 3D card deck of every past session._ | _Force-directed graph with project clusters + bridge edges + gap insights._ | _Turn-by-turn playback with Bash terminals & Edit diffs._ |
| ![Stack screenshot — placeholder until demo](docs/img/stack.svg) | ![Topology screenshot — placeholder until demo](docs/img/topology.svg) | ![Replay screenshot — placeholder until demo](docs/img/replay.svg) |

▶ **3-min walkthrough video**: _to be added (YouTube unlisted)_

---

## ✨ Five Qdrant-unique features

Most session-search tools index everything into one vector and call it a day. Memex puts **five different lenses** on the same session point and exposes each via a Qdrant primitive that would be hard to replicate elsewhere:

| # | Feature | Qdrant primitive | What you do with it |
|---|---|---|---|
| 1 | 🔍 **Lens slider** | Multiple **named vectors per point** + parallel `query()` + weighted Rust combine | 5 sliders (`content`, `tool`, `path`, `error`, `code`) — bias a search toward exactly the signal you want. Each result card shows per-vector contribution chips. |
| 2 | 🧪 **Mix & Match** | **Discovery API** (`DiscoverInput` + context pairs) | Drop sessions as **positives** and **negatives** — Qdrant returns sessions semantically near the positives, far from the negatives. |
| 3 | 🌌 **Topology galaxy** | **Distance Matrix API** (`search_matrix_pairs`) → 3D force-directed graph + auto-clustered project labels + gap analysis | A real WebGL scene of your session corpus. Yellow bridges = cross-project "ideas that jumped"; **Gap cards** flag pairs of projects that *should* connect but don't. |
| 4 | ⏯ **Replay engine** | Lightweight payload (`source_path`) → on-demand JSONL re-parse | Turn-by-turn animation of any past session with **Bash terminals**, **Edit `-`/`+` diffs**, **Read snippets**, **Task/Agent spawns**. Click to scrub, ⏮ ⏯ ⏭ controls, 1× / 2× / 4× / 8×. |
| 5 | 🔔 **Proactive recall** | `query()` on the `error` named vector with `has_errors=true` filter, polled every 12 s over `~/.claude/projects` | Working in another Claude Code session and hit a fresh error? A banner slides in: *"I've seen this error before — open the session that solved it."* |

Plus: **Snapshot** export/import via Qdrant's HTTP snapshot API, **portable** in one `.snapshot` file.

ColBERT v2 inline citations are on the roadmap; [`fastembed-rs`](https://github.com/Anush008/fastembed-rs) 5.x doesn't yet ship the model.

---

## 💬 What you can ask Memex

Concrete examples of questions Memex answers from real session history:

<table>
<tr><td><b>Search</b></td><td>

```
⌘K → "Tauri build failed missing icons"
```

→ The exact past session that fixed it, with the Edit diff in Replay.

</td></tr>
<tr><td><b>Lens</b></td><td>

```
memex lens "myproject memex Qdrant" --content 2 --tool 1 --path 0.5
```

→ Sessions weighted toward conversation prose, with file paths as a tiebreaker.

</td></tr>
<tr><td><b>Discovery</b></td><td>

```
Click + pos on workspace-a session, − neg on project-meeting session
```

→ "Other myproject-flavored sessions, but unlike chatty meeting work."

</td></tr>
<tr><td><b>Topology</b></td><td>

```
Open Topology → see auto-labeled project clusters
```

→ *"`project-marketing` (10 sessions) — code + shell · Bash×1350 Edit×1032"*<br>
→ Gap card: *"`project-redesign` ↔ `project-yc` — semantically similar (sim 0.97) but never bridged."*

</td></tr>
<tr><td><b>Recall</b></td><td>

Working on a fresh project, hit an error. 12 seconds later:

```
⚡ I've seen this error before:
   myproject/workspace-b — 2026-05-15 [Open replay]
```

</td></tr>
<tr><td><b>Replay</b></td><td>

```
Click Replay on any card → step through 600 turns at 4× speed
```

→ Watch your past self debug, browse, and ship — every Bash command and Edit diff rendered.

</td></tr>
</table>

---

## 🚀 Quick start

```bash
# 1. Clone + install JS deps
gh repo clone sgwannabe/memex ~/memex && cd ~/memex && npm install

# 2. Start Qdrant (binary path — or docker run -d -p 6333:6333 -p 6334:6334 qdrant/qdrant:v1.18.0)
mkdir -p .qdrant && curl -sL https://github.com/qdrant/qdrant/releases/download/v1.18.0/qdrant-aarch64-apple-darwin.tar.gz | tar xz -C .qdrant
./.qdrant/qdrant &

# 3. Index your ~/.claude/projects (downloads BGE-small ~130 MB on first run)
cargo build --release --manifest-path src-tauri/Cargo.toml
./src-tauri/target/release/memex scan --index

# 4. Launch the app
npm run tauri build   # produces src-tauri/target/release/bundle/macos/Memex.app
open src-tauri/target/release/bundle/macos/Memex.app
```

That's it. Hit **⌘K**, type something you worked on last month, watch the cards rank.

<details>
<summary><b>📋 Full prerequisites + step-by-step (click to expand)</b></summary>

### Prerequisites

- **macOS 11+** (Apple Silicon recommended; tested on macOS 26.5 / arm64)
- [**Rust**](https://rustup.rs) 1.88+
- [**Node.js**](https://nodejs.org) 22+ with npm
- [**Qdrant**](https://github.com/qdrant/qdrant/releases) 1.18+ (binary or Docker)

### Step 1 — Clone

```bash
gh repo clone sgwannabe/memex ~/memex
cd ~/memex
npm install
```

### Step 2 — Start Qdrant

Either download the prebuilt binary…

```bash
mkdir -p .qdrant && cd .qdrant
curl -sL https://github.com/qdrant/qdrant/releases/download/v1.18.0/qdrant-aarch64-apple-darwin.tar.gz | tar xz
./qdrant            # serves Qdrant on localhost:6333 (HTTP) + 6334 (gRPC)
```

…or run it via Docker:

```bash
docker run -d -p 6333:6333 -p 6334:6334 qdrant/qdrant:v1.18.0
```

Verify: `curl localhost:6333 | jq .title` should print `"qdrant - vector search engine"`.

### Step 3 — Authorize Full Disk Access

On **macOS Sequoia / Tahoe**, granting `Memex.app` **Full Disk Access** in System Settings → Privacy & Security is required so it can read `~/.claude/projects`. Memex never sends your sessions anywhere — every embedding and similarity call happens locally in Rust + Qdrant.

### Step 4 — First index

The CLI is the same binary as the GUI; it dispatches on `argv[1]`. The first run downloads the BGE-small-en-v1.5 ONNX model (~130 MB) into `.fastembed_cache/`.

```bash
cargo build --release --manifest-path src-tauri/Cargo.toml
./src-tauri/target/release/memex scan --index
```

You should see:
```
parsed 80 session(s) (shown: 80), 17752 total tool calls
indexed 79/80 session(s) into 'memex_sessions' (1 duplicate sessionId(s) skipped, 0 error(s))
```

### Step 5 — Launch

```bash
npm run tauri dev      # hot-reload dev mode
# OR
npm run tauri build    # → src-tauri/target/release/bundle/macos/Memex.app + .dmg
```

When the window opens, the bottom status bar should read:
```
Connected — 79 sessions indexed (memex_sessions)
```

</details>

---

## 🛠 CLI reference

Memex's CLI is a one-binary surface over the same backend the GUI uses:

```bash
memex scan [--index] [--path PATH] [--limit N]    # walk + (optionally) index
memex search "query"                              # plain content-vector search
memex lens "query" --content 2 --tool 1.5 --code 0.5
memex mix --pos <session_id> --neg <session_id>
memex topology --sample 80 --per-point 6 --out topo.json
memex recall "Tauri build failed missing icons"
memex snapshot export ./memex.snapshot
memex snapshot import ./memex.snapshot
```

Run `memex --help` for the full surface; each subcommand has `--help` too.

---

## 🏗 Architecture

```mermaid
flowchart TB
    subgraph fs["~/.claude/projects (your laptop)"]
        jsonl["<session-uuid>.jsonl<br>append-only"]
    end

    subgraph app["Memex.app · Tauri 2"]
        webview["Webview (HTML/CSS/JS)<br>Time Machine stack · 3D topology · replay · banner"]
        rustcore["Rust core<br>parser.rs · indexer.rs<br>commands.rs · cli.rs"]
        webview <-- "Tauri IPC<br>invoke('lens_search', …)" --> rustcore
    end

    subgraph qdrant["Local Qdrant 1.18"]
        coll["Collection memex_sessions<br>5 named vectors / point (384-d cosine)<br>payload-indexed: project_name, start_ts, has_errors, …"]
    end

    fs -- walkdir + serde_json --> rustcore
    rustcore -- "fastembed BGE-small<br>+ qdrant-client gRPC" --> coll
    rustcore -. "reqwest HTTP<br>(snapshots only)" .-> coll
```

Each session becomes **one point** with **five named vectors** (`content`, `tool`, `path`, `error`, `code`) all dense 384-d BGE-small. The payload carries only metadata — replay re-parses the JSONL on demand so Qdrant stays lean.

Deeper reading:
- [`docs/architecture.md`](docs/architecture.md) — data flow, schema, design trade-offs
- [`docs/qdrant-features.md`](docs/qdrant-features.md) — engineer's tour of each of the 5 features
- [`docs/memex/PLAN.md`](docs/memex/PLAN.md) — original 8-phase implementation plan

---

## 🔬 Tech stack

<table>
<tr>
<td><b>Frontend</b></td>
<td>

`vanilla HTML/CSS/JS` · `Tauri 2 webview` · [`3d-force-graph`](https://github.com/vasturiano/3d-force-graph) (Three.js) for topology · CSS 3D `translateZ` for the Time Machine layered stack

</td>
</tr>
<tr>
<td><b>Backend</b></td>
<td>

`Rust 1.88` · [`tauri 2`](https://tauri.app) · [`qdrant-client 1.18`](https://github.com/qdrant/rust-client) · [`fastembed 5`](https://github.com/Anush008/fastembed-rs) (BGE-small-en-v1.5) · [`petgraph 0.6`](https://github.com/petgraph/petgraph) for MST · [`tokio`](https://tokio.rs) · `walkdir` · `serde` · `regex`

</td>
</tr>
<tr>
<td><b>Storage</b></td>
<td>

[`Qdrant 1.18`](https://qdrant.tech) (local binary or Docker) — 5 named dense vectors per point (384-d cosine), payload-indexed on `project_name`, `git_branch`, `start_ts`, `has_errors`, etc.

</td>
</tr>
<tr>
<td><b>Embedding</b></td>
<td>

`fastembed-rs` running BGE-small-en-v1.5 entirely client-side. No Python sidecar, no network calls, ~130 MB ONNX model cached after first run.

</td>
</tr>
<tr>
<td><b>Bundle</b></td>
<td>

`Memex.app` ~45 MB · `Memex_0.1.0_aarch64.dmg` ~15 MB · No code signing in MVP — right-click → Open the first time.

</td>
</tr>
</table>

---

## 📊 Status & roadmap

This is a **hackathon MVP** built for [Qdrant Vector Space Day 2026](https://qdrant.tech) (deadline 2026-06-01). Functional path verified end-to-end on the author's `~/.claude/projects` (**79 sessions indexed, 17,938 tool calls covered**), with all five Qdrant features exercisable from both CLI and GUI.

**What ships in this MVP**

- ✅ 5 Qdrant-unique features (lens / mix / topology / replay / recall)
- ✅ Time Machine layered 3D card stack on boot
- ✅ 3D force-directed topology galaxy with project cluster auto-labels + gap analysis
- ✅ Snapshot export/import via Qdrant HTTP API
- ✅ Lazy AppState init — self-heals if Qdrant is started after Memex
- ✅ Honest duplicate-sessionId detection in indexer reporting
- ✅ `Memex.app` + `.dmg` for macOS arm64

**Deferred to post-MVP**

| Item | Why it's deferred | Path forward |
|---|---|---|
| ColBERT v2 inline citations | `fastembed-rs` doesn't yet expose the model | Fallback via `ort` crate + ONNX Jina-ColBERT-v2 |
| BM42 sparse on `path` vector | Same upstream gap | Same path |
| Real `notify` file watcher | Polling works and avoids fd-leak / macOS permission edge cases | Code path already in `Cargo.toml` — one-line swap when needed |
| Native file picker for snapshots | MVP uses `window.prompt()` | Add `tauri-plugin-dialog` |
| Code signing / notarization | Local-only MVP | Apple Developer cert when shipping publicly |

---

## 🤝 Contributing / feedback

This is a personal hackathon project, but PRs that don't break the demo are welcome — especially:
- Linux + Windows packaging
- Codex / Cursor / other CLI session formats (parser extension)
- ColBERT v2 integration via `ort`

For bugs or design feedback, [open an issue](https://github.com/sgwannabe/memex/issues/new).

---

## 📄 License

[Apache 2.0](LICENSE) © 2026 Sangguen Chang.

Built on the excellent open work of [Qdrant](https://github.com/qdrant/qdrant), [Tauri](https://github.com/tauri-apps/tauri), [fastembed-rs](https://github.com/Anush008/fastembed-rs), [petgraph](https://github.com/petgraph/petgraph), and [3d-force-graph](https://github.com/vasturiano/3d-force-graph).

<div align="center">
<sub>Made for <a href="https://qdrant.tech">Qdrant Vector Space Day 2026</a> · <a href="https://github.com/sgwannabe/memex">sgwannabe/memex</a></sub>
</div>
