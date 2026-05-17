// Memex frontend — vanilla JS shell wiring 5 Qdrant-backed commands.
// Tauri's `withGlobalTauri: true` puts the IPC bridge on window.__TAURI__,
// so we can stay plain ESM without a build step.

const { invoke } = window.__TAURI__.core;

const LENSES = ["content", "tool", "path", "error", "code"];

const state = {
  query: "",
  weights: Object.fromEntries(LENSES.map((k) => [k, 1.0])),
  hits: [],
  selected: null,
  mix: { positive: [], negative: [] },
  collectionPoints: 0,
  // B3: monotonically-increasing query id; renderResults drops responses
  // whose generation is older than the latest dispatched query.
  queryGen: 0,
  replay: {
    sessionId: null,
    turns: [],
    cursor: 0,
    playing: false,
    speedMs: 500,
    timer: null,
  },
  recall: {
    dismissedKeys: new Set(),
    lastBannerError: null,
    // P2: short-lived cache so repeated polls of the same error_text don't
    // re-embed + re-search every 12 s.
    cache: new Map(), // error_text → { hits, ts }
  },
};

const RECALL_CACHE_TTL_MS = 60_000;
const RECALL_CACHE_MAX = 50;

document.addEventListener("DOMContentLoaded", async () => {
  buildLensSliders();
  attachEvents();
  attachReplayEvents();
  attachRecallBannerEvents();
  await pollUntilReady();
  startRecallPolling();
});

async function pollUntilReady(attempt = 0) {
  // AppState is `.manage()`d eagerly now, but its slots (Qdrant + fastembed)
  // init lazily on the first command call. First-time launch may need ~10 s
  // to download the BGE-small ONNX model (~130 MB), so we poll patiently and
  // separate the "still warming up" UI from the "real problem" UI.
  const MAX_ATTEMPTS = 240; // 240 × 500 ms = 120 s, comfortable for cold start
  const RETRY_MS = 500;
  try {
    const info = await invoke("collection_info");
    state.collectionPoints = info.points_count;
    setStatus(
      `Connected — ${info.points_count} sessions indexed (${info.collection})`,
    );
    document.getElementById("collection-info").textContent =
      `· ${info.points_count} sessions`;
  } catch (err) {
    const msg = String(err);
    // Distinguish: still cold-starting vs. true failure (Qdrant down / model
    // load borked). Keep retrying in both cases but show an actionable hint
    // once we've waited a while.
    if (attempt < MAX_ATTEMPTS) {
      const sec = Math.round((attempt * RETRY_MS) / 1000);
      let hint = `Bootstrapping… (${sec}s)`;
      if (msg.includes("could not connect to Qdrant") || msg.includes("connection")) {
        hint = `Qdrant not reachable on :6334 — start it with \`./.qdrant/qdrant\` then this banner will clear automatically.`;
      } else if (msg.includes("fastembed") || msg.includes("BGE")) {
        hint = `Loading BGE-small embedder (first launch may take ~30s for model download)…`;
      } else if (msg.includes("state not managed")) {
        hint = `Warming up… (${sec}s)`;
      }
      setStatus(hint);
      setTimeout(() => pollUntilReady(attempt + 1), RETRY_MS);
      return;
    }
    setStatus(`Memex couldn't bootstrap after 2 minutes — last error: ${msg}`);
  }
}

function attachEvents() {
  const input = document.getElementById("search-input");
  input.addEventListener("input", debounce(onSearchInput, 200));
  document.addEventListener("keydown", (e) => {
    if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "k") {
      e.preventDefault();
      input.focus();
      input.select();
    }
  });

  document.getElementById("btn-topology").addEventListener("click", onTopology);
  document.getElementById("btn-mix").addEventListener("click", openMixModal);
  document.getElementById("btn-snapshot").addEventListener("click", onSnapshot);
  document.getElementById("btn-refresh").addEventListener("click", onRefresh);
  document.getElementById("btn-reset-lens").addEventListener("click", resetLens);
  document.getElementById("btn-recall").addEventListener("click", onRecall);
  document.getElementById("btn-run-mix").addEventListener("click", runMix);

  for (const closer of document.querySelectorAll("[data-close]")) {
    closer.addEventListener("click", (e) => {
      e.target.closest("dialog").close();
    });
  }
}

