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
};

document.addEventListener("DOMContentLoaded", async () => {
  buildLensSliders();
  attachEvents();
  await pollUntilReady();
});

async function pollUntilReady(attempt = 0) {
  // AppState is initialized async in Tauri setup. Poll until ready.
  try {
    const info = await invoke("collection_info");
    state.collectionPoints = info.points_count;
    setStatus(
      `Connected — ${info.points_count} sessions indexed (${info.collection})`,
    );
    document.getElementById("collection-info").textContent =
      `· ${info.points_count} sessions`;
  } catch (err) {
    if (attempt < 60) {
      setStatus(`Waiting for index… (${attempt}s)`);
      setTimeout(() => pollUntilReady(attempt + 1), 500);
      return;
    }
    setStatus(`Could not reach Qdrant: ${err}`);
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
  const t0 = performance.now();
  setStatus(`Searching "${state.query}"…`);
  try {
    const hits = await invoke("lens_search", {
      query: state.query,
      weights: state.weights,
      limit: 20,
    });
    state.hits = hits;
    renderResults(hits);
    setStatus(`${hits.length} hits for "${state.query}"`);
    setLatency(Math.round(performance.now() - t0));
  } catch (err) {
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
          const side =
            btn.dataset.action === "add-positive" ? "positive" : "negative";
          addToMix(side, h.session_id);
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
    const opacity = Math.max(0.1, Math.min(0.9, e.distance));
    line.setAttribute("stroke", `rgba(10, 132, 255, ${opacity})`);
    line.setAttribute("stroke-width", String(0.5 + e.distance * 2));
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
    const n = await invoke("refresh_index");
    state.collectionPoints = n;
    setStatus(`Re-index complete: ${n} sessions indexed.`);
    document.getElementById("collection-info").textContent = `· ${n} sessions`;
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
