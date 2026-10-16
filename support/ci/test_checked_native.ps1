# Windows: pwsh -NoProfile -File support/ci/test_checked_native.ps1
[CmdletBinding()]
param()
$ErrorActionPreference = 'Stop'
$repo = Split-Path (Split-Path $PSScriptRoot -Parent) -Parent
$scratch = Join-Path ([IO.Path]::GetTempPath()) "genet-native-$PID-$([guid]::NewGuid().ToString('N'))"
$fake = Join-Path $scratch 'fake'
$original = @{
    Path = $env:PATH; Fail = $env:AUDIT_FAIL; Log = $env:AUDIT_LOG
    Target = $env:AUDIT_TARGET; RustFlags = $env:RUSTFLAGS
    CargoTarget = $env:CARGO_TARGET_DIR
}
$hadCargoTarget = Test-Path Env:CARGO_TARGET_DIR
$nativePreference = Get-Variable PSNativeCommandUseErrorActionPreference -ErrorAction SilentlyContinue
if ($nativePreference) { $originalNativePreference = $nativePreference.Value }
function Assert([bool]$Condition, [string]$Message) {
    if (-not $Condition) { throw $Message }
}
try {
    . (Join-Path $repo 'support/checked-native.ps1')
    $preferenceCases = if ($nativePreference) { @($false, $true) } else { @($null) }
    foreach ($promote in $preferenceCases) {
        if ($nativePreference) { $PSNativeCommandUseErrorActionPreference = $promote }
        try {
            Invoke-CheckedNative -Description 'intentional native failure' -FilePath 'cmd.exe' -Arguments @('/d', '/c', 'exit 17')
            throw 'Expected a native failure.'
        } catch {
            Assert ($_.Exception.Message -match 'failed with exit code 17') "Wrong promoted-native-errors=$promote result: $_"
        }
        if ($nativePreference) {
            Assert ($PSNativeCommandUseErrorActionPreference -eq $promote) "Caller native-error preference changed for $promote."
        }
    }
    if ($nativePreference) { $PSNativeCommandUseErrorActionPreference = $originalNativePreference }

    New-Item -ItemType Directory -Path $fake -Force | Out-Null
    @'
$tool = $args[0]
$a = @($args | Select-Object -Skip 1)
$stage = switch ($tool) {
    'cargo' { if ($a[0] -eq 'metadata') { 'metadata' } else { 'receipt-build' }; break }
    'rustup' {
        if ($a[0] -eq 'component') { 'component' }
        elseif ($a[2] -eq 'rustc') { 'rustc' }
        elseif ($a[1] -eq 'nightly-2026-06-22') { 'nova-build' }
        else { 'boa-build' }
        break
    }
    'wasm-bindgen' {
        if ($a[0] -eq '--version') { 'version' }
        elseif ($a[0] -match 'wasm64') { 'nova-bindgen' }
        elseif ($a[0] -match 'scripted_worker') { 'boa-bindgen' }
        else { 'receipt-bindgen' }
        break
    }
    'python' { 'python'; break }
}
Add-Content $env:AUDIT_LOG $stage
if ($stage -eq $env:AUDIT_FAIL) { exit 17 }
if ($stage -eq 'metadata') { @{ target_directory = $env:AUDIT_TARGET } | ConvertTo-Json -Compress }
if ($stage -eq 'version') { 'wasm-bindgen 0.2.129' }
if ($stage -eq 'python') { 'C:\fake-python.exe' }
exit 0
'@ | Set-Content (Join-Path $fake 'fake-native.ps1')
    foreach ($name in @('cargo', 'rustup', 'wasm-bindgen', 'python')) {
        $wrapper = '@echo off' + [Environment]::NewLine +
            'powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0fake-native.ps1" ' + $name + ' %*'
        Set-Content (Join-Path $fake "$name.cmd") $wrapper
    }
    $env:PATH = "$fake;$($original.Path)"
    $env:AUDIT_TARGET = Join-Path $scratch 'stale-target'
    $env:RUSTFLAGS = 'original-flags'
    foreach ($arch in @('wasm64', 'wasm32')) {
        $dir = Join-Path $env:AUDIT_TARGET "$arch-unknown-unknown/release"
        New-Item -ItemType Directory $dir -Force | Out-Null
        Set-Content (Join-Path $dir 'genet_scripted_worker.wasm') 'stale'
    }
    foreach ($stage in @('metadata', 'version', 'rustc', 'component', 'nova-build',
                         'boa-build', 'nova-bindgen', 'boa-bindgen')) {
        $env:AUDIT_FAIL = $stage
        $env:AUDIT_LOG = Join-Path $scratch "$stage.log"
        $outName = "dist/checked-native-$PID-$stage"
        $out = Join-Path $repo $outName
        try {
            & (Join-Path $repo 'support/build-scripted-workers.ps1') -OutputDirectory $outName | Out-Null
            throw 'Expected a native failure.'
        } catch {
            Assert ($_.Exception.Message -match 'failed with exit code 17') "Wrong $stage result: $_"
        }
        Assert ((Get-Content $env:AUDIT_LOG | Select-Object -Last 1) -eq $stage) "$stage continued."
        Assert (-not (Test-Path (Join-Path $out 'loader.mjs'))) "$stage copied the loader."
        Assert ($env:RUSTFLAGS -eq 'original-flags') "$stage changed RUSTFLAGS."
        if (Test-Path $out) { Remove-Item $out -Recurse -Force }
    }
    $chrome = Join-Path $scratch 'chrome.exe'
    Set-Content $chrome 'unused'
    foreach ($stage in @('receipt-build', 'receipt-bindgen', 'python')) {
        $env:AUDIT_FAIL = $stage
        $env:AUDIT_LOG = Join-Path $scratch "$stage.log"
        $artifact = Join-Path $scratch "receipt-$stage"
        $stale = Join-Path $artifact 'target/wasm32-unknown-unknown/debug'
        New-Item -ItemType Directory $stale -Force | Out-Null
        Set-Content (Join-Path $stale 'ortet.wasm') 'stale'
        $env:CARGO_TARGET_DIR = 'original-target'
        try {
            & (Join-Path $repo 'support/ci/run_ortet_chromium_receipt.ps1') -ArtifactDir $artifact -ChromePath $chrome | Out-Null
            throw 'Expected a native failure.'
        } catch {
            Assert ($_.Exception.Message -match 'failed with exit code 17') "Wrong $stage result: $_"
        }
        Assert ((Get-Content $env:AUDIT_LOG | Select-Object -Last 1) -eq $stage) "$stage continued."
        Assert ($env:CARGO_TARGET_DIR -eq 'original-target') "$stage did not restore CARGO_TARGET_DIR."
        Assert (-not (Test-Path (Join-Path $artifact 'chromium-receipt.stderr.log'))) "$stage launched Chrome."
    }
    $env:AUDIT_FAIL = 'receipt-build'
    $env:AUDIT_LOG = Join-Path $scratch 'unset.log'
    Remove-Item Env:CARGO_TARGET_DIR
    try {
        & (Join-Path $repo 'support/ci/run_ortet_chromium_receipt.ps1') -ArtifactDir (Join-Path $scratch 'unset') -ChromePath $chrome | Out-Null
        throw 'Expected a native failure.'
    } catch {
        Assert ($_.Exception.Message -match 'failed with exit code 17') "Wrong unset result: $_"
    }
    Assert (-not (Test-Path Env:CARGO_TARGET_DIR)) 'Absent CARGO_TARGET_DIR was not restored.'
    'Checked-native regression passed.'
}
finally {
    if ($nativePreference) { $PSNativeCommandUseErrorActionPreference = $originalNativePreference }
    $env:PATH = $original.Path
    $env:AUDIT_FAIL = $original.Fail
    $env:AUDIT_LOG = $original.Log
    $env:AUDIT_TARGET = $original.Target
    $env:RUSTFLAGS = $original.RustFlags
    if ($hadCargoTarget) { $env:CARGO_TARGET_DIR = $original.CargoTarget }
    else { Remove-Item Env:CARGO_TARGET_DIR -ErrorAction SilentlyContinue }
    if (Test-Path $scratch) { Remove-Item $scratch -Recurse -Force }
}
