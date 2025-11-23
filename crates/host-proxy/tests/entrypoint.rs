#[cfg(unix)]
#[test]
fn host_proxy_main_delegates_to_library_entrypoint() {
    let source = include_str!("../src/main.rs");

    assert!(
        source.contains("run_host_proxy().await"),
        "main should delegate to run_host_proxy"
    );

    let line_count = source
        .lines()
        .filter(|line| !line.trim().is_empty())
        .count();
    assert!(
        line_count <= 20,
        "host-proxy main should remain a thin wrapper (got {line_count} lines)"
    );
}

#[cfg(not(unix))]
#[test]
fn host_proxy_main_reports_platform_unsupported() {
    let source = include_str!("../src/main.rs");
    assert!(
        source.contains("not supported on this platform"),
        "non-Unix main should clearly report unsupported platform"
    );
}
