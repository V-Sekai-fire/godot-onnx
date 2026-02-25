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

The script builds the **float** variant and, if `GODOT4_BIN` is set, the **doubles** variant:

- `libgodot_onnx.*` — for standard Godot (float real); always built
- `libgodot_onnx_doubles.*` — for Godot built with `precision=double`; only built when `GODOT4_BIN` points to that binary

## Double-precision (doubles) build

gdext requires a Godot binary built with **precision=double** to generate the correct API (see [godotengine/godot#86346](https://github.com/godotengine/godot/issues/86346)). You can build Godot from source with `precision=double`, or use a prebuilt double-precision build if available.

1. **Set `GODOT4_BIN`** to the path of your double-precision Godot executable (e.g. `C:\Godot\Godot_v4.x_doubles.exe` or `./godot.double`).
2. Run the same build script: `.\misc\build.ps1` or `./misc/build.sh`. It will build the float library, then the doubles library using `GODOT4_BIN`.
3. Or build doubles only:
   ```powershell
   $env:GODOT4_BIN = "C:\path\to\godot_double.exe"
   cargo build --release --no-default-features --features double-precision
   ```
   Then copy the output to `sample/addons/godot-onnx/libgodot_onnx_doubles.*`.

## Use in Godot

Open the `sample/` folder as a Godot project (Godot 4.4+) and run the main scene. The extension is loaded via `sample/godot-onnx.gdextension` (float). If your Godot is built with **double precision**, use `godot-onnx-doubles.gdextension` (or rename it to `godot-onnx.gdextension`) so the doubles library is loaded.

## Build only one variant

- **Float only:** `cargo build --release` then copy the library into `sample/addons/godot-onnx/` as `libgodot_onnx.*`.
- **Doubles only:** Set `GODOT4_BIN`, then `cargo build --release --no-default-features --features double-precision` and copy as `libgodot_onnx_doubles.*`.
