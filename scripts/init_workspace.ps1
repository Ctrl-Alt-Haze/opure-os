# OPURE OS Core Architecture - Workspace Initialization & Packaging Script
# Bootstraps, compiles, tests, packages UEFI image, and benchmarks the OPURE OS Cognitive Core.

param (
    [switch]$Release = $true,
    [switch]$RunBenchmark = $true,
    [switch]$PackageUefi = $true
)

$ErrorActionPreference = "Stop"

Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host "  INITIALIZING OPURE OS ARCHITECTURE WORKSPACE            " -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan

# 1. Verify Rust Toolchain
Write-Host "`n[1/5] Checking Rust Toolchain..." -ForegroundColor Yellow
cargo --version
rustc --version

# 2. Check and Compile Crates
Write-Host "`n[2/5] Compiling OPURE Workspace Crates..." -ForegroundColor Yellow
Set-Location -Path "$PSScriptRoot\.."

if ($Release) {
    cargo build --workspace --release --all-targets
} else {
    cargo build --workspace --all-targets
}

# 3. Execute Core Tests
Write-Host "`n[3/5] Running Workspace Unit & Integration Tests..." -ForegroundColor Yellow
if ($Release) {
    cargo test --workspace --release
} else {
    cargo test --workspace
}

# 4. Compile Standalone UEFI Kernel & Package Virtual Disk
if ($PackageUefi) {
    Write-Host "`n[4/5] Building Bare-Metal UEFI Bootloader, WASM Bundle & Packaging Disk Image..." -ForegroundColor Yellow
    cargo build -p opure-boot --target x86_64-unknown-uefi --release
    cargo rustc -p opure-wasm --target wasm32-unknown-unknown --release --crate-type cdylib
    Copy-Item "$PSScriptRoot\..\target\wasm32-unknown-unknown\release\opure_wasm.wasm" "$PSScriptRoot\..\web\opure_wasm.wasm" -Force
    python "$PSScriptRoot\package_uefi_disk.py"
}

# 5. Run Demonstration Runtime & Benchmarks
if ($RunBenchmark) {
    Write-Host "`n[5/5] Executing OPURE Cognitive Core Runtime & Benchmark Suite..." -ForegroundColor Green
    if ($Release) {
        cargo run --bin opure-core --release
    } else {
        cargo run --bin opure-core
    }
}

Write-Host "`n[DONE] OPURE Workspace initialization, UEFI packaging & verification complete." -ForegroundColor Green
