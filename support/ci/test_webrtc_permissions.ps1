# Socket-free regression checks. Never adds a rule to a firewall policy store.
[CmdletBinding()]
param([Parameter(Mandatory)][ValidateSet('Scope', 'Failure')][string]$Case)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$path = Join-Path $PSScriptRoot 'webrtc_permissions.ps1'
$parseErrors = $null; $tokens = $null
$ast = [Management.Automation.Language.Parser]::ParseFile($path, [ref]$tokens, [ref]$parseErrors)
if ($parseErrors.Count) { throw 'Firewall setup script has parse errors.' }
$assignment = @($ast.FindAll({
    param($node)
    $node -is [Management.Automation.Language.AssignmentStatementAst] -and
        $node.Left -is [Management.Automation.Language.VariableExpressionAst] -and
        $node.Left.VariablePath.UserPath -eq 'params'
}, $true))
if ($assignment.Count -ne 1) { throw 'Expected one actual installation parameter definition.' }
$name = 'Genet-RTC-regression-fixture'
$role = @{ Role = 'native'; Program = Join-Path $PSHOME 'powershell.exe'; Port = 'Any' }
$selectedProfile = 'Public'
. ([scriptblock]::Create($assignment[0].Extent.Text))

if ($Case -eq 'Scope') {
    # An unattached INetFwRule validates the actual scope without adding a rule.
    # A WhatIf preview alone does not reach this Windows API validation.
    $rule = New-Object -ComObject HNetCfg.FWRule
    try {
        $rule.RemoteAddresses = $params.RemoteAddress -join ','
        $scope = $rule.RemoteAddresses
        if ('LocalSubnet' -notin $params.RemoteAddress -or
            '127.0.0.0/8' -notin $params.RemoteAddress -or
            'Any' -in $params.RemoteAddress) { throw 'Expected the bounded local-peer scope.' }
    } finally {
        [void][Runtime.InteropServices.Marshal]::FinalReleaseComObject($rule)
    }
    @{ case = $Case; valid = $true; scope = $scope } | ConvertTo-Json -Compress
    exit 0
}

# Exercise the actual installation pipeline against a nonterminating rejection.
# The mock takes precedence over NetSecurity and never calls the real cmdlet.
function New-NetFirewallRule {
    [CmdletBinding()]
    param(
        $Name, $DisplayName, $Group, $Description, $Program, $Direction,
        $Action, $Enabled, $Profile, $Protocol, $LocalPort, $RemoteAddress,
        $EdgeTraversalPolicy
    )
    Write-Error 'Simulated firewall installation rejection'
}
$command = @($ast.FindAll({
    param($node)
    $node -is [Management.Automation.Language.CommandAst] -and
        $node.GetCommandName() -eq 'New-NetFirewallRule'
}, $true))
if ($command.Count -ne 1 -or
    $command[0].Parent -isnot [Management.Automation.Language.PipelineAst]) {
    throw 'Expected one actual firewall installation pipeline.'
}
$ErrorActionPreference = 'Continue'
try {
    & ([scriptblock]::Create($command[0].Parent.Extent.Text))
    throw 'Installation continued after a rejected rule.'
} catch {
    if ($_.Exception.Message -notmatch 'Simulated firewall installation rejection') { throw }
}
@{ case = $Case; stopped = $true } | ConvertTo-Json -Compress