async function onSearchInput(e) {
  state.query = e.target.value.trim();
  if (!state.query) {
    document.getElementById("results-empty").style.display = "";
    document
      .getElementById("results")
      .querySelectorAll(".card")
      .forEach((c) => c.remove());
    return;
  }
  await runLensSearch();
}

async function runLensSearch() {
  const gen = ++state.queryGen;
  const t0 = performance.now();
  setStatus(`Searching "${state.query}"…`);
  try {
    const hits = await invoke("lens_search", {
      query: state.query,
      weights: state.weights,
      limit: 20,
    });
    // B3: if a newer query has already been dispatched, drop this stale
    // response so we don't overwrite a fresher result list.
    if (gen !== state.queryGen) return;
    state.hits = hits;
    renderResults(hits);
    setStatus(`${hits.length} hits for "${state.query}"`);
    setLatency(Math.round(performance.now() - t0));
  } catch (err) {
    if (gen !== state.queryGen) return;
    setStatus(`Search failed: ${err}`);
  }
}

function buildLensSliders() {
  const root = document.getElementById("lens-sliders");
  root.innerHTML = "";
  for (const name of LENSES) {
    const wrap = document.createElement("div");
    wrap.className = "slider";
    wrap.innerHTML = `
      <div class="slider-label">
        <span>${name}</span>
        <span class="slider-value" data-for="${name}">1.00</span>
      </div>
      <input type="range" min="0" max="2" step="0.05" value="1.0" data-name="${name}" />
    `;
    const input = wrap.querySelector("input");
    const value = wrap.querySelector(".slider-value");
    input.addEventListener("input", (e) => {
      const v = parseFloat(e.target.value);
      state.weights[name] = v;
      value.textContent = v.toFixed(2);
    });
    input.addEventListener(
      "change",
      debounce(() => {
        if (state.query) runLensSearch();
      }, 150),
    );
    root.appendChild(wrap);
  }
}

function resetLens() {
  for (const name of LENSES) state.weights[name] = 1.0;
  document
    .querySelectorAll("#lens-sliders input")
    .forEach((i) => (i.value = "1.0"));
  document
    .querySelectorAll(".slider-value")
    .forEach((s) => (s.textContent = "1.00"));
  if (state.query) runLensSearch();
}

function renderResults(hits) {
  const root = document.getElementById("results");
  root.querySelectorAll(".card").forEach((c) => c.remove());
  document.getElementById("results-empty").style.display = hits.length ? "none" : "";
  for (const h of hits) {
    const card = document.createElement("article");
    card.className = "card";
    card.dataset.sessionId = h.session_id;
    const ts = (h.start_iso || "").slice(0, 16).replace("T", " ");
    const title = h.ai_title || "(untitled)";
    const vecBreak = Object.entries(h.vector_scores || {})
      .sort((a, b) => b[1] - a[1])
      .map(([k, v]) => `<span class="vec-chip">${k} ${v.toFixed(2)}</span>`)
      .join("");
    card.innerHTML = `
      <header>
        <span class="score">${(h.score ?? 0).toFixed(3)}</span>
        <span class="proj">${escapeHtml(h.project_name || "?")}</span>
        <span class="ts">${ts}</span>
      </header>
      <h3 class="title">${escapeHtml(title)}</h3>
      <div class="vec-breakdown">${vecBreak}</div>
      <footer>
        <code class="sid">${h.session_id}</code>
        <button class="btn ghost xs" data-action="replay">Replay</button>
        <button class="btn ghost xs" data-action="add-positive">+ pos</button>
        <button class="btn ghost xs" data-action="add-negative">− neg</button>
      </footer>
    `;
    card.addEventListener("click", () => selectSession(h.session_id));
    card
      .querySelectorAll("button[data-action]")
      .forEach((btn) =>
        btn.addEventListener("click", (e) => {
          e.stopPropagation();
          if (btn.dataset.action === "replay") {
            openReplay(h.session_id, h);
          } else {
            const side =
              btn.dataset.action === "add-positive" ? "positive" : "negative";
            addToMix(side, h.session_id);
          }
        }),
      );
    root.appendChild(card);
  }
}

