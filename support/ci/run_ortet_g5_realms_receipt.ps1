# Copyright 2026 Mark Alan Boykin
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
# SPDX-License-Identifier: MPL-2.0

# The full headed G5 realms acceptance.
#
# A parent document and a same-origin child iframe, driven by one native click:
# the parent's script builds associated state on the child's subtree (an open
# shadow root holding a nested closed one, plus a parser-created template),
# adopts that subtree into the parent, sends the parent's own link the other
# way, navigates the child twice, brings the subtree back into the child, keeps
# one link in the parent, and navigates the top level. Three things outside the
# script are then correlated with it, per engine: the presented frame's digest,
# the accessibility projection the host published, and the live-node census of
# the top document's arena.
#
# The fixture is served over one loopback http origin because this engine gives
# every `file:` URL an opaque origin, and an opaque origin is same-origin with
# nothing — the parent could not reach `frame.contentDocument` at all from the
# filesystem. See support/ci/ortet_g5_realms_receipt_server.mjs.
#
# The window is stated in physical pixels and the display scale is read back
# from Ortet's own `display scale=` line, so the CSS viewport this receipt
# asserts against is recorded rather than assumed (the 2026-09-10 correction to
# the O2 bridge-action receipt). Nothing in the fixture is drawn below 20 px, so
# its digest does not inherit the frames fixture's bimodal instability.

[CmdletBinding()]
param(
    [Parameter(Mandatory)]
    [string]$ArtifactDir,
    [string]$TargetDir,
    [ValidateRange(1, 32)]
    [int]$BuildJobs = 4,
    [switch]$Release,
    [ValidateRange(1, 65535)]
    [int]$HttpPort = 18795,
    [ValidateRange(1, 120000)]
    [int]$ReceiptTimeoutMs = 40000,
    [ValidateRange(1, 5000)]
    [int]$FailureTimeoutMs = 25,
    [ValidateRange(1, 10)]
    [int]$Repeats = 3,
    # Physical pixels. At this display's scale factor of 2.0 this is a 640x560
    # CSS viewport, which is what puts every link the projection advertises
    # inside the window.
    [string]$Size = '1280x1120'
)

$ErrorActionPreference = 'Stop'
$repo = Split-Path (Split-Path $PSScriptRoot -Parent) -Parent
$artifact = [IO.Path]::GetFullPath($ArtifactDir)
$target = if ($TargetDir) { [IO.Path]::GetFullPath($TargetDir) } else { Join-Path $artifact 'target' }
$serverScript = Join-Path $repo 'support\ci\ortet_g5_realms_receipt_server.mjs'
$completion = 'Ortet G5 realms sequence complete'
$impossible = 'Ortet impossible realms heading'
$keptLink = 'Adopted link kept in the parent'
$awayLink = 'Link adopted into the child'
New-Item -ItemType Directory -Path $artifact -Force | Out-Null

# Every replay must name exactly what it was built from. Same contract as the
# G5 arena receipt: clean checkouts for every local path dependency, the locked
# graph, the lockfile hash and the compiler.
function Get-SourceIdentity {
    $metadataText = cargo metadata --format-version 1 --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Cannot resolve the locked receipt dependency graph.' }
    $metadata = ($metadataText -join "`n") | ConvertFrom-Json
    $metadataText | Set-Content (Join-Path $artifact 'cargo-metadata.json')
    $roots = @($repo)
    foreach ($package in $metadata.packages) {
        if ($null -ne $package.source) { continue }
        $directory = Split-Path $package.manifest_path -Parent
        $root = git -C $directory rev-parse --show-toplevel 2>$null
        if ($LASTEXITCODE -ne 0) { throw "Local package has no source revision: $directory" }
        $roots += ($root -join '').Trim()
    }
    $identities = foreach ($root in ($roots | Sort-Object -Unique)) {
        $changedPaths = @(git -C $root diff HEAD --name-only)
        $changedPaths += @(git -C $root ls-files --others --exclude-standard)
        $workingFiles = foreach ($path in @($changedPaths | Sort-Object -Unique)) {
            $absolute = Join-Path $root $path
            [pscustomobject]@{
                path = $path
                sha256 = if (Test-Path -LiteralPath $absolute -PathType Leaf) {
                    (Get-FileHash -LiteralPath $absolute -Algorithm SHA256).Hash
                } else { $null }
            }
        }
        [pscustomobject]@{
            path = $root
            revision = (git -C $root rev-parse HEAD).Trim()
            dirty = @(git -C $root status --porcelain --untracked-files=normal).Count
            working_files = @($workingFiles)
        }
    }
    [pscustomobject]@{
        repositories = @($identities)
        lock_sha256 = (Get-FileHash (Join-Path $repo 'Cargo.lock') -Algorithm SHA256).Hash
        rustc = (rustc -Vv) -join "`n"
    } | ConvertTo-Json -Depth 5 -Compress
}

