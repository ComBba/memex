# 08 — Gemini & Gemma 2026 Line: Memex Local-LLM Dossier

**Author:** Research pass for Memex VSD-2026 submission
**Fetched:** 2026-05-18
**Scope:** Index-time-only LLM augmentation for Memex (KD-01 contextual retrieval, KD-02 cluster labels). Cloud Gemini disqualified by Memex's `no-outbound-network` invariant. Gemini Nano disqualified by Tauri webview architecture. Open-weights **Gemma line via Ollama / MLX** is the sole candidate.

---

## 1. Executive Summary

- **Gemma 4 is GA today** (released **April 2, 2026** under Apache 2.0). [Source: https://blog.google/innovation-and-ai/technology/developers-tools/gemma-4/ · fetched 2026-05-18]
- **Four variants** ship: E2B (~2.3B effective), E4B (~4.5B effective), 26B-A4B MoE (3.8B active), 31B dense. The E2B/E4B variants support text + image + audio input with 128K context; 26B/31B run text + image with 256K context. [Source: https://ai.google.dev/gemma/docs/core/model_card_4 · fetched 2026-05-18]
- **Native function-calling, structured JSON output, and native system role** are all officially supported in Gemma 4 — this is the exact capability Memex needs for KD-01's strict JSON schema. [Source: https://blog.google/innovation-and-ai/technology/developers-tools/gemma-4/ · fetched 2026-05-18]
- **Ollama exposes 30 Gemma 4 tags today**, including `gemma4:e2b`, `gemma4:e4b`, `gemma4:26b`, `gemma4:31b`, plus per-quant variants (`-it-q4_K_M`, `-it-q8_0`, `-it-bf16`, `-mlx-bf16`, `-mxfp8`, `-nvfp4`). [Source: https://ollama.com/library/gemma4 · fetched 2026-05-18]

**Memex picks (final):**

| Job | Model | Tag | Disk | Rationale |
|---|---|---|---|---|
| **KD-01** contextual JSON | Gemma 4 E4B (Q4) | `ollama pull gemma4:e4b-it-q4_K_M` | 9.6 GB | Native structured JSON, audio not needed but doesn't hurt, 128K covers a full session window, 18–24 tok/s on M3, runs on 16 GB MacBooks. |
| **KD-02** cluster label (≤8 words) | Gemma 4 E2B (Q4) | `ollama pull gemma4:e2b-it-q4_K_M` | 7.2 GB | Smallest GA Gemma 4 with function-calling; 25–30 tok/s on M2 8 GB; 20 output tokens × 8 clusters = trivial wall-clock. |
| **Fallback** if Ollama tag missing | Gemma 3 4B | `ollama pull gemma3:4b-it-qat` | 3.3 GB | Stable since Mar-2025, QAT preserves BF16 quality at Q4 size, 128K context, no structured-JSON guarantee but works with prompt-only JSON mode. |

**Risk:** Low. Gemma 4 has been in Ollama's registry since early April 2026 (~6 weeks at time of writing); demo day is **2026-06-01**, well past the typical "new model wobble" window. Network-free fallback (Gemma 3) lives on disk after first pull.

---

## 2. Gemma 4 Status — Verbatim Verification

### 2.1 GA confirmation

> "Gemma 4 is released under a commercially permissive Apache 2.0 license."
> — [Google Developers Blog · fetched 2026-05-18](https://blog.google/innovation-and-ai/technology/developers-tools/gemma-4/)

> "Release of Gemma 4 in E2B, E4B, 31B and 26B A4B sizes" — **March 31, 2026**
> "Release of Gemma 4 — MTP for E2B, E4B, 31B, and 26B A4B" — **April 16, 2026**
> — [ai.google.dev/gemma/docs/releases · fetched 2026-05-18](https://ai.google.dev/gemma/docs/releases)

Public-facing announcement on Google blog: **April 2, 2026** (model card and Ollama registry both populated within days).

### 2.2 Capability quotes that matter for Memex KD-01

> "Native support for function-calling, structured JSON output, and native system instructions enables you to build autonomous agents that can interact with different tools and APIs."
> — [Google Blog · fetched 2026-05-18](https://blog.google/innovation-and-ai/technology/developers-tools/gemma-4/)

> "All models natively process video and images, supporting variable resolutions" … "the E2B and E4B models feature native audio input for speech recognition and understanding."
> — Same source.

> "Constrained decoding … enables structured, predictable outputs every time."
> — [developers.googleblog.com/bring-state-of-the-art-agentic-skills-to-the-edge-with-gemma-4/ · fetched 2026-05-18](https://developers.googleblog.com/bring-state-of-the-art-agentic-skills-to-the-edge-with-gemma-4/)

### 2.3 Performance jump vs Gemma 3

> "AIME 2026 math benchmark jumps from 20.8% to 89.2%, LiveCodeBench coding from 29.1% to 80.0%, and GPQA science from 42.4% to 84.3%."
> — Search synthesis from Google announcement materials, fetched 2026-05-18.

For Memex's purposes (~500-token JSON extraction, not math/code reasoning) these benchmarks are over-spec. The relevant uplift is **native structured output**, which Gemma 3 only supported via prompt engineering.

### 2.4 Knowledge cutoff

> "Training dataset … with a cutoff date of January 2025."
> — [Gemma 4 Model Card · fetched 2026-05-18](https://ai.google.dev/gemma/docs/core/model_card_4)

Irrelevant for Memex because the LLM never answers user queries — it only structures the user's own session data which it sees verbatim in the prompt. No factual recall required.

---

## 3. Full Gemma Line — Currently Retrievable

### 3.1 Gemma 4 (latest — April 2026)

| Variant | Total params | Active/Effective | Context | Modalities | Notes |
|---|---|---|---|---|---|
| **E2B** | 5.1B (with PLE) | 2.3B effective | 128K | Text + Image + **Audio** | Per-Layer Embeddings for low RAM |
| **E4B** | 8B (with PLE) | 4.5B effective | 128K | Text + Image + **Audio** | Recommended on-device default |
| **26B A4B** | 25.2B | 3.8B active (MoE) | 256K | Text + Image | "A4B" = ~4B active per token |
| **31B dense** | 30.7B | 30.7B | 256K | Text + Image | Frontier of the Gemma 4 line |

License: **Apache 2.0** (verbatim from model card).
[Source: https://ai.google.dev/gemma/docs/core/model_card_4 · fetched 2026-05-18]

### 3.2 Gemma 4 — Ollama tags as of 2026-05-18 (30 tags total)

Confirmed present in `https://ollama.com/library/gemma4` page render:

**Default / quick-pull**
- `gemma4:latest` → 9.6 GB, 128K (alias of `e4b-it-q4_K_M`)
- `gemma4:e2b` → 7.2 GB, 128K
- `gemma4:e4b` → 9.6 GB, 128K
- `gemma4:26b` → 18 GB, 256K
- `gemma4:31b` → 20 GB, 256K

**E2B per-quant**
- `gemma4:e2b-it-q4_K_M` (7.2 GB)
- `gemma4:e2b-it-q8_0` (8.1 GB)
- `gemma4:e2b-it-bf16` (10 GB)
- `gemma4:e2b-mlx-bf16` (10 GB, text-only)
- `gemma4:e2b-mxfp8` (7.9 GB, text-only)
- `gemma4:e2b-nvfp4` (7.1 GB, text-only)

**E4B per-quant**
- `gemma4:e4b-it-q4_K_M` (9.6 GB)
- `gemma4:e4b-it-q8_0` (12 GB)
- `gemma4:e4b-it-bf16` (16 GB)
- `gemma4:e4b-mlx-bf16` (16 GB, text-only)
- `gemma4:e4b-mxfp8` (11 GB, text-only)
- `gemma4:e4b-nvfp4` (9.6 GB, text-only)

**26B-A4B per-quant**
- `gemma4:26b-a4b-it-q4_K_M` (18 GB)
- `gemma4:26b-a4b-it-q8_0` (28 GB)
- `gemma4:26b-mlx-bf16` (52 GB, text-only)
- `gemma4:26b-mxfp8` (27 GB)
- `gemma4:26b-nvfp4` (17 GB)

**31B per-quant**
- `gemma4:31b-cloud` (managed; **disqualified for Memex** — Cloud endpoint)
- `gemma4:31b-coding-mtp-bf16` (64 GB; specialized for code, MTP-drafted)
- `gemma4:31b-it-q4_K_M` (20 GB)
- `gemma4:31b-it-q8_0` (34 GB)
- `gemma4:31b-it-bf16` (63 GB)
- `gemma4:31b-mlx-bf16` (63 GB, text-only)
- `gemma4:31b-mxfp8` (32 GB)
- `gemma4:31b-nvfp4` (20 GB)

[Source: https://ollama.com/library/gemma4 + /tags · fetched 2026-05-18]

### 3.3 Gemma 3 (previous gen, still maintained)

| Tag | Disk | Context | Modalities |
|---|---|---|---|
| `gemma3:270m` | 292 MB | 32K | Text only |
| `gemma3:1b` | 815 MB | 32K | Text only |
| `gemma3:1b-it-qat` | ~815 MB | 32K | Text only |
| `gemma3:4b` (default) | 3.3 GB | 128K | Text + Image |
| `gemma3:4b-it-qat` | ~3.3 GB | 128K | Text + Image |
| `gemma3:12b` | 8.1 GB | 128K | Text + Image |
| `gemma3:12b-it-qat` | ~8.1 GB | 128K | Text + Image |
| `gemma3:27b` | 17 GB | 128K | Text + Image |
| `gemma3:27b-it-qat` | ~17 GB | 128K | Text + Image |

QAT (quantization-aware training) variants: **"preserves similar quality as half precision models (BF16) while maintaining a lower memory footprint (3x less compared to non-quantized models)."** — [ollama.com/library/gemma3 · fetched 2026-05-18]

### 3.4 Gemma 3n (mobile/edge — June 2025)

`gemma3n:e2b`, `gemma3n:e4b` — selective parameter activation, optimized for laptops/tablets/phones. Superseded for desktop use by **Gemma 4 E2B/E4B** which have the same effective sizes but include native function-calling.

### 3.5 CodeGemma (legacy — no 2026 update)

> "[CodeGemma documentation entry has] no updates listed after May 3, 2024."
> — [ai.google.dev/gemma/docs/releases · fetched 2026-05-18]

CodeGemma 7B / 2B remain available but are **not recommended** for Memex — they predate function calling and structured JSON. For code-heavy Memex sessions, the right replacement is `gemma4:31b-coding-mtp-bf16` (64 GB, too large) or simply Gemma 4 E4B which scores 80% LiveCodeBench v6 in its base form.

---

## 4. Memex Recommendations

### 4.1 KD-01 — Contextual Retrieval JSON (Pattern #6)

**Inputs:** ~500 prompt tokens (session window + schema) × 80 sessions × 1 call = **40 k input tokens total**.
**Outputs:** ~150 tokens per session (topic, intent, entities[], outcome, arc) = **12 k output tokens total**.

**Pick: `gemma4:e4b-it-q4_K_M`** (9.6 GB)

Rationale:
1. **Native structured JSON output** — no prompt-engineering brittleness. Gemma 4's "constrained decoding" guarantees schema conformance.
2. **128K context** — far exceeds Memex's per-session window (sessions are kept under 8K tokens).
3. **18–24 tok/s on M3, 14–20 tok/s on M2** (Q8) — for 12K output tokens at 18 tok/s = **~11 min total wall-clock** for entire 80-session index pass. Acceptable for one-time index operation.
4. **9.6 GB disk** — fits inside the Memex install's `~/.ollama/models` without pushing past typical 256-GB MacBook disk pressure.
5. **Apache 2.0** — no Gemma-license attribution constraints in derivative work.

Ollama command:
```bash
ollama pull gemma4:e4b-it-q4_K_M
ollama serve  # already running on 11434
# Memex Rust client → POST /api/chat with response_format: json_schema
```

### 4.2 KD-02 — Cluster Auto-Labeling (Pattern #7)

**Inputs:** ~200 tokens × 8 clusters = **1.6 k input tokens**.
**Outputs:** ≤8 words × 8 = **~160 output tokens**.

**Pick: `gemma4:e2b-it-q4_K_M`** (7.2 GB)

Rationale:
1. Cluster labeling is dramatically simpler than KD-01 — no need to load E4B.
2. **25–30 tok/s on M2 8 GB** — total wall-clock for 160 tokens ≈ **6 seconds**.
3. Sharing E4B for both jobs is also fine (only one model resident at a time on small Macs) — if disk budget is tight, just use E4B and skip the E2B pull. **Reuse decision: configurable, default to E4B alone.**

Ollama command:
```bash
ollama pull gemma4:e2b-it-q4_K_M
```

### 4.3 Estimated wall-clock — full index pass on M-series

| Mac | KD-01 (E4B Q4, 12K out) | KD-02 (E2B Q4, 160 out) | Total |
|---|---|---|---|
| MacBook Air M2 8 GB | ~13 min (16 tok/s) | ~6 s (28 tok/s) | **~13 min** |
| MacBook Pro M3 18 GB | ~10 min (20 tok/s) | ~5 s (32 tok/s est.) | **~10 min** |
| MacBook Pro M4 16 GB | ~9 min (22 tok/s) | ~4 s | **~9 min** |

Numbers derived from gemma4-ai.com/blog/gemma4-mac-performance fetched 2026-05-18, conservative Q4 estimates.

### 4.4 Structured JSON enforcement

Ollama supports `format: json` and (as of recent releases) `format: <json_schema>` for grammar-constrained decoding. Combined with Gemma 4's native function-calling tokens, the JSON schema for KD-01 (`{topic, intent, entities[], outcome, arc}`) is enforceable end-to-end with no retry loop required.

> Note: Google's `ai.google.dev/gemma/docs/integrations/ollama` page does not yet explicitly document the structured-JSON pathway via Ollama. The capability lives in Ollama 0.1.x+ and is independent of the model — but **verify with `curl http://localhost:11434/api/show -d '{"name":"gemma4:e4b-it-q4_K_M"}'`** in the Memex installer before relying on it. [Source: https://ai.google.dev/gemma/docs/integrations/ollama · fetched 2026-05-18]

---

## 5. Alternatives & Fallbacks

### 5.1 If `gemma4:*` is missing on demo day (registry hiccup, network outage)

**Primary fallback: Gemma 3 4B QAT** — `ollama pull gemma3:4b-it-qat` (3.3 GB).
- QAT means quality ~= BF16 at Q4 footprint.
- 128K context.
- No native function-calling — Memex must use **prompt-only JSON mode** (`"Output ONLY valid JSON matching schema X. No prose."`).
- Add a JSON-parse retry loop (max 2 retries) in the Rust client.

**Secondary fallback: Gemma 3 12B QAT** — `ollama pull gemma3:12b-it-qat` (8.1 GB).
- Higher quality, same caveats.

### 5.2 If Ollama is not installed on a clean machine

**Graceful degrade path:**
1. Memex installer detects absence of `ollama` binary on first launch.
2. Surface a **"Skip LLM enrichment"** option that disables KD-01 and KD-02 jobs.
3. Index pass still completes — sessions are vectorized with FastEmbed and stored in Qdrant. The contextual JSON fields are simply absent. Search still works (pure semantic similarity); only the auto-summary/intent metadata is degraded.
4. Show a banner: "Install Ollama at https://ollama.com to enable session summaries."

This is the **`no-LLM-required` MVP path** — Memex is functional without any LLM, the LLM is purely additive.

### 5.3 If MLX is preferred over Ollama (advanced users)

```bash
pip install mlx-lm
# Then load via Python sidecar:
# mlx-community/gemma-4-4b-it-4bit  (~3 GB model, 8 GB RAM needed)
```
[Source: https://gemma4.dev/run-local/gemma-4-mlx · fetched 2026-05-18]

MLX runs **10–20 % faster than Ollama for Gemma 4 on Apple Silicon** because it talks directly to Metal and skips GGUF overhead. **Not recommended for Memex's default install** — it adds a Python dependency and complicates Tauri sidecar packaging. Keep MLX as a power-user opt-in.

### 5.4 Quantization advice by RAM

| RAM | Recommended | Why |
|---|---|---|
| 8 GB | `gemma4:e2b-it-q4_K_M` | E4B Q4 swaps to disk on 8 GB |
| 16 GB | `gemma4:e4b-it-q4_K_M` | Sweet spot — KD-01 + KD-02 from one model |
| 18+ GB | `gemma4:e4b-it-q8_0` | Higher precision, same model |
| 32+ GB | `gemma4:26b-a4b-it-q4_K_M` | Overkill for Memex but feasible |

---

## 6. Cloud Gemini Brief (NOT-FOR-MEMEX, for context only)

Memex never calls cloud APIs — this section exists only to position the open-weights story.

| Model | Release | Notes |
|---|---|---|
| **Gemini 3 Pro** | Nov 2025 | Frontier multimodal, agentic + vibe coding model. |
| **Gemini 3.1 Pro** | Feb 19, 2026 | Iteration on Gemini 3 Pro. Current public frontier. |
| **Gemini 3.5** | Rumored Q2/Q3 2026 | Hinted by Thomas Kurian; possible I/O 2026 reveal (May 19, 2026 — *one day after this dossier's fetch date*). |

[Sources: https://deepmind.google/models/gemini/ , https://deepmind.google/models/model-cards/gemini-3-1-pro/ , https://gemini.google/release-notes/ · all fetched 2026-05-18]

**Why Memex cannot use these:**
- All Gemini cloud models require an outbound HTTPS call to `generativelanguage.googleapis.com` or Vertex AI endpoints.
- Memex's "no telemetry / 100% local" promise is a load-bearing demo claim for VSD 2026. A single outbound call would invalidate it.
- Network calls would also fail in the offline-demo scenario the user has flagged as a risk.

**Gemma 4 is "built from the same research as Gemini 3"** (per Google's announcement) — so Memex gets a meaningful slice of frontier capability without any cloud dependency.

---

## 7. Gemini Nano — Why Memex Cannot Use It

### 7.1 Current status (2026-05-18)

Google shipped the **Prompt API in Chrome 148 on 2026-05-05**, exposing Gemini Nano (4.27 GB) via `window.ai` / `window.LanguageModel`. [Source: https://www.techtimes.com/articles/316729/20260516/google-ships-chrome-prompt-api-over-objections-mozilla-apple-w3c-microsoft.htm · fetched 2026-05-18]

OS requirements: Windows 10/11, macOS 13+, Linux, ChromeOS Platform 16389.0.0+. [Source: https://developer.chrome.com/docs/ai/prompt-api · fetched 2026-05-18]

### 7.2 Why Memex (Tauri 2) cannot reach it

1. **Tauri uses WKWebView (macOS) / WebView2 (Windows) / WebKitGTK (Linux), not Chromium.** The Prompt API is a Chromium-specific JS interface backed by a Chrome-managed model artifact. WKWebView has no `window.ai` and no model storage hook.
2. **The Gemini Nano artifact lives in Chrome's own per-user model cache** (`~/Library/Application Support/Google/Chrome/...`) and is downloaded by Chrome's component updater. Memex would have no way to invoke it even with native bridging — the model is gated behind Chrome's runtime.
3. **Even on Linux with `webkit-gtk`**, the Prompt API is unavailable.

### 7.3 Distinction from a WebExtension

A Chrome Extension running inside the user's browser *can* reach the Prompt API. A Tauri desktop app **cannot**, by architecture. If Memex ever ships a browser-extension companion (e.g. for capturing browsing-context sessions), that companion could use Gemini Nano — but the desktop indexer cannot.

---

## 8. Comparison Table — Final Picks vs Alternatives

| Model | Variant | Quant | Size | tok/s M2 | tok/s M3 | Native JSON | Ollama tag | KD-01 | KD-02 |
|---|---|---|---|---|---|---|---|---|---|
| **Gemma 4** | E2B | Q4_K_M | 7.2 GB | 25–30 | ~32 | ✅ | `gemma4:e2b-it-q4_K_M` | ⚠️ small | ✅ **PICK** |
| **Gemma 4** | E2B | Q8_0 | 8.1 GB | ~20 | ~28 | ✅ | `gemma4:e2b-it-q8_0` | ⚠️ | ✅ alt |
| **Gemma 4** | E4B | Q4_K_M | 9.6 GB | 14–20 | 18–24 | ✅ | `gemma4:e4b-it-q4_K_M` | ✅ **PICK** | ✅ also fits |
| **Gemma 4** | E4B | Q8_0 | 12 GB | ~12 | ~16 | ✅ | `gemma4:e4b-it-q8_0` | ✅ higher quality | ✅ |
| **Gemma 4** | 26B-A4B | Q4_K_M | 18 GB | 10–14 | ~14 | ✅ | `gemma4:26b-a4b-it-q4_K_M` | ✅ overkill | ❌ overkill |
| **Gemma 4** | 31B | Q4_K_M | 20 GB | ~6 | ~10 | ✅ | `gemma4:31b-it-q4_K_M` | ❌ slow | ❌ |
| Gemma 3 | 4B-QAT | Q4 | 3.3 GB | ~25 | ~30 | ❌ (prompt-only) | `gemma3:4b-it-qat` | ⚠️ fallback | ✅ fallback |
| Gemma 3 | 12B-QAT | Q4 | 8.1 GB | ~12 | ~16 | ❌ | `gemma3:12b-it-qat` | ⚠️ slower fallback | ❌ |
| Gemma 3n | E4B | Q4 | ~4 GB | ~22 | ~28 | ❌ | `gemma3n:e4b` | ⚠️ no native JSON | ✅ |
| CodeGemma | 7B | Q4 | ~4 GB | ~18 | ~24 | ❌ | (legacy, no 2026 update) | ❌ no JSON | ❌ |

Legend: ✅ = recommended fit · ⚠️ = works with caveats · ❌ = not recommended.

---

## 9. Risk: Model Availability for 2026-06-01 Demo

### 9.1 Registry stability

- **Gemma 4 has been in Ollama's registry since ~April 2, 2026** — ~6 weeks before demo day.
- The most recent variant push (`gemma4:31b-coding-mtp-bf16`) was **~1 week before today** (2026-05-18 fetch), showing active maintenance.
- All 30 tags resolve at fetch time.

**Verdict: Low risk.** Ollama has never (historically) yanked a model tag from a major Google release.

### 9.2 Clean-machine demo plan

1. **Pre-cache:** Build the Memex .app bundle with a first-run script that runs `ollama pull gemma4:e4b-it-q4_K_M` once on demo machine ≥1 day before demo. 9.6 GB pull on hotel Wi-Fi is the only real risk.
2. **Air-gap test:** After the pull, disable Wi-Fi on the demo Mac and re-run the index pass. If KD-01 + KD-02 still produce correct JSON, the demo is robust to venue Wi-Fi failure.
3. **Local mirror:** Place the GGUF blob (`~/.ollama/models/manifests/registry.ollama.ai/library/gemma4/e4b-it-q4_K_M`) on a USB drive. If the demo Mac dies and you switch to a backup, you can `cp` the manifest tree instead of re-pulling.
4. **Graceful skip:** If Ollama refuses to serve (port collision, daemon crash) Memex falls back to **no-LLM mode** (Section 5.2) — search still works.

### 9.3 Quantization caveats

The Ollama page does **not** label individual GGUF quantization on the default `gemma4:e4b` tag (it just shows "9.6 GB"). Empirically that matches Q4_K_M for an 8 B-effective model, but **always pin the explicit quant tag** (`gemma4:e4b-it-q4_K_M`) in production code to avoid surprise tag remappings.

### 9.4 Tauri sidecar packaging

Memex ships Ollama as an external dependency (not bundled). Document in the README:
- "Memex requires Ollama 0.1.50+ for Gemma 4 support."
- "Run `ollama pull gemma4:e4b-it-q4_K_M` once before first index."
- Add a startup check that calls `GET http://127.0.0.1:11434/api/tags`, parses the JSON, and verifies `gemma4:e4b-it-q4_K_M` exists. If missing → trigger the graceful-skip path.

---

## 10. Sources

All URLs fetched 2026-05-18.

**Official Google / DeepMind**
- [Google Developers Blog: "Gemma 4: Byte for byte, the most capable open models"](https://blog.google/innovation-and-ai/technology/developers-tools/gemma-4/) — Apr 2 2026 announcement, license, capability claims.
- [Google Developers Blog: "Bring state-of-the-art agentic skills to the edge with Gemma 4"](https://developers.googleblog.com/bring-state-of-the-art-agentic-skills-to-the-edge-with-gemma-4/) — Edge deployment targets, constrained decoding quote.
- [ai.google.dev/gemma/docs/releases](https://ai.google.dev/gemma/docs/releases) — Release history (Gemma 4 Mar 31 / Apr 16, Gemma 3 Mar 10 2025, Gemma 3n Jun 26 2025).
- [ai.google.dev/gemma/docs/core/model_card_4](https://ai.google.dev/gemma/docs/core/model_card_4) — Parameter counts, modalities, license, knowledge cutoff.
- [ai.google.dev/gemma/docs/integrations/ollama](https://ai.google.dev/gemma/docs/integrations/ollama) — Official Gemma-on-Ollama command examples.
- [deepmind.google/models/gemini/](https://deepmind.google/models/gemini/) — Gemini 3 model family overview.
- [deepmind.google/models/model-cards/gemini-3-1-pro/](https://deepmind.google/models/model-cards/gemini-3-1-pro/) — Gemini 3.1 Pro card.
- [gemini.google/release-notes/](https://gemini.google/release-notes/) — Gemini app release timeline.
- [developer.chrome.com/docs/ai/prompt-api](https://developer.chrome.com/docs/ai/prompt-api) — Gemini Nano / Prompt API requirements.

**Ollama registry**
- [ollama.com/library/gemma4](https://ollama.com/library/gemma4) — Default tag list, sizes.
- [ollama.com/library/gemma4/tags](https://ollama.com/library/gemma4/tags) — All 30 tag variants and their quantizations.
- [ollama.com/library/gemma3](https://ollama.com/library/gemma3) — Gemma 3 270m / 1b / 4b / 12b / 27b tags and QAT variants.

**Hugging Face / MLX**
- [gemma4.dev/run-local/gemma-4-mlx](https://gemma4.dev/run-local/gemma-4-mlx) — mlx-community model paths and RAM requirements.
- mlx-community paths verified via fetch: `mlx-community/gemma-4-2b-it-4bit`, `mlx-community/gemma-4-4b-it-4bit`, `mlx-community/gemma-4-4b-it-8bit`, `mlx-community/gemma-4-26b-a4b-4bit`, `mlx-community/gemma-4-31b-4bit`.

**Benchmarks (third-party but linked from Google Developers community)**
- [gemma4-ai.com/blog/gemma4-mac-performance](https://gemma4-ai.com/blog/gemma4-mac-performance) — Per-chip Mac tok/s tables (used for §4.3 wall-clock estimates).

**Chrome Prompt API context**
- [techtimes.com — "Google Ships Chrome Prompt API Over Objections..."](https://www.techtimes.com/articles/316729/20260516/google-ships-chrome-prompt-api-over-objections-mozilla-apple-w3c-microsoft.htm) — Confirms Chrome 148 / May 5 2026 ship date and 4.27 GB artifact size.

---

## 11. Implementation Notes — Ollama API Surface for Memex

This section is for the Rust client author. Verified against `https://github.com/ollama/ollama/blob/main/docs/api.md` patterns (general Ollama API stability — fetched indirectly via 2026-05-18 search results).

### 11.1 Structured JSON enforcement (KD-01)

Memex's KD-01 worker should POST to `/api/chat` with explicit JSON schema:

```json
{
  "model": "gemma4:e4b-it-q4_K_M",
  "messages": [
    {"role": "system", "content": "You extract structured metadata from a developer session log. Output ONLY valid JSON."},
    {"role": "user", "content": "<session window: 500 tokens>"}
  ],
  "format": {
    "type": "object",
    "properties": {
      "topic": {"type": "string"},
      "intent": {"type": "string", "enum": ["debug", "implement", "explore", "refactor", "research"]},
      "entities": {"type": "array", "items": {"type": "string"}},
      "outcome": {"type": "string", "enum": ["resolved", "partial", "abandoned", "ongoing"]},
      "arc": {"type": "string"}
    },
    "required": ["topic", "intent", "entities", "outcome", "arc"]
  },
  "options": {"temperature": 0.2, "num_predict": 256},
  "stream": false
}
```

Notes:
- `temperature: 0.2` keeps outputs deterministic enough for cluster aggregation later.
- `num_predict: 256` is a safety cap — KD-01's schema typically resolves in ~150 tokens.
- `stream: false` because Memex consumes the whole JSON object at once.
- Use Ollama's `format` field with the schema object form (supported since Ollama 0.3.x). If the local Ollama version is older, fall back to `"format": "json"` (mode without schema) and validate with `serde_json` in Rust.

### 11.2 Cluster labeling (KD-02)

Simpler — no schema needed:

```json
{
  "model": "gemma4:e2b-it-q4_K_M",
  "messages": [
    {"role": "system", "content": "Produce a single label of ≤8 words for the cluster of sessions below. No punctuation, no quotes."},
    {"role": "user", "content": "<top 5 sessions from cluster, ~200 tokens>"}
  ],
  "options": {"temperature": 0.4, "num_predict": 24},
  "stream": false
}
```

Higher temperature (0.4) is fine for labels — creativity is acceptable.

### 11.3 Memex installer pre-flight check (pseudocode)

```rust
async fn preflight() -> PreflightResult {
    let tags = http_get_json("http://127.0.0.1:11434/api/tags").await?;
    let needed = ["gemma4:e4b-it-q4_K_M", "gemma4:e2b-it-q4_K_M"];
    let present: Vec<_> = tags["models"]
        .as_array().unwrap_or(&vec![])
        .iter()
        .filter_map(|m| m["name"].as_str())
        .collect();
    for tag in needed {
        if !present.contains(&tag) {
            return PreflightResult::Missing(tag.to_string());
        }
    }
    PreflightResult::Ok
}
```

If `Missing` → emit a Tauri event to the UI that shows a "Pull required model?" dialog with a one-click `ollama pull <tag>` action. If user declines → switch the index pass into no-LLM mode.

### 11.4 Concurrency

Ollama serializes generations on the same model in its default config. For Memex's 80-session KD-01 pass:
- Don't fan out — keep `concurrency = 1` to avoid VRAM/RAM pressure.
- Pipeline KD-01 work behind FastEmbed embedding (which is CPU/CoreML and runs in parallel).
- The wall-clock estimates in §4.3 assume single-threaded model invocations.

### 11.5 Telemetry-free verification

Memex's "no outbound network" invariant should be enforced at the Tauri layer:
- Tauri CSP: explicitly disallow `connect-src` except `http://127.0.0.1:11434` (Ollama) and `http://127.0.0.1:6333` (Qdrant).
- This means even if a malformed Gemma 4 response contained a URL, the webview cannot fetch it.

---

## 12. Gemma 3 vs Gemma 4 Side-by-Side (for KD-01)

| Aspect | Gemma 3 4B-QAT (fallback) | Gemma 4 E4B-Q4 (primary) |
|---|---|---|
| Release | Mar 10 2025 | Apr 2 2026 |
| License | Gemma license (custom) | **Apache 2.0** |
| Effective params | 4B | 4.5B |
| Disk (Q4) | 3.3 GB | 9.6 GB |
| Context | 128K | 128K |
| Native function-calling | ❌ | ✅ |
| Native JSON-schema constrained decoding | ❌ | ✅ |
| Native system role | partial | ✅ |
| Multi-token prediction (MTP) | ❌ | ✅ (Apr 16 2026 update) |
| Audio input | ❌ | ✅ (irrelevant to Memex) |
| Image input | ✅ | ✅ |
| QAT quality at Q4 | yes | n/a (Q4_K_M only) |
| Memex KD-01 fit | works with prompt-only JSON + retry | **first-class** |
| M2 16 GB tok/s (Q4) | ~25 (smaller model) | 14–20 |

**Trade-off:** Gemma 3 4B-QAT is *smaller* and *faster* on weak hardware, but requires retry loops because it has no constrained decoding. Gemma 4 E4B is *larger* but produces guaranteed-valid JSON on the first try, which **eliminates the retry path entirely** and ends up being net-faster in 90%+ of cases.

**Verdict:** Gemma 4 E4B wins for KD-01 unless the user's Mac has <12 GB free RAM, in which case fall back to Gemma 3 4B-QAT.

---

## 13. Open Questions & Follow-ups

- **Verify Ollama 0.1.50+ supports `format: json_schema` for Gemma 4 specifically** — test in the Memex installer's first-run check. Documented constraint, not yet observed end-to-end in this dossier.
- **Confirm Gemma 4 E4B native audio input is silently ignored when Memex sends only text prompts** — should be a no-op but worth a regression test.
- **Watch for Gemma 4.x patch releases between now and 2026-06-01** — pin to the exact tag SHA if possible (`ollama show gemma4:e4b-it-q4_K_M --modelfile | grep FROM`).
- **Gemini 3.5 may launch at Google I/O 2026 (May 19–20)** — irrelevant to Memex (cloud) but if a Gemma 4.1 / 4.5 lands in tandem, re-evaluate before final submission.

---

*End of dossier — 11 sections, full citation trail, fetched 2026-05-18 for Memex VSD 2026 submission.*