async function selectSession(sessionId) {
  state.selected = sessionId;
  for (const c of document.querySelectorAll(".card")) {
    c.classList.toggle("selected", c.dataset.sessionId === sessionId);
  }
  const inspector = document.getElementById("inspector");
  inspector.innerHTML = `<div class="empty">Loading ${sessionId.slice(0, 8)}…</div>`;
  try {
    const payload = await invoke("get_session", { sessionId });
    renderInspector(payload, sessionId);
  } catch (err) {
    inspector.innerHTML = `<div class="empty">Error: ${escapeHtml(String(err))}</div>`;
  }
}

function renderInspector(payload, sessionId) {
  const inspector = document.getElementById("inspector");
  if (!payload) {
    inspector.innerHTML = `<div class="empty">Session ${sessionId} not in index.</div>`;
    return;
  }
  const fields = [
    ["session_id", payload.session_id],
    ["project", payload.project_name],
    ["path", payload.project_path],
    ["branch", payload.git_branch],
    ["claude version", payload.claude_version],
    ["title", payload.ai_title || "(untitled)"],
    ["started", payload.start_iso],
    ["ended", payload.end_iso],
    ["user turns", payload.user_turns],
    ["assistant turns", payload.assistant_turns],
    ["tool calls", payload.tool_count],
    ["had errors", payload.has_errors],
  ];
  const rows = fields
    .map(
      ([k, v]) => `
      <div class="kv">
        <span class="k">${escapeHtml(k)}</span>
        <span class="v">${escapeHtml(String(v ?? ""))}</span>
      </div>`,
    )
    .join("");
  inspector.innerHTML = `
    <header class="inspector-head">
      <h3>${escapeHtml(payload.project_name || "session")}</h3>
      <code>${escapeHtml(sessionId)}</code>
    </header>
    <div class="kvs">${rows}</div>
    <details class="raw">
      <summary>Raw payload</summary>
      <pre>${escapeHtml(JSON.stringify(payload, null, 2))}</pre>
    </details>
  `;
}

async function onRecall() {
  const text = document.getElementById("recall-input").value.trim();
  if (!text) {
    setStatus("Paste an error message into the Recall box first.");
    return;
  }
  setStatus("Searching past errors…");
  try {
    const hits = await invoke("recall", { errorText: text, limit: 5 });
    renderResults(hits);
    setStatus(`Recall: ${hits.length} past session(s) match.`);
  } catch (err) {
    setStatus(`Recall failed: ${err}`);
  }
}

async function onTopology() {
  const modal = document.getElementById("topology-modal");
  modal.showModal();
  const canvas = document.getElementById("topology-canvas");
  canvas.innerHTML = `<div class="empty">Computing MST…</div>`;
  try {
    const topo = await invoke("topology", { sample: 80, perPoint: 4 });
    renderTopologySvg(topo, canvas);
  } catch (err) {
    canvas.innerHTML = `<div class="empty">Topology failed: ${escapeHtml(String(err))}</div>`;
  }
}

