// Copyright (c) 2026-present K. S. Ernest (iFire) Lee & godot-onnx contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT
// ONNXModule: Resource that loads an ONNX model and runs inference via ort.
// API matches IREEModule: load(path), unload(), call_module(func_name, args) -> Array of ONNXTensor.

use godot::builtin::{GString, PackedByteArray, VarArray};
use godot::classes::{FileAccess, Resource, ResourceLoader};
use godot::prelude::*;
use ndarray::ArrayD;
use ort::ep::{self, ExecutionProviderDispatch};
use ort::session::Session;
use ort::session::builder::SessionBuilder;
use ort::session::SessionOutputs;
use ort::value::{DynValue, Tensor};
use std::io::Write;
use std::sync::Mutex;

use crate::model_data::OnnxModelData;
use crate::tensor::OnnxTensor;

/// Build session with WebGPU execution provider (all platforms).
fn session_builder() -> ort::Result<SessionBuilder> {
    let webgpu_ep: ExecutionProviderDispatch = ep::WebGPU::default().into();
    Session::builder()
        .and_then(|b: SessionBuilder| b.with_execution_providers([webgpu_ep]))
}

/// Resource that loads an ONNX model from a Godot path (`res://` or `user://`) and runs inference via ONNX Runtime.
/// Create with `OnnxModule.new()`, then call [load] to load a model and [call_module] to run inference.
#[derive(GodotClass)]
#[class(base = Resource, init)]
pub struct OnnxModule {
    path: Mutex<GString>,
    session: Mutex<Option<Session>>,
    base: Base<Resource>,
}

#[godot_api]
impl OnnxModule {
    /// Load ONNX model from Godot path (res:// or user://). Uses ResourceLoader so imported .onnx files (engine import cache) are loaded; falls back to raw file read for user:// or unimported paths.
    #[func]
    pub fn load(&mut self, path: GString) {
        let path_str = path.to_string();
        self.unload();
        let bytes = Self::load_bytes(&path);
        if bytes.is_empty() {
            godot_error!("OnnxModule.load: empty or missing file: {}", path_str);
            return;
        }
        let bytes_slice = bytes.as_slice();
        // ort typically has commit_from_file; use temp file to load from bytes
        let temp_path = std::env::temp_dir().join("godot_onnx_model.onnx");
        if let Ok(mut f) = std::fs::File::create(&temp_path) {
            let _ = f.write_all(bytes_slice);
            drop(f);
            let builder = session_builder();
            match builder.and_then(|b: SessionBuilder| b.commit_from_file(&temp_path)) {
                Ok(session) => {
                    let _ = std::fs::remove_file(&temp_path);
                    *self.path.lock().unwrap() = path;
                    *self.session.lock().unwrap() = Some(session);
                    self.base_mut().notify_property_list_changed();
                    self.base_mut().emit_changed();
                }
                Err(e) => {
                    let _ = std::fs::remove_file(&temp_path);
                    godot_error!("OnnxModule.load: ort error: {}", e);
                }
            }
        } else {
            godot_error!("OnnxModule.load: cannot create temp file");
        }
    }

    /// Unload the current model and release the session.
    #[func]
    pub fn unload(&mut self) {
        *self.session.lock().unwrap() = None;
        *self.path.lock().unwrap() = GString::new();
    }

    /// Returns true if a model is currently loaded.
    #[func]
    pub fn is_loaded(&self) -> bool {
        self.session.lock().unwrap().is_some()
    }

    /// Load model bytes: try ResourceLoader first (imported .onnx from engine cache), then FileAccess.
    fn load_bytes(path: &GString) -> PackedByteArray {
        let mut loader = ResourceLoader::singleton();
        let path_str: String = path.to_string();
        if let Some(res) = loader.load(&path_str) {
            let v = res.to_variant();
            if let Ok(model_data) = Gd::<OnnxModelData>::try_from_variant(&v) {
                let data = model_data.bind().get_data();
                if !data.is_empty() {
                    return data;
                }
            }
        }
        FileAccess::get_file_as_bytes(path)
    }

    /// Run inference. func_name is ignored (ONNX has one graph); args are input tensors in model order.
    #[func]
    pub fn call_module(&mut self, _func_name: GString, args: VarArray) -> VarArray {
        let mut session_guard = self.session.lock().unwrap();
        let Some(session) = session_guard.as_mut() else {
            godot_error!("OnnxModule.call_module: module not loaded");
            return VarArray::new();
        };
        let input_count = session.inputs().len();
        if args.len() != input_count {
            godot_error!(
                "OnnxModule.call_module: expected {} inputs, got {}",
                input_count,
                args.len()
            );
            return VarArray::new();
        }
        let mut input_tensors: Vec<DynValue> = Vec::with_capacity(input_count);
        for i in 0..args.len() {
            let Some(ref arg) = args.get(i) else { continue };
            let Ok(tensor) = Gd::<OnnxTensor>::try_from_variant(arg) else {
                godot_error!("OnnxModule.call_module: arg {} is not an OnnxTensor", i);
                return VarArray::new();
            };
            let tensor = tensor.bind();
            let shape: Vec<usize> = tensor
                .shape_slice()
                .into_iter()
                .map(|d| d as usize)
                .collect();
            let data = tensor.data_slice();
            let arr = match ArrayD::from_shape_vec(shape.clone(), data) {
                Ok(a) => a,
                Err(e) => {
                    godot_error!("OnnxModule: shape/data mismatch: {}", e);
                    return VarArray::new();
                }
            };
            match Tensor::from_array(arr) {
                Ok(v) => input_tensors.push(v.into_dyn()),
                Err(e) => {
                    godot_error!("OnnxModule.call_module: input {} ort error: {}", i, e);
                    return VarArray::new();
                }
            }
        }
        // Build inputs: ort accepts named map. Use ordered inputs by name.
        let input_names: Vec<String> = session
            .inputs()
            .iter()
            .map(|o| o.name().to_string())
            .collect();
        let inputs_map: std::collections::HashMap<String, DynValue> = input_names
            .iter()
            .zip(input_tensors.into_iter())
            .map(|(name, v): (&String, DynValue)| (name.clone(), v))
            .collect();
        let output_names: Vec<String> = session
            .outputs()
            .iter()
            .map(|o| o.name().to_string())
            .collect();
        let run_result = session.run(inputs_map);
        let outputs: SessionOutputs<'_> = match run_result {
            Ok(out) => out,
            Err(e) => {
                godot_error!("OnnxModule.call_module: run error: {}", e);
                return VarArray::new();
            }
        };
        let mut result = VarArray::new();
        for out_name in output_names {
            let out_val: &DynValue = match outputs.get(&out_name) {
                Some(v) => v,
                None => continue,
            };
            let view: ndarray::ArrayViewD<'_, f32> = match out_val.try_extract_array::<f32>() {
                Ok(v) => v,
                Err(_) => continue,
            };
            let shape: Vec<i64> = view.shape().iter().map(|&s| s as i64).collect();
            let data: Vec<f32> = view.iter().copied().collect();
            let out_tensor = OnnxTensor::from_shape_and_data(shape, data);
            result.push(out_tensor.to_variant().to_godot());
        }
        result
    }
}
