param()
$ErrorActionPreference = "Stop"

$isWindows = $PSVersionTable.PSPlatform -eq "Win32NT"
if (-not $isWindows) {
  Write-Host "SKIP: world-sync Windows smoke (not Windows)"
  exit 0
}

function Assert-ContainsInsensitive([string]$haystack, [string]$needle) {
  if ($haystack.ToLowerInvariant().Contains($needle.ToLowerInvariant())) { return }
  throw "expected output to contain '$needle' (case-insensitive). Output:`n$haystack"
}

$slice = if ($env:SUBSTRATE_SMOKE_SLICE_ID) { $env:SUBSTRATE_SMOKE_SLICE_ID } else { "WS7" }

$ws = Join-Path $env:TEMP ("substrate-world-sync-" + [guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Force -Path $ws | Out-Null
Push-Location $ws
try {
  & substrate workspace init . *> $null

  switch ($slice) {
    "WS2" {
      $out = & substrate workspace sync --dry-run 2>&1 | Out-String
      $code = $LASTEXITCODE
      if ($code -ne 4) { throw "expected exit 4, got $code. Output:`n$out" }
      Assert-ContainsInsensitive $out "unsupported on windows"
      Write-Host "OK: world-sync Windows smoke ($slice)"
    }
    "WS5" {
      $out = & substrate workspace sync --dry-run --direction both 2>&1 | Out-String
      $code = $LASTEXITCODE
      if ($code -ne 4) { throw "expected exit 4, got $code. Output:`n$out" }
      Assert-ContainsInsensitive $out "unsupported on windows"
      Write-Host "OK: world-sync Windows smoke ($slice)"
    }
    "WS7" {
      & substrate workspace checkpoint --message "smoke" *> $null
      if ($LASTEXITCODE -ne 0) { throw "checkpoint expected exit 0, got $LASTEXITCODE" }

      Set-Content -LiteralPath (Join-Path $ws "mutation.txt") -Value "mutated" -Encoding UTF8

      $out = & substrate workspace rollback last 2>&1 | Out-String
      $code = $LASTEXITCODE
      if ($code -ne 5) { throw "expected rollback without --force to exit 5, got $code. Output:`n$out" }

      & substrate workspace rollback last --force *> $null
      if ($LASTEXITCODE -ne 0) { throw "rollback --force expected exit 0, got $LASTEXITCODE" }
      if (Test-Path -LiteralPath (Join-Path $ws "mutation.txt")) { throw "expected mutation.txt to be removed after rollback --force" }
      Write-Host "OK: world-sync Windows smoke ($slice)"
    }
    default {
      throw "unsupported SUBSTRATE_SMOKE_SLICE_ID=$slice"
    }
  }
}
finally {
  Pop-Location
  Remove-Item -Recurse -Force $ws
}
