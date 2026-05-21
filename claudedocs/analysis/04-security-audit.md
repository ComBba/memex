# Memex Security Audit Report

**Date**: 2026-05-18
**Auditor**: Security Engineer (Claude)
**Scope**: src-tauri/src/, src/main.js, src/vendor/, tauri.conf.json, capabilities/default.json, Cargo.toml
**Codebase**: /Users/kimsejun/Documents/GitHub/memex

---

## 1. Threat Model

### User Profile

A software developer running Claude Code locally on macOS. The application reads ~/.claude/projects/**/*.jsonl -- months of personal AI session history that may contain source code snippets, API responses, credential strings typed in prompts, internal project names, file paths, error messages with secrets, and sensitive command output.

### Assets at Risk

- **Raw JSONL files** (~/.claude/projects/): Highest-sensitivity personal and work data. A single session file may contain shell commands with tokens, partial secrets in tool output, or private project architecture.
- **Qdrant vector index** (localhost:6334): Derived representations of all JSONL content including 384-dimensional embeddings plus full text payloads (source_path, project_name, git_branch, ai_title). Leaking the index is nearly as bad as leaking the raw files.
- **fastembed model cache** (~/Library/Caches/dev.sgwannabe.memex/fastembed/): Not sensitive itself, but the cache directory could be targeted by other local processes.

### Adversaries In Scope

