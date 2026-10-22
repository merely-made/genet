# Copyright 2026 Mark Alan Boykin
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
# SPDX-License-Identifier: MPL-2.0

[CmdletBinding()]
param(
    [string]$Wasm64Toolchain = "nightly-2026-06-22",
    [string]$StableToolchain = "1.99.1",
    [string]$OutputDirectory = "dist/scripted-workers"
)

$ErrorActionPreference = "Stop"
$repo = Split-Path -Parent $PSScriptRoot
$out = Join-Path $repo $OutputDirectory
. (Join-Path $PSScriptRoot 'checked-native.ps1')
$metadata = Invoke-CheckedNative -Description 'cargo metadata' -FilePath 'cargo' -Arguments @(
    'metadata', '--manifest-path', (Join-Path $repo 'Cargo.toml'), '--format-version', '1', '--no-deps'
) | ConvertFrom-Json
$target = $metadata.target_directory

$wasmBindgen = Get-Command wasm-bindgen -ErrorAction SilentlyContinue
if (-not $wasmBindgen) {
    throw "wasm-bindgen CLI 0.2.129 is required: cargo install wasm-bindgen-cli --version 0.2.129 --locked"
}
$version = Invoke-CheckedNative -Description 'wasm-bindgen version' -FilePath $wasmBindgen.Source -Arguments @('--version')
if ($version -notmatch '^wasm-bindgen 0\.2\.129$') {
    throw "wasm-bindgen CLI must be 0.2.129; found: $version"
}

Invoke-CheckedNative -Description 'wasm64 rustc version' -FilePath 'rustup' -Arguments @(
    'run', $Wasm64Toolchain, 'rustc', '--version'
) | Out-Null
Invoke-CheckedNative -Description 'wasm64 rust-src installation' -FilePath 'rustup' -Arguments @(
    'component', 'add', 'rust-src', '--toolchain', $Wasm64Toolchain
) | Out-Null

$previousRustFlags = $env:RUSTFLAGS
try {
    $env:RUSTFLAGS = '--cfg getrandom_backend="wasm_js"'
    Invoke-CheckedNative -Description 'Nova wasm64 build' -FilePath 'rustup' -Arguments @(
        'run', $Wasm64Toolchain, 'cargo', 'build',
        '--manifest-path', (Join-Path $repo 'Cargo.toml'),
        '--release', '--package', 'genet-scripted-worker',
        '--no-default-features', '--features', 'engine-nova',
        '--target', 'wasm64-unknown-unknown', '-Z', 'build-std=std,panic_abort'
    )
} finally {
    $env:RUSTFLAGS = $previousRustFlags
}

Invoke-CheckedNative -Description 'Boa wasm32 build' -FilePath 'rustup' -Arguments @(
    'run', $StableToolchain, 'cargo', 'build',
    '--manifest-path', (Join-Path $repo 'Cargo.toml'),
    '--release', '--package', 'genet-scripted-worker',
    '--no-default-features', '--features', 'engine-boa',
    '--target', 'wasm32-unknown-unknown'
)

$novaOut = Join-Path $out "nova"
$boaOut = Join-Path $out "boa"
New-Item -ItemType Directory -Force -Path $novaOut | Out-Null
New-Item -ItemType Directory -Force -Path $boaOut | Out-Null

Invoke-CheckedNative -Description 'Nova wasm-bindgen' -FilePath $wasmBindgen.Source -Arguments @(
    (Join-Path $target 'wasm64-unknown-unknown/release/genet_scripted_worker.wasm'),
    '--target', 'web', '--out-dir', $novaOut, '--out-name', 'genet-scripted-nova-wasm64'
)

Invoke-CheckedNative -Description 'Boa wasm-bindgen' -FilePath $wasmBindgen.Source -Arguments @(
    (Join-Path $target 'wasm32-unknown-unknown/release/genet_scripted_worker.wasm'),
    '--target', 'web', '--out-dir', $boaOut, '--out-name', 'genet-scripted-boa-wasm32'
)

Copy-Item (Join-Path $repo "components/genet-scripted-worker/loader.mjs") $out -Force
Copy-Item (Join-Path $repo "components/genet-scripted-worker/worker-bootstrap.mjs") $out -Force

Write-Output "Generated genet-scripted-nova-wasm64 and genet-scripted-boa-wasm32 in $out"
