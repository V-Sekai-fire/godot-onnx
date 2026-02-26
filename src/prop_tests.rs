//! Copyright (c) 2026-present K. S. Ernest (iFire) Lee & godot-onnx contributors
//! SPDX-License-Identifier: Apache-2.0 OR MIT
//! Property tests for ONNX inference (identity and matmul) using proptest.
//! Runs ort directly; no Godot runtime required. Enable with `cargo test`.

#![cfg(test)]

use ndarray::{Array1, Array2, ArrayD, IxDyn};
use ort::session::builder::SessionBuilder;
use ort::session::Session;
use ort::value::Tensor;
use proptest::prelude::*;
use std::collections::HashMap;
use std::path::Path;

const IDENTITY_PATH: &str = "sample/models/identity.onnx";
const MATMUL_PATH: &str = "sample/models/matmul.onnx";

// Scene models (iree.gd equivalents; download via sample/models/download_onnx_models.py)
const ESRGAN_PATH: &str = "sample/models/esrgan.onnx";
// Preprocessed ESRGAN input: [1,3,128,128] NCHW f32 LE from baboon.png (generate with sample/models/generate_esrgan_input.py)
const ESRGAN_INPUT_RAW_PATH: &str = "sample/models/esrgan_input.raw";
const ESRGAN_RESULT_PNG_PATH: &str = "sample/models/esrgan_result.png";
const ESRGAN_INPUT_H: usize = 128;
const ESRGAN_INPUT_W: usize = 128;
const ESRGAN_OUTPUT_H: usize = 512;
const ESRGAN_OUTPUT_W: usize = 512;

fn identity_session() -> Option<Session> {
    if !Path::new(IDENTITY_PATH).exists() {
        return None;
    }
    Session::builder()
        .and_then(|b: SessionBuilder| b.commit_from_file(IDENTITY_PATH))
        .ok()
}

fn matmul_session() -> Option<Session> {
    if !Path::new(MATMUL_PATH).exists() {
        return None;
    }
    Session::builder()
        .and_then(|b: SessionBuilder| b.commit_from_file(MATMUL_PATH))
        .ok()
}

/// Load model from path, run one inference. `dims` = input shape; `use_f32` = true for float, false for int32.
fn run_scene_model(path: &str, dims: &[usize], use_f32: bool) -> Result<(), Box<dyn std::error::Error>> {
    let _ = run_scene_model_with_output(path, dims, use_f32)?;
    Ok(())
}

/// Like run_scene_model but returns the first output shape and f32 data (for assertion and saving output).
fn run_scene_model_with_output(
    path: &str,
    dims: &[usize],
    use_f32: bool,
) -> Result<(Vec<usize>, Vec<f32>), Box<dyn std::error::Error>> {
    if !Path::new(path).exists() {
        return Err("model file not found".into());
    }
    let mut session = Session::builder()?
        .commit_from_file(path)?;
    let input_name = session
        .inputs()
        .first()
        .ok_or("no inputs")?
        .name()
        .to_string();
    let first_output_name = session
        .outputs()
        .first()
        .ok_or("no outputs")?
        .name()
        .to_string();
    let tensor = if use_f32 {
        let arr = ArrayD::<f32>::zeros(IxDyn(dims));
        Tensor::from_array(arr)?.into_dyn()
    } else {
        let arr = ArrayD::<i32>::zeros(IxDyn(dims));
        Tensor::from_array(arr)?.into_dyn()
    };
    let mut feed = HashMap::new();
    feed.insert(input_name, tensor);
    let outputs = session.run(feed)?;
    let out_val = outputs
        .get(&first_output_name)
        .ok_or("missing first output")?;
    let view = out_val.try_extract_array::<f32>()?;
    let shape: Vec<usize> = view.shape().iter().copied().collect();
    let data: Vec<f32> = view.iter().copied().collect();
    Ok((shape, data))
}

