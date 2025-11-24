use super::report::{
    ManagerDoctorState, ShimDoctorReport, WorldDepsDoctorSection, WorldDoctorSnapshot,
};
use crate::builtins::world_deps::WorldDepGuestState;

pub(crate) fn print_text_report(report: &ShimDoctorReport) {
    println!("== substrate shim doctor ==");
    println!("Manifest: {}", report.manifest.base.display());
    if let Some(overlay) = &report.manifest.overlay {
        let status = if report.manifest.overlay_exists {
            "present"
        } else {
            "missing"
        };
        println!("Overlay: {} ({})", overlay.display(), status);
    } else {
        println!("Overlay: <not configured>");
    }
    println!(
        "Manager init skip requested: {}",
        bool_str(report.skip_all_requested)
    );
    println!();

    println!("PATH state:");
    println!(
        "  Shim dir: {} ({})",
        report.path.shim_dir.display(),
        if report.path.shim_dir_exists {
            "exists"
        } else {
            "missing"
        }
    );
    println!(
        "  Host PATH includes Substrate shims: {}",
        bool_str(report.path.host_contains_shims)
    );
    let first_entry = report.path.path_first_entry.as_deref().unwrap_or("<empty>");
    println!("  PATH first entry: {}", first_entry);
    println!(
        "  Shim in PATH: {} (first = {})",
        bool_str(report.path.host_contains_shims),
        bool_str(report.path.shim_first_in_path)
    );
    println!(
        "  Legacy bashenv: {} ({})",
        report.path.bashenv_path.display(),
        if report.path.bashenv_exists {
            "exists"
        } else {
            "missing"
        }
    );
    println!();

    println!("Managers:");
    if report.states.is_empty() {
        println!("  (no managers found in manifest)");
    } else {
        print_manager_table(&report.states);
    }
    println!();

    if report.hints.is_empty() {
        println!(
            "Recent hints: none recorded in {}",
            report.trace_log.display()
        );
    } else {
        println!("Recent hints ({}):", report.hints.len());
        for hint in &report.hints {
            println!(
                "  {} – {} ({})",
                hint.name,
                hint.hint,
                hint.last_seen.to_rfc3339()
            );
        }
    }

    println!();
    print_world_section(report.world.as_ref());
    println!();
    print_world_deps_section(report.world_deps.as_ref());
}

fn print_manager_table(states: &[ManagerDoctorState]) {
    let mut name_width = "Name".len();
    for state in states {
        name_width = name_width.max(state.name.len());
    }
    println!(
        "{:<name_width$} {:<9} {:<6} {:<7} Last Hint",
        "Name",
        "Detected",
        "Init",
        "Repair",
        name_width = name_width
    );
    println!(
        "{:-<name_width$} {:->9} {:->6} {:->7} {:-<20}",
        "",
        "",
        "",
        "",
        "",
        name_width = name_width
    );
    for state in states {
        let hint_text = state
            .last_hint
            .as_ref()
            .map(|hint| format!("{} {}", hint.last_seen.to_rfc3339(), hint.hint))
            .unwrap_or_else(|| "-".to_string());
        println!(
            "{:<name_width$} {:<9} {:<6} {:<7} {}",
            state.name,
            bool_str(state.detected),
            bool_str(state.init_sourced),
            bool_str(state.repair_available),
            hint_text,
            name_width = name_width
        );
    }
}

fn print_world_section(section: Option<&WorldDoctorSnapshot>) {
    println!("World backend:");
    match section {
        Some(snapshot) => {
            println!(
                "  Status: {}",
                if snapshot.ok {
                    "healthy"
                } else {
                    "needs attention"
                }
            );
            println!("  Platform: {}", snapshot.platform);
            if let Some(source) = &snapshot.source {
                println!("  Source: {}", source);
            }
            if let Some(code) = snapshot.exit_code {
                println!("  Exit code: {}", code);
            }
            if let Some(err) = &snapshot.error {
                println!("  Error: {}", err);
            } else if let Some(stderr) = &snapshot.stderr {
                if !stderr.is_empty() {
                    println!("  Notes: {}", stderr);
                }
            }
            println!("  Details: run `substrate world doctor --json` for full output.");
        }
        None => println!("  No data available."),
    }
}

fn print_world_deps_section(section: Option<&WorldDepsDoctorSection>) {
    println!("World deps:");
    match section {
        Some(section) => {
            if let Some(source) = &section.source {
                println!("  Source: {}", source);
            }
            if let Some(report) = &section.report {
                println!("  Manifest: {}", report.manifest.base.display());
                if let Some(overlay) = &report.manifest.overlay {
                    let status = if report.manifest.overlay_exists {
                        "present"
                    } else {
                        "missing"
                    };
                    println!("  Overlay: {} ({status})", overlay.display());
                } else {
                    println!("  Overlay: <not configured>");
                }
                if let Some(reason) = &report.world_disabled_reason {
                    println!("  Backend: disabled ({reason})");
                } else {
                    println!("  Backend: enabled");
                }
                if report.tools.is_empty() {
                    println!("  Tools: manifest empty");
                    return;
                }
                let missing: Vec<&str> = report
                    .tools
                    .iter()
                    .filter(|entry| entry.guest.status == WorldDepGuestState::Missing)
                    .map(|entry| entry.name.as_str())
                    .collect();
                if missing.is_empty() {
                    println!("  Guest tools: all present");
                } else {
                    println!(
                        "  Guest tools missing ({}): {}",
                        missing.len(),
                        missing.join(", ")
                    );
                }
            } else if let Some(err) = &section.error {
                println!("  Error: {}", err);
            } else {
                println!("  No data available.");
            }
        }
        None => println!("  No data available."),
    }
}

fn bool_str(value: bool) -> &'static str {
    if value {
        "yes"
    } else {
        "no"
    }
}
