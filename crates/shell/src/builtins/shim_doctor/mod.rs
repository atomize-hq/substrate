mod output;
mod repair;
mod report;

pub use repair::RepairOutcome;
#[cfg_attr(
    unix,
    allow(
        unused_imports,
        reason = "Unix checked-projection compatibility is retained until the R2-3 migration"
    )
)]
pub(crate) use report::collect_report;
#[cfg(unix)]
pub(crate) use report::collect_report_for_context;
pub use report::{ShimDoctorReport, WorldDepsDoctorStatus, WorldDoctorStatus};

use anyhow::Result;
use serde_json::to_string_pretty;
#[cfg(unix)]
use transport_api_types::InstallBootstrapContextCarrierV1;

#[cfg(unix)]
pub fn run_doctor(
    json_mode: bool,
    cli_no_world: bool,
    cli_force_world: bool,
    install_context: &InstallBootstrapContextCarrierV1,
) -> Result<()> {
    let report =
        report::collect_report_for_context(cli_no_world, cli_force_world, install_context)?;
    if json_mode {
        println!("{}", to_string_pretty(&report)?);
    } else {
        output::print_text_report(&report);
    }
    Ok(())
}

#[cfg(not(unix))]
pub fn run_doctor(json_mode: bool, cli_no_world: bool, cli_force_world: bool) -> Result<()> {
    let report = collect_report(cli_no_world, cli_force_world)?;
    if json_mode {
        println!("{}", to_string_pretty(&report)?);
    } else {
        output::print_text_report(&report);
    }
    Ok(())
}

#[cfg(unix)]
pub fn run_repair(
    manager: &str,
    auto_confirm: bool,
    install_context: &InstallBootstrapContextCarrierV1,
) -> Result<RepairOutcome> {
    repair::run_repair(manager, auto_confirm, install_context)
}

#[cfg(not(unix))]
pub fn run_repair(manager: &str, auto_confirm: bool) -> Result<RepairOutcome> {
    repair::run_repair(manager, auto_confirm)
}