/// Run model with custom float32 input (e.g. ESRGAN from photo); returns first output shape and f32 data.
fn run_scene_model_with_float_input(
    path: &str,
    input_arr: ArrayD<f32>,
) -> Result<(Vec<usize>, Vec<f32>), Box<dyn std::error::Error>> {
    if !Path::new(path).exists() {
        return Err("model file not found".into());
    }
    let mut session = Session::builder()?
        .commit_from_file(path)?;
    let input_name = session
        .inputs()
        .first()
        .ok_or("no inputs")?
        .name()
        .to_string();
    let first_output_name = session
        .outputs()
        .first()
        .ok_or("no outputs")?
        .name()
        .to_string();
    let tensor = Tensor::from_array(input_arr)?.into_dyn();
    let mut feed = HashMap::new();
    feed.insert(input_name, tensor);
    let outputs = session.run(feed)?;
    let out_val = outputs
        .get(&first_output_name)
        .ok_or("missing first output")?;
    let view = out_val.try_extract_array::<f32>()?;
    let shape: Vec<usize> = view.shape().iter().copied().collect();
    let data: Vec<f32> = view.iter().copied().collect();
    Ok((shape, data))
}

/// Load preprocessed ESRGAN input from raw file: [1,3,128,128] NCHW f32 LE (normalized 0–1).
fn load_esrgan_input_raw(path: &str) -> Result<ArrayD<f32>, Box<dyn std::error::Error>> {
    let buf = std::fs::read(Path::new(path))?;
    let n = 1 * 3 * ESRGAN_INPUT_H * ESRGAN_INPUT_W;
    if buf.len() != n * 4 {
        return Err(format!("esrgan input raw: expected {} bytes, got {}", n * 4, buf.len()).into());
    }
    let mut data = Vec::with_capacity(n);
    for chunk in buf.chunks_exact(4) {
        data.push(f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]));
    }
    let arr = ArrayD::from_shape_vec(
        IxDyn(&[1, 3, ESRGAN_INPUT_H, ESRGAN_INPUT_W]),
        data,
    )?;
    Ok(arr)
}

fn run_identity(session: &mut Session, x: [f32; 3]) -> Result<[f32; 3], Box<dyn std::error::Error>> {
    let arr = Array1::from_vec(x.to_vec());
    let input = Tensor::from_array(arr)?.into_dyn();
    let mut inputs = HashMap::new();
    inputs.insert("x".to_string(), input);
    let outputs = session.run(inputs)?;
    let out = outputs
        .get("y")
        .ok_or("missing output 'y'")?
        .try_extract_array::<f32>()?;
    let v: Vec<f32> = out.iter().copied().collect();
    Ok([v[0], v[1], v[2]])
}

fn run_matmul(
    session: &mut Session,
    a: [[f32; 3]; 2],
    b: [[f32; 2]; 3],
) -> Result<[[f32; 2]; 2], Box<dyn std::error::Error>> {
    let a_flat: Vec<f32> = a.iter().flat_map(|r| r.iter().copied()).collect();
    let b_flat: Vec<f32> = b.iter().flat_map(|r| r.iter().copied()).collect();
    let arr_a = Array2::from_shape_vec((2, 3), a_flat)?;
    let arr_b = Array2::from_shape_vec((3, 2), b_flat)?;
    let input_a = Tensor::from_array(arr_a)?.into_dyn();
    let input_b = Tensor::from_array(arr_b)?.into_dyn();
    let mut inputs = HashMap::new();
    inputs.insert("A".to_string(), input_a);
    inputs.insert("B".to_string(), input_b);
    let outputs = session.run(inputs)?;
    let out = outputs
        .get("Y")
        .ok_or("missing output 'Y'")?
        .try_extract_array::<f32>()?;
    let v: Vec<f32> = out.iter().copied().collect();
    Ok([[v[0], v[1]], [v[2], v[3]]])
}

