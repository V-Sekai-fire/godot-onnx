// Copyright (c) 2026-present K. S. Ernest (iFire) Lee & godot-onnx contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT
// GDExtension entry point for Godot ONNX (ort + tract backend)
// Reuses patterns from M:\iree.gd (IREEModule / IREETensor API)

use godot::init::gdextension;

pub mod module;
pub mod model_data;
pub(crate) mod sync_cell;
pub mod tensor;

#[cfg(all(test, not(target_arch = "wasm32")))]
mod prop_tests;

pub use module::OnnxModule;
pub use model_data::OnnxModelData;
pub use tensor::OnnxTensor;

#[gdextension]
unsafe impl godot::init::ExtensionLibrary for OnnxExtension {}

pub struct OnnxExtension;
