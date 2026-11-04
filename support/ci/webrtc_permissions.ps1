# Read-only by default. -Install is an explicit, one-time administrator action.
# Never elevates itself, changes firewall profiles, removes rules, or disables prompts.
[CmdletBinding(SupportsShouldProcess)]
param(
    [ValidateSet('Native', 'Headed', 'All')][string]$Kind = 'All',
    [ValidateSet('chrome', 'firefox', 'both')][string]$Browser = 'both',
    [ValidateSet('Domain', 'Private', 'Public')][string[]]$FirewallProfile = @(),
    [switch]$Install,
    [string]$NativeHost = 'C:\t\cargo-targets\genet\debug\examples\interop_host.exe',
    [string]$NodeProgram
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
if ([Environment]::OSVersion.Platform -ne [PlatformID]::Win32NT) {
    throw 'This preflight is for the Windows RTC receiving profile.'
}
$target = 'C:\t\cargo-targets\genet'
$identity = [Security.Principal.WindowsIdentity]::GetCurrent()
$principal = [Security.Principal.WindowsPrincipal]::new($identity)
$elevated = $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $FirewallProfile.Count) {
    $FirewallProfile = @(Get-NetConnectionProfile | ForEach-Object {
        if ([string]$_.NetworkCategory -eq 'DomainAuthenticated') { 'Domain' }
        else { [string]$_.NetworkCategory }
    } | Sort-Object -Unique)
}
if (-not $FirewallProfile.Count) { throw 'No active network profile; specify -FirewallProfile explicitly.' }
$roles = @()
if ($Kind -in @('Native', 'All')) {
    $roles += @{ Role = 'native'; Program = "$target\rtc-interop\native-browser-tests.exe"; Protocol = 'UDP'; Port = 'Any' }
}
if ($Kind -in @('Headed', 'All')) {
    if (-not $NodeProgram) { $NodeProgram = (Get-Command node -CommandType Application | Select-Object -First 1).Source }
    $roles += @{ Role = 'host'; Program = $NativeHost; Protocol = 'UDP'; Port = 'Any' }
    $roles += @{ Role = 'node'; Program = $NodeProgram; Protocol = 'UDP'; Port = 'Any' }
    if ($Browser -in @('firefox', 'both')) {
        $roles += @{ Role = 'firefox'; Program = 'C:\Program Files\Mozilla Firefox\firefox.exe'; Protocol = 'UDP'; Port = 'Any' }
    }
    if ($Browser -in @('chrome', 'both')) {
        $roles += @{ Role = 'chrome'; Program = 'C:\Program Files\Google\Chrome\Application\chrome.exe'; Protocol = 'UDP'; Port = '5353' }
    }
}
$filters = @(Get-NetFirewallApplicationFilter -PolicyStore ActiveStore)
$results = @()
foreach ($role in $roles) {
    $exists = Test-Path -LiteralPath $role.Program -PathType Leaf
    foreach ($selectedProfile in $FirewallProfile) {
        $allow = @(); $block = @()
        foreach ($filter in @($filters | Where-Object { $_.Program -eq $role.Program })) {
            foreach ($rule in @($filter | Get-NetFirewallRule)) {
                if ([string]$rule.Enabled -ne 'True' -or [string]$rule.Direction -ne 'Inbound') { continue }
                $profiles = @(([string]$rule.Profile -split ',') | ForEach-Object { $_.Trim() })
                if ('Any' -notin $profiles -and $selectedProfile -notin $profiles) { continue }
                $port = $rule | Get-NetFirewallPortFilter
                if ([string]$port.Protocol -notin @('Any', $role.Protocol)) { continue }
                $ports = @($port.LocalPort)
                if ('Any' -notin $ports -and $role.Port -ne 'Any' -and $role.Port -notin $ports) { continue }
                if ([string]$rule.Action -eq 'Block') { $block += $rule.Name; continue }
                if ($role.Port -eq 'Any' -and 'Any' -notin $ports) { continue }
                $address = $rule | Get-NetFirewallAddressFilter
                $interface = $rule | Get-NetFirewallInterfaceFilter
                $service = $rule | Get-NetFirewallServiceFilter
                $security = $rule | Get-NetFirewallSecurityFilter
                # Accept only the simple local-network application rule shape.
                # Other scopes are reported missing rather than guessed sufficient.
                if ([string]$rule.Action -eq 'Allow' -and
                    'Any' -in @($address.LocalAddress) -and
                    ('Any' -in @($address.RemoteAddress) -or 'LocalSubnet' -in @($address.RemoteAddress)) -and
                    'Any' -in @($interface.InterfaceAlias) -and
                    [string]$service.Service -eq 'Any' -and
                    [string]$security.Authentication -eq 'NotRequired') {
                    $allow += $rule.Name
                }
            }
        }
        $covered = $exists -and $allow.Count -gt 0 -and $block.Count -eq 0
        $planned = $Install -and -not $covered -and $block.Count -eq 0 -and $exists
        if ($planned) {
            $name = "Genet-RTC-$($role.Role)-$selectedProfile-UDP"
            $params = @{
                Name = $name; DisplayName = "Genet RTC $($role.Role) ($selectedProfile UDP)"
                Group = 'Genet RTC fixtures'; Description = 'Caller-run local RTC fixtures; exact executable, selected profile and local peers.'
                Program = $role.Program; Direction = 'Inbound'; Action = 'Allow'; Enabled = 'True'
                Profile = $selectedProfile; Protocol = 'UDP'; LocalPort = $role.Port
                RemoteAddress = @('LocalSubnet', '127.0.0.0/8', '::1'); EdgeTraversalPolicy = 'Block'
            }
            if (-not $WhatIfPreference -and -not $elevated) {
                throw 'Install requires an elevated PowerShell, once while you are present. This script never opens UAC itself.'
            }
            if ($PSCmdlet.ShouldProcess("$($role.Program), $selectedProfile, UDP/$($role.Port), local peers", 'Add application firewall rule')) {
                # An existing owned rule with different settings needs review, not replacement.
                New-NetFirewallRule @params | Out-Null
            }
        }
        $results += @{
            role = $role.Role; program = $role.Program; profile = $selectedProfile
            executableExists = $exists; applicationRuleCoverage = $covered
            allowRules = $allow; possibleBlockRules = $block; installPlanned = $planned
        }
    }
}
$missing = @($results | Where-Object { -not $_.applicationRuleCoverage })
if ($Install -and -not $WhatIfPreference) {
    # Re-read effective policy after an explicit installation; a local rule
    # creation is not proof that managed policy merged or accepted it.
    & $PSCommandPath -Kind $Kind -Browser $Browser -FirewallProfile $FirewallProfile -NativeHost $NativeHost -NodeProgram $NodeProgram
    exit $LASTEXITCODE
}
@{
    schema = 1; kind = $Kind; elevated = $elevated; profiles = $FirewallProfile
    applicationRuleCoverage = ($missing.Count -eq 0); results = $results
    scope = 'Read-only application-rule preflight, not a connectivity test or a guarantee against UAC, SmartScreen, antivirus, browser, global firewall or managed-policy restrictions.'
    install = 'Prepare the native binary first. Preview -Kind Native -Install -WhatIf; apply from an elevated PowerShell once, then audit again. Existing block rules require separate review.'
} | ConvertTo-Json -Depth 6 -Compress
if ($missing.Count -and -not $Install) { exit 2 }
exit 0
