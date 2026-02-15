mod output;
mod repair;
mod report;

pub use repair::RepairOutcome;
pub(crate) use report::collect_report;
pub use report::ShimDoctorReport;

use anyhow::Result;
use serde_json::to_string_pretty;

pub fn run_doctor(json_mode: bool, cli_no_world: bool, cli_force_world: bool) -> Result<()> {
    let report = collect_report(cli_no_world, cli_force_world)?;
    if json_mode {
        println!("{}", to_string_pretty(&report)?);
    } else {
        output::print_text_report(&report);
    }
    Ok(())
}

pub fn run_repair(manager: &str, auto_confirm: bool) -> Result<RepairOutcome> {
    repair::run_repair(manager, auto_confirm)
}
