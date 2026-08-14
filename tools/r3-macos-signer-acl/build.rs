use std::{env, fs, path::PathBuf};

use sha2::{Digest, Sha256};

fn literal(name: &str, default: &str) -> String {
    let value = env::var(name).unwrap_or_else(|_| default.to_owned());
    assert!(
        !value.is_empty() && !value.contains(['\0', '\n', '\r']),
        "{name} is not a safe build literal"
    );
    println!("cargo:rerun-if-env-changed={name}");
    value
}

fn main() {
    let zero64 = "0".repeat(64);
    let capability_bytes = include_bytes!("../r3-macos-finalizer/capability-v2.json");
    let capability_digest = format!("{:x}", Sha256::digest(capability_bytes));
    if let Ok(supplied) = env::var("R3_CAPABILITY_DIGEST") {
        assert_eq!(
            supplied, capability_digest,
            "R3_CAPABILITY_DIGEST differs from canonical ../r3-macos-finalizer/capability-v2.json"
        );
    }
    println!("cargo:rerun-if-changed=../r3-macos-finalizer/capability-v2.json");
    println!("cargo:rerun-if-env-changed=R3_CAPABILITY_DIGEST");
    let launch_plist_sha256 = literal("R3_FINALIZER_LAUNCHD_PLIST_SHA256", &zero64);

    let body = format!(
        "pub const CAPABILITY_DIGEST: &str = {capability_digest:?};\n\
         pub const LAUNCH_PLIST_SHA256: &str = {launch_plist_sha256:?};\n"
    );
    let out = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR"));
    fs::write(out.join("runner_identity.rs"), body).expect("write runner identity literals");
}
