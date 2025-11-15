use std::process::ExitCode;

fn main() -> ExitCode {
    match substrate_shim::run_shim() {
        Ok(code) => ExitCode::from(code as u8),
        Err(err) => {
            eprintln!("shim error: {err:?}");
            ExitCode::from(126)
        }
    }
}
