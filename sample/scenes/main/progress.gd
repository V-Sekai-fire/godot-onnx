# Copyright (c) 2026-present K. S. Ernest (iFire) Lee & godot-onnx contributors
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Matches iree.gd sample/scenes/main/progress.gd

extends ProgressBar

func _on_upscaling_start() -> void:
	visible = true

func _on_upscaling_stop() -> void:
	visible = false

func _on_upscaling_step(percentage: float) -> void:
	value = percentage