function Assert-ReceiptLog {
    param([string]$Output, [string]$Engine, [string]$ExpectedAddress)

    $engineId = if ($Engine -eq 'nova') { 'genet.scripted.nova' } else { 'genet.scripted' }
    $identity = "engine $engineId backend $Engine presented"
    if ($Output -notmatch [regex]::Escape($identity)) { throw "Ortet $Engine did not report engine/backend identity '$identity'." }
    if ($Output -notmatch [regex]::Escape("semantic heading `"$completion`"")) { throw "Ortet $Engine did not report semantic completion." }
    # The top-level navigation happens inside the session, so the host's own
    # address is still the one it opened. The completion heading only exists in
    # the document the top level navigated *to*; matching it is the proof.
    if ($Output -notmatch [regex]::Escape("settled at $ExpectedAddress")) { throw "Ortet $Engine did not settle at $ExpectedAddress." }
}

function Assert-CompletionPixels {
    param([string]$Png, [string]$ExpectedColor, [string]$RecordPath)

    Add-Type -AssemblyName System.Drawing
    $bitmap = [System.Drawing.Bitmap]::FromFile($Png)
    try {
        $point = [Math]::Max(0, $bitmap.Width - 8), [Math]::Max(0, $bitmap.Height - 8)
        $actual = $bitmap.GetPixel($point[0], $point[1])
        $expected = [System.Drawing.ColorTranslator]::FromHtml($ExpectedColor)
        [pscustomobject]@{
            x = $point[0]; y = $point[1]
            expected = ('#{0:X2}{1:X2}{2:X2}' -f $expected.R, $expected.G, $expected.B)
            actual = ('#{0:X2}{1:X2}{2:X2}{3:X2}' -f $actual.R, $actual.G, $actual.B, $actual.A)
            width = $bitmap.Width; height = $bitmap.Height
        } | ConvertTo-Json | Set-Content $RecordPath
        if ($actual.R -ne $expected.R -or $actual.G -ne $expected.G -or $actual.B -ne $expected.B) {
            throw "receipt pixel at ($($point[0]),$($point[1])) was $($actual.R),$($actual.G),$($actual.B), expected $($expected.R),$($expected.G),$($expected.B)"
        }
    }
    finally { $bitmap.Dispose() }
}

# Split the appended dump into one record per published revision.
function Read-Projections {
    param([string]$Path)

    if (-not (Test-Path $Path)) { throw "the accessibility dump was not written: $Path" }
    $revisions = @()
    $current = $null
    foreach ($line in Get-Content $Path) {
        if ($line -match '^projection revision=(?<revision>\d+) frames=(?<frames>\d+) address=(?<address>\S+) nodes=(?<nodes>\d+) root=(?<root>\d+)$') {
            if ($current) { $revisions += $current }
            $current = [pscustomobject]@{
                revision = [int]$Matches['revision']
                frames = [int]$Matches['frames']
                address = $Matches['address']
                root = $Matches['root']
                nodes = @()
            }
            continue
        }
        if ($line -match '^\s+node id=(?<id>\d+) role=(?<role>.+?) name="(?<name>.*)" value=".*" actions=\[(?<actions>.*)\] bounds=(?<bounds>\S+)$') {
            if ($current) {
                $current.nodes += [pscustomobject]@{
                    id = $Matches['id']
                    role = $Matches['role'].Trim()
                    name = $Matches['name']
                    actions = $Matches['actions']
                    bounds = $Matches['bounds']
                }
            }
        }
    }
    if ($current) { $revisions += $current }
    if (-not $revisions.Count) { throw "no projection was published into $Path" }
    return $revisions
}

# The accessibility half of the acceptance, asserted on one and the same
# published revision: the link that ended up in the parent is there, as a Link
# with a Click action, and the link that was adopted into the child and
# navigated away is not.
function Assert-Projection {
    param([object[]]$Revisions, [string]$Engine, [string]$RecordPath)

    $settled = $Revisions | Where-Object {
        ($_.nodes | Where-Object { $_.name -eq $keptLink }) -and
        ($_.nodes | Where-Object { $_.role -eq 'Heading { level: 1 }' -and $_.name -eq 'Ortet G5 realms adoption settled' })
    }
    if (-not $settled) { throw "Ortet $Engine never published a projection with the adopted link in the settled state." }
    $settled = @($settled)[-1]
    $kept = @($settled.nodes | Where-Object { $_.name -eq $keptLink })[0]
    if ($kept.role -ne 'Link') { throw "Ortet $Engine projected the adopted link as $($kept.role), not Link." }
    if ($kept.actions -notmatch 'Click') { throw "Ortet $Engine projected the adopted link without a Click action: [$($kept.actions)]." }
    if ($settled.nodes | Where-Object { $_.name -eq $awayLink }) {
        throw "Ortet $Engine still projects the link that was adopted into the child and navigated away."
    }
    # The subtree came from the child's arena, so the adopted link's node id is
    # tagged with a different arena than the parent's document root. That the
    # parent's projection carries it at all is the cross-arena half of this.
    if ($kept.id.Substring(0, 3) -eq $settled.root.Substring(0, 3)) {
        Write-Warning "Ortet $Engine projected the adopted link under a parent-arena id; the arena tag could not be distinguished."
    }
    $final = @($Revisions)[-1]
    if ($final.root -eq $settled.root) { throw "Ortet $Engine published no post-navigation projection: the top level did not replace the document." }
    if (-not ($final.nodes | Where-Object { $_.name -eq $completion })) {
        throw "Ortet $Engine's final projection does not carry the completion heading."
    }
    [pscustomobject]@{
        revisions = $Revisions.Count
        settled_revision = $settled.revision
        kept_link = $kept
        final_revision = $final.revision
        final_root = $final.root
        settled_root = $settled.root
    } | ConvertTo-Json -Depth 5 | Set-Content $RecordPath
}

# Run the host and capture everything it said.
#
# `& $exe ... 2>&1 | Tee-Object` cannot be used here for two reasons, both
# observed rather than assumed. Ortet writes progress lines to stderr, and with
# `$ErrorActionPreference = 'Stop'` a redirected native stderr line becomes a
# terminating error. Ortet also previously lingered after its event loop
# exited. Keep a process deadline and retain its log on failure; completion
# also requires a normal process exit.
function Invoke-Ortet {
    param([string]$Exe, [string[]]$Arguments, [string]$Log, [int]$GraceMs)

    $errorLog = "$Log.err"
    # A raw .NET process, not Start-Process: `Start-Process -PassThru` without
    # `-Wait` does not reliably populate `ExitCode`, and this receipt reads the
    # exit code.
    $info = [Diagnostics.ProcessStartInfo]::new()
    $info.FileName = $Exe
    foreach ($argument in $Arguments) { $info.ArgumentList.Add([string]$argument) }
    $info.RedirectStandardOutput = $true
    $info.RedirectStandardError = $true
    $info.UseShellExecute = $false
    $info.WorkingDirectory = (Get-Location).Path
    $process = [Diagnostics.Process]::Start($info)
    # Read both pipes on their own threads: a full pipe buffer would otherwise
    # stall the host before it finished writing its receipt.
    $outTask = $process.StandardOutput.ReadToEndAsync()
    $errTask = $process.StandardError.ReadToEndAsync()
    $exited = $process.WaitForExit($GraceMs)
    if (-not $exited) {
        $process.Kill($true)
        $null = $process.WaitForExit(5000)
    }
    $output = ''
    foreach ($task in @($outTask, $errTask)) {
        if ($task.Wait(5000)) { $output += $task.Result }
    }
    Set-Content -Path $Log -Value $output
    Set-Content -Path $errorLog -Value ''
    [pscustomobject]@{
        exited = $exited
        code = if ($exited) { $process.ExitCode } else { $null }
        output = $output
    }
}

Push-Location $repo
try {
    $env:CARGO_TARGET_DIR = $target
    $before = Get-SourceIdentity
    $before | Set-Content (Join-Path $artifact 'source-identity.json')
    $env:GENET_SOURCE_REVISION = (git rev-parse HEAD).Trim()
    $buildArguments = @('build', '-p', 'ortet', '--features', 'scripted-nova', '--offline', '--locked', '-j', "$BuildJobs")
    if ($Release) { $buildArguments += '--release' }
    & cargo @buildArguments
    if ($LASTEXITCODE -ne 0) { throw 'native scripted Ortet build failed.' }
    $profile = if ($Release) { 'release' } else { 'debug' }
    $exe = Join-Path $target "$profile\ortet.exe"
    if (-not (Test-Path $exe)) { throw "Ortet binary was not produced at $exe." }
    (Get-FileHash -LiteralPath $exe -Algorithm SHA256).Hash.ToLowerInvariant() + '  ortet.exe' |
        Set-Content (Join-Path $artifact 'ortet.sha256')

    $server = Start-Process -FilePath 'node' -ArgumentList @(
        $serverScript, '--artifact', $artifact, '--port', $HttpPort
    ) -PassThru -WindowStyle Hidden
    try {
        Start-Sleep -Milliseconds 500
        $origin = "http://127.0.0.1:$HttpPort"
        $url = "$origin/parent.html"
        $summary = @()

        foreach ($engine in @('boa', 'nova')) {
            $engineArtifact = Join-Path $artifact $engine
            New-Item -ItemType Directory -Path $engineArtifact -Force | Out-Null
            $digests = @()
            $censuses = @()

            for ($pass = 1; $pass -le $Repeats; $pass++) {
                $passArtifact = Join-Path $engineArtifact "pass$pass"
                New-Item -ItemType Directory -Path $passArtifact -Force | Out-Null
                $png = Join-Path $passArtifact 'receipt.png'
                $log = Join-Path $passArtifact 'receipt.log'
                $dump = Join-Path $passArtifact 'a11y.txt'
                # The dump is appended to, one block per published revision, so
                # a re-run must start from an empty file.
                if (Test-Path $dump) { Remove-Item $dump -Force }
                $run = Invoke-Ortet -Exe $exe -Log $log -GraceMs ($ReceiptTimeoutMs + 20000) -Arguments @(
                    '--url', $url, '--engine', $engine, '--size', $Size, '--artifact', $png,
                    '--a11y-dump', $dump, '--actions', 'click:170,62',
                    '--expect-heading', $completion, '--timeout-ms', $ReceiptTimeoutMs
                )
                $output = $run.output
                if ($run.exited -and $run.code -ne 0) {
                    throw "Ortet $engine pass $pass exited $($run.code) without completing the realms sequence; see $log."
                }
                if (-not $run.exited) {
                    throw "Ortet $engine pass $pass did not exit within the process deadline; see $log."
                }
                Assert-ReceiptLog -Output $output -Engine $engine -ExpectedAddress $url
                if (-not (Test-Path $png)) { throw "Ortet $engine pass $pass wrote no PNG." }
                (Get-FileHash -Algorithm SHA256 $png).Hash.ToLowerInvariant() + '  receipt.png' |
                    Set-Content (Join-Path $passArtifact 'receipt.sha256')
                Assert-CompletionPixels -Png $png -ExpectedColor '#2f6b3c' -RecordPath (Join-Path $passArtifact 'receipt-pixels.json')

                $scale = [regex]::Match($output, 'ortet: display scale=(?<scale>[\d.]+) physical=(?<w>\d+)x(?<h>\d+) logical=(?<lw>\d+)x(?<lh>\d+)')
                if (-not $scale.Success) { throw "Ortet $engine pass $pass did not report its display scale." }

                $digest = [regex]::Match($output, 'digest 0x(?<digest>[0-9a-f]{16})')
                if (-not $digest.Success) { throw "Ortet $engine pass $pass reported no frame digest." }
                $digests += $digest.Groups['digest'].Value

                # Live-node census: the sequence adopts a subtree out and back
                # and then navigates the top level. The incoming document is the
                # same shape as the outgoing one, so the arena must not end above
                # where it began — a leaked outgoing document would.
                $census = [regex]::Match($output, 'ortet: receipt live nodes first=(?<first>\d+) last=(?<last>\d+)')
                if (-not $census.Success) { throw "Ortet $engine pass $pass reported no live-node census." }
                $first = [int]$census.Groups['first'].Value
                $last = [int]$census.Groups['last'].Value
                if ($last -gt $first) { throw "Ortet $engine pass $pass left the arena larger than it began: first=$first last=$last." }
                $censuses += [pscustomobject]@{ pass = $pass; first = $first; last = $last }

                # Collection: the released subtree is actually reaped, through
                # the same production `Runtime::collect_garbage` accounting the
                # arena receipts already treat as authoritative.
                $stats = [regex]::Match($output, 'ortet: receipt collection unpinned=(?<unpinned>\d+) collected=(?<collected>\d+)')
                if (-not $stats.Success) { throw "Ortet $engine pass $pass reported no collection stats." }
                if ([int]$stats.Groups['collected'].Value -lt 2 -or [int]$stats.Groups['unpinned'].Value -lt 2) {
                    throw "Ortet $engine pass $pass did not confirm collection: $($stats.Value)."
                }

                Assert-Projection -Revisions (Read-Projections -Path $dump) -Engine $engine `
                    -RecordPath (Join-Path $passArtifact 'a11y-assertions.json')

                [pscustomobject]@{
                    engine = $engine
                    pass = $pass
                    digest = "0x$($digest.Groups['digest'].Value)"
                    display_scale = [double]$scale.Groups['scale'].Value
                    physical = "$($scale.Groups['w'].Value)x$($scale.Groups['h'].Value)"
                    css_viewport = "$($scale.Groups['lw'].Value)x$($scale.Groups['lh'].Value)"
                    live_nodes_first = $first
                    live_nodes_last = $last
                    collection = $stats.Value
                    process_exited = $run.exited
                    process_exit_code = $run.code
                } | ConvertTo-Json | Set-Content (Join-Path $passArtifact 'result.json')
            }

            $distinct = @($digests | Sort-Object -Unique)
            [pscustomobject]@{
                engine = $engine
                digests = $digests
                distinct = $distinct
                stable = ($distinct.Count -eq 1)
                censuses = $censuses
            } | ConvertTo-Json -Depth 5 | Set-Content (Join-Path $engineArtifact 'digests.json')
            if ($distinct.Count -ne 1) {
                throw "Ortet $engine produced $($distinct.Count) distinct digests over $Repeats runs: $($distinct -join ', '). This fixture is designed to be stable; do not average it."
            }
            $summary += [pscustomobject]@{ engine = $engine; digest = "0x$($distinct[0])"; runs = $Repeats }

            # Deliberate failing control: an unmet completion condition under a
            # short deadline must fail the run rather than silently succeed.
            $failArtifact = Join-Path $engineArtifact 'timeout'
            New-Item -ItemType Directory -Path $failArtifact -Force | Out-Null
            $failLog = Join-Path $failArtifact 'receipt.log'
            $control = Invoke-Ortet -Exe $exe -Log $failLog -GraceMs 60000 -Arguments @(
                '--url', $url, '--engine', $engine, '--size', $Size,
                '--artifact', (Join-Path $failArtifact 'receipt.png'),
                '--expect-heading', $impossible, '--timeout-ms', $FailureTimeoutMs
            )
            if (-not $control.exited) { throw "Ortet $engine failure control did not exit; see $failLog." }
            if ($control.exited -and $control.code -eq 0) { throw "Ortet $engine deliberate-failure control unexpectedly succeeded." }
            if ($control.output -notmatch [regex]::Escape("was absent before the ${FailureTimeoutMs}ms deadline")) {
                throw "Ortet $engine failure control did not report its bounded deadline."
            }
            if ($control.output -match [regex]::Escape("semantic heading")) {
                throw "Ortet $engine failure control reported a semantic completion it must not have."
            }
            [pscustomobject]@{
                process_exited = $control.exited
                process_exit_code = $control.code
                expected_failure = $true
            } | ConvertTo-Json | Set-Content (Join-Path $failArtifact 'result.json')
        }

        $summary | ConvertTo-Json -Depth 5 | Set-Content (Join-Path $artifact 'summary.json')
        if (@($summary | Select-Object -ExpandProperty digest -Unique).Count -ne 1) {
            Write-Warning 'The two engines produced different digests; both are recorded in summary.json.'
        }
    }
    finally {
        if ($server -and -not $server.HasExited) { Stop-Process -Id $server.Id -Force }
    }

    $after = Get-SourceIdentity
    $after | Set-Content (Join-Path $artifact 'source-identity-after.json')
    if ($before -ne $after) { throw 'Source or dependency state changed during the realms receipt.' }
}
finally {
    Pop-Location
}
exit 0
