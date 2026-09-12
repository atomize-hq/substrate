#[test]
fn world_service_main_parks_e3_transition_caps_before_runtime_threads() {
    let source = include_str!("../src/main.rs");

    assert!(!source.contains("#[tokio::main"));
    let internal_exec = source.find("internal_exec::run_landlock_exec").unwrap();
    let parking = source
        .find("park_e3_child_transition_capabilities_before_runtime()?")
        .unwrap();
    let runtime = source
        .find("build_primary_runtime_after_e3_capability_parking()?")
        .unwrap();
    assert!(internal_exec < parking && parking < runtime);
    assert!(source.contains("on_thread_start"));
    assert!(source.contains("verify_e3_transition_capabilities_parked"));
    assert!(source.contains("runtime.block_on(run_world_service())"));
}
