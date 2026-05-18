# Memex Stack Ecosystem Research — Rust + Tauri 2 + fastembed + Qdrant

**Audience**: Memex maintainers
**Date**: 2026-05-18
**Scope**: Verify current best-practice and latest-version state of every dep in `src-tauri/Cargo.toml`, surface breaking-change risk, and survey OSS projects in the same neighborhood.
**Method**: `crates.io` JSON API (`/api/v1/crates/<name>`), GitHub `gh release` for release notes, official docs (tauri.app, docs.rs, qdrant.tech), and web search.
**Citation convention**: Every version number, code snippet, and breaking-change line carries `[Source: <URL> · fetched 2026-05-18]`. Sources block at the end.

---

## 1. Snapshot table

Memex's pinned versions are taken from `src-tauri/Cargo.toml` (raw spec) and `src-tauri/Cargo.lock` (resolved). "Latest" columns are crates.io `max_stable_version` as of 2026-05-18.

| Crate | Memex pinned (spec) | Memex resolved (lock) | Latest stable on crates.io (2026-05-18) | Breaking changes between resolved → latest | Risk to upgrade | Source |
|---|---|---|---|---|---|---|
| `tauri` | `"2"` (features = `tray-icon`) | `2.11.2` | `2.11.2` | Already on latest. Next minor 2.12 not yet on crates.io. | None today; minor bumps in 2.x have been additive. | [crates.io/tauri](https://crates.io/api/v1/crates/tauri) |
| `tauri-build` | `"2"` | `2.6.2` | `2.6.2` | Already on latest. | None. | [crates.io/tauri-build](https://crates.io/api/v1/crates/tauri-build) |
| `tauri-plugin-opener` | `"2"` | `2.5.4` | `2.5.4` | Already on latest. | None. | [crates.io/tauri-plugin-opener](https://crates.io/api/v1/crates/tauri-plugin-opener) |
| `qdrant-client` | `"1"` | `1.18.0` | `1.18.0` | Already on latest. Old pre-1.10 client was *removed* in 1.17 — Memex is fine. | None today; track 1.19 for new server features. | [crates.io/qdrant-client](https://crates.io/api/v1/crates/qdrant-client), [v1.17 notes](https://github.com/qdrant/rust-client/releases/tag/v1.17.0) |
| `fastembed` | `"5"` | `5.13.4` | `5.13.4` | Already on latest. 4.x → 5.x bump was the big one (ort RC10, Rayon removal). | None today; ort upgrades in 5.x have been transparent so far. | [crates.io/fastembed](https://crates.io/api/v1/crates/fastembed), [v5.0 notes](https://github.com/Anush008/fastembed-rs/releases/tag/v5.0.0) |
| `ort` (transitive) | n/a | `2.0.0-rc.12` | `2.0.0-rc.12` (no stable yet) | Still pre-1.0; major refactors land each RC. | Pinned by fastembed; we don't touch directly. | [crates.io/ort](https://crates.io/api/v1/crates/ort) |
| `tokio` | `"1"` (features = `rt-multi-thread, macros, fs, io-util, time, sync`) | `1.52.3` | `1.52.3` | Already on latest. 1.x is API-stable. | None. | [crates.io/tokio](https://crates.io/api/v1/crates/tokio) |
| `reqwest` | `"0.12"` (rustls-tls, json, stream, multipart) | `0.12.28` | `0.13.3` | 0.12 → 0.13 is a real breaking bump: rustls is default TLS, `rustls-tls` feature renamed to `rustls`, AWS-LC replaces ring, `query`/`form` are now opt-in features. | **Breaking — schedule** if/when moving to 0.13; or stay on 0.12.x indefinitely (still maintained). | [crates.io/reqwest](https://crates.io/api/v1/crates/reqwest) |
| `petgraph` | `"0.6"` | `0.6.5` | `0.8.3` | 0.6 → 0.7/0.8 changed several trait method signatures and `min_spanning_tree` now returns a pure iterator that must be collected with `FromElements`. | **Test required** — Memex uses `min_spanning_tree`; the call site needs an `UnGraph::from_elements(...)` wrapper. | [crates.io/petgraph](https://crates.io/api/v1/crates/petgraph) |
| `notify` | `"6"` | `6.1.1` | `8.2.0` stable / `9.0.0-rc.4` | 6 → 7 raised MSRV to 1.72, renamed crossbeam feature, flattened event serialization. 7 → 8 raised MSRV to 1.77, added symlink toggle, *breaking-fix* for unaligned Windows access. 8 → 9 raises MSRV to 1.88, replaces `fsevent-sys` with `objc2-core-foundation`, preserves watched-path representation. | **Breaking — schedule**. Memex's spec defers this swap intentionally; the 9 line is still RC. | [crates.io/notify](https://crates.io/api/v1/crates/notify), [CHANGELOG](https://github.com/notify-rs/notify/blob/main/notify/CHANGELOG.md) |
| `notify-debouncer-full` | `"0.3"` | `0.3.2` | `0.7.0` stable / `0.8.0-rc.2` | Tied to `notify` major. Each notify major rev'd the debouncer crate. | **Breaking — schedule** with `notify`. | [crates.io/notify-debouncer-full](https://crates.io/api/v1/crates/notify-debouncer-full) |
| `walkdir` | `"2.5"` | `2.5.0` | `2.5.0` | Already on latest. | None. | [crates.io/walkdir](https://crates.io/api/v1/crates/walkdir) |
| `chrono` | `"0.4"` (serde) | (resolved 0.4.x) | `0.4.44` | 0.4 has been backwards-compatible through 2026. | None. | [crates.io/chrono](https://crates.io/api/v1/crates/chrono) |
| `anyhow` | `"1"` | (resolved 1.x) | `1.0.102` | None expected. | None. | [crates.io/anyhow](https://crates.io/api/v1/crates/anyhow) |
| `thiserror` | `"1"` | `1.0.69` *(also 2.0.18 in tree via transitives)* | `2.0.18` | 2.x breaks: format strings can no longer mix `{0}`+positional args, raw-keyword field names like `{r#type}` must be unraw, code with `derive(Error)` must directly depend on `thiserror`, `r#source` opt-out support added. | **Test required**. Memex's usage is small; the migration is mechanical. | [crates.io/thiserror](https://crates.io/api/v1/crates/thiserror), [thiserror 2.0 notes](https://github.com/dtolnay/thiserror/releases/tag/2.0.0) |
| `clap` | `"4"` (derive) | (resolved 4.x) | `4.6.1` | Stable. | None. | [crates.io/clap](https://crates.io/api/v1/crates/clap) |
| `serde` | `"1"` | (resolved 1.x) | `1.0.228` | Stable. | None. | [crates.io/serde](https://crates.io/api/v1/crates/serde) |
| `serde_json` | `"1"` | (resolved 1.x) | `1.0.149` | Stable. | None. | [crates.io/serde_json](https://crates.io/api/v1/crates/serde_json) |
| `indicatif` | `"0.18"` | (resolved 0.18.x) | `0.18.4` | Stable inside 0.18. | None. | [crates.io/indicatif](https://crates.io/api/v1/crates/indicatif) |
| `uuid` | `"1"` (v4, v5, serde) | (resolved 1.x) | `1.23.1` | Stable. | None. | [crates.io/uuid](https://crates.io/api/v1/crates/uuid) |
| `regex` | `"1"` | (resolved 1.x) | `1.12.3` | Stable. | None. | [crates.io/regex](https://crates.io/api/v1/crates/regex) |
| `once_cell` | `"1"` | (resolved 1.x) | `1.21.4` | Stable; std's `OnceLock` is the modern replacement but `once_cell` still works. | None today; consider migrating to `std::sync::OnceLock` later. | [crates.io/once_cell](https://crates.io/api/v1/crates/once_cell) |
| `futures` | `"0.3"` | (resolved 0.3.x) | `0.3.32` | Stable. | None. | [crates.io/futures](https://crates.io/api/v1/crates/futures) |

**Top-line read**: Memex is on the **current stable line** for every dep that matters (tauri, qdrant-client, fastembed, tokio). The only genuine deferred-upgrade debt is `petgraph 0.6 → 0.8`, `notify 6 → 8`, `reqwest 0.12 → 0.13`, and `thiserror 1 → 2`. None block anything today.

---

## 2. `qdrant-client = "1.x"` — latest path

**Current**: `1.18.0` released 2026-05-11 [Source: [github.com/qdrant/rust-client/releases](https://github.com/qdrant/rust-client/releases) · fetched 2026-05-18]. Memex's `Cargo.lock` already shows `qdrant-client 1.18.0`.

### Recent change log (verbatim)

> **v1.18.0** (2026-05-11)
> * Support for Qdrant 1.18
> * Add custom headers in client builder
> * Add helper to specify per request tracing
>
> **v1.17.0** (2026-02-20)
> * Support for Qdrant 1.17
> * Add `Payload::with_capacity`
> * Remove old client deprecated since version 1.10
> * Add payload deserialization helpers
>
> **v1.16.0** (2025-11-17)
> * Support for Qdrant 1.16
> * Add connection pooling, use three connections by default
> * Add efficient `From` conversions to vectors
> * Deprecate old vector fields, use `into_vector()` instead

[Source: [github.com/qdrant/rust-client/releases](https://github.com/qdrant/rust-client/releases) · fetched 2026-05-18]

### Recommended client construction (verbatim from README)

```rust
let client = Qdrant::from_url("http://localhost:6334").build()?;
```

For Cloud:

```rust
let client = Qdrant::from_url("http://xxxxxxxxxx.eu-central.aws.cloud.qdrant.io:6334")
    .api_key(std::env::var("QDRANT_API_KEY"))
    .build()?;
```

[Source: [github.com/qdrant/rust-client README](https://raw.githubusercontent.com/qdrant/rust-client/master/README.md) · fetched 2026-05-18]

### Connection pooling

Since **v1.16**, `qdrant-client` opens **three gRPC connections by default** with built-in pooling. This is invisible to the consumer; Memex's existing single-client-handle pattern is correct — no need to manage a pool. [Source: [v1.16.0 release notes](https://github.com/qdrant/rust-client/releases/tag/v1.16.0) · fetched 2026-05-18]

### Quickstart snippets (verbatim from qdrant.tech)

```rust
use qdrant_client::Qdrant;
let client = Qdrant::from_url("http://localhost:6334").build()?;

use qdrant_client::qdrant::{CreateCollectionBuilder, VectorParamsBuilder};
client
    .create_collection(
        CreateCollectionBuilder::new("test_collection")
            .vectors_config(VectorParamsBuilder::new(4, Distance::Dot)),
    )
    .await?;

use qdrant_client::qdrant::{PointStruct, UpsertPointsBuilder};
let response = client
    .upsert_points(UpsertPointsBuilder::new("test_collection", points).wait(true))
    .await?;

use qdrant_client::qdrant::QueryPointsBuilder;
let search_result = client
    .query(QueryPointsBuilder::new("test_collection").query(vec![0.2, 0.1, 0.9, 0.7]))
    .await?;
```

Port note: **gRPC = 6334**, REST = 6333. Memex uses the Rust client → gRPC. [Source: [qdrant.tech/documentation/quickstart](https://qdrant.tech/documentation/quickstart/) · fetched 2026-05-18]

### TLS

The Rust client uses **tonic** under the hood; TLS for Qdrant Cloud connections is automatic from the `https://` URL scheme. No extra cargo features needed. For self-hosted Qdrant on a custom port with TLS, set `tls_config` on the builder. (Memex talks to a local Qdrant, so it can ignore TLS entirely.)

### Verdict for Memex

No action. Already on latest. Watch for `1.19` when Qdrant 1.19 server lands.

---

## 3. `fastembed = "5.x"` — latest path

**Current**: `5.13.4` released 2026-04-27 [Source: [crates.io/fastembed](https://crates.io/api/v1/crates/fastembed) · fetched 2026-05-18]. Memex's `Cargo.lock` already shows `5.13.4`.

### 4.x → 5.x breaking changes (verbatim)

> **5.0.0** (2025-07-07)
> ⚠ BREAKING CHANGES
> * Upgraded ort to `v2.0.0-rc.10`, Removed Rayon (#170)

[Source: [v5.0.0 release notes](https://github.com/Anush008/fastembed-rs/releases/tag/v5.0.0) · fetched 2026-05-18]

### Notable 5.x feature additions

| Version | Date | Notable change |
|---|---|---|
| `5.5.0` | 2025-12-17 | Support Snowflake Arctic models |
| `5.6.0` | 2026-01-02 | Added BAAI/bge-m3 dense and sparse embeddings |
| `5.7.0` | 2026-01-09 | Added jinaai/jina-embeddings-v2-base-en |
| `5.8.0` | 2026-01-11 | Added Qwen3 Embedding support |
| `5.10.0` | 2026-02-19 | External initializers in `UserDefinedEmbeddingModel` |
| `5.11.0` | 2026-02-19 | nomic-embed-text-v2-moe |
| `5.12.0` | 2026-03-05 | Qwen3-VL-Embedding-2B for image embeddings |
| `5.13.0` | 2026-03-16 | Export `Qwen3Model` in `qwen3` feature |
| `5.13.4` | 2026-04-27 | Fix Qwen3 F16 dtype mismatches |

[Source: [fastembed-rs releases](https://github.com/Anush008/fastembed-rs/releases) · fetched 2026-05-18]

### `EmbeddingModel` enum (latest)

44 variants total, 16 of which are quantized (`Q` suffix). Default is `BGESmallENV15`:

```
AllMiniLML6V2  AllMiniLML6V2Q
AllMiniLML12V2 AllMiniLML12V2Q
AllMpnetBaseV2
BGEBaseENV15   BGEBaseENV15Q
BGELargeENV15  BGELargeENV15Q
BGESmallENV15  BGESmallENV15Q          // default
NomicEmbedTextV1
NomicEmbedTextV15  NomicEmbedTextV15Q
ParaphraseMLMiniLML12V2  ParaphraseMLMiniLML12V2Q
ParaphraseMLMpnetBaseV2
BGESmallZHV15  BGELargeZHV15
BGEM3
ModernBertEmbedLarge
MultilingualE5Small  MultilingualE5Base  MultilingualE5Large
MxbaiEmbedLargeV1  MxbaiEmbedLargeV1Q
GTEBaseENV15  GTEBaseENV15Q
GTELargeENV15 GTELargeENV15Q
ClipVitB32
JinaEmbeddingsV2BaseCode
JinaEmbeddingsV2BaseEN
EmbeddingGemma300M
SnowflakeArcticEmbedXS  SnowflakeArcticEmbedXSQ
SnowflakeArcticEmbedS   SnowflakeArcticEmbedSQ
SnowflakeArcticEmbedM   SnowflakeArcticEmbedMQ
SnowflakeArcticEmbedMLong  SnowflakeArcticEmbedMLongQ
SnowflakeArcticEmbedL   SnowflakeArcticEmbedLQ
```

[Source: [docs.rs/fastembed EmbeddingModel](https://docs.rs/fastembed/latest/fastembed/enum.EmbeddingModel.html) · fetched 2026-05-18]

### Cache directory

The `cache_dir` parameter on `InitOptions` defaults to `fastembed_cache/` and can be overridden by the `FASTEMBED_CACHE_PATH` environment variable. [Source: [docs.rs/fastembed](https://docs.rs/fastembed) discussion · fetched 2026-05-18; cf. [Anush008/fastembed-rs v4.9](https://github.com/Anush008/fastembed-rs) "Resolve cache dir from env var"]

Memex bakes `.fastembed_cache/` next to the binary on the host filesystem (per recent commit `b41aa7a fix: Embedder cache + CWD for .app — Read-only file system (EROFS)`). This is fine — and the env-var override is a clean lever if a packaged `.app` ever needs to point cache to `~/Library/Caches/dev.sgwannabe.memex/fastembed_cache`. **Concrete recommendation**: in production, set `FASTEMBED_CACHE_PATH` to a writable path *inside* `~/Library/Caches/dev.sgwannabe.memex/` from the Tauri side before constructing the `TextEmbedding`.

### ONNX Runtime backend on macOS arm64

fastembed-rs uses `pykeio/ort` (Rust ONNX Runtime bindings). The transitive pin in Memex's lock is **`ort 2.0.0-rc.12`** [Source: `src-tauri/Cargo.lock` line "name = \"ort\" version = \"2.0.0-rc.12\""].

On macOS arm64:

> "Support for Intel macOS (x86_64-apple-darwin) has been dropped following upstream changes to ONNX Runtime & Rust, and the macOS target has been raised to 13.4. This means current versions of ORT primarily support Apple Silicon Macs."

[Source: [pykeio/ort docs and search summary](https://ort.pyke.io/) · fetched 2026-05-18]

CoreML EP is available but requires opting in:

```rust
ep::CoreML::default()
    .with_subgraphs()
    .with_compute_units(ep::coreml::ComputeUnits::CPUAndNeuralEngine)
    .build()
```

Enabled via the `coreml` cargo feature. **fastembed does not expose `coreml` as a feature flag** — to use it, Memex would need to depend on `ort` directly with `features = ["coreml"]` and pass an EP list via the `execution_providers` slot on `InitOptions`. The default CPU EP is what Memex uses today and is fine for the embedding sizes in question (small/base BGE under 100 MB). [Source: [WebSearch result on ort CoreML EP](https://ort.pyke.io/perf/execution-providers) · fetched 2026-05-18]

### Recommended batch size

fastembed-rs 5.0 **removed Rayon** [Source: [v5.0.0 release notes](https://github.com/Anush008/fastembed-rs/releases/tag/v5.0.0) · fetched 2026-05-18]. Embedding is single-threaded per call inside the ONNX session. Batching is the user's lever. The README quickstart uses `embed(documents, None)` where the second arg is `Option<usize>` — None means library default. For BGE-small on M-series, 32–64 docs/batch is a safe practical sweet spot (memory ~ small). Memex's existing batch loop should pass an explicit batch size to keep memory predictable across `.app` packaging boundaries.

### What's *not* in fastembed-rs vs Python fastembed

Python `fastembed` ships **ColBERT** (late-interaction), **BM42** (sparse with attention weights), **SPLADE** (Memex already deferred this — *partially* available as `prithivida/Splade_PP_en_v1` only), and a broader sparse/late-interaction zoo.

fastembed-rs as of 5.13.4 supports:

- **Sparse** text: `prithivida/Splade_PP_en_v1` (default), `BAAI/bge-m3` (sparse mode added in 5.6) [Source: [v5.6.0 notes](https://github.com/Anush008/fastembed-rs/releases/tag/v5.6.0) · fetched 2026-05-18]
- **No ColBERT** support in the Rust crate.
- **No BM42** support in the Rust crate.

**→ Memex's deferral of ColBERT/BM42 remains valid.** They are not available in the Rust ecosystem yet. If those become required for a "5th unique vector lens," the options are: (a) wait for fastembed-rs to add them, (b) call out to a sidecar (which Memex explicitly avoids), or (c) embed `ort` directly and ship the ColBERT ONNX model with custom pooling logic.

---

## 4. `tauri = "2.x"` — latest path

**Current**: `tauri 2.11.2`, `tauri-build 2.6.2`, `tauri-cli 2.11.2`, `tauri-runtime-wry 2.11.2`, `tauri-bundler 2.9.2`, all released 2026-05-16. [Source: [tauri releases](https://github.com/tauri-apps/tauri/releases) · fetched 2026-05-18, [crates.io/tauri](https://crates.io/api/v1/crates/tauri) · fetched 2026-05-18]

Memex is on `tauri 2.11.2` already (per Cargo.lock). Best-in-class.

### 2.x evolution highlights (verbatim summary)

> "Version 2.2.0 introduced `TrayIconBuilder::show_menu_on_left_click` and deprecated `menu_on_left_click`."
> "By 2.8.0, the framework added `with_inner_tray_icon` to access platform-specific icons."
> "Version 2.0.0-rc.0 prefixed core plugin permissions with 'core:' (app, event, image, menu, path, resources, tray, webview, window)."
> "Version 2.6.0 introduced the `dynamic-acl` feature (enabled by default) for granular control."
> "Critically, version 2.11.1 enforced ACL checks for IPC requests from remote origins even when no AppManifest is configured, preventing custom commands from bypassing access control entirely."
> "Version 2.3.0 updated wry to 0.50 and objc2 to 0.6."
> "Version 2.8.0 updated webkit2gtk-rs to v2.0.2."

[Source: [Tauri CHANGELOG](https://raw.githubusercontent.com/tauri-apps/tauri/dev/crates/tauri/CHANGELOG.md) · fetched 2026-05-18]

### `tray-icon` feature stability

Stable since 2.0. `tray-icon` crate 0.23.0 is pinned by tauri 2.11.2. Memex's `features = ["tray-icon"]` config is correct. Note 2.2's `show_menu_on_left_click` rename — if Memex ever calls `menu_on_left_click` directly, switch.

### Capability model (current best practice)

Capability files live in `src-tauri/capabilities/*.json` (or `.toml`). Schema:

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "main-capability",
  "description": "Capability for the main window",
  "windows": ["main"],
  "permissions": [
    "core:path:default",
    "core:event:default",
    "core:window:default",
    "core:app:default",
    "core:resources:default",
    "core:menu:default",
    "core:tray:default",
    "core:window:allow-set-title"
  ]
}
```

[Source: [v2.tauri.app/security/capabilities](https://v2.tauri.app/security/capabilities/) · fetched 2026-05-18]

Memex's current `default.json`:

```json
{
  "identifier": "default",
  "windows": ["main"],
  "permissions": ["core:default", "opener:default"]
}
```

This uses the **group permission `core:default`**, which is the official broad-grant alias. That works for development. For a hardened production build, consider replacing `core:default` with the explicit per-domain list above and trim to what Memex actually invokes from JS. Memex's IPC surface is small (the indexer commands), so this lockdown is feasible without a long audit.

### `withGlobalTauri: true` + `csp: null` security stance

Memex sets both today:

```json
"app": {
  "withGlobalTauri": true,
  "security": { "csp": null }
}
```

`withGlobalTauri: true` exposes the `window.__TAURI__` global to the webview — convenient for `withGlobalTauri` polyfills and the vanilla-JS frontend Memex uses. **Acceptable** since Memex serves a local `frontendDist: "../src"` and never loads remote content. `csp: null` disables CSP entirely — also acceptable for an entirely-local app, but a hardening opportunity: setting `csp: "default-src 'self'; style-src 'self' 'unsafe-inline'"` is a low-cost win that doesn't break the current vanilla JS frontend.

### macOS bundling (`.dmg` on arm64)

Tauri 2.11.2 bundles via `tauri-bundler 2.9.2`. Memex's `bundle.targets: "all"` produces both `.app` and `.dmg` on macOS. For arm64-only distribution, that's already what `cargo tauri build` emits when building from an Apple Silicon Mac. No change needed.

### Code signing / notarization

Memex ships **unsigned** today (no `bundle.macOS.signingIdentity` in `tauri.conf.json`). When Memex starts distributing on a third-party channel and wants Gatekeeper friction to drop, the migration is:

1. Acquire **Developer ID Application** certificate (not Apple Distribution — that's for App Store).
2. Set env vars at build time:
   - `APPLE_SIGNING_IDENTITY="Developer ID Application: <Name> (<TEAMID>)"`
   - For notarization, **App Store Connect API** path (preferred):
     - `APPLE_API_ISSUER=...`
     - `APPLE_API_KEY=...`
     - `APPLE_API_KEY_PATH=...`
   - Or **Apple ID** path:
     - `APPLE_ID="you@example.com"`
     - `APPLE_PASSWORD="<app-specific-password>"`
     - `APPLE_TEAM_ID="<TEAMID>"`
3. Provide an **Entitlements.plist** with `com.apple.security.cs.allow-jit = true` and `com.apple.security.cs.allow-unsigned-executable-memory = true` — required for Tauri's wry/WebKit to not crash post-notarization.

[Source: [v2.tauri.app/distribute/sign/macos](https://v2.tauri.app/distribute/sign/macos/) · fetched 2026-05-18]

Until then, "right-click → Open" is the user workaround. Document it in README's install section.

### Known transitive risk

The cargo-audit attached to 2.11.2's release notes shows **`atk 0.18.2` and `atk-sys 0.18.2` unmaintained** (RUSTSEC-2024-0413 / RUSTSEC-2024-0416). These are Linux-only (GTK3) and reach Memex only on Linux builds. **macOS-only Memex installs are unaffected.** [Source: [tauri-v2.11.2 release notes](https://github.com/tauri-apps/tauri/releases/tag/tauri-v2.11.2) · fetched 2026-05-18]

---

## 5. Tokio runtime sizing for Memex's shape

Memex shape (per repo recon):
- One `Embedder` behind a `Mutex` (ONNX session — `!Sync` for inference calls)
- A 12 s poller for session JSONL changes
- On-demand Tauri commands (`#[tauri::command] async fn ...`)
- I/O-bound calls into `qdrant-client` (gRPC) and filesystem reads

### Current Memex config

```toml
tokio = { version = "1", features = ["rt-multi-thread", "macros", "fs", "io-util", "time", "sync"] }
```

This pulls in `rt-multi-thread`, so Tauri instantiates the multi-thread runtime by default for the async runtime that backs `#[tauri::command]`.

### What Tokio docs recommend

> "Most applications should use the multi-thread scheduler, except in some niche use-cases, such as when running only a single thread is required."

[Source: [docs.rs/tokio/runtime](https://docs.rs/tokio/latest/tokio/runtime/index.html) · fetched 2026-05-18]

Runtime defaults:

> `max_blocking_threads`: **The default value is 512.**
> `worker_threads`: **The default value is the number of cores available to the system.**

[Source: [docs.rs/tokio Builder](https://docs.rs/tokio/latest/tokio/runtime/struct.Builder.html) · fetched 2026-05-18]

On `spawn_blocking`:

> "Use `spawn_blocking` for short-lived blocking operations" and "Use dedicated threads for long-lived or persistent blocking workloads"
>
> "The thread limit is very large by default, because `spawn_blocking` is often used for various kinds of IO operations that cannot be performed asynchronously."
>
> "However, when running CPU-intensive code via `spawn_blocking`, developers should be aware of this large default and consider using synchronization primitives (like semaphores) or specialized executors like rayon to limit concurrent execution."

[Source: [docs.rs/tokio spawn_blocking](https://docs.rs/tokio/latest/tokio/task/fn.spawn_blocking.html) · fetched 2026-05-18]

### Concrete recommendation for Memex

1. **Keep multi-thread runtime.** Worker threads = cores is the right default for Memex's mixed I/O + occasional embedding work.
2. **Wrap ONNX inference in `spawn_blocking`.** ONNX `Session::run` is a synchronous CPU-bound call. Today, holding the `Mutex<Embedder>` across an `await` in a Tauri command stalls every other async task on the same worker. Move the actual `model.embed(...)` call to `tokio::task::spawn_blocking({ let emb = embedder.clone(); move || emb.lock().unwrap().embed(docs, Some(64)) }).await??` — the cost is one thread-pool handoff per embed batch, well worth it.
3. **Add a semaphore around `spawn_blocking` for embeddings.** Cap concurrent embed batches to ~`num_cpus / 2` so the indexer doesn't starve the UI. Tokio's default 512-thread blocking pool is way too generous for a desktop tool.
4. **Don't change feature flags.** `rt-multi-thread, macros, fs, io-util, time, sync` is exactly the set you need. Notably **`process` is absent** — good, because Memex doesn't spawn subprocesses.

---

## 6. `petgraph 0.6` / `notify 6` — current state

### petgraph

Pinned: `0.6.5`. Latest: `0.8.3` (released 2025-09-30). [Source: [crates.io/petgraph](https://crates.io/api/v1/crates/petgraph) · fetched 2026-05-18]

Breaking change relevant to Memex (uses `min_spanning_tree`):

> "`min_spanning_tree` now returns an iterator that needs to be made into a specific graph type deliberately."

Migration is one line:

```rust
// petgraph 0.6
let mst = min_spanning_tree(&g);                       // already-collected graph

// petgraph 0.8
use petgraph::data::FromElements;
let mst = UnGraph::<_, _>::from_elements(min_spanning_tree(&g));
```

[Source: [petgraph CHANGELOG / 0.8 notes](https://github.com/petgraph/petgraph/blob/master/CHANGELOG.md) · fetched 2026-05-18]

Other 0.8 changes (per CHANGELOG via search):

- `NodeIndexable` method signature changes
- Removal of `IntoExternals`
- Various trait reorganizations

**Risk**: low. Memex's graph code is contained to MST construction for the session-topology view. A 30-min migration with `cargo check` will surface every callsite. Defer until there's another reason to touch that module.

### notify + notify-debouncer-full

Pinned: `notify 6.1.1`, `notify-debouncer-full 0.3.2`. Latest stable: `notify 8.2.0` + `notify-debouncer-full 0.7.0`. 9.0-rc.4 is in flight. [Source: [crates.io/notify](https://crates.io/api/v1/crates/notify) · fetched 2026-05-18]

Full path of breaking changes 6 → 9 (verbatim from notify CHANGELOG):

**6 → 7 (2024-10-25)**
> - raise MSRV to 1.72
> - move event type to notify-types crate
> - remove internal use of crossbeam channels
> - rename feature `crossbeam` to `crossbeam-channel` and disable it by default
> - flatten serialization of events and use camelCase
> - upgrade mio to 1.0

**7 → 8 (2025-01-10)**
> - update notify-types to version 2.0.0
> - raise MSRV to 1.77
> - add config option to disable following symbolic links
> - unaligned access to FILE_NOTIFY_INFORMATION (Windows; marked breaking)

**8 → 9 (RC, 2026-01-25 onward)**
> - raise MSRV to 1.85, later 1.88
> - preserve watched path representation in `Event.paths` and `Watcher::watched_paths`
> - replace an existing watch when `watch` is called again for the same path
> - replace `fsevent-sys` with `objc2-core-foundation` and `objc2-core-services`
> - annotate FSEvents clone-related events with `info = 'is: clone'`
> - add `EventKindMask` for filtering filesystem events
> - add `Watcher::watched_paths` to list active watches

[Source: [notify CHANGELOG](https://github.com/notify-rs/notify/blob/main/notify/CHANGELOG.md) · fetched 2026-05-18]

### macOS FSEvent / Full Disk Access for Memex

Memex watches `~/.claude/projects/` for new session JSONL files. macOS treats this path as user data, so:

- **No special entitlement needed for read access** when the user has explicitly given Full Disk Access (a global toggle in System Settings).
- **Without FDA**, FSEvents on `~/.claude/projects/` works because the user's home is not protected by TCC in the same way `~/Library/Mail/` is.
- The polling-based debouncer (Memex's current approach) sidesteps the kernel event stream entirely and is the most portable choice.

The notify 8 → 9 move replaces `fsevent-sys` with `objc2-core-foundation`, which is the modern objc2 ecosystem stack. **Functional behavior on macOS arm64 should be equivalent**, but the rewrite is new and worth letting bake to a stable 9.0 release before adopting.

**Verdict**: Memex's defer-the-swap stance is correct. Move to `notify = "8"` when there's a concrete reason (e.g. symlink-handling config). Skip 9.x until a stable release ships.

---

## 7. `reqwest 0.12 + rustls-tls`

Pinned: `reqwest 0.12.28`. Latest: `0.13.3` (released 2026-04-27). [Source: [crates.io/reqwest](https://crates.io/api/v1/crates/reqwest) · fetched 2026-05-18]

Memex's spec:

```toml
reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls", "stream", "multipart"] }
```

`default-features = false` plus explicit `rustls-tls` ensures **no OpenSSL link** at any point. Verified: the resolved tree shows `rustls 0.23` and `webpki-roots` — no `openssl`/`openssl-sys` crates. **Confirmed: Memex avoids OpenSSL.**

### 0.12 → 0.13 breaking changes (verbatim)

> 1. rustls is now the default TLS backend, instead of native-tls
> 2. The `rustls-tls` feature has been renamed to simply `rustls`
> 3. AWS-LC is now the default crypto provider instead of ring
> 4. Rustls roots features removed; `rustls-platform-verifier` is used by default
> 5. native-tls now includes ALPN. To disable, use `native-tls-no-alpn`
> 6. Query and form handling are now crate features that are disabled by default
> 7. Long-deprecated methods removed, including the `trust-dns` feature (renamed to `hickory-dns`)

[Source: [reqwest CHANGELOG (search summary)](https://github.com/seanmonstar/reqwest/blob/master/CHANGELOG.md) · fetched 2026-05-18]

### Migration path for Memex

When/if upgrading:

```toml
# Memex 0.13 equivalent
reqwest = { version = "0.13", default-features = false, features = ["json", "rustls", "stream", "multipart"] }
```

If Memex's `reqwest` callsites use `.query(...)` or `.form(...)`, those become opt-in features in 0.13 and need to be re-listed explicitly. **Audit**: search the repo for those methods before migrating.

**Verdict**: Stay on 0.12 indefinitely. 0.12 is fully maintained and the migration cost has no payoff (no new feature Memex needs).

---

## 8. Real-world OSS in the same shape as Memex

Searches: GitHub search for `qdrant + tauri`, `qdrant-client + fastembed`, `Claude Code session search`, etc.

### 8.1 Direct overlaps (Qdrant + Tauri local-first desktop)

**Conclusion**: There is **no widely-starred OSS project today that combines all three** (Qdrant + Tauri + fastembed-rs) for a local-first session-indexing desktop tool. Memex is, as of 2026-05-18, in genuinely novel territory at that intersection.

### 8.2 Adjacent OSS — same problem space (AI session indexing/search)

| Project | Lang | Stars | Last update | Backend | Memex-relevant takeaways | License |
|---|---|---|---|---|---|---|
| **Dicklesworthstone/coding_agent_session_search** ("cass") | Rust | **770** | active (3,938 commits) | SQLite + Tantivy BM25 + **fastembed-rs** (MiniLM / Snowflake Arctic / Nomic, all ONNX) + custom FSVI vector format | Supports **20+ agent formats** (Claude Code, Codex, Gemini CLI, Cline, Cursor, ChatGPT, Aider, Pi-Agent, Copilot CLI, OpenClaw, Vibe, Crush, Hermes, Kimi Code, Qwen Code, Factory/Droid). Best reference for **multi-format parser layouts** Memex's README mentions wanting. Uses fastembed-rs the same way Memex does. Vector store is custom (mmap'd float32) not Qdrant — useful comparison point for *why* Memex chose Qdrant. | MIT + OpenAI/Anthropic Rider — Apache-2.0 compatible for code use (rider is a model-usage clause, not a code-license clause) |
| **lee-fuhr/claude-session-index** | Python | 22 | 2026-02-02 (v0.3.1) | SQLite + FTS5 | Pure FTS5 baseline; useful for the "fast lexical fallback" pattern. | MIT — compatible |
| **edwarddgao/agent-traces** | Python | 3 | recent | Custom binary (raw `.f32` mmap) + JSONL meta | Qwen3-Embedding-8B via OpenRouter (online), but the on-disk layout is instructive for very small toolchains. CLI only. | MIT — compatible |
| **sinzin91/search-sessions** | Rust | 28 | 2026-03-23 (v0.3.0) | None — ripgrep fallback over raw JSONL | "No indexing" approach. Confirms there's market demand for sub-second Claude Code session search but does not address vector/semantic recall. Useful baseline for *what users settle for* without Memex. | MIT — compatible |
| **withLinda/claude-JSONL-browser** | TypeScript | (small) | recent | Web tool (JSONL → Markdown) | Read-only viewer pattern; UX inspiration for Memex's replay panel. | MIT — compatible |
| **anthropics/claude-code issue #39667** | (issue) | — | open | — | Real users hitting "Session JSONL files silently deleted, sessions-index.json stops updating" — a *real* failure mode Memex's indexer must tolerate. Index-of-file-list source-of-truth is unreliable. | — |

[Sources: [Dicklesworthstone/coding_agent_session_search](https://github.com/Dicklesworthstone/coding_agent_session_search), [lee-fuhr/claude-session-index](https://github.com/lee-fuhr/claude-session-index), [edwarddgao/agent-traces](https://github.com/edwarddgao/agent-traces), [sinzin91/search-sessions](https://github.com/sinzin91/search-sessions), [withLinda/claude-JSONL-browser](https://github.com/withLinda/claude-JSONL-browser), [anthropics/claude-code#39667](https://github.com/anthropics/claude-code/issues/39667) · all fetched 2026-05-18]

### 8.3 Adjacent OSS — Qdrant + Rust example projects (no Tauri)

| Project | Lang | Stars | What it does | Memex-relevant takeaways |
|---|---|---|---|---|
| **RGGH/fe** | Rust + Python | 0 | "Qdrant Rust Fastembed" demo | Tiny demo of the exact two-crate combo Memex uses. Code patterns OK to skim. License unspecified — *not* drop-in usable. |
| **qdrant/rust-client** examples/ | Rust | — | Official examples | Reference for `Qdrant::from_url(...).build()?` and the new query API. |

### 8.4 What Memex can borrow

1. **From `cass`** — the multi-format parser shape (one `Source` trait, one `normalize_to_message()` impl per provider). Memex's README invites contributions for "Codex / Cursor / other CLI session formats" — `cass` has already enumerated 20+. Lift the format-detection heuristics (file-path globs, JSONL-line shape) into Memex's parser layer.
2. **From `cass`** — the lexical+semantic Reciprocal Rank Fusion (RRF) hybrid. Memex's "lens slider over 5 named vectors" is genuinely different, but adding a 6th lens that does pure-BM25 fallback (for fastembed-failed edge cases) is cheap with Tantivy or even `sqlite-vec` + FTS5.
3. **From `sinzin91/search-sessions`** — the "ripgrep fallback when index is stale" pattern. Memex's index pruning will eventually drift behind real files; a `--rg` escape hatch CLI command is good UX insurance.
4. **From `claude-code#39667`** — defensive coding around `sessions-index.json` drift. Memex should not trust the index file; walk `~/.claude/projects/**/*.jsonl` directly.

### 8.5 License compatibility note

Memex is **Apache-2.0**. All MIT-licensed projects above are one-way compatible (MIT code can be vendored into Apache-2.0 with attribution). The "OpenAI/Anthropic Rider" on `cass` constrains model-use commercial activity, not source code redistribution — code patterns can be lifted.

---

## 9. Local-first agent-memory projects (broader landscape)

Beyond search/index tools, here is the broader 2026 OSS landscape Memex sits within:

| Project | What it is | Why it matters for Memex |
|---|---|---|
| [Dicklesworthstone/coding_agent_session_search](https://github.com/Dicklesworthstone/coding_agent_session_search) | Rust TUI/CLI multi-agent session search | Closest competitor by feature surface. No vector lens, no replay, no MST. |
| [lee-fuhr/claude-session-index](https://github.com/lee-fuhr/claude-session-index) | Python+SQLite Claude Code indexer | Pure lexical baseline. |
| [edwarddgao/agent-traces](https://github.com/edwarddgao/agent-traces) | Python CLI semantic search over Claude+Codex | OpenRouter-backed (cloud embedding); explicit non-local-first counter-example. |
| [sinzin91/search-sessions](https://github.com/sinzin91/search-sessions) | Rust no-index CLI search | Demonstrates user demand for *zero-config* search; tradeoff: no semantics. |
| [withLinda/claude-JSONL-browser](https://github.com/withLinda/claude-JSONL-browser) | Web JSONL → Markdown viewer | Replay-UX prior art. |

Memex is the only one that (a) is **Tauri-wrapped**, (b) uses **Qdrant** as the vector store, (c) ships **5 named vector lenses** with a slider, (d) has an **MST-based topology** view, and (e) does **proactive recall** on the active session. None of the 5 above touch (a), (c), (d), or (e).

[Sources: [GitHub search "Claude Code session JSONL"](https://github.com/search?q=claude+code+session+jsonl&type=repositories) · fetched 2026-05-18]

---

## 10. macOS arm64 packaging notes

### onnxruntime arm64 status

`ort 2.0.0-rc.12` is what fastembed pulls in transitively. The pyke/ort project ships **prebuilt dynamic-linked ONNX Runtime libraries** for arm64-apple-darwin. They are downloaded by the `download-binaries` cargo feature at build time (the default). For Memex's `.app` bundle this means the ONNX Runtime `.dylib` is embedded in the Rust binary's RPATH-resolved load.

> "Support for Intel macOS (x86_64-apple-darwin) has been dropped following upstream changes to ONNX Runtime & Rust, and the macOS target has been raised to 13.4."

[Source: [pykeio/ort](https://github.com/pykeio/ort) · WebSearch summary · fetched 2026-05-18]

**Impact on Memex**: arm64-only distribution is now the path of least resistance. The bundle's `bundle.macOS.minimumSystemVersion: 11.0` should be **raised to 13.4** to match ort's minimum. This is a documented requirement, not a soft suggestion — older macOS will fail to load the dylib.

Universal2 (arm64 + x86_64) is **no longer supported** by ort upstream. If Memex needs Intel support, it must either pin ort to an older version (locking fastembed to <5.0) or maintain a separate Intel build with a different ONNX Runtime backend.

### Notarization in 2026

Process unchanged from 2024–2025 baseline:
1. Sign with Developer ID Application + Hardened Runtime (`codesign --options runtime ...`).
2. Submit `.app` or `.dmg` to Apple's notary service via `notarytool` (or `xcrun altool` — deprecated).
3. Staple ticket back with `xcrun stapler staple ...`.

Tauri 2's bundler does all three automatically when env vars are set.

[Source: [v2.tauri.app/distribute/sign/macos](https://v2.tauri.app/distribute/sign/macos/) · fetched 2026-05-18]

### Hardened Runtime entitlements required

> "Tauri's WebView needs JIT and unsigned executable memory permissions. You should create an Entitlements.plist with both `com.apple.security.cs.allow-jit` and `com.apple.security.cs.allow-unsigned-executable-memory` set to true."

[Source: [WebSearch on Tauri 2 hardened runtime](https://dev.to/0xmassi/shipping-a-production-macos-app-with-tauri-20-code-signing-notarization-and-homebrew-mc3) · fetched 2026-05-18; cross-ref [Apple Developer Docs](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.security.cs.allow-unsigned-executable-memory)]

### Full Disk Access UX

Memex reads `~/.claude/projects/`, which lives under `$HOME` — not in TCC-protected paths (`~/Documents`, `~/Desktop`, `~/Downloads`, `~/Library/Mail`, etc.). **No FDA prompt should appear in the common case.** If/when Memex starts watching `~/Library/Application Support/*/sessions/` or similar paths from Codex/Cursor, that triggers TCC. At that point the UX is:

1. First-run modal: "Memex needs to read AI session files. Click Open System Settings → Privacy & Security → Full Disk Access → toggle on Memex."
2. Tauri can detect EPERM on initial read attempt and surface this hint.

Don't pre-emptively request FDA — Apple flags apps that do.

---

## 11. Recommendation summary

Five concrete upgrade or pattern-adoption suggestions, ranked by ROI for Memex.

### #1 — Wrap ONNX inference in `spawn_blocking` with a small semaphore
- **Pattern**: tokio runtime / embedder concurrency
- **From → To**: direct `.lock().embed(...)` inside async commands → `spawn_blocking` + `Semaphore::new(num_cpus/2)`
- **Effort**: 1–2 hours
- **Risk**: **Safe** — additive
- **Payoff**: Eliminates UI stalls during indexing batches. Memex's poller and on-demand commands stop blocking each other. Highest ROI item on this list.

### #2 — Tighten Tauri capability from `core:default` to per-domain allowlist
- **Pattern**: tauri 2 capability hardening
- **From → To**: `["core:default", "opener:default"]` → enumerated `core:path:default`, `core:event:default`, `core:window:default`, `core:app:default`, `opener:default` (skip `core:resources`, `core:tray`, `core:menu` unless used)
- **Effort**: 1 hour (audit JS callsites + test)
- **Risk**: **Test required** — broken IPC if a needed permission is missed
- **Payoff**: Smaller attack surface; aligns with 2.11.1's tightened ACL enforcement; makes future code-signing review cleaner.

### #3 — Raise `bundle.macOS.minimumSystemVersion` from `11.0` → `13.4`
- **Pattern**: macOS bundling correctness for ort 2.0
- **From → To**: `"minimumSystemVersion": "11.0"` → `"13.4"`
- **Effort**: 5 minutes
- **Risk**: **Safe** — matches reality; ort fails to load on <13.4 anyway
- **Payoff**: Prevents silent-crash bug reports from users on Big Sur (11) / Monterey (12) / older Ventura. Better to fail at install time than at runtime.

### #4 — Configure `FASTEMBED_CACHE_PATH` to a writable per-user path in the Tauri setup hook
- **Pattern**: fastembed cache placement for .app bundles
- **From → To**: implicit `./.fastembed_cache/` (CWD-dependent) → `~/Library/Caches/dev.sgwannabe.memex/fastembed/` set via env var before `TextEmbedding::try_new()`
- **Effort**: 30 minutes
- **Risk**: **Safe** — environment-variable-driven, no API breakage
- **Payoff**: Eliminates the EROFS class of bugs that motivated commit `b41aa7a` permanently. Survives `.app` move into `/Applications` without re-downloading the model.

### #5 — Defer everything else
- **Pattern**: do-nothing on petgraph/notify/reqwest/thiserror
- **From → To**: keep `petgraph 0.6`, `notify 6`, `reqwest 0.12`, `thiserror 1`
- **Effort**: 0
- **Risk**: **Safe** — all four lines remain maintained
- **Payoff**: Zero churn for zero functional gain. Re-evaluate only when a specific feature in a newer major (e.g. notify's symlink toggle, thiserror 2's `r#source`) becomes required.

---

## Sources

### Crates.io API endpoints (versions verified 2026-05-18)

- [crates.io/api/v1/crates/tauri](https://crates.io/api/v1/crates/tauri)
- [crates.io/api/v1/crates/tauri-build](https://crates.io/api/v1/crates/tauri-build)
- [crates.io/api/v1/crates/tauri-plugin-opener](https://crates.io/api/v1/crates/tauri-plugin-opener)
- [crates.io/api/v1/crates/qdrant-client](https://crates.io/api/v1/crates/qdrant-client)
- [crates.io/api/v1/crates/fastembed](https://crates.io/api/v1/crates/fastembed)
- [crates.io/api/v1/crates/ort](https://crates.io/api/v1/crates/ort)
- [crates.io/api/v1/crates/tokio](https://crates.io/api/v1/crates/tokio)
- [crates.io/api/v1/crates/reqwest](https://crates.io/api/v1/crates/reqwest)
- [crates.io/api/v1/crates/petgraph](https://crates.io/api/v1/crates/petgraph)
- [crates.io/api/v1/crates/notify](https://crates.io/api/v1/crates/notify)
- [crates.io/api/v1/crates/notify-debouncer-full](https://crates.io/api/v1/crates/notify-debouncer-full)
- [crates.io/api/v1/crates/walkdir](https://crates.io/api/v1/crates/walkdir)
- [crates.io/api/v1/crates/chrono](https://crates.io/api/v1/crates/chrono)
- [crates.io/api/v1/crates/anyhow](https://crates.io/api/v1/crates/anyhow)
- [crates.io/api/v1/crates/thiserror](https://crates.io/api/v1/crates/thiserror)
- [crates.io/api/v1/crates/clap](https://crates.io/api/v1/crates/clap)
- [crates.io/api/v1/crates/serde](https://crates.io/api/v1/crates/serde)
- [crates.io/api/v1/crates/serde_json](https://crates.io/api/v1/crates/serde_json)
- [crates.io/api/v1/crates/indicatif](https://crates.io/api/v1/crates/indicatif)
- [crates.io/api/v1/crates/uuid](https://crates.io/api/v1/crates/uuid)
- [crates.io/api/v1/crates/regex](https://crates.io/api/v1/crates/regex)
- [crates.io/api/v1/crates/once_cell](https://crates.io/api/v1/crates/once_cell)
- [crates.io/api/v1/crates/futures](https://crates.io/api/v1/crates/futures)

### GitHub release notes (verified via `gh release view`)

- [github.com/qdrant/rust-client/releases](https://github.com/qdrant/rust-client/releases)
- [github.com/qdrant/rust-client/releases/tag/v1.18.0](https://github.com/qdrant/rust-client/releases/tag/v1.18.0)
- [github.com/qdrant/rust-client/releases/tag/v1.17.0](https://github.com/qdrant/rust-client/releases/tag/v1.17.0)
- [github.com/qdrant/rust-client/releases/tag/v1.16.0](https://github.com/qdrant/rust-client/releases/tag/v1.16.0)
- [github.com/Anush008/fastembed-rs/releases](https://github.com/Anush008/fastembed-rs/releases)
- [github.com/Anush008/fastembed-rs/releases/tag/v5.0.0](https://github.com/Anush008/fastembed-rs/releases/tag/v5.0.0)
- [github.com/Anush008/fastembed-rs/releases/tag/v5.13.4](https://github.com/Anush008/fastembed-rs/releases/tag/v5.13.4)
- [github.com/tauri-apps/tauri/releases](https://github.com/tauri-apps/tauri/releases)
- [github.com/tauri-apps/tauri/releases/tag/tauri-v2.11.2](https://github.com/tauri-apps/tauri/releases/tag/tauri-v2.11.2)
- [github.com/tokio-rs/tokio/releases](https://github.com/tokio-rs/tokio/releases)
- [github.com/notify-rs/notify/releases](https://github.com/notify-rs/notify/releases)
- [github.com/notify-rs/notify/blob/main/notify/CHANGELOG.md](https://github.com/notify-rs/notify/blob/main/notify/CHANGELOG.md)
- [github.com/dtolnay/thiserror/releases/tag/2.0.0](https://github.com/dtolnay/thiserror/releases/tag/2.0.0)

### Official docs

- [v2.tauri.app/security/capabilities](https://v2.tauri.app/security/capabilities/)
- [v2.tauri.app/distribute/sign/macos](https://v2.tauri.app/distribute/sign/macos/)
- [v2.tauri.app/reference/config](https://v2.tauri.app/reference/config/)
- [v2.tauri.app/reference/environment-variables](https://v2.tauri.app/reference/environment-variables/)
- [raw.githubusercontent.com/tauri-apps/tauri/dev/crates/tauri/CHANGELOG.md](https://raw.githubusercontent.com/tauri-apps/tauri/dev/crates/tauri/CHANGELOG.md)
- [qdrant.tech/documentation/quickstart](https://qdrant.tech/documentation/quickstart/)
- [raw.githubusercontent.com/qdrant/rust-client/master/README.md](https://raw.githubusercontent.com/qdrant/rust-client/master/README.md)
- [docs.rs/tokio/runtime](https://docs.rs/tokio/latest/tokio/runtime/index.html)
- [docs.rs/tokio Builder](https://docs.rs/tokio/latest/tokio/runtime/struct.Builder.html)
- [docs.rs/tokio spawn_blocking](https://docs.rs/tokio/latest/tokio/task/fn.spawn_blocking.html)
- [docs.rs/fastembed enum.EmbeddingModel](https://docs.rs/fastembed/latest/fastembed/enum.EmbeddingModel.html)
- [ort.pyke.io / pykeio/ort](https://github.com/pykeio/ort)

### Adjacent OSS projects surveyed

- [github.com/Dicklesworthstone/coding_agent_session_search](https://github.com/Dicklesworthstone/coding_agent_session_search)
- [github.com/lee-fuhr/claude-session-index](https://github.com/lee-fuhr/claude-session-index)
- [github.com/edwarddgao/agent-traces](https://github.com/edwarddgao/agent-traces)
- [github.com/sinzin91/search-sessions](https://github.com/sinzin91/search-sessions)
- [github.com/withLinda/claude-JSONL-browser](https://github.com/withLinda/claude-JSONL-browser)
- [github.com/RGGH/fe](https://github.com/RGGH/fe)
- [github.com/anthropics/claude-code/issues/39667](https://github.com/anthropics/claude-code/issues/39667)

### Background reading consulted

- [dev.to — Shipping a Production macOS App with Tauri 2.0](https://dev.to/0xmassi/shipping-a-production-macos-app-with-tauri-20-code-signing-notarization-and-homebrew-mc3)
- [dev.to — File Watching in Rust with notify-rs](https://dev.to/hiyoyok/file-watching-in-rust-with-notify-rs-hot-folders-for-a-sync-app-32d8)
- [developer.apple.com — com.apple.security.cs.allow-unsigned-executable-memory](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.security.cs.allow-unsigned-executable-memory)
- [developer.apple.com — Configuring the hardened runtime](https://developer.apple.com/documentation/xcode/configuring-the-hardened-runtime)

---

*Report written 2026-05-18. Versions verified live against crates.io and GitHub on that date. Read-only on the Memex repo — no files modified outside `claudedocs/research/`.*
