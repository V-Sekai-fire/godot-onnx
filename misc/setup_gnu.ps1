# Copyright (c) 2026-present K. S. Ernest (iFire) Lee & godot-onnx contributors
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Option B: Use GNU toolchain so we don't need Visual Studio.
# Run this once (from a terminal where Rust is in PATH), then build with: cargo build --release

$ErrorActionPreference = "Stop"

Write-Host "Setting up Rust for Windows GNU (MinGW) build..."

# Find rustup: PATH or common install locations
$rustupExe = $null
foreach ($p in @("rustup", "$env:USERPROFILE\.cargo\bin\rustup.exe", "C:\Users\$env:USERNAME\.cargo\bin\rustup.exe")) {
    if ($p -eq "rustup") {
        $r = Get-Command rustup -ErrorAction SilentlyContinue
        if ($r) { $rustupExe = "rustup"; break }
    } elseif (Test-Path $p) { $rustupExe = $p; break }
}
if (-not $rustupExe) {
    Write-Host "rustup not found. Install Rust from https://rustup.rs, then in a new terminal run:"
    Write-Host "  rustup target add x86_64-pc-windows-gnu"
    Write-Host "  rustup default stable-x86_64-pc-windows-gnu"
    Write-Host "Then install MinGW-w64 (gcc in PATH) and run: cargo build --release"
    exit 1
}

# 1. Ensure GNU target is installed and set as default (so build scripts use gcc too)
Write-Host "Adding and defaulting to x86_64-pc-windows-gnu..."
& $rustupExe target add x86_64-pc-windows-gnu
& $rustupExe default stable-x86_64-pc-windows-gnu

# 2. Check for gcc (MinGW)
$gcc = Get-Command gcc -ErrorAction SilentlyContinue
if (-not $gcc) {
    Write-Host ""
    Write-Host "gcc not found. Install MinGW-w64 and add its bin/ to PATH:"
    Write-Host "  - https://www.mingw-w64.org/"
    Write-Host "  - Or: winget install -e --id mingw-w64.mingw-w64"
    Write-Host "  Then restart the terminal and run: cargo build --release"
    exit 1
}

Write-Host "gcc: $($gcc.Source)"
Write-Host "Done. Run: cargo build --release"
Write-Host "Then: .\misc\build.ps1"
