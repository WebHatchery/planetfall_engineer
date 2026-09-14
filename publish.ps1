# RustGames project publisher wrapper.
# Build/deploy behavior lives in the workspace root publish.ps1.

param(
    [switch]$SkipBuild = $false,
    [switch]$WindowsOnly = $false,
    [switch]$WebGLOnly = $false,
    [switch]$DeployOnly = $false,
    [Alias('p')] [switch]$Production = $false,
    [switch]$FTP = $false,
    [switch]$Archive = $false,
    [switch]$Unarchive = $false,
    [switch]$DryRun = $false
)

$ErrorActionPreference = "Stop"
$rootPublisher = Join-Path (Split-Path $PSScriptRoot -Parent) "publish.ps1"

function Invoke-ReleaseGate {
    param(
        [string]$Name,
        [string[]]$Arguments
    )

    Write-Host "[gate] $Name" -ForegroundColor Yellow
    & cargo @Arguments
    if ($LASTEXITCODE -ne 0) {
        throw "Release gate failed: $Name"
    }
}

if (-not (Test-Path $rootPublisher)) {
    Write-Error "RustGames root publisher not found: $rootPublisher"
    exit 1
}

if ($Archive -or $Unarchive) {
    & $rootPublisher -RustGameArchive -ProjectDir $PSScriptRoot -Unarchive:$Unarchive -Production:$Production -DryRun:$DryRun
    if (-not $?) { exit 1 }
    exit 0
}

Push-Location -LiteralPath $PSScriptRoot
try {
    if (-not $DeployOnly -and -not $DryRun) {
        Invoke-ReleaseGate "format" @("fmt", "--all", "--", "--check")
        Invoke-ReleaseGate "warnings-denied Clippy" @("clippy", "--all-targets", "--all-features", "--", "-D", "warnings")
        Invoke-ReleaseGate "tests, content validation, and source-line gate" @("test", "--all-targets", "--all-features")
    }

    & $rootPublisher -RustGamePublish -ProjectDir $PSScriptRoot `
        -SkipBuild:$SkipBuild `
        -WindowsOnly:$WindowsOnly `
        -WebGLOnly:$WebGLOnly `
        -DeployOnly:$DeployOnly `
        -Production:$Production `
        -FTP:$FTP `
        -DryRun:$DryRun

    if (-not $?) { exit 1 }
}
finally {
    Pop-Location
}
