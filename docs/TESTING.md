# Testing

Both **Rust** and **Godot** tests run the same ONNX path (identity, matmul, and optionally the scene models). Use both to validate the extension.

## Rust tests (no Godot)

```bash
cargo test
```

From the repo root. Requires generated models in `sample/models/` (identity, matmul, benchmark are created by CI; see [Usage](USAGE.md#test-models-identity--matmul)).

**What runs:**

- **identity_preserves_input** — For any `[f32; 3]`, identity model output equals input.
- **matmul_matches_reference** — For random 2×3 and 3×2 matrices, matmul output matches a reference within tolerance.
- **scene_esrgan_runs** — Loads `esrgan.onnx`, runs one inference (NCHW [1,3,128,128] float). Skip if file missing.
- **scene_esrgan_output_shape_and_finite** — ESRGAN: asserts output shape [1,3,512,512] and all values finite. Skip if model missing.
- **scene_esrgan_with_photo** — ESRGAN with preprocessed photo (baboon.png → `esrgan_input.raw`): same assertions. Generate input: `python sample/models/generate_esrgan_input.py`. Skip if model or raw missing.
- **scene_esrgan_save_png** — Runs ESRGAN with photo input and writes the upscaled image to **`sample/models/esrgan_result.png`** (512×512 RGB). Run `cargo test scene_esrgan_save_png` to refresh the PNG.

Scene models (ESRGAN) are optional (download with `sample/models/download_onnx_models.py`). More proptest cases:

```bash
PROPTEST_CASES=500 cargo test
```

## Godot tests (headless)

Run the sample project’s inference test script (same API as the scenes, no UI):

```bash
godot --path sample --headless -s res://scripts/test_inference.gd
```

**Exit code:** 0 = all run tests passed, 1 = failure (e.g. identity/matmul missing or inference error).

**What runs:**

- **identity** — Loads `res://models/identity.onnx`, runs [1,2,3], asserts output [1,2,3].
- **matmul** — Loads `res://models/matmul.onnx`, runs 2×3 @ 3×2, asserts output [22,28,49,64].
- **Scene models** — If present: load esrgan, pose_detection, pose_detection_thunder, mediapipe_pose_landmark_full; run one inference with dummy input; assert at least one output. Missing files are **skipped**.

Requires the extension built and present in `sample/addons/godot-onnx/`. Generate identity/matmul/benchmark with the Python scripts in `sample/models/` (see [Usage](USAGE.md)).

## Generating inference images (Godot)

To compare Godot output with Rust (same models, different runtime):

- **In editor:** Run each scene (main, pose_detection, pose_detection_thunder, mediapipe_pose_landmark_full), then click **Save image**. Files go to `user://inference_output/`.
- **Headless (all scenes):**  
  `godot --path sample --headless -s res://scripts/export_inference_images.gd`  
  Writes `esrgan.png`, `pose_detection.png`, `pose_detection_thunder.png`, `mediapipe_pose_landmark_full.png` to `user://inference_output/`.

See [Scenes](SCENES.md#generating-inference-images-validate-vs-rust-tests).

## CI

- **Rust:** `cargo test` runs in the desktop build job (after generating identity/matmul/benchmark). Scene model tests skip if those ONNX files are not present.
- **Godot:** On Ubuntu, the workflow installs Godot 4.2, runs **Godot inference tests** (`res://scripts/test_inference.gd`); the job fails if the script exits non-zero (e.g. identity/matmul missing or inference error). Then it optionally downloads scene models, runs the export script, and uploads inference images as an artifact.
