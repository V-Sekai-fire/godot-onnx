# Build

## Requirements

- **Rust** (stable)
- **Godot 4.4+**
- **Windows**: MSVC toolchain (Visual Studio Build Tools with C++ or “Developer Command Prompt”). ort’s prebuilt ONNX Runtime for Windows is MSVC-only; the GNU toolchain is not supported for this crate.

## Build the extension

From the repo root:

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
- `libgodot_onnx_doubles.*` — for Godot built with `precision=double` (may be skipped if `GODOT4_BIN` / `GODOT4_GDEXTENSION_JSON` are not set; see [gdext double-precision](https://github.com/godot-rust/gdext)).

## Use in Godot

Open the `sample/` folder as a Godot project (Godot 4.4+) and run the main scene. The extension is loaded via `sample/godot-onnx.gdextension` (float). If your Godot is built with **double precision**, rename or swap so that `godot-onnx-doubles.gdextension` is the one that loads (e.g. rename it to `godot-onnx.gdextension` and disable the float one).

## Build only one variant

- **Float only:** `cargo build --release` then copy the library into `sample/addons/godot-onnx/` as `libgodot_onnx.*`.
- **Doubles only:** `cargo build --release --features double-precision` then copy as `libgodot_onnx_doubles.*`.
