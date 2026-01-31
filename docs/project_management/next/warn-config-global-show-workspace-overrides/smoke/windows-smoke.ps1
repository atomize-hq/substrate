Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

if (-not $IsWindows) {
  Write-Host "SKIP: windows-smoke.ps1 intended for Windows"
  exit 0
}

$SubstrateBin = $env:SUBSTRATE_BIN
if ([string]::IsNullOrWhiteSpace($SubstrateBin)) { $SubstrateBin = "substrate" }

try {
  Get-Command $SubstrateBin -ErrorAction Stop | Out-Null
} catch {
  Write-Error "FAIL: substrate binary not found (SUBSTRATE_BIN='$SubstrateBin')"
  exit 2
}

$tmp = Join-Path $env:TEMP ("substrate-smoke-" + [guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Force -Path $tmp | Out-Null

try {
  $homeDir = Join-Path $tmp "home"
  $substrateHome = Join-Path $tmp "substrate-home"
  $ws = Join-Path $tmp "ws"

  New-Item -ItemType Directory -Force -Path $homeDir, $substrateHome, $ws | Out-Null

  $env:HOME = $homeDir
  $env:USERPROFILE = $homeDir
  $env:SUBSTRATE_HOME = $substrateHome

  & $SubstrateBin workspace init $ws | Out-Null

  @"
world:
  caged: false
"@ | Set-Content -Path (Join-Path $substrateHome "config.yaml") -NoNewline

  Push-Location $ws
  try {
    $workspaceYaml = Join-Path $ws ".substrate\workspace.yaml"

    # Empty workspace patch => no note.
    "{}" | Set-Content -Path $workspaceYaml -NoNewline

    $stdoutEmpty = Join-Path $tmp "stdout-empty.txt"
    $stderrEmpty = Join-Path $tmp "stderr-empty.txt"
    & $SubstrateBin config global show 1> $stdoutEmpty 2> $stderrEmpty

    if (Select-String -Path $stderrEmpty -Pattern "substrate: note: workspace config" -Quiet) {
      Write-Error "FAIL: unexpected workspace-override note for empty workspace patch"
      Get-Content -Path $stderrEmpty | Write-Host
      exit 1
    }

    # Non-empty workspace patch => note present.
    @"
world:
  caged: true
"@ | Set-Content -Path $workspaceYaml -NoNewline

    $stdoutNon = Join-Path $tmp "stdout-nonempty.txt"
    $stderrNon = Join-Path $tmp "stderr-nonempty.txt"
    & $SubstrateBin config global show 1> $stdoutNon 2> $stderrNon

    foreach ($pat in @(
      "substrate: note: workspace config",
      "workspace.yaml",
      "overrides global config here",
      "substrate config show --explain"
    )) {
      if (-not (Select-String -Path $stderrNon -Pattern $pat -Quiet)) {
        Write-Error "FAIL: missing expected pattern in stderr: $pat"
        Get-Content -Path $stderrNon | Write-Host
        exit 1
      }
    }

    if (Select-String -Path $stdoutNon -Pattern "substrate: note:" -Quiet) {
      Write-Error "FAIL: stdout is contaminated with note text"
      Get-Content -Path $stdoutNon | Write-Host
      exit 1
    }

    # Invalid YAML workspace patch => command still succeeds and emits note.
    "world: [this is not valid" | Set-Content -Path $workspaceYaml -NoNewline

    $stdoutInvalid = Join-Path $tmp "stdout-invalid.txt"
    $stderrInvalid = Join-Path $tmp "stderr-invalid.txt"
    & $SubstrateBin config global show 1> $stdoutInvalid 2> $stderrInvalid

    if (-not (Select-String -Path $stderrInvalid -Pattern "substrate: note: workspace config" -Quiet)) {
      Write-Error "FAIL: missing workspace-override note (invalid YAML)"
      Get-Content -Path $stderrInvalid | Write-Host
      exit 1
    }

    # --json stdout remains valid JSON when note is present.
    @"
world:
  caged: true
"@ | Set-Content -Path $workspaceYaml -NoNewline

    $stdoutJson = Join-Path $tmp "stdout-json.txt"
    $stderrJson = Join-Path $tmp "stderr-json.txt"
    & $SubstrateBin config global show --json 1> $stdoutJson 2> $stderrJson

    Get-Content -Path $stdoutJson -Raw | ConvertFrom-Json | Out-Null

    if (-not (Select-String -Path $stderrJson -Pattern "substrate: note: workspace config" -Quiet)) {
      Write-Error "FAIL: missing workspace-override note (--json mode)"
      Get-Content -Path $stderrJson | Write-Host
      exit 1
    }
  } finally {
    Pop-Location
  }

  Write-Host "PASS: windows smoke"
  exit 0
} finally {
  Remove-Item -Recurse -Force -Path $tmp
}
