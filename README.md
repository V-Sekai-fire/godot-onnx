# godot-onnx

Rust GDExtension for Godot 4 that runs ONNX models using [ort](https://github.com/pykeio/ort) (ONNX Runtime). API similar to [iree.gd](https://github.com/godotengine/iree.gd): load a model, build tensors, run inference, read results from GDScript.

## Features

- **OnnxModule** (Resource): load ONNX from `res://` or `user://`, run inference with `call_module`.
- **OnnxTensor** (RefCounted): create from `PackedFloat32Array`/`PackedByteArray` + shape; read with `get_data()` / `get_dimension()`.
- **WebGPU EP** (optional): GPU acceleration via Dawn; disable in `Cargo.toml` for CPU only.
- **Property tests**: [proptest](https://docs.rs/proptest) for identity/matmul (no Godot required).

## Quick start

**Requirements:** Rust (stable), Godot 4.4+. On Windows: MSVC toolchain (see [Build](docs/BUILD.md)).

```powershell
# Windows
.\misc\build.ps1
```

```bash
# Linux / macOS
./misc/build.sh
```

Open `sample/` as a Godot project and run the main scene.

## Documentation

- [Build](docs/BUILD.md) — Requirements, build steps, float/doubles variants
- [Usage](docs/USAGE.md) — GDScript API, test models (identity/matmul)
- [Testing](docs/TESTING.md) — `cargo test`, Godot sample
- [Project layout](docs/LAYOUT.md) — Directory structure, pre-commit

## License

Apache-2.0 or MIT (same as ort and gdext).