function renderTopologySvg(topo, mount) {
  const W = mount.clientWidth || 900;
  const H = mount.clientHeight || 540;
  const { nodes, edges } = topo;
  if (!nodes.length) {
    mount.innerHTML = `<div class="empty">No nodes yet — re-index first.</div>`;
    return;
  }
  const positions = new Map();
  const cx = W / 2,
    cy = H / 2;
  const r = Math.min(W, H) * 0.42;
  nodes.forEach((n, i) => {
    const t = (i / nodes.length) * 2 * Math.PI;
    positions.set(n.session_id, [cx + r * Math.cos(t), cy + r * Math.sin(t)]);
  });

  const svgNS = "http://www.w3.org/2000/svg";
  const svg = document.createElementNS(svgNS, "svg");
  svg.setAttribute("width", "100%");
  svg.setAttribute("height", "100%");
  svg.setAttribute("viewBox", `0 0 ${W} ${H}`);

  for (const e of edges) {
    const a = positions.get(e.a);
    const b = positions.get(e.b);
    if (!a || !b) continue;
    const line = document.createElementNS(svgNS, "line");
    line.setAttribute("x1", a[0]);
    line.setAttribute("y1", a[1]);
    line.setAttribute("x2", b[0]);
    line.setAttribute("y2", b[1]);
    // After B1 the Rust side returns true distance (low = similar). For the
    // SVG we render *similarity* — closer pairs = thicker, more opaque edges.
    const sim = Math.max(0, Math.min(1, 1 - e.distance));
    const opacity = Math.max(0.12, Math.min(0.85, sim));
    line.setAttribute("stroke", `rgba(10, 132, 255, ${opacity})`);
    line.setAttribute("stroke-width", String(0.5 + sim * 2));
    svg.appendChild(line);
  }

  for (const n of nodes) {
    const p = positions.get(n.session_id);
    if (!p) continue;
    const g = document.createElementNS(svgNS, "g");
    g.setAttribute("transform", `translate(${p[0]}, ${p[1]})`);
    g.style.cursor = "pointer";
    g.addEventListener("click", () => {
      document.getElementById("topology-modal").close();
      selectSession(n.session_id);
    });
    const circle = document.createElementNS(svgNS, "circle");
    circle.setAttribute("r", Math.max(4, Math.min(12, Math.sqrt((n.user_turns || 0) + 1))));
    circle.setAttribute("fill", projectColor(n.project_name));
    circle.setAttribute("stroke", "#fff");
    circle.setAttribute("stroke-opacity", "0.4");
    const label = document.createElementNS(svgNS, "text");
    label.setAttribute("x", "10");
    label.setAttribute("y", "4");
    label.setAttribute("fill", "rgba(255,255,255,0.7)");
    label.setAttribute("font-size", "10");
    label.textContent = (n.project_name || "?").slice(0, 18);
    g.appendChild(circle);
    g.appendChild(label);
    svg.appendChild(g);
  }

  mount.innerHTML = "";
  mount.appendChild(svg);
}

const COLOR_PALETTE = [
  "#ff9f0a", "#bf5af2", "#30d158", "#ff375f", "#ffd60a",
  "#0a84ff", "#ff6b35", "#af52de", "#66d4cf", "#ffb340",
  "#5e5ce6", "#ff453a", "#d8e056", "#4cc9b3", "#5ac8fa",
];

function projectColor(name) {
  if (!name) return "#888";
  let h = 0;
  for (let i = 0; i < name.length; i++) h = (h * 31 + name.charCodeAt(i)) | 0;
  return COLOR_PALETTE[Math.abs(h) % COLOR_PALETTE.length];
}

function openMixModal() {
  renderMixDropzones();
  document.getElementById("mix-results").innerHTML = "";
  document.getElementById("mix-modal").showModal();
}

function addToMix(side, sessionId) {
  if (!state.mix[side].includes(sessionId)) {
    state.mix[side].push(sessionId);
  }
  renderMixDropzones();
}

function removeFromMix(side, sessionId) {
  state.mix[side] = state.mix[side].filter((s) => s !== sessionId);
  renderMixDropzones();
}

