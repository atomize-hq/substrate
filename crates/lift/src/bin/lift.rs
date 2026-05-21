#![allow(unused_crate_dependencies)]

fn main() {
    #[cfg(feature = "cli")]
    {
        if let Err(err) = substrate_lift::run_cli() {
            eprintln!("{err}");
            std::process::exit(1);
        }
    }

    #[cfg(not(feature = "cli"))]
    {
        eprintln!("lift was built without the 'cli' feature");
        std::process::exit(1);
    }
}
