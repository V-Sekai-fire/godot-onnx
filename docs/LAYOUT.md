# Project layout

| Path                              | Description                                                                                       |
| --------------------------------- | ------------------------------------------------------------------------------------------------- |
| `src/lib.rs`                      | GDExtension entry; exports `OnnxModule`, `OnnxTensor`, `OnnxModelData`.                           |
| `src/module.rs`                   | `OnnxModule`: load via ResourceLoader (imported .onnx) or FileAccess, run via ort with WebGPU EP. |
| `src/model_data.rs`               | `OnnxModelData`: Resource holding ONNX bytes; used by the import system (engine import cache).   |
| `src/tensor.rs`                   | `OnnxTensor`: storage and `from_float32s` / `from_bytes` / `get_data` / `get_dimension`.          |
| `src/prop_tests.rs`               | Property tests (proptest); compiled only with `cargo test`.                                       |
| `sample/`                         | Godot project: `project.godot`, `main.tscn`, `main.gd`, `godot-onnx.gdextension`.                 |
| `sample/addons/godot-onnx/`       | Extension library, editor plugin (plugin.cfg, plugin.gd, import_plugin.gd) for .onnx import.      |
| `sample/models/`                  | Python scripts and generated `identity.onnx`, `matmul.onnx`.                                      |
| `misc/build.ps1`, `misc/build.sh` | Build and copy the library into `sample/addons/godot-onnx/`.                                      |
| `misc/retry_build.ps1`            | Convenience script: build + copy on Windows (Rust in PATH).                                       |
| `misc/check-license-headers.py`   | Ensures tracked source files have an SPDX license header; used by pre-commit.                     |

## Pre-commit (license headers)

Optional: install [pre-commit](https://pre-commit.com/) and run:

```bash
pre-commit install
```

This runs a license check before each commit: all tracked `.rs`, `.gd`, `.py`, `.ps1`, and `.sh` files must contain `SPDX-License-Identifier: Apache-2.0 OR MIT` in the first 25 lines. To add the header to files that lack it:

```bash
python misc/check-license-headers.py --add
```
