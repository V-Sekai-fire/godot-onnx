#!/usr/bin/env python3
# Copyright (c) 2026-present K. S. Ernest (iFire) Lee & godot-onnx contributors
# SPDX-License-Identifier: Apache-2.0 OR MIT
"""Generate esrgan_input.raw from baboon.png for Rust ESRGAN photo test.
Run from repo root: python sample/models/generate_esrgan_input.py
Requires: pip install Pillow
Output: [1,3,128,128] NCHW float32 LE, normalized 0-1.
"""
from __future__ import annotations

import struct
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent.parent
ESRGAN_IMAGE = REPO_ROOT / "sample" / "assets" / "example_images" / "baboon.png"
OUT_RAW = Path(__file__).resolve().parent / "esrgan_input.raw"
H, W = 128, 128


def main() -> int:
    try:
        from PIL import Image
    except ImportError:
        print("pip install Pillow", file=sys.stderr)
        return 1
    if not ESRGAN_IMAGE.exists():
        print(f"Image not found: {ESRGAN_IMAGE}", file=sys.stderr)
        return 1
    img = Image.open(ESRGAN_IMAGE)
    img = img.convert("RGB")
    img = img.resize((W, H), Image.Resampling.BILINEAR)
    pixels = list(img.getdata())
    # NCHW: channel 0 (R), then channel 1 (G), then channel 2 (B), each 0-1 float
    with open(OUT_RAW, "wb") as f:
        for c in (0, 1, 2):
            for (r, g, b) in pixels:
                v = [r, g, b][c] / 255.0
                f.write(struct.pack("<f", v))
    print(f"Wrote {OUT_RAW} ({OUT_RAW.stat().st_size} bytes)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