function renderMixDropzones() {
  for (const side of ["positive", "negative"]) {
    const root = document.getElementById(`mix-${side}`);
    root.innerHTML = "";
    if (!state.mix[side].length) {
      const hint = document.createElement("span");
      hint.className = "dropzone-hint";
      hint.textContent = "click + pos / − neg on a card to add…";
      root.appendChild(hint);
      continue;
    }
    for (const sid of state.mix[side]) {
      const chip = document.createElement("span");
      chip.className = "chip";
      chip.textContent = sid.slice(0, 8) + "…";
      const close = document.createElement("button");
      close.type = "button";
      close.textContent = "×";
      close.className = "chip-close";
      close.addEventListener("click", () => removeFromMix(side, sid));
      chip.appendChild(close);
      root.appendChild(chip);
    }
  }
}

async function runMix() {
  if (!state.mix.positive.length && !state.mix.negative.length) {
    document.getElementById("mix-results").textContent =
      "Add at least one positive or negative session first.";
    return;
  }
  const out = document.getElementById("mix-results");
  out.textContent = "Running discovery…";
  try {
    const hits = await invoke("mix_match", {
      positive: state.mix.positive,
      negative: state.mix.negative,
      limit: 10,
    });
    if (!hits.length) {
      out.textContent = "No discovery hits.";
      return;
    }
    out.innerHTML = "<h4>Discovered</h4>";
    for (const h of hits) {
      const row = document.createElement("div");
      row.className = "mix-row";
      row.textContent = `${h.score.toFixed(3)}  ${h.project_name}  ${h.session_id}`;
      out.appendChild(row);
    }
  } catch (err) {
    out.textContent = `Mix failed: ${err}`;
  }
}

async function onSnapshot() {
  const path = prompt(
    "Snapshot destination path:",
    `/tmp/memex-${new Date().toISOString().slice(0, 10)}.snapshot`,
  );
  if (!path) return;
  setStatus("Exporting snapshot…");
  try {
    const name = await invoke("snapshot_export", { path });
    setStatus(`Snapshot '${name}' → ${path}`);
  } catch (err) {
    setStatus(`Snapshot failed: ${err}`);
  }
}

async function onRefresh() {
  setStatus("Re-indexing ~/.claude/projects…");
  try {
    const r = await invoke("refresh_index");
    state.collectionPoints = r.indexed;
    const dup = r.duplicates_skipped
      ? ` (${r.duplicates_skipped} duplicate sessionId(s) skipped)`
      : "";
    const errs = r.errors ? ` · ${r.errors} error(s)` : "";
    setStatus(`Re-indexed ${r.indexed}/${r.total_scanned} sessions${dup}${errs}.`);
    document.getElementById("collection-info").textContent = `· ${r.indexed} sessions`;
  } catch (err) {
    setStatus(`Re-index failed: ${err}`);
  }
}

function setStatus(msg) {
  document.getElementById("status-text").textContent = msg;
}

function setLatency(ms) {
  document.getElementById("status-latency").textContent = ms ? `${ms} ms` : "";
}

function debounce(fn, wait) {
  let t = null;
  return (...args) => {
    if (t) clearTimeout(t);
    t = setTimeout(() => fn(...args), wait);
  };
}

function escapeHtml(s) {
  return String(s).replace(/[&<>"']/g, (c) =>
    ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c],
  );
}

// ---------------------------------------------------------------------------
// Phase 5 — Replay engine
// ---------------------------------------------------------------------------

function attachReplayEvents() {
  document.getElementById("replay-prev").addEventListener("click", () => stepReplay(-1));
  document.getElementById("replay-next").addEventListener("click", () => stepReplay(+1));
  document.getElementById("replay-play").addEventListener("click", toggleReplayPlay);
  document
    .getElementById("replay-speed")
    .addEventListener("change", (e) => {
      state.replay.speedMs = parseInt(e.target.value, 10);
      if (state.replay.playing) {
        stopReplayTimer();
        startReplayTimer();
      }
    });
  document.getElementById("replay-modal").addEventListener("close", () => {
    stopReplayTimer();
    state.replay.playing = false;
  });
}

