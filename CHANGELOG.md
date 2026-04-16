# Changelog

All notable changes to this project are tracked here. Dates use UTC.

## [Unreleased]

### Added

- `substrate health` command with aggregated shim/world summaries plus upgraded
  `substrate shim doctor --json` payloads for support bundle collection.
- Dedicated ChatGPT Codex `backend-api/codex/responses` transport handling in
  the gateway, including stream-native sync/stream assembly, route-specific
  request shaping, and Codex conformance fixtures.

### Docs

- Updated README Quick Start plus INSTALLATION/USAGE/CONFIGURATION to describe
  the pass-through shim model, manager manifest/overlays, shim doctor/repair,
  and the upcoming `substrate world enable` / `world deps` CLI. These sections
  now document how to run doctor commands inside temporary HOMEs, how to apply
  repair snippets safely, and where to find the manifest files.
- Added replay crate module-level docs with a runnable selection example and
  documented the `substrate-common::prelude` for shared types and helpers.
- Added Codex route/auth/conformance contract docs plus OAuth setup/testing
  guidance for ChatGPT Codex integrated and standalone handoff behavior.

### Changed

- Split the `substrate-trace` and `world-windows-wsl` crates into focused
  modules while preserving their public surfaces and platform guards.
- ChatGPT Codex OAuth requests now resolve `ChatGPT-Account-ID` from the
  selected Substrate or standalone auth context, preserve tool-call
  continuations on the Responses route, and fail before upstream when Codex
  identity cannot be resolved.

## [0.2.0-beta.1] - 2025-10-30

### Added

- Cross-platform world execution parity (Windows WSL, macOS Lima, Linux native)
with transport telemetry.
- Beta installer bundles that package world provisioning scripts and host
helpers.
- Updated distribution plan documenting beta release process and artifact
layout.

### Known Issues

- Cross-platform beta build still in soak testing; report regressions with
attached evidence logs.
