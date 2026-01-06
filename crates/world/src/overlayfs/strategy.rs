use anyhow::{Context, Result};
use std::path::Path;
use std::process::{Command, Stdio};
use substrate_common::{
    WorldFsMode, WorldFsStrategy, WorldFsStrategyFallbackReason, WorldFsStrategyProbe,
    WorldFsStrategyProbeResult,
};

use super::OverlayFs;

pub const ENUMERATION_PROBE_ID: &str = "enumeration_v1";
pub const ENUMERATION_PROBE_FILE: &str = ".substrate_enum_probe";

#[derive(Debug, Clone)]
pub struct StrategySelection {
    pub primary: WorldFsStrategy,
    pub final_strategy: WorldFsStrategy,
    pub fallback_reason: WorldFsStrategyFallbackReason,
    pub probe: WorldFsStrategyProbe,
}

#[derive(Debug, Clone)]
struct ProbeOutcome {
    mount_ok: bool,
    probe: WorldFsStrategyProbe,
}

pub fn run_enumeration_probe(
    world_id: &str,
    strategy: WorldFsStrategy,
    project: &Path,
) -> WorldFsStrategyProbe {
    probe_strategy(world_id, strategy, project).probe
}

fn probe_enumeration_in_dir(dir: &Path) -> Result<WorldFsStrategyProbe> {
    let probe_path = dir.join(ENUMERATION_PROBE_FILE);
    let mut failure_reason: Option<String> = None;

    if let Err(err) = std::fs::write(&probe_path, b"probe") {
        failure_reason = Some(format!("failed to create probe file: {err}"));
    } else {
        let output = Command::new("ls")
            .arg("-a1")
            .current_dir(dir)
            .stdin(Stdio::null())
            .output()
            .context("failed to run ls -a1 for enumeration probe")?;

        if !output.status.success() {
            failure_reason = Some(format!(
                "ls -a1 failed with status {}",
                output.status.code().unwrap_or(-1)
            ));
        } else {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let found = stdout.lines().any(|line| line == ENUMERATION_PROBE_FILE);
            if !found {
                failure_reason = Some("probe file missing from directory enumeration".to_string());
            }
        }
    }

    let _ = std::fs::remove_file(&probe_path);

    Ok(WorldFsStrategyProbe {
        id: ENUMERATION_PROBE_ID.to_string(),
        probe_file: ENUMERATION_PROBE_FILE.to_string(),
        result: if failure_reason.is_some() {
            WorldFsStrategyProbeResult::Fail
        } else {
            WorldFsStrategyProbeResult::Pass
        },
        failure_reason,
    })
}

fn probe_strategy(world_id: &str, strategy: WorldFsStrategy, project: &Path) -> ProbeOutcome {
    let probe_id = format!(
        "{world_id}-probe-{}-{}",
        strategy.as_str(),
        uuid::Uuid::now_v7()
    );

    let mut overlay = match OverlayFs::new(&probe_id) {
        Ok(overlay) => overlay,
        Err(err) => {
            return ProbeOutcome {
                mount_ok: false,
                probe: WorldFsStrategyProbe {
                    id: ENUMERATION_PROBE_ID.to_string(),
                    probe_file: ENUMERATION_PROBE_FILE.to_string(),
                    result: WorldFsStrategyProbeResult::Fail,
                    failure_reason: Some(format!("failed to initialize probe overlay: {err}")),
                },
            };
        }
    };

    // ADR-0004 enumeration probe contract: the probe MUST create a file in the merged view,
    // so it must run against a writable overlay regardless of the session fs_mode.
    let mount_res = match strategy {
        WorldFsStrategy::Overlay => overlay.mount_kernel_only(project),
        WorldFsStrategy::Fuse => overlay.mount_fuse_only(project),
        WorldFsStrategy::Host => Err(anyhow::anyhow!("host is not a probeable world fs strategy")),
    };

    let (mount_ok, probe) = match mount_res {
        Ok(merged) => {
            let probe = probe_enumeration_in_dir(&merged).unwrap_or(WorldFsStrategyProbe {
                id: ENUMERATION_PROBE_ID.to_string(),
                probe_file: ENUMERATION_PROBE_FILE.to_string(),
                result: WorldFsStrategyProbeResult::Fail,
                failure_reason: Some("probe execution failed unexpectedly".to_string()),
            });
            (true, probe)
        }
        Err(err) => (
            false,
            WorldFsStrategyProbe {
                id: ENUMERATION_PROBE_ID.to_string(),
                probe_file: ENUMERATION_PROBE_FILE.to_string(),
                result: WorldFsStrategyProbeResult::Fail,
                failure_reason: Some(format!("mount failed: {err:#}")),
            },
        ),
    };

    let _ = overlay.cleanup();
    ProbeOutcome { mount_ok, probe }
}

#[cfg(target_os = "linux")]
fn fuse_available() -> bool {
    std::path::Path::new("/dev/fuse").exists() && which::which("fuse-overlayfs").is_ok()
}

#[cfg(not(target_os = "linux"))]
fn fuse_available() -> bool {
    false
}

pub fn select_strategy(
    world_id: &str,
    project: &Path,
    fs_mode: WorldFsMode,
) -> Result<StrategySelection> {
    let primary = WorldFsStrategy::Overlay;
    let _ = fs_mode;
    let primary_outcome = probe_strategy(world_id, WorldFsStrategy::Overlay, project);

    if primary_outcome.mount_ok && primary_outcome.probe.result == WorldFsStrategyProbeResult::Pass
    {
        return Ok(StrategySelection {
            primary,
            final_strategy: WorldFsStrategy::Overlay,
            fallback_reason: WorldFsStrategyFallbackReason::None,
            probe: primary_outcome.probe,
        });
    }

    let primary_failure = if primary_outcome.mount_ok {
        WorldFsStrategyFallbackReason::PrimaryProbeFailed
    } else {
        WorldFsStrategyFallbackReason::PrimaryMountFailed
    };

    if !fuse_available() {
        anyhow::bail!(
            "WORLD_FS_STRATEGY_UNAVAILABLE fallback_reason={} primary_probe_result={} primary_failure_reason={}",
            WorldFsStrategyFallbackReason::FallbackUnavailable.as_str(),
            primary_outcome.probe.result == WorldFsStrategyProbeResult::Pass,
            primary_outcome
                .probe
                .failure_reason
                .clone()
                .unwrap_or_else(|| "unknown".to_string())
        );
    }

    let fallback_outcome = probe_strategy(world_id, WorldFsStrategy::Fuse, project);
    if fallback_outcome.mount_ok
        && fallback_outcome.probe.result == WorldFsStrategyProbeResult::Pass
    {
        return Ok(StrategySelection {
            primary,
            final_strategy: WorldFsStrategy::Fuse,
            fallback_reason: primary_failure,
            probe: fallback_outcome.probe,
        });
    }

    let fallback_failure = if fallback_outcome.mount_ok {
        WorldFsStrategyFallbackReason::FallbackProbeFailed
    } else {
        WorldFsStrategyFallbackReason::FallbackMountFailed
    };

    anyhow::bail!(
        "WORLD_FS_STRATEGY_UNAVAILABLE fallback_reason={} primary_failure_reason={} fallback_failure_reason={}",
        fallback_failure.as_str(),
        primary_outcome
            .probe
            .failure_reason
            .clone()
            .unwrap_or_else(|| "unknown".to_string()),
        fallback_outcome
            .probe
            .failure_reason
            .clone()
            .unwrap_or_else(|| "unknown".to_string())
    );
}