async function openReplay(sessionId, hit) {
  const modal = document.getElementById("replay-modal");
  modal.showModal();
  document.getElementById("replay-title").textContent = `Replay — ${hit?.project_name || sessionId.slice(0, 8)}`;
  document.getElementById("replay-detail").innerHTML =
    `<div class="empty">Loading session…</div>`;
  document.getElementById("replay-list").innerHTML = "";
  state.replay = {
    sessionId,
    turns: [],
    cursor: 0,
    playing: false,
    speedMs: parseInt(document.getElementById("replay-speed").value, 10),
    timer: null,
  };
  try {
    const session = await invoke("get_session_turns", { sessionId });
    state.replay.turns = session.turns || [];
    if (!state.replay.turns.length) {
      document.getElementById("replay-detail").innerHTML =
        `<div class="empty">Session has no turns to replay.</div>`;
      return;
    }
    renderReplayList(session);
    renderReplayTurn(0);
  } catch (err) {
    document.getElementById("replay-detail").innerHTML =
      `<div class="empty">Failed to load: ${escapeHtml(String(err))}</div>`;
  }
}

function renderReplayList(session) {
  const list = document.getElementById("replay-list");
  list.innerHTML = "";
  state.replay.turns.forEach((turn, i) => {
    const row = document.createElement("button");
    row.type = "button";
    row.className = "replay-row";
    row.dataset.index = i;
    const role = turn.role === "user" ? "U" : turn.role === "assistant" ? "A" : "S";
    const preview = (turn.text || "")
      .replace(/\s+/g, " ")
      .slice(0, 60);
    const tools = (turn.tool_calls || [])
      .map((t) => t.name)
      .join(", ");
    row.innerHTML = `
      <span class="replay-row-role role-${turn.role}">${role}</span>
      <span class="replay-row-preview">${escapeHtml(preview || tools || "(empty)")}</span>
      <span class="replay-row-meta">${turn.tool_calls?.length ? "🔧" + turn.tool_calls.length : ""}</span>
    `;
    row.addEventListener("click", () => {
      state.replay.cursor = i;
      renderReplayTurn(i);
      stopReplayTimer();
      state.replay.playing = false;
      updatePlayButton();
    });
    list.appendChild(row);
  });
  updateProgress();
}

function renderReplayTurn(i) {
  const turn = state.replay.turns[i];
  if (!turn) return;
  const detail = document.getElementById("replay-detail");
  const ts = turn.timestamp ? new Date(turn.timestamp).toISOString().slice(0, 19).replace("T", " ") : "";

  const toolViz = (turn.tool_calls || [])
    .map((tc) => renderToolCall(tc, turn.tool_results || []))
    .join("");

  const toolResults = (turn.tool_results || [])
    .filter((r) => !(turn.tool_calls || []).some((tc) => tc.id === r.tool_use_id))
    .map(renderStrayResult)
    .join("");

  detail.innerHTML = `
    <div class="turn-meta">
      <span class="turn-role role-${turn.role}">${turn.role}</span>
      <span class="muted">${escapeHtml(ts)}</span>
      ${turn.is_sidechain ? '<span class="badge">sidechain</span>' : ""}
    </div>
    ${turn.text ? `<pre class="turn-text">${escapeHtml(turn.text)}</pre>` : ""}
    ${toolViz}
    ${toolResults}
  `;

  // Highlight the selected row + scroll into view.
  for (const r of document.querySelectorAll(".replay-row")) {
    r.classList.toggle("selected", parseInt(r.dataset.index, 10) === i);
  }
  const selectedRow = document.querySelector(`.replay-row[data-index="${i}"]`);
  if (selectedRow) selectedRow.scrollIntoView({ block: "nearest" });
  state.replay.cursor = i;
  updateProgress();
}

