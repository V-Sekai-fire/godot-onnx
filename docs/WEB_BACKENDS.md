# Web (WASM) backends for ort

This project uses the [ort](https://github.com/pykeio/ort) crate for ONNX inference. On the web (wasm32) we need a backend that does not link to the C++ ONNX Runtime. Options:

## Alternative backends (ort API)

These implement the same `ort` API and are enabled with `ort`’s **`alternative-backend`** feature. You call `ort::set_api(backend::api())` before creating sessions.

| Backend      | Crate       | ort version | Notes |
|-------------|-------------|-------------|--------|
| **ort-tract** | `ort-tract` | 2.0.0-rc.11 | Pure Rust, [tract](https://github.com/sonos/tract). **CPU + WebAssembly.** Good operator support. [Docs](https://ort.pyke.io/backends/tract). |
| **ort-candle** | `ort-candle` | 2.0.0-rc.11 | [Hugging Face Candle](https://github.com/huggingface/candle). **CPU, WebAssembly.** Good for transformers; limited ops. [Docs](https://ort.pyke.io/backends/candle). |
| **ort-web**   | `ort-web`   | =2.0.0-rc.11 (exact) | Emscripten-compiled ONNX Runtime, WebGPU. We reverted it due to `ort` 2.0.0-rc.11’s smallvec 1.15 API mismatch on wasm. |

**ort-tract** and **ort-candle** require **ort 2.0.0-rc.11**. The published `ort` 2.0.0-rc.11 has a [SmallVec API mismatch](https://github.com/pykeio/ort) with smallvec 1.15 when building for wasm. Using `ort` from git (branch `main`) avoids that; the repo has been updated to the array form `SmallVec<[T; N]>`.

To use **ort-tract** on wasm you can:

1. **Patch ort to git (main)** in `Cargo.toml` so the build uses the fixed SmallVec API, then add `ort-tract` for wasm and call `ort::set_api(ort_tract::api())` before building sessions.
2. Wait for a new `ort` release (e.g. 2.0.0-rc.12) that fixes smallvec and use that with `ort-tract` or `ort-web`.

## Standalone runtimes (different API)

- **WONNX** ([wonnx](https://github.com/webonnx/wonnx), [lib.rs](https://lib.rs/crates/wonnx))  
  Pure Rust ONNX runtime, WebGPU via wgpu, WebAssembly. Different API from `ort`; would require a separate integration path. Project archived May 2025.

## Current state in this repo

- **All platforms:** **Tract-only.** `ort` 2.0.0-rc.11 with `alternative-backend` and **ort-tract**; no ONNX Runtime C++ (no CoreML, WebGPU, NNAPI). Models are loaded with `commit_from_memory`; inference runs on CPU (or in wasm via tract). `ort` is patched to git `main` so wasm builds (smallvec fix).
