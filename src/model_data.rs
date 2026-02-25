// Copyright (c) 2026-present K. S. Ernest (iFire) Lee & godot-onnx contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Resource that holds ONNX model bytes; used by the import system (engine import cache).

use godot::builtin::PackedByteArray;
use godot::classes::Resource;
use godot::prelude::*;

/// Resource holding raw ONNX model bytes. The import plugin saves this to the engine's import cache; [OnnxModule] loads it via [ResourceLoader].
#[derive(GodotClass)]
#[class(base = Resource, init)]
pub struct OnnxModelData {
    data: PackedByteArray,
    base: Base<Resource>,
}

#[godot_api]
impl OnnxModelData {
    /// Get the ONNX model bytes.
    #[func]
    pub fn get_data(&self) -> PackedByteArray {
        self.data.clone()
    }

    /// Set the ONNX model bytes (used by the import plugin).
    #[func]
    pub fn set_data(&mut self, data: PackedByteArray) {
        self.data = data;
    }
}
