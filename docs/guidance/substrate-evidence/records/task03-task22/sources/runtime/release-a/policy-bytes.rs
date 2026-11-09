// PREVIEW ONLY: compile after owner release against exact Cargo-reported P artifacts.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let policy = substrate_broker::Policy::default();
    let args: Vec<_> = std::env::args_os().collect();
    if args.len() != 3 { return Err("two output files required".into()); }
    std::fs::write(&args[1], serde_json::to_vec(&policy)?)?;
    std::fs::write(&args[2], policy.to_yaml()?.as_bytes())?;
    Ok(())
}