function renderToolCall(tc, results) {
  const result = results.find((r) => r.tool_use_id === tc.id);
  const input = tc.input || {};
  // B4: tool name comes from the model + jsonl source. For Claude Code it's
  // a fixed whitelist, but a foreign jsonl could carry arbitrary HTML — be
  // consistent and escape it everywhere.
  const name = escapeHtml(tc.name || "?");
  // Inline `tool-error` class up front so we don't rely on String.replace.
  const errCls = result && result.is_error ? " tool-error" : "";
  let body = "";
  switch (tc.name) {
    case "Bash": {
      body = `
        <div class="tool-block tool-bash${errCls}">
          <div class="tool-head">Bash · ${escapeHtml(input.description || "")}</div>
          <pre class="terminal">$ ${escapeHtml(input.command || "")}</pre>
          ${result ? `<pre class="terminal output">${escapeHtml(result.content.slice(0, 1200))}</pre>` : ""}
        </div>`;
      break;
    }
    case "Edit":
    case "MultiEdit": {
      body = `
        <div class="tool-block tool-edit${errCls}">
          <div class="tool-head">Edit · <code>${escapeHtml(input.file_path || "")}</code></div>
          ${input.old_string ? `<pre class="diff diff-old">- ${escapeHtml(input.old_string.slice(0, 600))}</pre>` : ""}
          ${input.new_string ? `<pre class="diff diff-new">+ ${escapeHtml(input.new_string.slice(0, 600))}</pre>` : ""}
        </div>`;
      break;
    }
    case "Write": {
      body = `
        <div class="tool-block tool-edit${errCls}">
          <div class="tool-head">Write · <code>${escapeHtml(input.file_path || "")}</code></div>
          <pre class="diff diff-new">${escapeHtml(String(input.content || "").slice(0, 800))}</pre>
        </div>`;
      break;
    }
    case "Read": {
      body = `
        <div class="tool-block tool-read${errCls}">
          <div class="tool-head">Read · <code>${escapeHtml(input.file_path || "")}</code></div>
          ${result ? `<pre class="terminal output">${escapeHtml(result.content.slice(0, 800))}</pre>` : ""}
        </div>`;
      break;
    }
    case "WebFetch":
    case "WebSearch": {
      body = `
        <div class="tool-block${errCls}">
          <div class="tool-head">${name} · ${escapeHtml(input.url || input.query || "")}</div>
          ${result ? `<pre class="terminal output">${escapeHtml(result.content.slice(0, 600))}</pre>` : ""}
        </div>`;
      break;
    }
    case "Task":
    case "Agent": {
      body = `
        <div class="tool-block${errCls}">
          <div class="tool-head">${name} · ${escapeHtml(input.subagent_type || input.description || "")}</div>
          <pre class="diff diff-new">${escapeHtml((input.prompt || "").slice(0, 600))}</pre>
        </div>`;
      break;
    }
    default: {
      body = `
        <div class="tool-block${errCls}">
          <div class="tool-head">${name}</div>
          <pre class="terminal">${escapeHtml(JSON.stringify(input).slice(0, 600))}</pre>
          ${result ? `<pre class="terminal output">${escapeHtml(result.content.slice(0, 600))}</pre>` : ""}
        </div>`;
    }
  }
  return body;
}

function renderStrayResult(r) {
  return `
    <div class="tool-block ${r.is_error ? "tool-error" : ""}">
      <div class="tool-head">tool_result</div>
      <pre class="terminal output">${escapeHtml(r.content.slice(0, 800))}</pre>
    </div>`;
}

function stepReplay(delta) {
  const total = state.replay.turns.length;
  if (!total) return;
  const next = Math.max(0, Math.min(total - 1, state.replay.cursor + delta));
  renderReplayTurn(next);
}

