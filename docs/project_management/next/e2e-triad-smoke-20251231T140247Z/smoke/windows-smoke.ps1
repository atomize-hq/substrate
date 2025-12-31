Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

Write-Host "== E2E smoke: cargo test -p triad_e2e_smoke_demo =="
cargo test -p triad_e2e_smoke_demo
