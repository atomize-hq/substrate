use super::shim_doctor::{self, ShimDoctorReport};
use super::world_deps::{WorldDepGuestState, WorldDepsStatusReport};
use anyhow::Result;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct HealthReport {
    pub shim: ShimDoctorReport,
    pub summary: HealthSummary,
}

#[derive(Debug, Serialize)]
pub struct HealthSummary {
    pub ok: bool,
    pub missing_managers: Vec<String>,
    pub missing_guest_tools: Vec<String>,
    pub skip_manager_init: bool,
    pub world_ok: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub world_disabled_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub world_error: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub failures: Vec<String>,
}

pub fn run(json_mode: bool, cli_no_world: bool, cli_force_world: bool) -> Result<()> {
    let report = shim_doctor::collect_report(cli_no_world, cli_force_world)?;
    let summary = HealthSummary::from_report(&report);
    let payload = HealthReport {
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

impl HealthSummary {
    fn from_report(report: &ShimDoctorReport) -> Self {
        let missing_managers = report
            .states
            .iter()
            .filter(|state| !state.detected)
            .map(|state| state.name.clone())
            .collect::<Vec<_>>();

        let world_ok = report.world.as_ref().map(|world| world.ok);
        let world_disabled_reason = report
            .world_deps
            .as_ref()
            .and_then(|section| section.report.as_ref())
            .and_then(|world_report| world_report.world_disabled_reason.clone());
        let world_error = report.world.as_ref().and_then(|world| {
            if let Some(err) = &world.error {
                Some(err.clone())
            } else if !world.ok {
                world.stderr.clone()
            } else {
                None
            }
        });

        let missing_guest_tools = report
            .world_deps
            .as_ref()
            .and_then(|section| section.report.as_ref())
            .map(extract_missing_tools)
            .unwrap_or_default();

        let mut failures = Vec::new();
        if report.skip_all_requested {
            failures.push("manager init skipped via SUBSTRATE_SKIP_MANAGER_INIT".to_string());
        }
        if !missing_managers.is_empty() {
            failures.push(format!(
                "managers missing detection: {}",
                missing_managers.join(", ")
            ));
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
        if let Some(section) = &report.world_deps {
            if section.report.is_none() {
                if let Some(err) = &section.error {
                    failures.push(format!("world deps unavailable: {err}"));
                }
            }
        }
        if !missing_guest_tools.is_empty() {
            failures.push(format!(
                "guest missing tools: {}",
                missing_guest_tools.join(", ")
            ));
        }

        let ok = failures.is_empty();

        Self {
            ok,
            missing_managers,
            missing_guest_tools,
            skip_manager_init: report.skip_all_requested,
            world_ok,
            world_disabled_reason,
            world_error,
            failures,
        }
    }
}

fn extract_missing_tools(report: &WorldDepsStatusReport) -> Vec<String> {
    report
        .tools
        .iter()
        .filter(|entry| entry.guest.status == WorldDepGuestState::Missing)
        .map(|entry| entry.name.clone())
        .collect()
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
        println!("  Missing: {}", report.summary.missing_managers.join(", "));
    }
    if report.summary.skip_manager_init {
        println!("  Manager init skipped via SUBSTRATE_SKIP_MANAGER_INIT");
    }

    match report.summary.world_ok {
        Some(true) => println!("World backend: healthy"),
        Some(false) => println!("World backend: needs attention"),
        None => println!("World backend: unknown"),
    }
    if let Some(reason) = &report.summary.world_disabled_reason {
        println!("  Reason: {reason}");
    }
    if let Some(err) = &report.summary.world_error {
        println!("  Error: {err}");
    }

    if report.summary.missing_guest_tools.is_empty() {
        println!("Guest tool sync: all present");
    } else {
        println!(
            "Guest tool sync: missing {} ({})",
            report.summary.missing_guest_tools.len(),
            report.summary.missing_guest_tools.join(", ")
        );
    }

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
