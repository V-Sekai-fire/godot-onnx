# Contributing to godot-onnx

Thanks for your interest in contributing. This document covers how to get set up, run tests, and submit changes.

## Getting started

1. **Clone the repo** and open it in your editor.

2. **Install requirements**
   - [Rust](https://rustup.rs/) (stable)
   - Godot 4.4+ (for running the sample)
   - **Windows only**: MSVC (Visual Studio Build Tools with C++). Use “Developer PowerShell” or ensure `link.exe` is in `PATH`.

3. **Build**
   ```bash
   cargo build --release
   ```
   Then copy the library into the sample (see README):
   - Windows: `.\misc\build.ps1`
   - Linux/macOS: `./misc/build.sh`

4. **Generate test models** (needed for tests and the sample):
   ```bash
   cd sample/models
   pip install onnx
   python create_identity_onnx.py
   python create_matmul_onnx.py
   python create_benchmark_onnx.py
   ```

## Running tests

- **Property tests** (Rust, no Godot):
  ```bash
  cargo test
  ```
  Optional: `PROPTEST_CASES=500 cargo test` for more random cases.

- **Godot sample**: open `sample/` as a Godot project and run the main scene to confirm the extension loads and runs identity/matmul.

Before submitting a PR, please ensure `cargo test` passes and the sample runs without errors. CI runs the same tests on push and pull requests (see [.github/workflows/ci.yml](.github/workflows/ci.yml)).

## Code and patches

- **Rust**: format with `cargo fmt`, check with `cargo clippy` (optional but recommended).
- **API**: the extension aims to mirror [iree.gd](https://github.com/godotengine/iree.gd)–style usage (`OnnxModule` / `OnnxTensor`, `load`, `call_module`, `from_float32s`, `get_data`, `get_dimension`). Keep that surface consistent.
- **New features**: if you add execution providers or options, document them in the README and any new public API.

## Commit messages

Use clear, descriptive commit messages. Avoid conventional-commit prefixes (`feat:`, `fix:`, etc.) unless the project adopts them later.

## Questions

Open an issue for bugs, feature ideas, or questions. For project layout and build details, see [README.md](README.md).
