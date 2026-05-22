use std::process::Command;

#[test]
fn helper_fails_closed_with_exit_4_on_invalid_enforcement_plan() {
    let bin = env!("CARGO_BIN_EXE_world-service");

    let output = Command::new(bin)
        .arg(world_service::internal_exec::LANDLOCK_EXEC_ARG)
        .env("SUBSTRATE_WORLD_FS_ENFORCEMENT_PLAN_B64", "not-base64")
        .output()
        .expect("run world-service helper");

    assert_eq!(output.status.code(), Some(4));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("world-fs-enforcement-plan"),
        "stderr should include feature tag; got: {stderr}"
    );
}
