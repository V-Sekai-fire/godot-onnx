#!/usr/bin/env python3
# Copyright (c) 2026-present K. S. Ernest (iFire) Lee & godot-onnx contributors
# SPDX-License-Identifier: Apache-2.0 OR MIT
"""Download ONNX models for sample. Run from sample/models/ (writes esrgan.onnx for Rust tests)."""
from __future__ import annotations

import os
import sys
import urllib.request
from pathlib import Path

# Run from sample/models/
SCRIPT_DIR = Path(__file__).resolve().parent
os.chdir(SCRIPT_DIR)

# (url, local_filename) or (url, local_filename, dict) for optional headers
MODELS = [
    # Qualcomm ESRGAN (4× upscale, input [1,3,128,128] NCHW float)
    (
        "https://huggingface.co/qualcomm/ESRGAN/resolve/77059f2407d67ddd813aa5055f61424039ab6154/ESRGAN.onnx",
        "esrgan.onnx",
    ),
]


def download(url: str, dest: Path, headers: dict | None = None) -> None:
    req = urllib.request.Request(url, headers=headers or {})
    with urllib.request.urlopen(req, timeout=120) as resp:
        data = resp.read()
    dest.write_bytes(data)
    size_mb = len(data) / (1024 * 1024)
    print(f"  -> {dest.name} ({size_mb:.2f} MB)")


def main() -> int:
    print("Downloading ONNX models into sample/models/ ...")
    for entry in MODELS:
        url = entry[0]
        name = entry[1]
        opts = entry[2] if len(entry) > 2 else {}
        dest = SCRIPT_DIR / name
        if dest.exists():
            print(f"Skip (exists): {name}")
            continue
        print(f"Fetching {name} ...")
        try:
            download(url, dest, opts.get("headers"))
        except Exception as e:
            print(f"  Error: {e}", file=sys.stderr)
            return 1

    print("Done.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
