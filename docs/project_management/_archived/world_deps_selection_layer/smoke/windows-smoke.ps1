$ErrorActionPreference = "Stop"

if (-not $IsWindows) {
  Write-Host "SKIP: world_deps_selection_layer Windows smoke (not Windows)"
  exit 0
}

if (-not (Get-Command substrate -ErrorAction SilentlyContinue)) {
  Write-Error "FAIL: missing required dependency: substrate"
  exit 3
}

function Run-Expect {
  param(
    [int]$ExpectedExit,
    [string]$ExpectedSubstring,
    [Parameter(Mandatory = $true)]
    [string[]]$Command
  )

  $output = & $Command[0] @($Command[1..($Command.Length - 1)]) 2>&1 | Out-String
  $code = $LASTEXITCODE

  if ($code -ne $ExpectedExit) {
    Write-Error ("FAIL: expected exit {0}, got {1}: {2}`n{3}" -f $ExpectedExit, $code, ($Command -join " "), $output)
    exit 1
  }

  if ($ExpectedSubstring -ne "" -and ($output -notlike ("*" + $ExpectedSubstring + "*"))) {
    Write-Error ("FAIL: expected output substring not found: {0}`nCMD: {1}`n{2}" -f $ExpectedSubstring, ($Command -join " "), $output)
    exit 1
  }
}

$help = & substrate world deps --help 2>$null | Out-String
if ($help -notmatch "\bstatus\b" -or $help -notmatch "\bsync\b" -or $help -notmatch "\binstall\b") {
  Write-Error "FAIL: substrate world deps subcommands missing (expected status/sync/install)"
  exit 1
}
if ($help -notmatch "\binit\b" -or $help -notmatch "\bselect\b") {
  Write-Error "FAIL: substrate world deps subcommands missing (expected init/select for WDL0)"
  exit 1
}

$hasProvision = ($help -match "\bprovision\b")

$tmpRoot = Join-Path $env:TEMP ("substrate-wdl-smoke-" + [guid]::NewGuid().ToString("N"))
$workspace = Join-Path $tmpRoot "workspace"
$substrateHome = Join-Path $tmpRoot "substrate-home"