fn ref_matmul(a: [[f32; 3]; 2], b: [[f32; 2]; 3]) -> [[f32; 2]; 2] {
    let mut c = [[0.0f32; 2]; 2];
    for i in 0..2 {
        for j in 0..2 {
            for k in 0..3 {
                c[i][j] += a[i][k] * b[k][j];
            }
        }
    }
    c
}

/// Compare two f32 for "semantic" equality: same value, or both NaN, or both same-sign inf. 0.0 and -0.0 are equal.
fn float_eq(a: f32, b: f32) -> bool {
    if a == b {
        return true;
    }
    if a.is_nan() && b.is_nan() {
        return true;
    }
    if a.is_infinite() && b.is_infinite() && a.signum() == b.signum() {
        return true;
    }
    false
}

/// Assert two f32s are equal (including NaN == NaN, 0 == -0, inf == same inf).
fn assert_float_eq(actual: f32, expected: f32) -> Result<(), TestCaseError> {
    if float_eq(actual, expected) {
        Ok(())
    } else {
        Err(TestCaseError::fail(format!(
            "got {} expected {}",
            actual, expected
        )))
    }
}

/// Assert two f32s are close: both non-finite in a compatible way, or within absolute tol, or within relative tol.
fn assert_float_close(actual: f32, expected: f32, abs_tol: f32, rel_tol: f32) -> Result<(), TestCaseError> {
    if float_eq(actual, expected) {
        return Ok(());
    }
    if !actual.is_finite() || !expected.is_finite() {
        return Err(TestCaseError::fail(format!(
            "got {} expected {} (one or both non-finite)",
            actual, expected
        )));
    }
    let diff = (actual - expected).abs();
    if diff < abs_tol {
        return Ok(());
    }
    let scale = expected.abs().max(actual.abs()).max(1e-6);
    if diff / scale < rel_tol {
        return Ok(());
    }
    Err(TestCaseError::fail(format!(
        "got {} expected {} (abs_tol {} rel_tol {})",
        actual, expected, abs_tol, rel_tol
    )))
}

fn identity_input() -> impl Strategy<Value = [f32; 3]> {
    prop::array::uniform3(prop::num::f32::ANY)
}

fn matmul_inputs() -> impl Strategy<Value = ([[f32; 3]; 2], [[f32; 2]; 3])> {
    (
        prop::array::uniform2(prop::array::uniform3(prop::num::f32::NORMAL)),
        prop::array::uniform3(prop::array::uniform2(prop::num::f32::NORMAL)),
    )
}

proptest! {
    #[test]
    fn identity_preserves_input(x in identity_input()) {
        let Some(mut session) = identity_session() else {
            return Ok(());
        };
        let out = run_identity(&mut session, x).map_err(|e| TestCaseError::fail(e.to_string()))?;
        for i in 0..3 {
            assert_float_eq(out[i], x[i])?;
        }
    }

    #[test]
    fn matmul_matches_reference(inputs in matmul_inputs()) {
        let Some(mut session) = matmul_session() else {
            return Ok(());
        };
        let (a, b) = inputs;
        let expected = ref_matmul(a, b);
        let out = run_matmul(&mut session, a, b).map_err(|e| TestCaseError::fail(e.to_string()))?;
        const ABS_TOL: f32 = 1e-4;
        const REL_TOL: f32 = 1e-4;
        for i in 0..2 {
            for j in 0..2 {
                assert_float_close(out[i][j], expected[i][j], ABS_TOL, REL_TOL)?;
            }
        }
    }
}

// --- Scene model tests (iree.gd equivalents); skip if model not downloaded ---

#[test]
fn scene_esrgan_runs() {
    // Qualcomm ESRGAN: NCHW input [1, 3, 128, 128]
    let r = run_scene_model(ESRGAN_PATH, &[1, 3, 128, 128], true);
    if let Err(e) = &r {
        if e.to_string().contains("not found") {
            return;
        }
    }
    r.expect("esrgan inference");
}

