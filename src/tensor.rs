// Copyright (c) 2026-present K. S. Ernest (iFire) Lee & godot-onnx contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT
// ONNXTensor: RefCounted type holding tensor data for Godot ↔ ort.
// API matches IREETensor: from_bytes, from_float32s, get_data, get_dimension.

use godot::builtin::{PackedByteArray, PackedFloat32Array, PackedInt64Array};
use godot::classes::RefCounted;
use godot::obj::NewGd;
use godot::prelude::*;
use std::sync::Mutex;

/// RefCounted type holding tensor data for Godot and ONNX Runtime. Create with [from_float32s] or [from_bytes], pass as inputs to [OnnxModule::call_module], and read outputs with [get_data] and [get_dimension].
#[derive(GodotClass)]
#[class(base = RefCounted, init)]
pub struct OnnxTensor {
    /// Stored as f32 for inference; from_bytes interprets bytes as f32 (little-endian).
    data: Mutex<Vec<f32>>,
    dimension: Mutex<Vec<i64>>,
    base: Base<RefCounted>,
}

fn packed_i64_to_vec(arr: &PackedInt64Array) -> Vec<i64> {
    (0..arr.len()).map(|i| arr.get(i).unwrap_or(0)).collect()
}

#[godot_api]
impl OnnxTensor {
    /// Create tensor from raw bytes (interpreted as f32, little-endian).
    #[func]
    pub fn from_bytes(bytes: PackedByteArray, dimension: PackedInt64Array) -> Gd<Self> {
        let dim = packed_i64_to_vec(&dimension);
        let n: i64 = dim.iter().product();
        let expected_len = (n * 4) as usize; // f32 = 4 bytes
        if bytes.len() != expected_len {
            godot_error!(
                "OnnxTensor.from_bytes: bytes len {} != expected {}",
                bytes.len(),
                expected_len
            );
            return OnnxTensor::new_gd();
        }
        let mut data = Vec::with_capacity(n as usize);
        for chunk in bytes.as_slice().chunks_exact(4) {
            data.push(f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]));
        }
        let mut out = OnnxTensor::new_gd();
        out.bind_mut().set_data_and_dimension(data, dim);
        out
    }

    /// Create tensor from float32 array (matches IREETensor.from_float32s).
    #[func]
    pub fn from_float32s(float32s: PackedFloat32Array, dimension: PackedInt64Array) -> Gd<Self> {
        let dim = packed_i64_to_vec(&dimension);
        let n: i64 = dim.iter().product();
        let slice = float32s.as_slice();
        if slice.len() != n as usize {
            godot_error!(
                "OnnxTensor.from_float32s: data len {} != dimension product {}",
                slice.len(),
                n
            );
            return OnnxTensor::new_gd();
        }
        let data = slice.to_vec();
        let mut out = OnnxTensor::new_gd();
        out.bind_mut().set_data_and_dimension(data, dim);
        out
    }

    fn set_data_and_dimension(&mut self, data: Vec<f32>, dimension: Vec<i64>) {
        *self.data.lock().unwrap() = data;
        *self.dimension.lock().unwrap() = dimension;
    }

    /// Raw bytes (f32 little-endian), matches IREETensor.get_data().
    #[func]
    pub fn get_data(&self) -> PackedByteArray {
        let data = self.data.lock().unwrap();
        let mut out = PackedByteArray::new();
        out.resize(data.len() * 4);
        let slice = out.as_mut_slice();
        for (i, &v) in data.iter().enumerate() {
            let bytes = v.to_le_bytes();
            slice[i * 4..(i + 1) * 4].copy_from_slice(&bytes);
        }
        out
    }

    /// Shape as array of integers (matches IREETensor.get_dimension).
    #[func]
    pub fn get_dimension(&self) -> PackedInt64Array {
        let dim = self.dimension.lock().unwrap();
        let mut out = PackedInt64Array::new();
        out.resize(dim.len());
        out.as_mut_slice().copy_from_slice(&dim);
        out
    }

    /// Returns true if the tensor holds data (non-empty).
    #[func]
    pub fn is_captured(&self) -> bool {
        !self.data.lock().unwrap().is_empty()
    }

    /// Internal: get shape as slice for ort.
    pub fn shape_slice(&self) -> Vec<i64> {
        self.dimension.lock().unwrap().clone()
    }

    /// Internal: get data as slice for ort.
    pub fn data_slice(&self) -> Vec<f32> {
        self.data.lock().unwrap().clone()
    }

    /// Internal: create OnnxTensor from ort output (shape + f32 data).
    pub fn from_shape_and_data(shape: Vec<i64>, data: Vec<f32>) -> Gd<Self> {
        let mut out = OnnxTensor::new_gd();
        out.bind_mut().set_data_and_dimension(data, shape);
        out
    }
}
