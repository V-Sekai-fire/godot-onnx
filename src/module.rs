// Copyright (c) 2026-present K. S. Ernest (iFire) Lee & godot-onnx contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT
// ONNXModule: Resource that loads an ONNX model and runs inference via ort.
// API matches IREEModule: load(path), unload(), call_module(func_name, args) -> Array of ONNXTensor.
// On wasm32 we use ort-web; load() is async (spawns future), call_module requires run_async (use call_module_async or stub).

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
use std::sync::{Arc, Mutex};

use crate::model_data::OnnxModelData;
use crate::tensor::OnnxTensor;

/// Shared session store so wasm32 async load can set the session from a spawned future.
#[derive(Clone)]
pub struct SessionStore(Arc<Mutex<Option<Session>>>);

impl Default for SessionStore {
    fn default() -> Self {
        Self(Arc::new(Mutex::new(None)))
    }
}

/// Build session with platform-appropriate execution providers: CoreML on macOS; WebGPU on Linux/Windows; CPU only on Android (Dawn built separately if needed).
fn session_builder() -> ort::Result<SessionBuilder> {
    #[cfg(target_os = "macos")]
    {
        let coreml_ep: ExecutionProviderDispatch = ep::CoreML::default().into();
        let cpu_ep: ExecutionProviderDispatch = ep::CPU::default().into();
        Session::builder()
            .and_then(|b: SessionBuilder| b.with_execution_providers([coreml_ep, cpu_ep]))
    }

    #[cfg(target_os = "android")]
    {
        let cpu_ep: ExecutionProviderDispatch = ep::CPU::default().into();
        Session::builder()
            .and_then(|b: SessionBuilder| b.with_execution_providers([cpu_ep]))
    }

    #[cfg(any(target_os = "linux", target_os = "windows"))]
    {
        let webgpu_ep: ExecutionProviderDispatch = ep::WebGPU::default().into();
        let cpu_ep: ExecutionProviderDispatch = ep::CPU::default().into();
        Session::builder()
            .and_then(|b: SessionBuilder| b.with_execution_providers([webgpu_ep, cpu_ep]))
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows", target_os = "android")))]
    {
        let cpu_ep: ExecutionProviderDispatch = ep::CPU::default().into();
        Session::builder()
            .and_then(|b: SessionBuilder| b.with_execution_providers([cpu_ep]))
    }
}

/// Resource that loads an ONNX model from a Godot path (`res://` or `user://`) and runs inference via ONNX Runtime.
/// Create with `OnnxModule.new()`, then call [load] to load a model and [call_module] to run inference.
#[derive(GodotClass)]
#[class(base = Resource, init)]
pub struct OnnxModule {
    path: Mutex<GString>,
    session: SessionStore,
    base: Base<Resource>,
}

#[godot_api]
impl OnnxModule {
    /// Load ONNX model from Godot path (res:// or user://). Uses ResourceLoader so imported .onnx files (engine import cache) are loaded; falls back to raw file read for user:// or unimported paths.
    /// On web (wasm32) this starts an async load; poll [is_loaded] or wait a frame for the session to be ready.
    #[func]
    pub fn load(&mut self, path: GString) {
        let path_str = path.to_string();
        self.unload();
        let bytes = Self::load_bytes(&path);
        if bytes.is_empty() {
            godot_error!("OnnxModule.load: empty or missing file: {}", path_str);
            return;
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let bytes_slice = bytes.as_slice();
            let temp_path = std::env::temp_dir().join("godot_onnx_model.onnx");
            if let Ok(mut f) = std::fs::File::create(&temp_path) {
                let _ = f.write_all(bytes_slice);
                drop(f);
                let builder = session_builder();
                match builder.and_then(|b: SessionBuilder| b.commit_from_file(&temp_path)) {
                    Ok(session) => {
                        let _ = std::fs::remove_file(&temp_path);
                        *self.path.lock().unwrap() = path;
                        *self.session.0.lock().unwrap() = Some(session);
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

        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen_futures::spawn_local;
            let path_clone = path.clone();
            let bytes_vec: Vec<u8> = bytes.as_slice().to_vec();
            let session_store = self.session.clone();
            let path_for_emit = path.clone();
            spawn_local(async move {
                let api = match ort_web::api(ort_web::FEATURE_WEBGPU).await {
                    Ok(a) => a,
                    Err(e) => {
                        godot_error!("OnnxModule.load (web): ort-web init failed: {}", e);
                        return;
                    }
                };
                ort::set_api(api);
                let builder = match session_builder() {
                    Ok(b) => b,
                    Err(e) => {
                        godot_error!("OnnxModule.load (web): session_builder failed: {}", e);
                        return;
                    }
                };
                let session = match builder.commit_from_memory(&bytes_vec).await {
                    Ok(s) => s,
                    Err(e) => {
                        godot_error!("OnnxModule.load (web): commit_from_memory failed: {}", e);
                        return;
                    }
                };
                *session_store.0.lock().unwrap() = Some(session);
                // Note: we cannot call notify_property_list_changed/emit_changed from here without
                // a reference to self; the script can poll is_loaded() or wait a frame.
            });
            *self.path.lock().unwrap() = path_clone;
        }
    }

    /// Unload the current model and release the session.
    #[func]
    pub fn unload(&mut self) {
        *self.session.0.lock().unwrap() = None;
        *self.path.lock().unwrap() = GString::new();
    }

    /// Returns true if a model is currently loaded.
    #[func]
    pub fn is_loaded(&self) -> bool {
        self.session.0.lock().unwrap().is_some()
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
    /// On web (wasm32) only run_async is available; call_module returns empty — use call_module_async when implemented.
    #[func]
    pub fn call_module(&mut self, _func_name: GString, args: VarArray) -> VarArray {
        let mut session_guard = self.session.0.lock().unwrap();
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

        #[cfg(target_arch = "wasm32")]
        {
            godot_error!("OnnxModule.call_module: on web use call_module_async (sync run not supported)");
            return VarArray::new();
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
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
}
