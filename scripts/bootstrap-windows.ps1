[CmdletBinding()]
param(
    [switch]$SkipValidation
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path

function Write-Step {
    param([string]$Message)

    Write-Host "==> $Message"
}

function Get-Executable {
    param([string]$Name)

    return Get-Command $Name -ErrorAction SilentlyContinue | Select-Object -First 1
}

function Add-PathEntry {
    param([string]$PathEntry)

    if (-not $PathEntry -or -not (Test-Path $PathEntry)) {
        return $false
    }

    $separator = [System.IO.Path]::PathSeparator
    $entries = @($env:PATH -split [regex]::Escape([string]$separator))

    if ($entries -contains $PathEntry) {
        return $false
    }

    $env:PATH = "$PathEntry$separator$env:PATH"
    return $true
}

function Import-BatchEnvironment {
    param(
        [string]$BatchFile,
        [string[]]$Arguments = @()
    )

    if (-not (Test-Path $BatchFile)) {
        return $false
    }

    $argumentString = ($Arguments | ForEach-Object { $_.Trim() } | Where-Object { $_ }) -join " "
    $command = if ($argumentString) {
        'call "{0}" {1} >nul && set' -f $BatchFile, $argumentString
    }
    else {
        'call "{0}" >nul && set' -f $BatchFile
    }

    $lines = & cmd.exe /d /c $command
    if ($LASTEXITCODE -ne 0) {
        return $false
    }

    foreach ($line in $lines) {
        if ($line -match "^(.*?)=(.*)$") {
            Set-Item -Path "Env:$($matches[1])" -Value $matches[2]
        }
    }

    return $true
}

function Import-MsvcBuildEnvironment {
    if (Get-Executable "link.exe") {
        return $true
    }

    $vswhereRoot = ${env:ProgramFiles(x86)}
    if (-not $vswhereRoot) {
        return $false
    }

    $vswhere = Join-Path $vswhereRoot "Microsoft Visual Studio\Installer\vswhere.exe"
    if (-not (Test-Path $vswhere)) {
        return $false
    }

    $installationPath = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath | Select-Object -First 1
    if ($installationPath) {
        $installationPath = $installationPath.Trim()
    }

    if (-not $installationPath) {
        return $false
    }

    $candidates = @(
        @{
            Path      = Join-Path $installationPath "Common7\Tools\VsDevCmd.bat"
            Arguments = @("-no_logo")
        },
        @{
            Path      = Join-Path $installationPath "VC\Auxiliary\Build\vcvars64.bat"
            Arguments = @()
        }
    )

    foreach ($candidate in $candidates) {
        if (Import-BatchEnvironment -BatchFile $candidate.Path -Arguments $candidate.Arguments) {
            return $true
        }
    }

    return $false
}

Write-Step "Preparing Windows developer shell for CPX"

$defaultCargoBin = Join-Path $env:USERPROFILE ".cargo\bin"
if (-not (Get-Executable "cargo.exe") -and (Add-PathEntry -PathEntry $defaultCargoBin)) {
    Write-Step "Added $defaultCargoBin to PATH for this PowerShell session."
}

if (-not (Get-Executable "cargo.exe")) {
    $message = @"
Rust tooling is not available in this shell.
Expected cargo.exe on PATH or at:
  $defaultCargoBin

Next steps:
  1. Install the stable MSVC Rust toolchain (rustup default stable-x86_64-pc-windows-msvc).
  2. Re-open PowerShell or add $defaultCargoBin to PATH.
  3. Re-run .\scripts\bootstrap-windows.ps1
"@
    throw $message
}

if (-not (Get-Executable "link.exe") -and (Import-MsvcBuildEnvironment)) {
    Write-Step "Imported Visual Studio C++ build tools into the current session."
}

$link = Get-Executable "link.exe"
if (-not $link) {
    $message = @"
Microsoft C++ build tools are not available in this shell.
Expected link.exe after loading Visual Studio Build Tools.

Next steps:
  1. Install Visual Studio Build Tools with the Desktop development with C++ workload.
  2. Re-run .\scripts\bootstrap-windows.ps1 from a new PowerShell session.
"@
    throw $message
}

Push-Location $repoRoot
try {
    Write-Step "Toolchain summary"
    & cargo --version
    & rustc --version

    $rustup = Get-Executable "rustup.exe"
    if ($rustup) {
        & rustup show active-toolchain
    }

    Write-Host "link.exe -> $($link.Source)"

    if (-not $SkipValidation) {
        Write-Step "Running local validation"
        & (Join-Path $PSScriptRoot "validate-local.ps1")
    }
}
finally {
    Pop-Location
}
