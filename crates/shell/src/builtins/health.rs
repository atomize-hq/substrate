use super::shim_doctor::{self, ShimDoctorReport, WorldDepsDoctorStatus, WorldDoctorStatus};
use crate::execution::config_model::DoctorDisableSource;
use anyhow::Result;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct HealthReport {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub world_disable_reason: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub world_disable_source: Option<DoctorDisableSource>,
    pub shim: ShimDoctorReport,
    pub summary: HealthSummary,
}

#[derive(Debug, Serialize)]
pub struct HealthSummary {
    pub ok: bool,
    pub missing_managers: Vec<String>,
    pub skip_manager_init: bool,

    pub world_ok: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub world_error: Option<String>,

    pub world_deps_missing: Vec<String>,
    pub world_deps_blocked: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub world_deps_error: Option<String>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub failures: Vec<String>,
}

pub fn run(json_mode: bool, cli_no_world: bool, cli_force_world: bool) -> Result<()> {
    let report = shim_doctor::collect_report(cli_no_world, cli_force_world)?;
    let summary = HealthSummary::from_report(&report);
    let (world_disable_reason, world_disable_source) = world_disable_attribution(&report);
    let payload = HealthReport {
        world_disable_reason,
        world_disable_source,
        shim: report,
        summary,
    };

    if json_mode {
        println!("{}", serde_json::to_string_pretty(&payload)?);
    } else {
        print_health_summary(&payload);
    }

    Ok(())
}

fn world_disable_attribution(
    report: &ShimDoctorReport,
) -> (Option<&'static str>, Option<DoctorDisableSource>) {
    let Some(world) = report.world.as_ref() else {
        return (None, None);
    };
    (
        world.world_disable_reason,
        world.world_disable_source.clone(),
    )
}

fn health_world_disable_reason(report: &HealthReport) -> Option<&'static str> {
    report.world_disable_reason.or_else(|| {
        report
            .shim
            .world
            .as_ref()
            .and_then(|world| world.world_disable_reason)
    })
}

impl HealthSummary {
    fn from_report(report: &ShimDoctorReport) -> Self {
        let missing_managers = report
            .states
            .iter()
            .filter(|state| !state.detected)
            .map(|state| state.name.clone())
            .collect::<Vec<_>>();

        let disabled = report
            .world
            .as_ref()
            .map(|world| world.status == WorldDoctorStatus::Disabled)
            .unwrap_or(false)
            || report
                .world_deps
                .as_ref()
                .map(|section| section.status == WorldDepsDoctorStatus::SkippedDisabled)
                .unwrap_or(false);

        let world_ok = if disabled {
            None
        } else {
            report.world.as_ref().map(|world| world.ok)
        };
        let world_error = if disabled {
            None
        } else {
            report.world.as_ref().and_then(|world| {
                if let Some(err) = &world.error {
                    Some(err.clone())
                } else if !world.ok {
                    world.stderr.clone()
                } else {
                    None
                }
            })
        };

        let mut world_deps_missing = Vec::new();
        let mut world_deps_blocked = Vec::new();
        let mut world_deps_error = None;

        if !disabled {
            if let Some(section) = &report.world_deps {
                if let Some(err) = &section.error {
                    world_deps_error = Some(err.clone());
                } else if let Some(snapshot) = &section.report {
                    if let Some(err) = &snapshot.applied_error {
                        world_deps_error = Some(err.clone());
                    } else {
                        for item in &snapshot.applied {
                            let enabled = item.enabled.unwrap_or(false);
                            if !enabled {
                                continue;
                            }
                            let Some(world) = item.world.as_deref() else {
                                continue;
                            };
                            if world == "missing" {
                                world_deps_missing.push(item.name.clone());
                            } else if world == "blocked" {
                                world_deps_blocked.push(item.name.clone());
                            }
                        }
                    }
                }
            }

            world_deps_missing.sort();
            world_deps_missing.dedup();
            world_deps_blocked.sort();
            world_deps_blocked.dedup();
        }

        let mut failures = Vec::new();
        if report.skip_all_requested {
            failures.push("manager init skipped via SUBSTRATE_SKIP_MANAGER_INIT".to_string());
        }
        if let Some(false) = world_ok {
            failures.push("world backend health check failed".to_string());
        }
        if let Some(err) = &world_error {
            if !err.is_empty() {
                failures.push(format!("world backend error: {err}"));
            }
        } else if report.world.is_none() {
            failures.push("world backend status unavailable".to_string());
        }
        if let Some(err) = &world_deps_error {
            failures.push(format!("world deps unavailable: {err}"));
        }
        if !world_deps_missing.is_empty() {
            failures.push(format!(
                "world deps missing (enabled): {}",
                world_deps_missing.join(", ")
            ));
        }
        if !world_deps_blocked.is_empty() {
            failures.push(format!(
                "world deps blocked (manual): {}",
                world_deps_blocked.join(", ")
            ));
        }

        let ok = failures.is_empty();

        Self {
            ok,
            missing_managers,
            skip_manager_init: report.skip_all_requested,
            world_ok,
            world_error,
            world_deps_missing,
            world_deps_blocked,
            world_deps_error,
            failures,
        }
    }
}

