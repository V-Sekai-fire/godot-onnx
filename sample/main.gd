# Copyright (c) 2026-present K. S. Ernest (iFire) Lee & godot-onnx contributors
# SPDX-License-Identifier: Apache-2.0 OR MIT
extends Node
## Sample script: load ONNX module and run identity / matmul (same API as IREE.gd).
## Place identity.onnx and matmul.onnx in res://models/ or set paths below.

var _module: OnnxModule

func _ready() -> void:
	_module = OnnxModule.new()
	# Identity: one input, same shape output
	_module.load("res://models/identity.onnx")
	if _module.is_loaded():
		_run_identity()
	else:
		push_warning("identity.onnx not found; add res://models/identity.onnx to test identity")
	# MatMul: two inputs [A, B], one output
	_module.unload()
	_module.load("res://models/matmul.onnx")
	if _module.is_loaded():
		_run_matmul()
	else:
		push_warning("matmul.onnx not found; add res://models/matmul.onnx to test matmul")
	# Benchmark: heavier model to tell CPU vs accelerated by timing
	_module.unload()
	_module.load("res://models/benchmark.onnx")
	if _module.is_loaded():
		_run_benchmark()
	else:
		push_warning("benchmark.onnx not found; run create_benchmark_onnx.py in sample/models")

func _run_identity() -> void:
	var x := PackedFloat32Array([1.0, 2.0, 3.0])
	var dim := PackedInt64Array([3])
	var input_tensor := OnnxTensor.from_float32s(x, dim)
	var result: Array = _module.call_module("", [input_tensor])
	if result.size() > 0:
		var out: OnnxTensor = result[0] as OnnxTensor
		print("Identity output: ", out.get_data().to_float32_array(), " dim: ", out.get_dimension())

func _run_matmul() -> void:
	# Example: 2x3 @ 3x2 -> 2x2
	var a := PackedFloat32Array([1.0, 2.0, 3.0, 4.0, 5.0, 6.0])
	var b := PackedFloat32Array([1.0, 2.0, 3.0, 4.0, 5.0, 6.0])
	var dim_a := PackedInt64Array([2, 3])
	var dim_b := PackedInt64Array([3, 2])
	var ta := OnnxTensor.from_float32s(a, dim_a)
	var tb := OnnxTensor.from_float32s(b, dim_b)
	var result: Array = _module.call_module("", [ta, tb])
	if result.size() > 0:
		var out: OnnxTensor = result[0] as OnnxTensor
		print("MatMul output dim: ", out.get_dimension(), " data: ", out.get_data().to_float32_array())

func _run_benchmark() -> void:
	# Run benchmark model multiple times and measure; CPU is much slower than accelerated
	var a := PackedFloat32Array([1.0, 2.0, 3.0, 4.0, 5.0, 6.0])
	var b := PackedFloat32Array([1.0, 2.0, 3.0, 4.0, 5.0, 6.0])
	var dim := PackedInt64Array([2, 3])
	var ta := OnnxTensor.from_float32s(a, dim)
	var tb := OnnxTensor.from_float32s(b, dim)
	var n := 100
	var start := Time.get_ticks_usec()
	for i in n:
		_module.call_module("", [ta, tb])
	var elapsed_ms := (Time.get_ticks_usec() - start) / 1000.0
	print("Benchmark: %d runs in %.2f ms (%.3f ms/run) — lower = accelerated" % [n, elapsed_ms, elapsed_ms / n])