function toggleReplayPlay() {
  state.replay.playing = !state.replay.playing;
  updatePlayButton();
  if (state.replay.playing) startReplayTimer();
  else stopReplayTimer();
}

function startReplayTimer() {
  stopReplayTimer();
  state.replay.timer = setInterval(() => {
    const next = state.replay.cursor + 1;
    if (next >= state.replay.turns.length) {
      stopReplayTimer();
      state.replay.playing = false;
      updatePlayButton();
      return;
    }
    renderReplayTurn(next);
  }, state.replay.speedMs);
}

function stopReplayTimer() {
  if (state.replay.timer) {
    clearInterval(state.replay.timer);
    state.replay.timer = null;
  }
}

function updatePlayButton() {
  document.getElementById("replay-play").textContent = state.replay.playing ? "❚❚" : "▶";
}

function updateProgress() {
  document.getElementById("replay-progress").textContent =
    `${state.replay.cursor + 1} / ${state.replay.turns.length}`;
}

// ---------------------------------------------------------------------------
// Phase 6 — Proactive recall (polling)
// ---------------------------------------------------------------------------

const RECALL_POLL_MS = 12_000;

async function startRecallPolling() {
  try {
    await pollRecall();
  } catch {}
  setInterval(pollRecall, RECALL_POLL_MS);
}

async function pollRecall() {
  try {
    const recent = await invoke("tail_recent_errors", { sinceSeconds: 90 });
    if (!recent || !recent.length) return;
    // Pick the most-recent error not already shown.
    for (const ev of recent) {
      const key = `${ev.session_id}::${ev.error_text.slice(0, 80)}`;
      if (state.recall.dismissedKeys.has(key)) continue;
      const hits = await recallCached(ev.error_text, 3);
      // Filter out the current (still-failing) session itself.
      const useful = hits.filter((h) => h.session_id !== ev.session_id);
      if (!useful.length) continue;
      showRecallBanner(ev, useful);
      state.recall.lastBannerError = { ev, key, hits: useful };
      return; // only one banner at a time
    }
  } catch (err) {
    // Silent — Qdrant may not be ready yet.
  }
}

// P2: deduplicate recall calls by error_text within a short TTL so the
// 12 s polling loop doesn't re-embed the same error over and over.
async function recallCached(errorText, limit) {
  const cache = state.recall.cache;
  const prev = cache.get(errorText);
  if (prev && Date.now() - prev.ts < RECALL_CACHE_TTL_MS) {
    return prev.hits;
  }
  const hits = await invoke("recall", { errorText, limit });
  cache.set(errorText, { hits, ts: Date.now() });
  if (cache.size > RECALL_CACHE_MAX) {
    const oldest = cache.keys().next().value;
    cache.delete(oldest);
  }
  return hits;
}

function showRecallBanner(ev, hits) {
  const banner = document.getElementById("recall-banner");
  const detail = document.getElementById("recall-banner-detail");
  detail.textContent = `${ev.project_name || "?"} just hit: "${ev.error_text.slice(0, 120).replace(/\s+/g, " ")}" — ${hits.length} past session(s) may help`;
  banner.classList.remove("hidden");
}

function attachRecallBannerEvents() {
  document.getElementById("recall-banner-dismiss").addEventListener("click", () => {
    if (state.recall.lastBannerError) {
      state.recall.dismissedKeys.add(state.recall.lastBannerError.key);
    }
    document.getElementById("recall-banner").classList.add("hidden");
  });
  document.getElementById("recall-banner-open").addEventListener("click", () => {
    const ctx = state.recall.lastBannerError;
    if (!ctx) return;
    document.getElementById("recall-banner").classList.add("hidden");
    state.recall.dismissedKeys.add(ctx.key);
    // Open replay for the first past-fix candidate.
    const target = ctx.hits[0];
    openReplay(target.session_id, target);
  });
}
