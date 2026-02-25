#!/bin/bash
# Copyright (c) 2026-present K. S. Ernest (iFire) Lee & godot-onnx contributors
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Build script for Godot ONNX GDExtension (Linux/macOS)
# Copies the Rust library to sample/addons/godot-onnx/

set -e

case "$(uname -s)" in
    Linux)   LIB_EXT="so" ;;
    Darwin)  LIB_EXT="dylib" ;;
    *)       echo "Unsupported platform"; exit 1 ;;
esac

mkdir -p sample/addons/godot-onnx

# Float (default) build
echo "Building Godot ONNX GDExtension (float)..."
cargo build --release
LIB_NAME="libgodot_onnx.$LIB_EXT"
TARGET_PATH="target/release/$LIB_NAME"
[ -f "$TARGET_PATH" ] || TARGET_PATH="target/release/godot_onnx.$LIB_EXT"
if [ -f "$TARGET_PATH" ]; then
    cp "$TARGET_PATH" "sample/addons/godot-onnx/$LIB_NAME"
    if [ "$LIB_EXT" = "dylib" ]; then
        echo "Code signing GDExtension library for macOS..."
        codesign --force --sign - "sample/addons/godot-onnx/$LIB_NAME"
    fi
    echo "Copied to sample/addons/godot-onnx/$LIB_NAME"
else
    echo "Error: Built library not found at $TARGET_PATH"
    exit 1
fi

# Doubles build (for Godot with precision=double). Requires api-custom or api-custom-json; skip if not set up.
echo "Building Godot ONNX GDExtension (doubles)..."
LIB_NAME_DOUBLES="libgodot_onnx_doubles.$LIB_EXT"
if cargo build --release --features double-precision 2>/dev/null; then
    if [ -f "target/release/$LIB_NAME" ]; then
        cp "target/release/$LIB_NAME" "sample/addons/godot-onnx/$LIB_NAME_DOUBLES"
    elif [ -f "target/release/godot_onnx.$LIB_EXT" ]; then
        cp "target/release/godot_onnx.$LIB_EXT" "sample/addons/godot-onnx/$LIB_NAME_DOUBLES"
    fi
    if [ -f "sample/addons/godot-onnx/$LIB_NAME_DOUBLES" ]; then
        [ "$LIB_EXT" = "dylib" ] && codesign --force --sign - "sample/addons/godot-onnx/$LIB_NAME_DOUBLES"
        echo "Copied to sample/addons/godot-onnx/$LIB_NAME_DOUBLES"
    fi
else
    echo "Warning: Doubles build skipped (needs GODOT4_BIN or GODOT4_GDEXTENSION_JSON for gdext). Float build is ready."
fi
