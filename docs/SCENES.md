# Sample scenes (match iree.gd layout)

Scenes mirror [iree.gd sample/scenes](https://github.com/iree-gd/iree.gd/tree/main/sample/scenes). Each scene uses **OnnxModule** and **OnnxTensor** (same API shape as IREEModule/IREETensor). Place the required ONNX model in `sample/models/` and assign any texture in the editor where needed.

| Scene | Model path | Input | Output | Source |
|-------|------------|--------|--------|--------|
| **scenes/main** | `res://models/esrgan.onnx` | [1,50,50,3] float | [1,200,200,3] float | ESRGAN-style 4× upscale (50×50 → 200×200 patch) |
| **scenes/pose_detection** | `res://models/pose_detection.onnx` | [1,256,256,3] | 17×3 keypoints (y,x,conf) | Single pose (e.g. MoveNet, MediaPipe Lite) |
| **scenes/pose_detection_thunder** | `res://models/pose_detection_thunder.onnx` | [1,256,256,3] | 17×3 keypoints | Thunder variant |
| **scenes/mediapipe_pose_landmark_full** | `res://models/mediapipe_pose_landmark_full.onnx` | [1,256,256,3] | 3 outputs: landmarks, …, depth | Full-body 33 points + depth |

## Running

- **Console demo:** run **res://main.tscn** (identity, matmul, benchmark; no extra models beyond generated ones).
- **Main (ESRGAN):** run **res://scenes/main/main.tscn**. Assign a texture to the TextureRect (e.g. put an image in `sample/assets/example_images/` and reference it). Download or export an ESRGAN ONNX model to `sample/models/esrgan.onnx`.
- **Pose / MediaPipe:** run the scene, assign a texture (e.g. person image). Download the matching ONNX model to `sample/models/`.

## Where to get ONNX models

**Recommended:** run the download script (requires Python 3, no extra deps):

```bash
cd sample/models
python download_onnx_models.py
```

This fetches **esrgan.onnx** only (Qualcomm ESRGAN, 128×128 input). Rust tests use this model.

For pose and MediaPipe scenes, obtain the ONNX files from other sources:

- **pose_detection.onnx** — Xenova MoveNet SinglePose Lightning (Hugging Face: `Xenova/movenet-singlepose-lightning`).
- **pose_detection_thunder.onnx** — Xenova MoveNet SinglePose Thunder (Hugging Face: `Xenova/movenet-singlepose-thunder`).
- **mediapipe_pose_landmark_full.onnx** — Qualcomm MediaPipe Pose Landmark (from their ONNX float zip on qaihub-public-assets).

Other sources:

- [ONNX Model Zoo (Hugging Face)](https://huggingface.co/onnxmodelzoo) — vision, classification, etc.
- [ONNX Model Zoo (GitHub)](https://github.com/onnx/models) — validated models.
- MediaPipe / MoveNet: export from TensorFlow or use community ONNX conversions; ensure input [1,256,256,3] and output shapes match the scripts.

## Generating inference images (validate vs Rust tests)

Each scene can export its inference result to **user://inference_output/** so you can compare with Rust test output or keep reference images.

- **In editor:** Run the scene, then click **Save image**. Files written: `esrgan.png`, `pose_detection.png`, `pose_detection_thunder.png`, `mediapipe_pose_landmark_full.png`.
- **Headless (all scenes):** From repo root, with models and assets in place:
  ```bash
  godot --path sample --headless -s res://scripts/export_inference_images.gd
  ```
  Exits after writing all four images to `user://inference_output/`.

Rust tests in `src/prop_tests.rs` run identity, matmul, and ESRGAN; the Godot sample runs the full pipeline (texture → tensor → OnnxModule → image) for all scene models. Use the exported images to visually confirm the addon matches the Godot runtime.
