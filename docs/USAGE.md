# Usage (GDScript)

**.onnx import:** With the addon enabled, `.onnx` files are imported as `OnnxModelData` resources (engine import cache). `mod.load("res://models/identity.onnx")` then loads via [ResourceLoader](https://docs.godotengine.org/en/stable/classes/class_resourceloader.html); no raw file read at runtime for `res://` paths. For `user://` or unimported paths, loading falls back to reading the file directly.

Pattern matches [iree.gd](https://github.com/godotengine/iree.gd):

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

## API summary

| Class          | Methods / static methods                                                                                                                       |
| -------------- | ---------------------------------------------------------------------------------------------------------------------------------------------- |
| **OnnxModule** | `load(path)`, `unload()`, `is_loaded()`, `call_module(func_name, args)` → `Array` of `OnnxTensor`. `func_name` is ignored (single ONNX graph). |
| **OnnxTensor** | `OnnxTensor.from_float32s(float32s, dimension)`, `OnnxTensor.from_bytes(bytes, dimension)`, `get_data()`, `get_dimension()`, `is_captured()`.  |

Inputs to `call_module` must be `OnnxTensor` instances; pass them in the same order as the model’s inputs. Outputs are returned as an array of `OnnxTensor`.

**Backends:** The extension uses ONNX Runtime's WebGPU execution provider on all platforms.

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