#[test]
fn scene_esrgan_output_shape_and_finite() {
    // Qualcomm ESRGAN: input [1,3,128,128] -> output [1,3,512,512] (4× upscale), all finite
    let Ok((shape, data)) = run_scene_model_with_output(ESRGAN_PATH, &[1, 3, 128, 128], true) else {
        return; // skip if model missing
    };
    assert_eq!(shape.len(), 4, "ESRGAN output should be 4D [N,C,H,W]");
    assert_eq!(shape[0], 1);
    assert_eq!(shape[1], 3);
    assert_eq!(shape[2], 512, "4×128 = 512");
    assert_eq!(shape[3], 512);
    assert_eq!(data.len(), 1 * 3 * 512 * 512);
    for (i, &v) in data.iter().enumerate() {
        assert!(v.is_finite(), "ESRGAN output[{}] = {} not finite", i, v);
    }
}

#[test]
fn scene_esrgan_with_photo() {
    // ESRGAN with preprocessed photo (baboon.png → esrgan_input.raw): output shape and all finite.
    // Generate input: python sample/models/generate_esrgan_input.py
    let input_img = match load_esrgan_input_raw(ESRGAN_INPUT_RAW_PATH) {
        Ok(a) => a,
        Err(_) => return, // skip if raw missing (run generate_esrgan_input.py from repo root)
    };
    let Ok((shape, data)) = run_scene_model_with_float_input(ESRGAN_PATH, input_img) else {
        return; // skip if model missing
    };
    assert_eq!(shape.len(), 4, "ESRGAN output should be 4D [N,C,H,W]");
    assert_eq!(shape[0], 1);
    assert_eq!(shape[1], 3);
    assert_eq!(shape[2], 512, "4×128 = 512");
    assert_eq!(shape[3], 512);
    assert_eq!(data.len(), 1 * 3 * 512 * 512);
    for (i, &v) in data.iter().enumerate() {
        assert!(v.is_finite(), "ESRGAN output[{}] = {} not finite", i, v);
    }
}

/// Save ESRGAN output [1,3,H,W] f32 (NCHW) to PNG; values clamped to 0..1 then scaled to 0..255.
fn save_esrgan_output_as_png(
    data: &[f32],
    out_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let (h, w) = (ESRGAN_OUTPUT_H, ESRGAN_OUTPUT_W);
    assert_eq!(data.len(), 1 * 3 * h * w);
    let mut img = image::RgbImage::new(w as u32, h as u32);
    for y in 0..h {
        for x in 0..w {
            let r = (data[0 * h * w + y * w + x].clamp(0.0, 1.0) * 255.0).round() as u8;
            let g = (data[1 * h * w + y * w + x].clamp(0.0, 1.0) * 255.0).round() as u8;
            let b = (data[2 * h * w + y * w + x].clamp(0.0, 1.0) * 255.0).round() as u8;
            img.put_pixel(x as u32, y as u32, image::Rgb([r, g, b]));
        }
    }
    img.save(Path::new(out_path))?;
    Ok(())
}

#[test]
fn scene_esrgan_save_png() {
    // Run ESRGAN with photo input and save upscaled result as sample/models/esrgan_result.png
    let input_img = match load_esrgan_input_raw(ESRGAN_INPUT_RAW_PATH) {
        Ok(a) => a,
        Err(_) => return, // skip if raw missing
    };
    let Ok((shape, data)) = run_scene_model_with_float_input(ESRGAN_PATH, input_img) else {
        return; // skip if model missing
    };
    assert_eq!(shape.len(), 4);
    assert_eq!(shape[0], 1);
    assert_eq!(shape[1], 3);
    assert_eq!(shape[2], ESRGAN_OUTPUT_H);
    assert_eq!(shape[3], ESRGAN_OUTPUT_W);
    if let Err(e) = save_esrgan_output_as_png(&data, ESRGAN_RESULT_PNG_PATH) {
        panic!("failed to save ESRGAN result PNG: {}", e);
    }
    println!("Saved {}", ESRGAN_RESULT_PNG_PATH);
}
