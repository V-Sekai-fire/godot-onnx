// Copyright (c) 2026-present K. S. Ernest (iFire) Lee & godot-onnx contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT
// GDExtension entry point for Godot ONNX (ort-based)
// Reuses patterns from M:\iree.gd (IREEModule / IREETensor API)

use godot::init::gdextension;

pub mod module;
pub mod model_data;
pub mod tensor;

#[cfg(test)]
mod prop_tests;

pub use module::OnnxModule;
pub use model_data::OnnxModelData;
pub use tensor::OnnxTensor;

#[gdextension]
unsafe impl godot::init::ExtensionLibrary for OnnxExtension {}

pub struct OnnxExtension;