fn print_health_summary(report: &HealthReport) {
    println!("== substrate health ==");
    let total = report.shim.states.len();
    let detected = report
        .shim
        .states
        .iter()
        .filter(|state| state.detected)
        .count();
    println!("Managers detected: {detected}/{total}");
    if !report.summary.missing_managers.is_empty() {
        println!(
            "  Not detected on host (info): {}",
            report.summary.missing_managers.join(", ")
        );
        println!("    Install them locally if you expect Substrate to manage them.");
    }
    if report.summary.skip_manager_init {
        println!("  Manager init skipped via SUBSTRATE_SKIP_MANAGER_INIT");
    }

    if let Some(reason) = health_world_disable_reason(report) {
        println!("{reason}");
    }

    if report
        .shim
        .world
        .as_ref()
        .map(|world| world.status == WorldDoctorStatus::Disabled)
        .unwrap_or(false)
    {
        println!("World backend: disabled");
        println!("  Next: run `substrate world enable` to provision");
    } else {
        match report.summary.world_ok {
            Some(true) => println!("World backend: healthy"),
            Some(false) => println!("World backend: needs attention"),
            None => println!("World backend: unknown"),
        }
        if let Some(err) = &report.summary.world_error {
            println!("  Error: {err}");
        }
    }

    if report
        .shim
        .world_deps
        .as_ref()
        .map(|section| section.status == WorldDepsDoctorStatus::SkippedDisabled)
        .unwrap_or(false)
    {
        println!("World deps: skipped (world disabled)");
    } else if let Some(err) = &report.summary.world_deps_error {
        println!("World deps: unavailable ({})", err.trim());
    } else if report.summary.world_deps_missing.is_empty()
        && report.summary.world_deps_blocked.is_empty()
    {
        println!("World deps: all enabled deps present");
    } else {
        if !report.summary.world_deps_missing.is_empty() {
            println!(
                "World deps: missing ({}): {}",
                report.summary.world_deps_missing.len(),
                report.summary.world_deps_missing.join(", ")
            );
            println!("  Next: run `substrate world deps current sync` then `substrate world deps current list applied`");
        }
        if !report.summary.world_deps_blocked.is_empty() {
            println!(
                "World deps: blocked/manual ({}): {}",
                report.summary.world_deps_blocked.len(),
                report.summary.world_deps_blocked.join(", ")
            );
            println!("  Next: inspect with `substrate world deps current show <name> --explain`");
        }
    }

    println!();
    println!("Hints recorded: {}", report.shim.hints.len());

    if report.summary.failures.is_empty() {
        println!("Overall status: healthy");
    } else {
        println!("Overall status: attention required");
        for failure in &report.summary.failures {
            println!("  - {failure}");
        }
    }

    println!("Run `substrate health --json` or `substrate shim doctor --json` for full details.");
}
