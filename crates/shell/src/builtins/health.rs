use super::shim_doctor::{self, ManagerDoctorState, ShimDoctorReport};
use super::world_deps::{WorldDepGuestState, WorldDepsStatusReport};
use anyhow::Result;
use serde::Serialize;
use std::collections::{HashMap, HashSet};

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
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub attention_required_managers: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub world_only_managers: Vec<String>,
    pub skip_manager_init: bool,
    pub world_ok: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub world_disabled_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub world_error: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub failures: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub manager_states: Vec<ManagerStateSummary>,
}

#[derive(Debug, Serialize, Clone)]
pub struct ManagerStateSummary {
    pub name: String,
    pub host_present: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub world: Option<ManagerWorldStatus>,
    pub attention_required: bool,
    pub world_only: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct ManagerWorldStatus {
    pub status: WorldDepGuestState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
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
        let world_report = report
            .world_deps
            .as_ref()
            .and_then(|section| section.report.as_ref());
        let (manager_states, attention_required, world_only) =
            classify_manager_states(&report.states, world_report);
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
        if !attention_required.is_empty() {
            failures.push(format!(
                "managers require world sync: {}",
                attention_required.join(", ")
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
        let ok = failures.is_empty();

        Self {
            ok,
            missing_managers,
            missing_guest_tools,
            attention_required_managers: attention_required,
            world_only_managers: world_only,
            skip_manager_init: report.skip_all_requested,
            world_ok,
            world_disabled_reason,
            world_error,
            failures,
            manager_states,
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

fn classify_manager_states(
    host_states: &[ManagerDoctorState],
    world_report: Option<&WorldDepsStatusReport>,
) -> (Vec<ManagerStateSummary>, Vec<String>, Vec<String>) {
    let mut host_by_name: HashMap<String, &ManagerDoctorState> = HashMap::new();
    let mut seen = HashSet::new();
    let mut ordered_names = Vec::new();
    for state in host_states {
        let key = state.name.to_ascii_lowercase();
        host_by_name.insert(key.clone(), state);
        if seen.insert(key) {
            ordered_names.push(state.name.clone());
        }
    }

    let mut world_by_name: HashMap<String, usize> = HashMap::new();
    if let Some(report) = world_report {
        for (idx, entry) in report.tools.iter().enumerate() {
            let key = entry.name.to_ascii_lowercase();
            world_by_name.insert(key.clone(), idx);
            if seen.insert(key) {
                ordered_names.push(entry.name.clone());
            }
        }
    }

    let mut manager_states = Vec::new();
    let mut attention_required = Vec::new();
    let mut world_only = Vec::new();

    for name in ordered_names {
        let key = name.to_ascii_lowercase();
        let host_state = host_by_name.get(&key);
        let world_entry = world_report.and_then(|report| {
            world_by_name
                .get(&key)
                .and_then(|idx| report.tools.get(*idx))
        });

        let host_present = host_state
            .map(|state| state.detected)
            .or_else(|| world_entry.map(|entry| entry.host_detected))
            .unwrap_or(false);
        let host_reason = host_state.and_then(|state| state.reason.clone());

        let world_status = world_entry.map(|entry| ManagerWorldStatus {
            status: entry.guest.status,
            reason: entry.guest.reason.clone(),
        });

        let world_missing = world_status
            .as_ref()
            .map(|state| {
                matches!(
                    state.status,
                    WorldDepGuestState::Missing | WorldDepGuestState::Unavailable
                )
            })
            .unwrap_or(false);
        let world_present = world_status
            .as_ref()
            .map(|state| state.status == WorldDepGuestState::Present)
            .unwrap_or(false);

        let needs_attention = host_present && world_missing;
        if needs_attention {
            attention_required.push(name.clone());
        }

        let world_only_entry = !host_present && world_present;
        if world_only_entry {
            world_only.push(name.clone());
        }

        manager_states.push(ManagerStateSummary {
            name,
            host_present,
            host_reason,
            world: world_status,
            attention_required: needs_attention,
            world_only: world_only_entry,
        });
    }

    (manager_states, attention_required, world_only)
}
