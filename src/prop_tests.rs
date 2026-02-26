//! Copyright (c) 2026-present K. S. Ernest (iFire) Lee & godot-onnx contributors
//! SPDX-License-Identifier: Apache-2.0 OR MIT
//! Property tests for ONNX inference (identity and matmul) using proptest.
//! Runs ort directly; no Godot runtime required. Enable with `cargo test`.

#![cfg(test)]

use ndarray::{Array1, Array2};
use ort::session::builder::SessionBuilder;
use ort::session::Session;
use ort::value::Tensor;
use proptest::prelude::*;
use std::collections::HashMap;
use std::path::Path;

const IDENTITY_PATH: &str = "sample/models/identity.onnx";
const MATMUL_PATH: &str = "sample/models/matmul.onnx";

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
