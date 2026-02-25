# Copyright (c) 2026-present K. S. Ernest (iFire) Lee & godot-onnx contributors
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Run from "Developer PowerShell for VS" (or a terminal where link.exe is in PATH).
# ort only has prebuilt ONNX Runtime for Windows MSVC, not MinGW.

$ErrorActionPreference = "Stop"
Set-Location $PSScriptRoot\..

# Optional: ensure Rust is in PATH
$cargo = Get-Command cargo -ErrorAction SilentlyContinue
if (-not $cargo) {
    $env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"
}

Write-Host "Building release (MSVC; ort prebuilts require Windows MSVC)..."
cargo build --release
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "Copying DLL to sample/addons/godot-onnx/..."
$dll = "target\release\godot_onnx.dll"
$alt = "target\release\libgodot_onnx.dll"
$destName = "libgodot_onnx.dll"
if (Test-Path $dll) {
    $src = $dll
} elseif (Test-Path $alt) {
    $src = $alt
} else {
    Write-Host "No DLL found at $dll or $alt"
    exit 1
}
New-Item -ItemType Directory -Path "sample\addons\godot-onnx" -Force | Out-Null
Copy-Item $src -Destination "sample\addons\godot-onnx\$destName" -Force
Write-Host "Done. Open sample/ in Godot 4.4+ and run the main scene."
