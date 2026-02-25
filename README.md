# godot-onnx

Rust GDExtension for Godot 4 that runs ONNX models using [ort](https://github.com/pykeio/ort) (ONNX Runtime). It exposes an API similar to [iree.gd](https://github.com/godotengine/iree.gd) so you can load a model, build tensors, run inference, and read results from GDScript.

## Features

- **OnnxModule** (Resource): load an ONNX file from a Godot path (`res://` or `user://`), run inference with `call_module`.
- **OnnxTensor** (RefCounted): create tensors from `PackedFloat32Array`/`PackedByteArray` and shape; read back with `get_data()` and `get_dimension()`.
- **WebGPU execution provider**: sessions are created with the WebGPU EP when the feature is enabled (GPU acceleration via Dawn). You can disable it in `Cargo.toml` to use CPU only.
- **Property tests**: [proptest](https://docs.rs/proptest)-based tests for identity and matmul models (no Godot runtime required).

## Requirements

- **Rust** (stable)
- **Godot 4.4+**
- **Windows**: MSVC toolchain (Visual Studio Build Tools with C++ or “Developer Command Prompt”). ort’s prebuilt ONNX Runtime for Windows is MSVC-only; the GNU toolchain is not supported for this crate.

## Build

1. **Build the extension** (from the repo root):

   **Windows (PowerShell)**  
   Use a shell where Rust and MSVC are available (e.g. “Developer PowerShell for VS”, or add `%USERPROFILE%\.cargo\bin` to `PATH`):

   ```powershell
   .\misc\build.ps1
   ```

   **Linux / macOS**

   ```bash
   ./misc/build.sh
   ```

   The script builds **both** the default (float) and **double-precision** variants and copies them into `sample/addons/godot-onnx/`:
   - `libgodot_onnx.*` — for standard Godot (float real)
   - `libgodot_onnx_doubles.*` — for Godot built with `precision=double`

2. **Use in Godot**  
   Open the `sample/` folder as a Godot project (Godot 4.4+) and run the main scene. The extension is loaded via `sample/godot-onnx.gdextension` (float). If your Godot is built with **double precision**, rename or swap so that `godot-onnx-doubles.gdextension` is the one that loads (e.g. rename it to `godot-onnx.gdextension` and disable the float one).

   **Build only one variant:** Float only: `cargo build --release` then copy the library into `sample/addons/godot-onnx/` as `libgodot_onnx.*`. Doubles only: `cargo build --release --features double-precision` then copy as `libgodot_onnx_doubles.*`.

## Usage (GDScript)

Pattern matches IREE.gd:

```gdscript
var mod := OnnxModule.new()
mod.load("res://models/identity.onnx")
if not mod.is_loaded():
    push_error("Failed to load model")
    return

var x := PackedFloat32Array([1.0, 2.0, 3.0])
var dim := PackedInt64Array([3])
var input_tensor := OnnxTensor.from_float32s(x, dim)
var result: Array = mod.call_module("", [input_tensor])
if result.size() > 0:
    var out := result[0] as OnnxTensor
    print(out.get_data().to_float32_array(), " ", out.get_dimension())
```

### API summary

| Class        | Methods / static methods |
|-------------|---------------------------|
| **OnnxModule** | `load(path)`, `unload()`, `is_loaded()`, `call_module(func_name, args)` → `Array` of `OnnxTensor`. `func_name` is ignored (single ONNX graph). |
| **OnnxTensor** | `OnnxTensor.from_float32s(float32s, dimension)`, `OnnxTensor.from_bytes(bytes, dimension)`, `get_data()`, `get_dimension()`, `is_captured()`. |

Inputs to `call_module` must be `OnnxTensor` instances; pass them in the same order as the model’s inputs. Outputs are returned as an array of `OnnxTensor`.

## Test models (identity & matmul)

The sample expects two small ONNX models under `sample/models/`:

- **identity.onnx**: one input `x` [3], one output `y` [3] (Identity op).
- **matmul.onnx**: inputs `A` [2,3], `B` [3,2]; output `Y` [2,2] (MatMul op).

Scripts to generate them (run from `sample/models/`):

```bash
cd sample/models
pip install onnx
python create_identity_onnx.py   # writes identity.onnx
python create_matmul_onnx.py     # writes matmul.onnx
```

Then run the Godot sample scene; it will load these from `res://models/...` (project root for the sample is `sample/`, so `res://models/` is `sample/models/`).

## Testing

### Property tests (Rust, no Godot)

The crate uses [proptest](https://docs.rs/proptest) for property-based tests of the identity and matmul models:

```bash
cargo test
```

- **identity_preserves_input**: for any `[f32; 3]`, the identity model output equals the input.
- **matmul_matches_reference**: for any 2×3 and 3×2 matrices, the matmul model output matches a reference implementation within tolerance.

Tests live in `src/prop_tests.rs` and require the same ONNX files in `sample/models/` (see above). To increase the number of random cases:

```bash
PROPTEST_CASES=500 cargo test
```

### Godot sample

Open `sample/` in Godot 4.4+ and run the main scene. It loads identity and matmul, runs one inference each, and prints results to the console.

## Project layout

| Path | Description |
|------|-------------|
| `src/lib.rs` | GDExtension entry; exports `OnnxModule`, `OnnxTensor`. |
| `src/module.rs` | `OnnxModule`: load from path (via temp file), run via ort with WebGPU EP. |
| `src/tensor.rs` | `OnnxTensor`: storage and `from_float32s` / `from_bytes` / `get_data` / `get_dimension`. |
| `src/prop_tests.rs` | Property tests (proptest); compiled only with `cargo test`. |
| `sample/` | Godot project: `project.godot`, `main.tscn`, `main.gd`, `godot-onnx.gdextension`, `godot-onnx-doubles.gdextension` (for double-precision Godot). |
| `sample/addons/godot-onnx/` | Target directory for the built extension library. |
| `sample/models/` | Python scripts and generated `identity.onnx`, `matmul.onnx`. |
| `misc/build.ps1`, `misc/build.sh` | Build and copy the library into `sample/addons/godot-onnx/`. |
| `misc/retry_build.ps1` | Convenience script: build + copy on Windows (Rust in PATH). |
| `misc/check-license-headers.py` | Ensures tracked source files have an SPDX license header; used by pre-commit. |

## Pre-commit (license headers)

Optional: install [pre-commit](https://pre-commit.com/) and run:

```bash
pre-commit install
```

This runs a license check before each commit: all tracked `.rs`, `.gd`, `.py`, `.ps1`, and `.sh` files must contain `SPDX-License-Identifier: Apache-2.0 OR MIT` in the first 25 lines. To add the header to files that lack it:

```bash
python misc/check-license-headers.py --add
```

## License

Apache-2.0 or MIT (same as ort and gdext).
