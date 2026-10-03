# Copyright 2026 Mark Alan Boykin
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
# SPDX-License-Identifier: MPL-2.0

[CmdletBinding()]
param(
    [switch]$NoBuild
)

# Reftest baselines are a LOCAL guard, not default CI: reftests render through
# the GPU (`Renderer::boot`), which the headless CI runner does not have. Run
# this locally to confirm `unexpected=0` on the checked reftest slices.

$ErrorActionPreference = "Stop"
$repo = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)

if (-not $NoBuild) {
    cargo build --manifest-path (Join-Path $repo "Cargo.toml") --release -p genet-wpt
    if ($LASTEXITCODE -ne 0) {
        throw "cargo build -p genet-wpt --release failed with exit code $LASTEXITCODE"
    }
}

$metadata = cargo metadata --manifest-path (Join-Path $repo "Cargo.toml") --format-version 1 --no-deps | ConvertFrom-Json
if ($LASTEXITCODE -ne 0) {
    throw "cargo metadata failed with exit code $LASTEXITCODE"
}

$exeName = if ($IsWindows -or $env:OS -eq "Windows_NT") { "genet-wpt.exe" } else { "genet-wpt" }
$runner = Join-Path $metadata.target_directory (Join-Path "release" $exeName)
if (-not (Test-Path $runner)) {
    throw "release genet-wpt binary not found at $runner; rerun without -NoBuild"
}

$baselines = @(
    @{
        Subset = "css/mediaqueries"
        Engine = "boa"
        Expectations = "ports/genet-wpt/expectations/reftest/css_mediaqueries_boa.json"
    },
    @{
        Subset = "css/css-position"
        Engine = "boa"
        Expectations = "ports/genet-wpt/expectations/reftest/css_position_boa.json"
    }
    @{
        Subset = "css/css-text/text-align"
        Engine = "boa"
        Expectations = "ports/genet-wpt/expectations/reftest/css_css-text_text-align_boa.json"
    }
    @{
        Subset = "css/css-text/white-space"
        Engine = "boa"
        Expectations = "ports/genet-wpt/expectations/reftest/css_css-text_white-space_boa.json"
    }
    @{
        Subset = "css/css-text/text-indent"
        Engine = "boa"
        Expectations = "ports/genet-wpt/expectations/reftest/css_css-text_text-indent_boa.json"
    }
    @{
        Subset = "css/css-text/text-justify"
        Engine = "boa"
        Expectations = "ports/genet-wpt/expectations/reftest/css_css-text_text-justify_boa.json"
    }
    @{
        Subset = "css/css-text/line-breaking"
        Engine = "boa"
        Expectations = "ports/genet-wpt/expectations/reftest/css_css-text_line-breaking_boa.json"
    }
    @{
        Subset = "css/css-text/overflow-wrap"
        Engine = "boa"
        Expectations = "ports/genet-wpt/expectations/reftest/css_css-text_overflow-wrap_boa.json"
    }
    @{
        Subset = "css/css-text/word-break"
        Engine = "boa"
        Expectations = "ports/genet-wpt/expectations/reftest/css_css-text_word-break_boa.json"
    }
    @{
        Subset = "css/css-text/letter-spacing"
        Engine = "boa"
        Expectations = "ports/genet-wpt/expectations/reftest/css_css-text_letter-spacing_boa.json"
    }
    @{
        Subset = "css/css-text/hanging-punctuation"
        Engine = "boa"
        Expectations = "ports/genet-wpt/expectations/reftest/css_css-text_hanging-punctuation_boa.json"
    }
    @{
        Subset = "css/css-inline"
        Engine = "boa"
        Expectations = "ports/genet-wpt/expectations/reftest/css_css-inline_boa.json"
    }
    @{
        Subset = "css/CSS2/linebox"
        Engine = "boa"
        Expectations = "ports/genet-wpt/expectations/reftest/css_CSS2_linebox_boa.json"
    }
    @{
        Subset = "css/CSS2/text"
        Engine = "boa"
        Expectations = "ports/genet-wpt/expectations/reftest/css_CSS2_text_boa.json"
    }
    @{
        Subset = "css/CSS2/floats"
        Engine = "boa"
        Expectations = "ports/genet-wpt/expectations/reftest/css_CSS2_floats_boa.json"
    }
    @{
        Subset = "css/CSS2/bidi-text"
        Engine = "boa"
        Expectations = "ports/genet-wpt/expectations/reftest/css_CSS2_bidi-text_boa.json"
    }
)

foreach ($baseline in $baselines) {
    $expectations = Join-Path $repo $baseline.Expectations
    $renderer = if ($baseline.Renderer) { $baseline.Renderer } else { "livery" }
    Write-Output "Checking WPT reftest baseline: $($baseline.Subset) [$($baseline.Engine)/$renderer]"
    & $runner reftest $baseline.Subset --engine $baseline.Engine --renderer $renderer --expectations $expectations
    if ($LASTEXITCODE -ne 0) {
        throw "WPT reftest baseline failed: $($baseline.Subset) [$($baseline.Engine)]"
    }
}

Write-Output "WPT reftest baselines: unexpected=0"
