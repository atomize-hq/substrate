//! Performance benchmarks for Substrate shim
//!
//! These benchmarks measure key performance characteristics of the shim system:
//! - Path processing and deduplication
//! - JSON serialization for logging
//! - String operations for credential redaction patterns
//! - File system operations
//!
//! Run with: `cargo bench`
//! Generate HTML reports: `cargo bench -- --output-format html`

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use serde_json::json;
use std::collections::HashSet;
use std::ffi::OsString;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use tempfile::TempDir;

/// Benchmark PATH string processing and deduplication
fn bench_path_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("path_operations");

    // Test with realistic PATH values
    let path_cases = [
        "/usr/bin:/bin:/usr/sbin:/sbin",
        "/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin",
        "/home/user/.local/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin:/opt/bin:/snap/bin",
    ];

    for (i, path_str) in path_cases.iter().enumerate() {
        group.bench_with_input(
            BenchmarkId::new("parse_and_dedupe", i),
            path_str,
            |b, path_str| {
                b.iter(|| {
                    // Parse PATH string into components
                    let paths: Vec<PathBuf> = std::env::split_paths(path_str).collect();

                    // Deduplicate paths (common operation in shim)
                    let mut seen = HashSet::new();
                    let deduped: Vec<PathBuf> = paths
                        .into_iter()
                        .filter(|path| seen.insert(path.as_os_str().to_owned()))
                        .collect();

                    criterion::black_box(deduped);
                })
            },
        );
    }

    group.finish();
}

/// Benchmark credential redaction pattern matching
fn bench_credential_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("credential_redaction");

    // Realistic command line arguments
    let test_cases = [
        // Clean case (no redaction needed)
        vec!["git", "status", "--porcelain", "-b"],
        // Simple token case
        vec!["curl", "--token", "abc123", "https://api.example.com"],
        // Complex headers case
        vec![
            "curl",
            "-H",
            "Authorization: Bearer secret123",
            "-H",
            "X-API-Key: key456",
            "https://api.example.com",
        ],
        // Mixed sensitive and normal args
        vec![
            "myapp",
            "--config",
            "/path/to/config",
            "--token",
            "secret",
            "--verbose",
            "true",
        ],
    ];

    for (i, case) in test_cases.iter().enumerate() {
        let args: Vec<String> = case.iter().map(|s| s.to_string()).collect();

        group.bench_with_input(BenchmarkId::new("pattern_matching", i), &args, |b, args| {
            b.iter(|| {
                // Simulate credential redaction logic
                let redacted: Vec<String> = args
                    .iter()
                    .map(|arg| {
                        if arg.contains("token")
                            || arg.contains("password")
                            || arg.contains("secret")
                            || arg.contains("key=")
                            || arg.starts_with("Authorization:")
                            || arg.starts_with("X-API-Key:")
                        {
                            "***".to_string()
                        } else {
                            arg.clone()
                        }
                    })
                    .collect();

                criterion::black_box(redacted);
            })
        });
    }

    group.finish();
}

/// Benchmark JSON serialization for log entries
fn bench_json_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("json_serialization");

    // Sample log entries of different sizes
    let log_entries = [
        // Small entry
        json!({
            "ts": "2024-01-15T10:30:45.123Z",
            "command": "git",
            "argv": ["git", "status"],
            "exit_code": 0,
            "duration_ms": 50
        }),
        // Medium entry
        json!({
            "ts": "2024-01-15T10:30:45.123Z",
            "command": "npm",
            "argv": ["npm", "install", "express", "lodash", "moment"],
            "cwd": "/Users/developer/my-project",
            "exit_code": 0,
            "duration_ms": 2340,
            "pid": 12345,
            "hostname": "dev-machine",
            "session_id": "018d1234-5678-7abc-def0-123456789abc"
        }),
        // Large entry
        json!({
            "ts": "2024-01-15T10:30:45.123Z",
            "command": "docker",
            "argv": ["docker", "run", "-it", "--rm", "-v", "/host/path:/container/path",
                    "-e", "ENV_VAR=value", "-p", "8080:80", "nginx:latest"],
            "cwd": "/Users/developer/docker-project",
            "exit_code": 0,
            "duration_ms": 15670,
            "pid": 54321,
            "hostname": "docker-host.local",
            "platform": "darwin-aarch64",
            "depth": 1,
            "session_id": "018d1234-5678-7abc-def0-123456789abc",
            "resolved_path": "/usr/local/bin/docker",
            "shim_fingerprint": "sha256:abc123def456789",
            "isatty_stdin": true,
            "isatty_stdout": true,
            "isatty_stderr": false
        }),
    ];

    for (i, entry) in log_entries.iter().enumerate() {
        group.bench_with_input(BenchmarkId::new("serialize", i), entry, |b, entry| {
            b.iter(|| {
                let json_string = serde_json::to_string(entry).unwrap();
                let log_line = format!("{json_string}\n");
                criterion::black_box(log_line);
            })
        });
    }

    group.finish();
}

