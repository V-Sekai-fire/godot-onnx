# Copyright (c) 2026-present K. S. Ernest (iFire) Lee & godot-onnx contributors
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Run with: godot --path sample --headless -s res://scripts/export_inference_images.gd
# Exports inference result images to user://inference_output/ for validation (e.g. vs Rust tests).

extends Node

func _ready() -> void:
	await _export_all()

func _export_all() -> void:
	var root := get_tree().root

	# Main (ESRGAN): run upscale then save
	var main_scene := load("res://scenes/main/main.tscn") as PackedScene
	if main_scene:
		var main_inst := main_scene.instantiate()
		root.add_child(main_inst)
		await get_tree().process_frame
		var tex_rect = main_inst.get_node_or_null("UI/TextureRect")
		if tex_rect and tex_rect.has_method("upscale") and tex_rect.has_method("save_inference_image"):
			tex_rect.upscale()
			tex_rect.save_inference_image()
		main_inst.queue_free()
		await get_tree().process_frame

	# Pose detection
	var pose_scene := load("res://scenes/pose_detection/pose_detection.tscn") as PackedScene
	if pose_scene:
		var pose_inst := pose_scene.instantiate()
		root.add_child(pose_inst)
		await get_tree().process_frame
		await get_tree().process_frame
		if pose_inst.has_method("save_inference_image"):
			pose_inst.save_inference_image()
		pose_inst.queue_free()
		await get_tree().process_frame

	# Pose thunder
	var thunder_scene := load("res://scenes/pose_detection_thunder/pose_detection_thunder.tscn") as PackedScene
	if thunder_scene:
		var thunder_inst := thunder_scene.instantiate()
		root.add_child(thunder_inst)
		await get_tree().process_frame
		await get_tree().process_frame
		if thunder_inst.has_method("save_inference_image"):
			thunder_inst.save_inference_image()
		thunder_inst.queue_free()
		await get_tree().process_frame

	# MediaPipe
	var mp_scene := load("res://scenes/mediapipe_pose_landmark_full/mediapipe_pose_landmark_full.tscn") as PackedScene
	if mp_scene:
		var mp_inst := mp_scene.instantiate()
		root.add_child(mp_inst)
		await get_tree().process_frame
		await get_tree().process_frame
		if mp_inst.has_method("save_inference_image"):
			mp_inst.save_inference_image()
		mp_inst.queue_free()

	print("Inference images exported to user://inference_output/")
	get_tree().quit()