try {
  New-Item -ItemType Directory -Force -Path $workspace | Out-Null
  New-Item -ItemType Directory -Force -Path $substrateHome | Out-Null

  $env:SUBSTRATE_HOME = $substrateHome
  Set-Location $workspace

  Write-Host "== WDL0: selection missing no-op (prove no world calls via invalid socket) =="
  Remove-Item -Recurse -Force -ErrorAction SilentlyContinue .substrate
  Remove-Item -Force -ErrorAction SilentlyContinue (Join-Path $env:SUBSTRATE_HOME "world-deps.selection.yaml")
  $env:SUBSTRATE_WORLD_SOCKET = Join-Path $tmpRoot "does-not-exist.sock"

  Run-Expect -ExpectedExit 0 -ExpectedSubstring "world deps not configured (selection file missing)" -Command @("substrate", "world", "deps", "status")
  Run-Expect -ExpectedExit 0 -ExpectedSubstring "world deps not configured (selection file missing)" -Command @("substrate", "world", "deps", "sync")
  Run-Expect -ExpectedExit 0 -ExpectedSubstring "world deps not configured (selection file missing)" -Command @("substrate", "world", "deps", "install", "nvm")
  if ($hasProvision) {
    Run-Expect -ExpectedExit 0 -ExpectedSubstring "world deps not configured (selection file missing)" -Command @("substrate", "world", "deps", "provision")
  }

  Write-Host "== WDL0: configured-but-empty selection is valid and makes no world calls =="
  Run-Expect -ExpectedExit 0 -ExpectedSubstring "" -Command @("substrate", "world", "deps", "init", "--workspace", "--force")

  $statusJson = (& substrate world deps status --json) | ConvertFrom-Json
  if (-not $statusJson.selection.configured) { throw "expected selection.configured=true" }
  if ($statusJson.selection.active_scope -ne "workspace") { throw "expected selection.active_scope=workspace" }
  if ($statusJson.selection.active_path -ne ".substrate/world-deps.selection.yaml") { throw "expected selection.active_path=.substrate/world-deps.selection.yaml" }
  if ($statusJson.selection.selected.Count -ne 0) { throw "expected empty selection.selected" }

  Run-Expect -ExpectedExit 0 -ExpectedSubstring "No tools selected; nothing to do." -Command @("substrate", "world", "deps", "sync")
  Run-Expect -ExpectedExit 2 -ExpectedSubstring "tool not selected" -Command @("substrate", "world", "deps", "install", "nvm")

  Write-Host "== WDL0: select updates scope deterministically =="
  Run-Expect -ExpectedExit 0 -ExpectedSubstring "" -Command @("substrate", "world", "deps", "select", "--workspace", "nvm", "bun")
  $statusJson = (& substrate world deps status --json) | ConvertFrom-Json
  if ($statusJson.selection.selected -notcontains "nvm") { throw "expected selection.selected to contain nvm" }
  if ($statusJson.selection.selected -notcontains "bun") { throw "expected selection.selected to contain bun" }

  Write-Host "== WDL1/WDL2: backend-required checks (capability-gated) =="
  Remove-Item Env:SUBSTRATE_WORLD_SOCKET -ErrorAction SilentlyContinue

  $statusAll = (& substrate world deps status --all --json) | ConvertFrom-Json
  $installClassPresent = $false
  if ($statusAll.tools.Count -gt 0) {
    $installClassPresent = ($null -ne $statusAll.tools[0].PSObject.Properties["install_class"])
  }

  if ($installClassPresent) {
    Write-Host "OK: WDL1 capability detected (install_class present)"

    $overlay = @"
version: 2
managers:
  - name: wdl-smoke-system-packages
    guest_detect:
      command: "dpkg -s cowsay >/dev/null 2>&1"
    guest_install:
      class: system_packages
      system_packages:
        apt:
          - cowsay
"@
    $overlayPath = Join-Path $env:SUBSTRATE_HOME "manager_hooks.local.yaml"
    Set-Content -LiteralPath $overlayPath -Value $overlay -Encoding UTF8

    Run-Expect -ExpectedExit 0 -ExpectedSubstring "" -Command @("substrate", "world", "deps", "init", "--workspace", "--force")
    Run-Expect -ExpectedExit 0 -ExpectedSubstring "" -Command @("substrate", "world", "deps", "select", "--workspace", "wdl-smoke-system-packages")

    $doctor = (& substrate world doctor --json) | ConvertFrom-Json
    if (-not $doctor.world.ok) {
      Write-Error "FAIL: world backend not ready for backend-required smoke steps (expected world.ok=true). Remediation: run scripts/windows/wsl-warm.ps1 and re-check substrate world doctor --json."
      exit 3
    }

    $statusAll = (& substrate world deps status --all --json) | ConvertFrom-Json
    $tool = $statusAll.tools | Where-Object { $_.name -eq "wdl-smoke-system-packages" } | Select-Object -First 1
    if ($null -eq $tool) { throw "expected wdl-smoke-system-packages to appear in status --all --json tools[]" }
    if ($tool.install_class -ne "system_packages") { throw "expected install_class=system_packages for wdl-smoke-system-packages" }

    $preGuestStatus = $tool.guest.status
    if ($preGuestStatus -eq "skipped") {
      Run-Expect -ExpectedExit 4 -ExpectedSubstring "substrate world deps provision" -Command @("substrate", "world", "deps", "sync")
    } elseif ($preGuestStatus -eq "present") {
      Run-Expect -ExpectedExit 0 -ExpectedSubstring "" -Command @("substrate", "world", "deps", "sync")
    } else {
      throw ("unexpected guest.status for wdl-smoke-system-packages: " + $preGuestStatus)
    }

    if ($hasProvision) {
      Write-Host "OK: WDL2 capability detected (provision present)"
      Run-Expect -ExpectedExit 0 -ExpectedSubstring "cowsay" -Command @("substrate", "world", "deps", "provision", "--dry-run")
      Run-Expect -ExpectedExit 0 -ExpectedSubstring "system packages installed" -Command @("substrate", "world", "deps", "provision")
      Run-Expect -ExpectedExit 0 -ExpectedSubstring "system packages installed" -Command @("substrate", "world", "deps", "provision")

      $statusAll = (& substrate world deps status --all --json) | ConvertFrom-Json
      $tool = $statusAll.tools | Where-Object { $_.name -eq "wdl-smoke-system-packages" } | Select-Object -First 1
      if ($tool.guest.status -ne "present") { throw ("expected guest.status=present after provision, got: " + $tool.guest.status) }

      Run-Expect -ExpectedExit 0 -ExpectedSubstring "" -Command @("substrate", "world", "deps", "sync")
    } else {
      Write-Host "INFO: WDL2 capability not detected (provision absent); skipping provisioning assertions"
    }
  } else {
    Write-Host "INFO: WDL1 capability not detected (install_class absent); skipping WDL1/WDL2 assertions"
  }

  Write-Host "OK: world_deps_selection_layer Windows smoke"
  exit 0
} finally {
  try { Set-Location $env:TEMP } catch {}
  try { Remove-Item -Recurse -Force -ErrorAction SilentlyContinue $tmpRoot } catch {}
}
