#[test]
fn world_service_main_delegates_to_library_entrypoint() {
    let source = include_str!("../src/main.rs");

    assert!(
        source.contains("run_world_service().await"),
        "main should delegate to run_world_service"
    );

    let line_count = source
        .lines()
        .filter(|line| !line.trim().is_empty())
        .count();
    assert!(
        line_count <= 12,
        "world-service main should stay a thin wrapper around the library (got {line_count} lines)"
    );
}
