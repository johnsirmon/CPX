[CmdletBinding()]
param(
    [switch]$WriteFormatting,
    [switch]$SkipCorpusGate
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path

function Write-Step {
    param([string]$Message)

    Write-Host "==> $Message"
}

function Invoke-RepoCommand {
    param(
        [string]$FilePath,
        [string[]]$Arguments
    )

    $renderedCommand = ((@($FilePath) + $Arguments) -join " ")
    Write-Step $renderedCommand

    & $FilePath @Arguments
    if ($LASTEXITCODE -ne 0) {
        throw "Command failed with exit code ${LASTEXITCODE}: $renderedCommand"
    }
}

if (-not (Get-Command "cargo.exe" -ErrorAction SilentlyContinue)) {
    throw "cargo.exe is not available. Run .\scripts\bootstrap-windows.ps1 first or add %USERPROFILE%\.cargo\bin to PATH."
}

Push-Location $repoRoot
try {
    if ($WriteFormatting) {
        Invoke-RepoCommand -FilePath "cargo" -Arguments @("fmt", "--all")
    }
    else {
        Invoke-RepoCommand -FilePath "cargo" -Arguments @("fmt", "--all", "--check")
    }

    Invoke-RepoCommand -FilePath "cargo" -Arguments @("test", "--workspace")

    if (-not $SkipCorpusGate) {
        Invoke-RepoCommand -FilePath "cargo" -Arguments @("test", "-p", "cpx-core", "--test", "corpus", "corpus_cases_match_expected_outputs")
    }

    Invoke-RepoCommand -FilePath "cargo" -Arguments @("run", "-p", "cpx-cli", "--", "--help")
}
finally {
    Pop-Location
}