1. **Local malware or other desktop apps** reading Qdrant REST on localhost:6333 (no authentication by default on Qdrant open-source).
2. **Malicious .jsonl planted in ~/.claude/projects/** by a compromised tool injecting payloads, oversized content, or path-traversal strings into session fields.
3. **Supply-chain compromise** of Cargo.toml dependencies, especially hf-hub which makes outbound HTTPS calls on first run.
4. **Malicious Qdrant snapshot** imported by the user via snapshot import path.
5. **MITM on the BGE-small model download** -- the Hugging Face download on first launch goes over HTTPS but the application performs no additional checksum pinning beyond what hf-hub provides.

### Out of Scope

Physical machine theft, compromise of the Qdrant binary itself, macOS sandbox escapes, network-level MITMs after the model is already cached.

---

## 2. Network-Call Audit

**Claim**: zero telemetry, zero network calls at runtime, 100% local.

### Verdict: PARTIALLY TRUE -- with two important caveats

The app emits **no telemetry** and no analytics beacons. However:

### 2a. reqwest -- snapshot-only, localhost only (CONFIRMED SAFE at runtime)

Two reqwest::Client instances are created in indexer.rs:

| Location | URL | Condition |
|---|---|---|
| indexer.rs:1531 | http://localhost:6333/collections/memex_sessions/snapshots (POST) | Only when snapshot_export is invoked |
| indexer.rs:1563-1573 | http://localhost:6333/collections/memex_sessions/snapshots/upload (POST) | Only when snapshot_import is invoked |

Both URLs resolve to localhost (configurable via MEMEX_QDRANT_HTTP env var). These are not outbound Internet calls.

### 2b. Qdrant gRPC -- localhost only (CONFIRMED SAFE)

indexer.rs:127 connects to http://localhost:6334 via gRPC (configurable via MEMEX_QDRANT_URL). All Qdrant traffic stays loopback.

### 2c. fastembed / hf-hub -- OUTBOUND on first run (SEC-001)

indexer.rs:60-70 calls TextEmbedding::try_new with EmbeddingModel::BGESmallENV15. On the very first launch (or absent cache), hf-hub 0.5.0 downloads the BGE-small-en-v1.5 ONNX model (~130 MB) from huggingface.co over HTTPS.

- No application-level checksum pinning is performed; the app trusts hf-hub SHA-256 verification against LFS repository metadata.
- Network is NOT required after first run (model cached at ~/Library/Caches/dev.sgwannabe.memex/fastembed/).
- The 100% local at runtime claim is accurate for steady-state but **false on first launch**.

### 2d. Frontend -- no outbound calls (CONFIRMED)

src/main.js contains zero fetch() calls, zero XMLHttpRequest, and zero hardcoded external URLs. All communication is via Tauri IPC invoke().

### 2e. Vendored library

src/vendor/3d-force-graph.min.js (v1.73.4, 690 KB, SHA-256: 0ddfb909b551f4fdeb6b43cca567e71f2bc82b2e2eadf1d7cda65af8961b2d0f) contains exactly one fetch() call -- the Three.js GLTF/object loader triggered only by .jsonUrl() which Memex never calls. **This fetch path is dead code and never triggered.**

---

## 3. Tauri 2 Capability Surface

src-tauri/capabilities/default.json grants two permissions:

- **core:default** -- app, event, window, webview, menu, tray, path APIs. Does NOT include fs:default, fs:read, fs:write, or http:default.
- **opener:default** -- open files/URLs with OS default application handler.

No filesystem capability is declared in the manifest. All file I/O is performed via Rust tauri::command functions, not via the frontend calling tauri-plugin-fs directly. This is correct Tauri 2 architecture.

**opener:default risk**: Memex does not expose any IPC command that calls the opener with user-controlled paths. The only opener-adjacent code is the tray handler which calls a fixed window API. Risk is Low.

---

## 4. CSP and Window Security

tauri.conf.json:24-27 sets csp: null.

**csp: null means Tauri applies NO Content Security Policy to the webview.** (SEC-002)

Without a CSP, inline script tags injected via innerHTML assignment can execute; dynamic code execution APIs are unrestricted; any img/iframe/link element pointing to external URLs can be injected by untrusted content.

**withGlobalTauri: true** (tauri.conf.json:10) places window.__TAURI__ in the global scope. If a cross-site scripting vulnerability allows arbitrary script execution, the attacker gains access to all Tauri IPC commands including snapshot_export and refresh_index. This materially raises the blast radius of any injection finding from DOM manipulation to full filesystem read and Qdrant data exfiltration.

---

## 5. IPC Surface Input Validation

All tauri::command functions analyzed:

| Command | Inputs from webview | Validation | Blast radius if hostile |
|---|---|---|---|
| lens_search | query String, weights LensWeights, limit u64 | None | Qdrant embed+query; no file access |
| mix_match | positive/negative Vec String, limit u64 | None | Qdrant query only |
| topology | sample u32, per_point u32, path Option PathBuf | None on path | Walks any directory (SEC-005) |
| recall | error_text String, limit u64 | None | Qdrant query only |
| get_session | session_id String | None | Qdrant payload lookup |
| get_session_turns | session_id String | None on source_path from payload | Reads arbitrary file from Qdrant payload (SEC-003) |
| snapshot_export | path PathBuf | None | Writes snapshot to any filesystem path (SEC-004) |
| snapshot_import | path PathBuf | None | Reads any file; uploads to Qdrant |
| refresh_index | path Option PathBuf | None | Walks and reads any directory (SEC-005) |
| list_sessions | path Option PathBuf, limit usize | None | Reads any directory |
| tail_recent_errors | path Option PathBuf, since_seconds u64 | None | Reads any directory |
| predict_next_actions | session_id String, last_n_turns, horizon, neighbors | None | Reads file at source_path from Qdrant payload (SEC-003) |

### SEC-003 Detail: Qdrant payload poisoning via source_path

commands.rs:195, indexer.rs:1268, indexer.rs:1342 all call:

parser::parse_session(std::path::Path::new(&source))

The source_path field from the Qdrant payload is passed directly to File::open with **no path containment check**. If an attacker poisons the Qdrant collection (e.g., via malicious snapshot import), source_path can point to any file on the filesystem readable by the process.

---

## 6. JSONL Parsing Safety

parser.rs analysis:

- **Malformed JSON** (parser.rs:114-116): serde_json::from_str returns a Result; errors are propagated and the session parse fails. scan_dir at parser.rs:350-354 catches per-session errors and continues -- no panic.
- **Deeply nested structures**: serde_json::Value uses recursive parsing. Claude Code output format is bounded in practice; no stack overflow scenarios identified.
- **Gigabyte-sized files** (parser.rs:83): BufReader streams lines rather than reading the full file. However, a single JSON line with millions of characters is read entirely into memory. No per-line byte cap exists. (SEC-006)
- **Path traversal in content fields**: source_path stored in Session is always the actual filesystem path used to open the session (parser.rs:93). Fields like cwd inside the JSONL content only affect display strings (project_path, project_name), not file I/O.
- **NUL bytes and invalid UTF-8**: reader.lines() in Rust returns an error on invalid UTF-8 bytes; errors are propagated and logged -- no crash.
- **No fuzzing or property-based tests** exist. The dev-dependencies section only lists pretty_assertions.

---

## 7. Path Handling

All file-opening call sites:

| Location | Path source | Containment check |
|---|---|---|
| parser.rs:81 | WalkDir over caller-supplied root | None beyond WalkDir natural traversal |
| commands.rs:195 | source_path from Qdrant payload | **None** |
| indexer.rs:1268 | source_path from Qdrant payload | **None** |
| indexer.rs:1342 | source from neighbor session Qdrant payload | **None** |
| indexer.rs:1556 | dest.parent() of IPC-supplied PathBuf | None; creates arbitrary directories |
| indexer.rs:1564 | src: PathBuf from IPC snapshot_import | No validation |

topology command (commands.rs:124-137): The path: Option PathBuf parameter accepts any directory from the webview with no whitelist check. Calling invoke("topology", { path: "/" }) will walk the entire filesystem. (SEC-005)

snapshot_export (commands.rs:247-249, indexer.rs:1556): tokio::fs::create_dir_all(dest.parent()...) creates arbitrary directory trees, then tokio::fs::write(dest, bytes) writes to any writable path. (SEC-004)

---

## 8. Replay Rendering XSS Risk

**Safe paths** (confirmed via code review):

- renderToolCall (main.js:1322-1395): All tc.name, input.command, input.file_path, input.description, input.prompt, and result.content are wrapped in escapeHtml() before insertion into innerHTML template literals.
- renderStrayResult (main.js:1398-1403): r.content passed through escapeHtml().
- renderReplayList (main.js:1254-1283): turn.text preview uses escapeHtml().
- renderReplayTurn (main.js:1286-1319): turn.text rendered via escapeHtml() inside a pre element.
- topologyTooltip (main.js:1005-1021): escapeHtml(n.project) and escapeHtml(n.title) used correctly. Safe.

**Unsafe path found** (SEC-007):

main.js:381-383 builds the vector score breakdown:

.map(([k, v]) => span class vec-chip -- k inserted without escapeHtml(k)

The vector key k is inserted into innerHTML **without escapeHtml()**. Currently safe because keys are the fixed set content/tool/path/error/code set by the application itself. If the Qdrant collection is poisoned via snapshot import and a malicious key is stored, this becomes an HTML injection vector.

**Unescaped sessionId** (main.js:427, main.js:615):

inspector.innerHTML uses sessionId.slice(0, 8) and sessionId directly without escapeHtml(). UUIDs are safe hex+hyphens today. A poisoned collection with a non-UUID session_id could make this exploitable. (SEC-011)

---

## 9. Dependency Audit

Direct dependencies from Cargo.toml (2026-05 status):

| Crate | Declared | Locked | Status |
|---|---|---|---|
| tauri | 2.x | current | Maintained |
| qdrant-client | 1.x | current | Maintained |
| fastembed | 5.x | 5.13.4 | Maintained |
| reqwest | 0.12 | 0.12.28 | Maintained; rustls-tls feature confirmed |
| tokio | 1.x | current | Maintained |
| walkdir | 2.5 | current | Maintained |
| petgraph | 0.6 | 0.6.x | **Effectively unmaintained** since 2023-04; no CVEs known (SEC-009) |
| notify | 6.x | present | **Not imported anywhere in source** -- dead dependency (SEC-010) |
| notify-debouncer-full | 0.3 | present | **Not imported anywhere in source** -- dead dependency (SEC-010) |

**reqwest TLS**: Cargo.toml:37 declares features = ["rustls-tls"]. Confirmed correct for the application's own reqwest usage.

**However**: Cargo.lock reveals that hf-hub 0.5.0 (transitive via fastembed) depends on native-tls 0.2.18 and reqwest 0.12.28 with native-tls. This pulls openssl 0.10.80 into the build despite the application's own reqwest using rustls. On macOS 11+ native-tls uses Security.framework so direct OpenSSL exposure is minimal, but the supply chain is wider than the Cargo.toml feature choice implies. (SEC-008)

No direct CVEs found in declared dependencies as of 2026-05. cargo audit should be added to CI.

---

## 10. Snapshot Import Attack Surface

commands.rs:252-254: snapshot_import accepts a PathBuf from the webview with no validation, reads the file into memory, and POSTs it raw to Qdrant (indexer.rs:1561-1581). Qdrant 1.18 processes the snapshot server-side and replaces the collection.

**Worst-case attack chain if a user imports a malicious snapshot from the Internet**:

1. Qdrant collection is replaced with attacker-controlled points.
2. All payload fields (source_path, session_id, project_name) become attacker-controlled.
3. Any call to get_session_turns or predict_next_actions invokes parser::parse_session(Path::new(&source_path)) on the attacker-specified path with no containment check.
4. Arbitrary file content (any file readable by the app process) can be returned over the IPC bridge if the file happens to parse as valid JSONL.
5. Non-UUID session_id values trigger unescaped insertion at main.js:615.

This attack chain combines SEC-003, SEC-004, and SEC-011. The overall risk is High when the import functionality is accessible from the GUI.

---

## 11. Tray Icon and macOS Persistence

lib.rs:28-59 registers a tray icon with three menu items: Open Memex, Export Snapshot, Quit. No LaunchAgent, SMLoginItem, NSLoginItem, or launchd plist registration is performed. The snapshot tray handler fires document.getElementById(btn-snapshot)?.click() in the webview -- a fixed DOM element id, not user-controlled input.

**Confirmed: No auto-start or login-item behavior is added without user consent.**

---

## 12. Concrete Findings

| ID | Severity | Title | Location | Description | Recommendation |
|---|---|---|---|---|---|
| SEC-001 | Medium | First-launch outbound HTTPS to huggingface.co | indexer.rs:60-70 via fastembed and hf-hub | 100% local claim is false on first run; ~130 MB BGE-small model downloaded from huggingface.co. No application-level checksum pinning. | Document the network requirement clearly. Consider verifying the cached model SHA-256 at startup. |
| SEC-002 | High | No Content Security Policy in webview | tauri.conf.json:24-27 | csp: null disables all CSP. Combined with withGlobalTauri: true, any HTML injection gives full access to all Tauri IPC commands. | Add a restrictive CSP. Migrate away from withGlobalTauri: true. |
| SEC-003 | High | Arbitrary file read via Qdrant source_path payload poisoning | commands.rs:195, indexer.rs:1268, indexer.rs:1342 | source_path from Qdrant payload used in File::open with no path containment check. Malicious snapshot import enables arbitrary file read. | Validate source_path via canonicalize() plus starts_with(projects_root) before any File::open. |
| SEC-004 | High | Arbitrary file write via snapshot_export path | commands.rs:247-249, indexer.rs:1556 | path: PathBuf from webview with no validation. A hostile webview can write Qdrant snapshot bytes to any writable filesystem path. | Use a file-save dialog (tauri::dialog) instead of a raw prompt(). Validate the resulting path. |
| SEC-005 | Medium | Unrestricted directory walk via path parameter | commands.rs:124, 282, 366 | path: Option PathBuf accepts any directory from the webview. topology, refresh_index, list_sessions, tail_recent_errors can walk the entire filesystem. | Validate that caller-supplied paths are within ~/.claude/projects/ or reject non-default GUI overrides. |
| SEC-006 | Low | No per-line byte cap in JSONL parser | parser.rs:107-116 | A single JSON line of many megabytes is read entirely into memory. Adversarial JSONL with a very large single-line value could exhaust process memory. | Add a per-line byte limit (e.g., 10 MB) in the reader.lines() loop. |
| SEC-007 | Low | Unescaped vector key in renderResults | main.js:383 | Vector score keys interpolated into innerHTML without escapeHtml(). Latent HTML injection if key names ever come from a poisoned Qdrant collection. | Wrap the k variable in escapeHtml(k). |
| SEC-008 | Low | Transitive native-tls and OpenSSL in dependency tree | Cargo.lock via hf-hub to native-tls to openssl | Despite reqwest using rustls-tls, hf-hub pulls in native-tls 0.2.18 and openssl 0.10.80 transitively. Wider attack surface than intended. | Pin hf-hub or patch it to use a rustls backend via patch.crates-io in Cargo.toml. |
| SEC-009 | Low | petgraph 0.6 effectively unmaintained | Cargo.toml:39 | Last published release 2023-04; no security patches will arrive if a vulnerability is found. No current CVEs. | Monitor for a petgraph 0.7 release or consider a minimal in-house MST implementation. |
| SEC-010 | Info | Dead dependencies notify and notify-debouncer-full | Cargo.toml:41-42 | Both crates listed in Cargo.toml but not imported in any source file. Unnecessary supply-chain surface and build cost. | Remove both dependencies from Cargo.toml. |
| SEC-011 | Info | Unescaped sessionId in loading placeholders | main.js:427, main.js:615 | sessionId (a UUID) interpolated into innerHTML without escapeHtml(). Safe today; exploitable if a poisoned snapshot provides a non-UUID session_id. | Add escapeHtml(sessionId.slice(0, 8)) and escapeHtml(sessionId) at those two sites. |
| SEC-012 | Info | No authentication on Qdrant REST/gRPC (localhost:6333/6334) | Architecture | Qdrant default configuration has no API key. Any local process can read, write, or delete the memex_sessions collection. | Document this limitation. For sensitive environments, enable Qdrant API key authentication and pass it via MEMEX_QDRANT_URL. |
