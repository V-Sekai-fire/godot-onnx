# Copyright (c) 2026-present K. S. Ernest (iFire) Lee & godot-onnx contributors
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Matches iree.gd sample/scenes/main/esrgan_texture.gd — uses OnnxModule instead of IREEModule.
# Place an ESRGAN-style ONNX model at res://models/esrgan.onnx (input [1,50,50,3] float, output [1,200,200,3] float).

extends TextureRect

@export var model_path: String = "res://models/esrgan.onnx"

var _module: OnnxModule

signal on_upscaling_start()
signal on_upscaling_step(percentage: float)
signal on_upscaling_stop()

func _ready() -> void:
	_module = OnnxModule.new()
	_module.load(model_path)

func _module_ok() -> bool:
	return _module != null and _module.is_loaded()

func downscale() -> void:
	if not texture: return
	var image := texture.get_image()
	image.resize(int(float(image.get_width()) / 4), int(float(image.get_height()) / 4))
	texture = ImageTexture.create_from_image(image)

func upscale() -> void:
	if not _module_ok():
		push_warning("Load esrgan.onnx (see docs) to run upscale.")
		return
	var width := texture.get_width()
	var last_box_width := width % 50 if (width % 50) != 0 else 50
	var box_column_count := int(float(width) / 50.0) if last_box_width == 50 else int(ceilf(float(width) / 50.0))
	var height := texture.get_height()
	var last_box_height := height % 50 if (height % 50) != 0 else 50
	var box_row_count := int(float(height) / 50.0) if last_box_height == 50 else int(ceilf(float(height) / 50.0))
	var old_image := texture.get_image()
	var new_image_data := PackedByteArray()
	new_image_data.resize(width * 4 * height * 4 * 3)
	var new_image := Image.create_from_data(width * 4, height * 4, false, Image.FORMAT_RGB8, new_image_data)
	old_image.convert(Image.FORMAT_RGB8)
	new_image.convert(Image.FORMAT_RGB8)

	var processing_image_data := PackedByteArray()
	processing_image_data.resize(50 * 50 * 3)

	on_upscaling_start.emit()
	print("Start upscaling.")
	for i in box_column_count:
		for j in box_row_count:
			on_upscaling_step.emit(float(i * box_row_count + j) / float(box_column_count * box_row_count) * 100.0)
			var x_offset := i * 50
			var y_offset := j * 50
			var box_width := 50 if i != box_column_count - 1 else last_box_width
			var box_height := 50 if j != box_row_count - 1 else last_box_height
			var processing_image := Image.create_from_data(50, 50, false, Image.FORMAT_RGB8, processing_image_data)
			processing_image.convert(Image.FORMAT_RGB8)
			processing_image.blit_rect(old_image, Rect2i(x_offset, y_offset, box_width, box_height), Vector2i.ZERO)
			var raw_input_data := processing_image.get_data()
			var clean_input_data := PackedFloat32Array()
			clean_input_data.resize(raw_input_data.size())
			for k in raw_input_data.size():
				clean_input_data[k] = float(raw_input_data[k])
			var dim := PackedInt64Array([1, 50, 50, 3])
			var input_tensor := OnnxTensor.from_float32s(clean_input_data, dim)
			var result: Array = _module.call_module("", [input_tensor])
			if result.is_empty():
				push_error("No result")
				return
			var output_tensor: OnnxTensor = result[0] as OnnxTensor
			var raw_output_data := output_tensor.get_data().to_float32_array()
			var clean_output_data := PackedByteArray()
			clean_output_data.resize(raw_output_data.size())
			for k in raw_output_data.size():
				clean_output_data[k] = clampi(int(raw_output_data[k]), 0, 255)
			var output_image := Image.create_from_data(200, 200, false, Image.FORMAT_RGB8, clean_output_data)
			new_image.blit_rect(output_image, Rect2i(0, 0, box_width * 4, box_height * 4), Vector2i(x_offset * 4, y_offset * 4))
	on_upscaling_stop.emit()
	texture = ImageTexture.create_from_image(new_image)

const INFERENCE_OUTPUT_DIR := "user://inference_output"
const INFERENCE_ESRGAN_FILE := "esrgan.png"

func save_inference_image() -> void:
	if not texture:
		push_warning("No texture to save.")
		return
	var dir := DirAccess.open("user://")
	if dir and not dir.dir_exists("inference_output"):
		dir.make_dir_recursive("inference_output")
	var path := INFERENCE_OUTPUT_DIR.path_join(INFERENCE_ESRGAN_FILE)
	var err := texture.get_image().save_png(path)
	if err != OK:
		push_error("Save failed: %s" % error_string(err))
	else:
		print("Saved inference image: ", path)
