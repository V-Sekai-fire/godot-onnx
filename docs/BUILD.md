# Build

## Requirements

- **Rust** (stable)
- **[Janet](https://janet-lang.org/)** (build script; install: **Windows** `scoop install janet`, **macOS** `brew install janet`, **Linux** build from source or use your distro's package)
- **Godot 4.6** (official builds from [godotengine.org](https://godotengine.org/download)) for running the sample and day-to-day use.
- **Windows**: MSVC toolchain (Visual Studio Build Tools with C++ or “Developer Command Prompt”). ort’s prebuilt ONNX Runtime for Windows is MSVC-only; the GNU toolchain is not supported for this crate.

## Build the extension

From the repo root, use the Janet build script (CI and local use the same script):

```bash
janet misc/build.janet
```

Optional flags: `--skip-test`, `--skip-build`, `--skip-doubles`, `--ci`. Wrappers (require Janet on PATH): `.\misc\build.ps1` (Windows), `./misc/build.sh` (Linux/macOS).

The script builds the **float** variant and, if `GODOT4_BIN` is set, the **doubles** variant:

- `libgodot_onnx.*` — for standard Godot (float real); always built
- `libgodot_onnx_doubles.*` — for Godot built with `precision=double`; only built when `GODOT4_BIN` points to that binary

## API generation (GODOT4_BIN)

Builds that use gdext’s **api-custom** (Web/wasm, or double-precision) need a Godot 4 binary at build time so gdext can generate the correct API. Use the **V-Sekai world-godot** editor for that:

- **Download:** [V-Sekai/world-godot — latest.v-sekai-editor-281](https://github.com/V-Sekai/world-godot/releases/tag/latest.v-sekai-editor-281)  
  Get the editor archive for your OS (e.g. `v-sekai-godot-windows.zip`, `v-sekai-godot-linuxbsd.zip`, `v-sekai-godot-macos.zip`), extract it, then set **`GODOT4_BIN`** to the executable inside (e.g. `Godot_v4.x_win64.exe` or `Godot.x86_64`).

Use **official Godot 4.6** from [godotengine.org](https://godotengine.org/download) for running the sample; use the V-Sekai build only for API generation.

## Double-precision (doubles) build

gdext requires a Godot binary built with **precision=double** to generate the correct API (see [godotengine/godot#86346](https://github.com/godotengine/godot/issues/86346)). Use the V-Sekai world-godot editor (see **API generation** above) or another double-precision build, and set **`GODOT4_BIN`** to that executable.

1. **Set `GODOT4_BIN`** to the path of your double-precision Godot executable (e.g. from the V-Sekai release or `./godot.double`).
2. Run the build script: `janet misc/build.janet`. It will build the float library, then the doubles library using `GODOT4_BIN`.
3. Or build doubles only:
   ```powershell
   $env:GODOT4_BIN = "C:\path\to\godot_double.exe"
   cargo build --release --no-default-features --features double-precision
   ```
   Then copy the output to `sample/addons/godot-onnx/libgodot_onnx_doubles.*`.

## Use in Godot

Open the `sample/` folder as a Godot project with **official Godot 4.6** and run the main scene. The extension is loaded via `sample/godot-onnx.gdextension` (float). If your Godot is built with **double precision**, use `godot-onnx-doubles.gdextension` (or rename it to `godot-onnx.gdextension`) so the doubles library is loaded.

## Build only one variant

- **Float only:** `cargo build --release` then copy the library into `sample/addons/godot-onnx/` as `libgodot_onnx.*`.
- **Doubles only:** Set `GODOT4_BIN`, then `cargo build --release --no-default-features --features double-precision` and copy as `libgodot_onnx_doubles.*`.

## Web, Android, iOS (experimental)

The `.gdextension` file includes library entries for **web** (wasm32), **Android** (arm64-v8a, armeabi-v7a, x86_64, x86), and **iOS** (device arm64, simulator arm64/x86_64). Building for these platforms requires:

- **Web:** Rust target `wasm32-unknown-emscripten`, Emscripten SDK, and **`GODOT4_BIN`** pointing to the V-Sekai world-godot editor (see **API generation** above) so gdext can generate bindings. Build with `cargo build --release --target wasm32-unknown-emscripten --no-default-features`. **Windows:** load Emscripten with `& "%USERPROFILE%\scoop\apps\emscripten\current\emsdk_env.ps1"` if you need emcc. Godot export templates must be built with `dlink_enabled=yes` for GDExtension on web. **Backend:** All platforms use **ort + ort-tract** (tract-only; no ONNX Runtime C++). `ort` is patched to git `main` for wasm (see [docs/WEB_BACKENDS.md](WEB_BACKENDS.md)).

- **Android:** Rust targets `aarch64-linux-android`, `armv7-linux-androideabi`, `i686-linux-android`, `x86_64-linux-android`, and Android NDK. Copy the resulting `.so` files into `sample/addons/godot-onnx/` as `libgodot_onnx.arm64-v8a.so`, `libgodot_onnx.armeabi-v7a.so`, etc.
- **iOS:** Rust targets `aarch64-apple-ios`, `aarch64-apple-ios-sim`, `x86_64-apple-ios` (macOS host with Xcode). Copy the `.dylib` files as `libgodot_onnx.ios.arm64.dylib`, `libgodot_onnx.ios.sim.arm64.dylib`, `libgodot_onnx.ios.sim.x86_64.dylib`.

The same CI workflow (`.github/workflows/ci.yml`) runs desktop builds plus optional jobs for web, Android, and iOS; those jobs use `continue-on-error` because **ort** may not provide prebuilt binaries for these targets. See the workflow file for exact commands and artifact layout.
