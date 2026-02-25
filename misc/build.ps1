# Copyright (c) 2026-present K. S. Ernest (iFire) Lee & godot-onnx contributors
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Build script for Godot ONNX GDExtension (Windows)
# Copies the Rust library to sample/addons/godot-onnx/

$ErrorActionPreference = "Stop"
$libName = "godot_onnx"
$ext = "dll"
$destDir = "sample\addons\godot-onnx"
if (-not (Test-Path $destDir)) { New-Item -ItemType Directory -Path $destDir -Force | Out-Null }

# Float (default) build
Write-Host "Building Godot ONNX GDExtension (float)..."
cargo build --release
$targetPath = "target\release\lib$libName.$ext"
if (-not (Test-Path $targetPath)) { $targetPath = "target\release\$libName.$ext" }
if (Test-Path $targetPath) {
    Copy-Item $targetPath -Destination "$destDir\lib$libName.$ext" -Force
    Write-Host "Copied to $destDir\lib$libName.$ext"
} else {
    Write-Error "Built library not found (expected $targetPath or target\release\lib$libName.$ext)"
    exit 1
}

# Doubles build (for Godot with precision=double). Requires GODOT4_BIN set to a Godot binary built with precision=double.
if ($env:GODOT4_BIN) {
    Write-Host "Building Godot ONNX GDExtension (doubles)... GODOT4_BIN=$env:GODOT4_BIN"
    $errPreference = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    & cargo build --release --no-default-features --features double-precision 2>&1 | Out-Host
    $dc = $LASTEXITCODE
    $ErrorActionPreference = $errPreference
    if ($dc -eq 0) {
        $tp = "target\release\lib$libName.$ext"
        if (-not (Test-Path $tp)) { $tp = "target\release\$libName.$ext" }
        if (Test-Path $tp) {
            Copy-Item $tp -Destination "$destDir\lib${libName}_doubles.$ext" -Force
            Write-Host "Copied to $destDir\lib${libName}_doubles.$ext"
        }
    } else {
        Write-Warning "Doubles build failed. Float build is ready."
    }
} else {
    Write-Host "Doubles build skipped (set GODOT4_BIN to a double-precision Godot binary to build)."
}
exit 0
