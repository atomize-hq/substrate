mod output;
mod repair;
mod report;

pub use repair::RepairOutcome;
pub(crate) use report::collect_report;
pub use report::{ShimDoctorReport, WorldDepsDoctorStatus, WorldDoctorStatus};

use anyhow::Result;
use serde_json::to_string_pretty;
use transport_api_types::InstallBootstrapContextCarrierV1;

pub fn run_doctor(
    json_mode: bool,
    cli_no_world: bool,
    cli_force_world: bool,
    install_context: &InstallBootstrapContextCarrierV1,
) -> Result<()> {
    let report = report::collect_report_for_context(
        cli_no_world,
        cli_force_world,
        install_context,
    )?;
    if json_mode {
        println!("{}", to_string_pretty(&report)?);
    } else {
        output::print_text_report(&report);
    }
    Ok(())
}

pub fn run_repair(
    manager: &str,
    auto_confirm: bool,
    install_context: &InstallBootstrapContextCarrierV1,
) -> Result<RepairOutcome> {
    repair::run_repair(manager, auto_confirm, install_context)
}
