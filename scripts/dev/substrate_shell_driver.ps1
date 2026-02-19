Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Usage {
    Write-Output @"
substrate_shell_driver - host-only wrapper around the substrate CLI

Usage:
  substrate_shell_driver.ps1 [--bin <path>] [--help]
  substrate_shell_driver.ps1 [substrate args...]

Options:
  --bin <path>  Explicit substrate binary to run (env SUBSTRATE_BIN works too)
  --help        Show this message

By default the driver uses SUBSTRATE_BIN if set, otherwise it searches
<repo>\target\debug\substrate.exe and <repo>\target\release\substrate.exe. The
wrapper injects --no-world by default (unless --world/--no-world is already
provided) to prevent world provisioning during tests and developer scripts.
"@
}

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\\..")).Path

$binOverride = $null
$passthrough = @()

for ($i = 0; $i -lt $args.Count; $i++) {
    $arg = $args[$i]
    switch ($arg) {
        "--bin" {
            if ($i + 1 -ge $args.Count) {
                Write-Error "error: --bin requires a path"
                exit 2
            }
            $binOverride = $args[$i + 1]
            $i++
        }
        "--help" {
            Usage
            exit 0
        }
        "--" {
            if ($i + 1 -lt $args.Count) {
                $passthrough += $args[($i + 1)..($args.Count - 1)]
            }
            break
        }
        default {
            $passthrough += $arg
        }
    }
}

$substrateBin = $null
if ($null -ne $binOverride -and $binOverride -ne "") {
    $substrateBin = $binOverride
} elseif ($env:SUBSTRATE_BIN) {
    $substrateBin = $env:SUBSTRATE_BIN
} else {
    $debug = Join-Path $RepoRoot "target\\debug\\substrate.exe"
    $release = Join-Path $RepoRoot "target\\release\\substrate.exe"
    if (Test-Path -LiteralPath $debug) {
        $substrateBin = $debug
    } elseif (Test-Path -LiteralPath $release) {
        $substrateBin = $release
    }
}

if (-not $substrateBin -or -not (Test-Path -LiteralPath $substrateBin)) {
    Write-Error "error: substrate binary not found. Build it (cargo build -p substrate) or set SUBSTRATE_BIN"
    exit 2
}

$needsNoWorld = $true
foreach ($a in $passthrough) {
    if ($a -eq "--world" -or $a -eq "--no-world") {
        $needsNoWorld = $false
        break
    }
}
if ($needsNoWorld) {
    $passthrough = @("--no-world") + $passthrough
}

& $substrateBin @passthrough
exit $LASTEXITCODE