/// Benchmark file system operations
fn bench_filesystem_ops(c: &mut Criterion) {
    let temp = TempDir::new().unwrap();
    let mut group = c.benchmark_group("filesystem_operations");

    // Create test files
    let test_files = ["git", "npm", "python", "docker", "kubectl"];
    for file in &test_files {
        let path = temp.path().join(file);
        std::fs::write(&path, "#!/bin/bash\necho test").unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&path).unwrap().permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&path, perms).unwrap();
        }
    }

    // Benchmark file existence checks
    group.bench_function("file_exists_checks", |b| {
        let paths: Vec<PathBuf> = test_files.iter().map(|f| temp.path().join(f)).collect();

        b.iter(|| {
            let results: Vec<bool> = paths.iter().map(|path| path.exists()).collect();
            criterion::black_box(results);
        })
    });

    // Benchmark executable permission checks
    #[cfg(unix)]
    group.bench_function("executable_checks", |b| {
        use std::os::unix::fs::PermissionsExt;

        let paths: Vec<PathBuf> = test_files.iter().map(|f| temp.path().join(f)).collect();

        b.iter(|| {
            let results: Vec<bool> = paths
                .iter()
                .map(|path| {
                    path.metadata()
                        .map(|m| m.permissions().mode() & 0o111 != 0)
                        .unwrap_or(false)
                })
                .collect();
            criterion::black_box(results);
        })
    });

    group.finish();
}

/// Benchmark timestamp formatting operations
fn bench_timestamp_formatting(c: &mut Criterion) {
    let mut group = c.benchmark_group("timestamp_formatting");

    group.bench_function("rfc3339_with_millis", |b| {
        let timestamp = SystemTime::now();

        b.iter(|| {
            let dt: chrono::DateTime<chrono::Utc> = timestamp.into();
            let formatted = dt.to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
            criterion::black_box(formatted);
        })
    });

    group.finish();
}

/// Benchmark memory allocation patterns
fn bench_memory_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_allocations");

    // Benchmark vector allocations for argv processing
    group.bench_function("argv_vector_creation", |b| {
        let args = ["command", "arg1", "arg2", "arg3", "arg4", "arg5"];

        b.iter(|| {
            let os_strings: Vec<OsString> = args.iter().map(|s| OsString::from(*s)).collect();

            let processed: Vec<String> = os_strings
                .iter()
                .map(|os_str| os_str.to_string_lossy().to_string())
                .collect();

            criterion::black_box(processed);
        })
    });

    // Benchmark string concatenation for PATH building
    group.bench_function("path_string_building", |b| {
        let components = [
            "/opt/homebrew/bin",
            "/usr/local/bin",
            "/usr/bin",
            "/bin",
            "/usr/sbin",
            "/sbin",
        ];

        b.iter(|| {
            let path_string = components.join(":");
            criterion::black_box(path_string);
        })
    });

    group.finish();
}

/// Benchmark overall shim processing simulation
fn bench_shim_simulation(c: &mut Criterion) {
    let mut group = c.benchmark_group("shim_simulation");
    group.measurement_time(Duration::from_secs(10));

    // Simulate the key operations a shim performs
    group.bench_function("complete_cycle", |b| {
        let command = "git";
        let args = ["status", "--porcelain"];
        let cwd = "/Users/developer/project";
        let original_path = "/usr/bin:/bin";

        b.iter(|| {
            // 1. Parse PATH
            let paths: Vec<PathBuf> = std::env::split_paths(original_path).collect();

            // 2. Create log entry
            let log_entry = json!({
                "ts": chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
                "command": command,
                "argv": std::iter::once(command).chain(args.iter().copied()).collect::<Vec<_>>(),
                "cwd": cwd,
                "exit_code": 0,
                "duration_ms": 123,
                "pid": std::process::id(),
            });

            // 3. Serialize to JSON
            let json_string = serde_json::to_string(&log_entry).unwrap();
            let log_line = format!("{json_string}\n");

            criterion::black_box((paths, log_line));
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_path_operations,
    bench_credential_patterns,
    bench_json_operations,
    bench_filesystem_ops,
    bench_timestamp_formatting,
    bench_memory_patterns,
    bench_shim_simulation
);

criterion_main!(benches);
