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

function Resolve-CargoPath {
    $cargoCommand = Get-Command "cargo.exe" -ErrorAction SilentlyContinue
    if ($cargoCommand) {
        return $cargoCommand.Source
    }

    $defaultCargoPath = Join-Path $env:USERPROFILE ".cargo\bin\cargo.exe"
    if (Test-Path $defaultCargoPath) {
        return $defaultCargoPath
    }

    throw "cargo.exe is not available. Run .\scripts\bootstrap-windows.ps1 first or add %USERPROFILE%\.cargo\bin to PATH."
}

$cargoPath = Resolve-CargoPath

Push-Location $repoRoot
try {
    if ($WriteFormatting) {
        Invoke-RepoCommand -FilePath $cargoPath -Arguments @("fmt", "--all")
    }
    else {
        Invoke-RepoCommand -FilePath $cargoPath -Arguments @("fmt", "--all", "--check")
    }

    Invoke-RepoCommand -FilePath $cargoPath -Arguments @("test", "--workspace")

    if (-not $SkipCorpusGate) {
        Invoke-RepoCommand -FilePath $cargoPath -Arguments @("test", "-p", "cpx-core", "--test", "corpus", "corpus_cases_match_expected_outputs")
    }

    Invoke-RepoCommand -FilePath $cargoPath -Arguments @("run", "-p", "cpx-cli", "--", "--help")
}
finally {
    Pop-Location
}
