#![cfg(target_os = "linux")]

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn private_home() -> tempfile::TempDir {
    let home = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    fs::set_permissions(home.path(), fs::Permissions::from_mode(0o700)).unwrap();
    home
}

#[test]
fn test_cli_status_is_nonmutating_before_shell_initialization() {
    use clap::Parser;
    let home = private_home();
    let config = home.path().join("config.yaml");
    fs::write(&config, b"deliberately not parsed by catalog status\n").unwrap();
    let args = vec![
        "substrate".into(),
        "--install-prefix".into(),
        home.path().as_os_str().to_owned(),
        "agents".into(),
        "catalog".into(),
        "status".into(),
        "--json".into(),
    ];
    let cli = substrate_shell::Cli::try_parse_from(args).unwrap();
    assert_eq!(substrate_shell::run_shell_with_cli(cli).unwrap(), 0);
    assert_eq!(
        fs::read(config).unwrap(),
        b"deliberately not parsed by catalog status\n"
    );
    assert_eq!(
        fs::read_dir(home.path()).unwrap().count(),
        1,
        "status created product state"
    );
}

#[cfg(substrate_catalog_test_support)]
mod http {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::mpsc;
    use std::thread;
    use std::time::Duration;
    use substrate_shell::execution::model_catalog::test_support;

    fn body() -> Vec<u8> {
        serde_json::to_vec(&serde_json::json!({
            "models":{"lab/base":{"id":"lab/base","name":"Base","description":"Metadata"}},
            "providers":{"public":{"id":"public","name":"Public","npm":"never-executed","doc":"https://example.invalid","env":["NEVER_READ"],"models":{
                "offered":{"id":"offered","name":"Offered","description":"Metadata","canonical_model_id":"lab/base","attachment":false,"reasoning":false,"tool_call":true,"open_weights":false,"release_date":"2026-01-01","last_updated":"2026-01-01","modalities":{"input":["text"],"output":["text"]},"limit":{"context":1000,"output":100}}
            }}}
        })).unwrap()
    }

    struct Server {
        origin: String,
        request: mpsc::Receiver<String>,
        thread: thread::JoinHandle<()>,
    }

    impl Server {
        fn new(status: &str, headers: &str, body: Vec<u8>, length: usize, delay: Duration) -> Self {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            let origin = format!(
                "http://{}/catalog.json?type=chat",
                listener.local_addr().unwrap()
            );
            let head = format!("HTTP/1.1 {status}\r\nContent-Length: {length}\r\nConnection: close\r\n{headers}\r\n");
            let (sender, request) = mpsc::channel();
            let thread = thread::spawn(move || {
                let (mut stream, _) = listener.accept().unwrap();
                stream
                    .set_read_timeout(Some(Duration::from_secs(3)))
                    .unwrap();
                let mut request = Vec::new();
                let mut byte = [0];
                while !request.ends_with(b"\r\n\r\n") {
                    assert!(request.len() < 16 * 1024);
                    if stream.read(&mut byte).unwrap_or(0) == 0 {
                        break;
                    }
                    request.push(byte[0]);
                }
                sender.send(String::from_utf8(request).unwrap()).unwrap();
                thread::sleep(delay);
                let _ = stream.write_all(head.as_bytes());
                let _ = stream.write_all(&body);
            });
            Self {
                origin,
                request,
                thread,
            }
        }

        fn ok(body: Vec<u8>) -> Self {
            let length = body.len();
            Self::new(
                "200 OK",
                "ETag: \"fixture-v1\"\r\nLast-Modified: Wed, 21 Oct 2026 07:28:00 GMT\r\n",
                body,
                length,
                Duration::ZERO,
            )
        }

        fn finish(self) -> String {
            let request = self.request.recv_timeout(Duration::from_secs(3)).unwrap();
            self.thread.join().unwrap();
            request
        }
    }

    fn preserved(home: &Path) -> Vec<(PathBuf, Vec<u8>)> {
        ["config.yaml", "agents/local.yaml", "sessions/pinned.json"].into_iter().map(|name| {
            let path = home.join(name);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            let bytes = format!("actual-byte-preservation-marker:{name}:private/model-absent-from-public-catalog").into_bytes();
            fs::write(&path, &bytes).unwrap();
            (path, bytes)
        }).collect()
    }

    #[test]
    fn test_http_client_publisher_and_offline_last_valid_preserve_unrelated_artifacts() {
        // These are byte-preservation sentinels, NOT a genuine V3 Start/Resume qualification.
        let home = private_home();
        let sentinels = preserved(home.path());
        let server = Server::ok(body());
        let status =
            test_support::refresh(home.path(), &server.origin, Duration::from_secs(3)).unwrap();
        assert_eq!(status["available"], true);
        assert_eq!(status["etag"], "\"fixture-v1\"");
        assert_eq!(status["stale"], false);
        assert_eq!(status["source_sha256"], sha256(&body()));
        let request = server.finish().to_ascii_lowercase();
        assert!(request.starts_with("get /catalog.json?type=chat http/1.1\r\n"));
        for forbidden in [
            "authorization:",
            "proxy-authorization:",
            "cookie:",
            "never_read",
        ] {
            assert!(!request.contains(forbidden));
        }
        let path = home.path().join("model-catalog/snapshot.json");
        let before = fs::read(&path).unwrap();
        let server = Server::ok(b"{bad".to_vec());
        assert!(
            test_support::refresh(home.path(), &server.origin, Duration::from_secs(3)).is_err()
        );
        server.finish();
        assert_eq!(fs::read(&path).unwrap(), before);
        let status = test_support::status(home.path()).unwrap();
        assert_eq!(status["available"], true);
        assert_eq!(status["offline"], true);
        let refused = TcpListener::bind("127.0.0.1:0").unwrap();
        let origin = format!("http://{}/refused", refused.local_addr().unwrap());
        drop(refused);
        assert!(test_support::refresh(home.path(), &origin, Duration::from_millis(200)).is_err());
        assert_eq!(fs::read(path).unwrap(), before);
        for (path, bytes) in sentinels {
            assert_eq!(fs::read(path).unwrap(), bytes);
        }
    }

    #[test]
    fn test_http_redirect_truncated_timeout_and_oversize_refuse_without_snapshot() {
        for (status, headers, bytes, length, delay, timeout) in [
            (
                "302 Found",
                "Location: http://127.0.0.1:1/never-follow\r\n",
                Vec::new(),
                0,
                Duration::ZERO,
                Duration::from_secs(3),
            ),
            (
                "200 OK",
                "",
                body(),
                body().len() + 100,
                Duration::ZERO,
                Duration::from_secs(3),
            ),
            (
                "200 OK",
                "",
                body(),
                body().len(),
                Duration::from_millis(150),
                Duration::from_millis(30),
            ),
            (
                "503 Unavailable",
                "",
                Vec::new(),
                0,
                Duration::ZERO,
                Duration::from_secs(3),
            ),
            (
                "200 OK",
                "",
                vec![b' '; 32 * 1024 * 1024 + 1],
                32 * 1024 * 1024 + 1,
                Duration::ZERO,
                Duration::from_secs(3),
            ),
        ] {
            let home = private_home();
            let server = Server::new(status, headers, bytes, length, delay);
            let error = test_support::refresh(home.path(), &server.origin, timeout).unwrap_err();
            if status.starts_with("302") {
                assert!(
                    error.to_string().contains("status 302"),
                    "redirect was followed: {error:#}"
                );
            }
            server.finish();
            assert!(!home.path().join("model-catalog/snapshot.json").exists());
            let status = test_support::status(home.path()).unwrap();
            assert_eq!(status["available"], false);
            assert_eq!(status["offline"], true);
        }
    }

    #[test]
    fn test_http_truncation_timeout_and_oversize_preserve_last_valid() {
        let home = private_home();
        let server = Server::ok(body());
        test_support::refresh(home.path(), &server.origin, Duration::from_secs(3)).unwrap();
        server.finish();
        let path = home.path().join("model-catalog/snapshot.json");
        let before = fs::read(&path).unwrap();
        for (bytes, length, delay, timeout) in [
            (
                body(),
                body().len() + 1,
                Duration::ZERO,
                Duration::from_secs(3),
            ),
            (
                body(),
                body().len(),
                Duration::from_millis(100),
                Duration::from_millis(20),
            ),
            (
                vec![b' '; 32 * 1024 * 1024 + 1],
                32 * 1024 * 1024 + 1,
                Duration::ZERO,
                Duration::from_secs(3),
            ),
        ] {
            let server = Server::new("200 OK", "", bytes, length, delay);
            assert!(test_support::refresh(home.path(), &server.origin, timeout).is_err());
            server.finish();
            assert_eq!(fs::read(&path).unwrap(), before);
            let status = test_support::status(home.path()).unwrap();
            assert_eq!(status["available"], true);
            assert_eq!(status["stale"], true);
            assert_eq!(status["offline"], true);
        }
    }

    #[test]
    fn test_http_one_refresh_in_flight_and_concurrent_coherent_readers() {
        let home = private_home();
        let server = Server::ok(body());
        test_support::refresh(home.path(), &server.origin, Duration::from_secs(3)).unwrap();
        server.finish();
        let mut removed = serde_json::from_slice::<Value>(&body()).unwrap();
        removed["models"] = serde_json::json!({});
        removed["providers"] = serde_json::json!({});
        let server = Server::new(
            "200 OK",
            "",
            serde_json::to_vec(&removed).unwrap(),
            serde_json::to_vec(&removed).unwrap().len(),
            Duration::from_millis(100),
        );
        let origin = server.origin.clone();
        let path = home.path().to_path_buf();
        let writer =
            thread::spawn(move || test_support::refresh(&path, &origin, Duration::from_secs(3)));
        server.request.recv_timeout(Duration::from_secs(2)).unwrap();
        let second =
            test_support::refresh(home.path(), &server.origin, Duration::from_secs(1)).unwrap_err();
        assert!(second.to_string().contains("in flight"));
        let mut successful_reads = 0;
        while !writer.is_finished() {
            match test_support::status(home.path()) {
                Ok(value) => {
                    assert_eq!(value["available"], true);
                    assert!(value["provider_count"] == 0 || value["provider_count"] == 1);
                    successful_reads += 1;
                }
                Err(error) => assert!(
                    error.to_string().contains("changed") || error.to_string().contains("replaced")
                ),
            }
        }
        assert!(successful_reads > 0);
        assert_eq!(writer.join().unwrap().unwrap()["provider_count"], 0);
        server.thread.join().unwrap();
        assert_eq!(
            test_support::status(home.path()).unwrap()["provider_count"],
            0
        );
    }

    #[test]
    fn test_http_decompressed_body_limit_is_independent_of_content_length() {
        let mut gzip = Command::new("gzip")
            .arg("-c")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("Linux HTTP compression fixture requires gzip");
        let mut input = gzip.stdin.take().unwrap();
        let writer = thread::spawn(move || {
            let chunk = [b' '; 1024 * 1024];
            for _ in 0..33 {
                input.write_all(&chunk).unwrap();
            }
        });
        let compressed = gzip.wait_with_output().unwrap();
        writer.join().unwrap();
        assert!(compressed.status.success());
        assert!(compressed.stdout.len() < 64 * 1024);
        let length = compressed.stdout.len();
        let server = Server::new(
            "200 OK",
            "Content-Encoding: gzip\r\n",
            compressed.stdout,
            length,
            Duration::ZERO,
        );
        let home = private_home();
        let error = test_support::refresh(home.path(), &server.origin, Duration::from_secs(10))
            .unwrap_err();
        assert!(error.to_string().contains("exceeds 32 MiB"), "{error:#}");
        server.finish();
        assert!(!home.path().join("model-catalog/snapshot.json").exists());
    }

    #[test]
    #[ignore = "subprocess helper for test_http_sigkill_before_rename_retains_last_valid"]
    fn test_interrupted_stage_child() {
        let home = std::env::var_os("SUBSTRATE_CATALOG_INTERRUPTION_HOME")
            .expect("subprocess home is required");
        test_support::stage_until_terminated(Path::new(&home)).unwrap();
    }

    #[test]
    fn test_http_sigkill_before_rename_retains_last_valid() {
        let home = private_home();
        let server = Server::ok(body());
        test_support::refresh(home.path(), &server.origin, Duration::from_secs(3)).unwrap();
        server.finish();
        let snapshot = home.path().join("model-catalog/snapshot.json");
        let before = fs::read(&snapshot).unwrap();
        let mut child = Command::new(std::env::current_exe().unwrap())
            .args([
                "--ignored",
                "--exact",
                "http::test_interrupted_stage_child",
                "--nocapture",
            ])
            .env("SUBSTRATE_CATALOG_INTERRUPTION_HOME", home.path())
            .spawn()
            .unwrap();
        let deadline = std::time::Instant::now() + Duration::from_secs(3);
        while !home.path().join("interruption-ready").exists()
            && std::time::Instant::now() < deadline
        {
            thread::sleep(Duration::from_millis(10));
        }
        let ready = home.path().join("interruption-ready").exists();
        child.kill().unwrap();
        assert!(!child.wait().unwrap().success());
        assert!(ready, "subprocess never reached synced pre-rename stage");
        assert!(home.path().join("model-catalog/publication.stage").exists());
        assert_eq!(
            test_support::status(home.path()).unwrap()["available"],
            true
        );
        assert_eq!(fs::read(&snapshot).unwrap(), before);
        let server = Server::ok(b"{bad".to_vec());
        assert!(
            test_support::refresh(home.path(), &server.origin, Duration::from_secs(3)).is_err()
        );
        server.finish();
        assert!(!home.path().join("model-catalog/publication.stage").exists());
        assert_eq!(fs::read(snapshot).unwrap(), before);
    }
    mod cli {
        use super::*;
        use std::collections::BTreeMap;
        use std::ffi::CStr;

        type Tree = BTreeMap<PathBuf, (u32, u32, u32, Vec<u8>)>;

        fn tree(root: &Path) -> Tree {
            fn visit(root: &Path, path: &Path, result: &mut Tree) {
                let meta = fs::symlink_metadata(path).unwrap();
                let bytes = if meta.is_file() {
                    fs::read(path).unwrap()
                } else if meta.file_type().is_symlink() {
                    fs::read_link(path)
                        .unwrap()
                        .as_os_str()
                        .as_encoded_bytes()
                        .to_vec()
                } else {
                    assert!(meta.is_dir(), "unexpected fixture type");
                    Vec::new()
                };
                result.insert(
                    path.strip_prefix(root).unwrap().into(),
                    (meta.mode(), meta.uid(), meta.gid(), bytes),
                );
                if meta.is_dir() {
                    for entry in fs::read_dir(path).unwrap() {
                        visit(root, &entry.unwrap().path(), result);
                    }
                }
            }
            let mut result = Tree::new();
            visit(root, root, &mut result);
            result
        }

        fn outside_catalog(mut state: Tree) -> Tree {
            state.retain(|path, _| !path.starts_with("model-catalog"));
            state
        }

        fn fixture() -> tempfile::TempDir {
            let home = private_home();
            // These are custody sentinels, not genuine V3 Start/Resume evidence.
            for name in [
                "config.yaml",
                "agents/local.yaml",
                "sessions/pinned.json",
                "deps/local",
                "shims/sentinel",
                "trace.jsonl",
            ] {
                let path = home.path().join(name);
                fs::create_dir_all(path.parent().unwrap()).unwrap();
                fs::write(&path, format!("unparsed-private-selection:{name}")).unwrap();
                fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
            }
            home
        }

        fn command(home: &Path, action: &str, origin: &str) -> Command {
            let binary = std::env::var_os("SUBSTRATE_CATALOG_CLI_TEST_BINARY")
                .expect("full CLI proof requires the dedicated test-build substrate binary");
            let mut command = Command::new(binary);
            command
                .env_clear()
                .env("PATH", "/usr/bin:/bin")
                .env("SUBSTRATE_CATALOG_TEST_ORIGIN", origin)
                .args(["--install-prefix"])
                .arg(home)
                .args(["agents", "catalog", action, "--json"]);
            command
        }

        fn output(command: &mut Command, expected: i32) -> Value {
            let output = command.output().unwrap();
            assert_eq!(
                output.status.code(),
                Some(expected),
                "stdout={} stderr={}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            serde_json::from_slice(&output.stdout).expect("one JSON CLI response")
        }

        fn carrier(home: &Path) -> String {
            let mut passwd = std::mem::MaybeUninit::<libc::passwd>::uninit();
            let mut buffer = vec![0u8; 64 * 1024];
            let mut result = std::ptr::null_mut();
            // SAFETY: buffers and result pointer are valid for this reentrant lookup.
            let uid = unsafe { libc::geteuid() };
            assert_eq!(
                unsafe {
                    libc::getpwuid_r(
                        uid,
                        passwd.as_mut_ptr(),
                        buffer.as_mut_ptr().cast(),
                        buffer.len(),
                        &mut result,
                    )
                },
                0
            );
            assert!(!result.is_null());
            // SAFETY: a successful nonnull result initialized passwd and its account string.
            let account = unsafe { CStr::from_ptr(passwd.assume_init().pw_name) }
                .to_str()
                .unwrap();
            let context = transport_api_types::InstallBootstrapContextV1::new_unix(
                home.to_str().unwrap(),
                account,
                uid,
            )
            .unwrap();
            transport_api_types::InstallBootstrapContextCarrierV1::from_context(context)
                .unwrap()
                .encode()
                .unwrap()
        }

        #[test]
        fn test_full_cli_status_missing_cached_failure_and_refusal_are_nonmutating() {
            let home = fixture();
            let before = tree(home.path());
            let status = output(
                &mut command(home.path(), "status", "invalid-unused-origin"),
                0,
            );
            assert_eq!(status["available"], false);
            assert_eq!(status["offline"], false);
            assert_eq!(tree(home.path()), before);
            let server = Server::ok(body());
            output(&mut command(home.path(), "refresh", &server.origin), 0);
            server.finish();
            let before = tree(home.path());
            assert_eq!(
                output(&mut command(home.path(), "status", "unused"), 0)["available"],
                true
            );
            assert_eq!(tree(home.path()), before);
            let server = Server::ok(b"{bad".to_vec());
            output(&mut command(home.path(), "refresh", &server.origin), 1);
            server.finish();
            let before = tree(home.path());
            let status = output(&mut command(home.path(), "status", "unused"), 0);
            assert_eq!(status["available"], true);
            assert_eq!(status["offline"], true);
            assert_eq!(tree(home.path()), before);
            fs::set_permissions(
                home.path().join("model-catalog"),
                fs::Permissions::from_mode(0o755),
            )
            .unwrap();
            let before = tree(home.path());
            assert_eq!(
                output(&mut command(home.path(), "status", "unused"), 1)["ok"],
                false
            );
            assert_eq!(tree(home.path()), before);
            let missing = home.path().join("absent-home");
            output(&mut command(&missing, "status", "unused"), 1);
            assert!(!missing.exists());
            assert_eq!(tree(home.path()), before);
        }

        #[test]
        fn test_full_cli_refresh_writes_only_catalog_and_keeps_last_valid_on_failure() {
            let home = fixture();
            let before = tree(home.path());
            let server = Server::ok(body());
            let status = output(&mut command(home.path(), "refresh", &server.origin), 0);
            assert_eq!(status["available"], true);
            assert_eq!(status["source_sha256"], sha256(&body()));
            let request = server.finish().to_ascii_lowercase();
            assert!(request.starts_with("get /catalog.json?type=chat http/1.1\r\n"));
            for forbidden in ["authorization:", "proxy-authorization:", "cookie:"] {
                assert!(!request.contains(forbidden));
            }
            assert_eq!(outside_catalog(tree(home.path())), before);
            let snapshot = home.path().join("model-catalog/snapshot.json");
            let valid = fs::read(&snapshot).unwrap();
            let snapshot_mode = fs::metadata(&snapshot).unwrap().mode();
            let server = Server::ok(b"{bad".to_vec());
            output(&mut command(home.path(), "refresh", &server.origin), 1);
            server.finish();
            let refused = TcpListener::bind("127.0.0.1:0").unwrap();
            let origin = format!("http://{}/refused", refused.local_addr().unwrap());
            drop(refused);
            output(&mut command(home.path(), "refresh", &origin), 1);
            assert_eq!(fs::read(&snapshot).unwrap(), valid);
            assert_eq!(fs::metadata(&snapshot).unwrap().mode(), snapshot_mode);
            assert_eq!(outside_catalog(tree(home.path())), before);
            let empty = fixture();
            output(&mut command(empty.path(), "refresh", &origin), 1);
            assert!(!empty.path().join("model-catalog/snapshot.json").exists());
            let before = tree(empty.path());
            assert_eq!(
                output(&mut command(empty.path(), "status", "unused"), 0)["available"],
                false
            );
            assert_eq!(tree(empty.path()), before);
        }

        #[test]
        fn test_full_cli_invalid_context_projection_and_bootstrap_action_refuse_before_effects() {
            let home = fixture();
            let before = tree(home.path());
            let encoded = carrier(home.path());
            for (context, projection) in [
                ("malformed", None),
                (encoded.as_str(), Some("/conflicting-home")),
            ] {
                let mut cmd = command(home.path(), "refresh", "invalid-unused-origin");
                cmd.args(["--install-bootstrap-context-v1", context]);
                if let Some(projection) = projection {
                    cmd.env("SUBSTRATE_HOME", projection);
                }
                let result = cmd.output().unwrap();
                assert_eq!(result.status.code(), Some(2));
                assert!(String::from_utf8_lossy(&result.stderr)
                    .contains("invalid install bootstrap context"));
                assert_eq!(tree(home.path()), before);
            }
            let binary = std::env::var_os("SUBSTRATE_CATALOG_CLI_TEST_BINARY").unwrap();
            let result = Command::new(&binary)
                .env_clear()
                .env("PATH", "/usr/bin:/bin")
                .args(["--install-prefix"])
                .arg(home.path())
                .args([
                    "--install-bootstrap-context-v1",
                    &encoded,
                    "--install-bootstrap-home-v1",
                    "agents",
                    "catalog",
                    "status",
                    "--json",
                ])
                .output()
                .unwrap();
            assert_eq!(result.status.code(), Some(2));
            assert!(String::from_utf8_lossy(&result.stderr)
                .contains("invalid install bootstrap action"));
            assert_eq!(tree(home.path()), before);
            for flag in [
                "--shim-deploy",
                "--shim-remove",
                "--shim-status",
                "--world",
                "--no-world",
            ] {
                let result = Command::new(&binary)
                    .env_clear()
                    .args(["--install-prefix"])
                    .arg(home.path())
                    .args([flag, "agents", "catalog", "refresh", "--json"])
                    .output()
                    .unwrap();
                assert_eq!(result.status.code(), Some(2));
                assert!(String::from_utf8_lossy(&result.stderr)
                    .contains("catalog action must be exclusive"));
                assert_eq!(tree(home.path()), before);
            }
            let result = Command::new(binary)
                .env_clear()
                .args([
                    "--version-json",
                    "--install-bootstrap-context-v1",
                    "malformed",
                    "agents",
                    "catalog",
                    "refresh",
                    "--json",
                ])
                .output()
                .unwrap();
            assert!(result.status.success());
            let version: Value = serde_json::from_slice(&result.stdout).unwrap();
            assert!(version["version"].is_string());
            assert_eq!(tree(home.path()), before);
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct InstalledProfile {
    version: u32,
    installed_binary: PathBuf,
    installed_sha256: String,
    source_manifest: PathBuf,
    source_manifest_sha256: String,
    accepted_home: PathBuf,
    empty_home: PathBuf,
    preserved_artifacts: Vec<PreservedArtifact>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceManifest {
    version: u32,
    source_root: PathBuf,
    head: String,
    files: Vec<ManifestFile>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ManifestFile {
    path: PathBuf,
    sha256: String,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct PreservedArtifact {
    path: PathBuf,
    sha256: String,
    kind: String,
}

#[test]
fn test_installed_profile_parser_rejects_unknown_duplicate_and_missing_fields() {
    let value = serde_json::json!({"version":1,"installed_binary":"/binary","installed_sha256":"digest","source_manifest":"/manifest","source_manifest_sha256":"digest","accepted_home":"/home/valid","empty_home":"/home/empty","preserved_artifacts":[]});
    let valid = serde_json::to_string(&value).unwrap();
    assert!(serde_json::from_str::<InstalledProfile>(&valid).is_ok());
    let mut unknown = value.clone();
    unknown["origin"] = Value::String("http://untrusted.invalid".into());
    assert!(serde_json::from_value::<InstalledProfile>(unknown).is_err());
    assert!(serde_json::from_str::<InstalledProfile>(&valid.replacen(
        "\"version\":1",
        "\"version\":1,\"version\":2",
        1
    ))
    .is_err());
    let mut missing = value;
    missing.as_object_mut().unwrap().remove("source_manifest");
    assert!(serde_json::from_value::<InstalledProfile>(missing).is_err());
}

fn protected_file(path: &Path, max: u64) -> Vec<u8> {
    assert!(path.is_absolute(), "proof paths must be absolute");
    assert_eq!(
        fs::canonicalize(path).unwrap(),
        path,
        "proof path must be physical"
    );
    let metadata = fs::symlink_metadata(path).unwrap();
    assert!(
        metadata.is_file()
            && metadata.nlink() == 1
            && metadata.mode() & 0o077 == 0
            && metadata.len() <= max,
        "proof input must be a bounded protected single-link regular file"
    );
    // SAFETY: geteuid has no pointer preconditions.
    assert_eq!(metadata.uid(), unsafe { libc::geteuid() });
    fs::read(path).unwrap()
}

fn installed_binary(path: &Path) -> Vec<u8> {
    assert!(path.is_absolute());
    assert_eq!(fs::canonicalize(path).unwrap(), path);
    let metadata = fs::symlink_metadata(path).unwrap();
    assert!(
        metadata.is_file()
            && metadata.nlink() == 1
            && metadata.mode() & 0o022 == 0
            && metadata.mode() & 0o111 != 0
            && metadata.len() <= 512 * 1024 * 1024,
        "installed executable custody is invalid"
    );
    // SAFETY: geteuid has no pointer preconditions.
    assert!(metadata.uid() == 0 || metadata.uid() == unsafe { libc::geteuid() });
    fs::read(path).unwrap()
}

fn installed_command(
    profile: &InstalledProfile,
    home: &Path,
    action: &str,
    offline: bool,
) -> Output {
    assert_eq!(
        sha256(&installed_binary(&profile.installed_binary)),
        profile.installed_sha256,
        "installed executable changed before invocation"
    );
    let mut command = if offline {
        let mut command = Command::new("/usr/bin/unshare");
        command.args(["--net", "--"]).arg(&profile.installed_binary);
        command
    } else {
        Command::new(&profile.installed_binary)
    };
    command
        .args(["--install-prefix"])
        .arg(home)
        .args(["agents", "catalog", action, "--json"]);
    // Bootstrap uses the explicit prefix and actual principal, not ambient overrides.
    command.env_clear().env("PATH", "/usr/bin:/bin");
    let result = command
        .output()
        .expect("execute exact installed proof binary");
    println!(
        "installed action={action} offline={offline} status={} stdout={} stderr={}",
        result.status,
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    result
}

#[test]
#[ignore = "requires an explicit protected installed-proof profile and network namespace capability; never run during source qualification"]
fn test_installed_catalog_refresh_and_offline() {
    let profile_path = std::env::var_os("SUBSTRATE_CATALOG_PROOF_PROFILE")
        .map(PathBuf::from)
        .expect("SUBSTRATE_CATALOG_PROOF_PROFILE is required; absence is not a skip");
    let profile: InstalledProfile =
        serde_json::from_slice(&protected_file(&profile_path, 1024 * 1024))
            .expect("strict installed profile");
    assert_eq!(profile.version, 1);
    assert_eq!(
        sha256(&installed_binary(&profile.installed_binary)),
        profile.installed_sha256
    );
    let manifest_bytes = protected_file(&profile.source_manifest, 16 * 1024 * 1024);
    assert_eq!(sha256(&manifest_bytes), profile.source_manifest_sha256);
    let manifest: SourceManifest =
        serde_json::from_slice(&manifest_bytes).expect("strict source manifest");
    assert_eq!(manifest.version, 1);
    assert_eq!(
        fs::canonicalize(&manifest.source_root).unwrap(),
        manifest.source_root
    );
    let actual = Command::new("/usr/bin/git")
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .current_dir(&manifest.source_root)
        .args(["rev-parse", "HEAD"])
        .output()
        .unwrap();
    assert!(actual.status.success());
    assert_eq!(
        String::from_utf8(actual.stdout).unwrap().trim(),
        manifest.head
    );
    let mut seen = std::collections::BTreeSet::new();
    for file in &manifest.files {
        assert!(
            file.path.is_relative()
                && file
                    .path
                    .components()
                    .all(|c| matches!(c, std::path::Component::Normal(_)))
        );
        assert!(
            seen.insert(file.path.clone()),
            "duplicate source manifest file"
        );
        let path = manifest.source_root.join(&file.path);
        assert_eq!(fs::canonicalize(&path).unwrap(), path);
        assert_eq!(sha256(&fs::read(path).unwrap()), file.sha256);
    }
    for required in [
        "crates/shell/src/execution/model_catalog.rs",
        "crates/shell/src/execution/cli.rs",
        "crates/shell/src/execution/mod.rs",
        "crates/shell/src/execution/agents_cmd.rs",
        "crates/shell/src/execution/routing.rs",
        "Cargo.lock",
        "crates/shell/Cargo.toml",
    ] {
        assert!(
            seen.contains(Path::new(required)),
            "source manifest omits {required}"
        );
    }
    let tracked = Command::new("/usr/bin/git")
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .current_dir(&manifest.source_root)
        .args(["ls-files", "-z"])
        .output()
        .unwrap();
    assert!(tracked.status.success());
    let tracked: std::collections::BTreeSet<PathBuf> = tracked
        .stdout
        .split(|b| *b == 0)
        .filter(|p| !p.is_empty())
        .map(|p| {
            PathBuf::from(std::str::from_utf8(p).expect("manifest tracked paths must be UTF-8"))
        })
        .collect();
    assert!(
        tracked.is_subset(&seen),
        "source manifest must cover the complete tracked source tree"
    );
    assert_ne!(profile.accepted_home, profile.empty_home);
    for home in [&profile.accepted_home, &profile.empty_home] {
        assert_eq!(fs::canonicalize(home).unwrap(), *home);
        assert_eq!(fs::metadata(home).unwrap().mode() & 0o7777, 0o700);
        assert!(
            !home.join("model-catalog").exists(),
            "installed proof must start from isolated homes without catalog storage"
        );
    }
    let kinds: std::collections::BTreeSet<_> = profile
        .preserved_artifacts
        .iter()
        .map(|a| a.kind.as_str())
        .collect();
    for required in ["config", "selection", "retained_session"] {
        assert!(
            kinds.contains(required),
            "genuine preserved {required} artifact required"
        );
    }
    let assert_preserved = || {
        let mut paths = std::collections::BTreeSet::new();
        for artifact in &profile.preserved_artifacts {
            assert!(artifact.path.starts_with(&profile.accepted_home));
            assert!(paths.insert(&artifact.path), "duplicate preserved artifact");
            let metadata = fs::symlink_metadata(&artifact.path).unwrap();
            assert!(
                metadata.is_file()
                    && metadata.nlink() == 1
                    && metadata.mode() & 0o022 == 0
                    && metadata.len() <= 32 * 1024 * 1024
            );
            assert_eq!(fs::canonicalize(&artifact.path).unwrap(), artifact.path);
            assert_eq!(sha256(&fs::read(&artifact.path).unwrap()), artifact.sha256);
        }
    };
    assert_preserved();
    let refresh = installed_command(&profile, &profile.accepted_home, "refresh", false);
    assert!(
        refresh.status.success(),
        "real models.dev refresh failed; installed qualification remains incomplete"
    );
    let refreshed: Value = serde_json::from_slice(&refresh.stdout).unwrap();
    assert_eq!(refreshed["available"], true);
    assert_eq!(
        refreshed["origin"],
        "https://models.dev/catalog.json?type=chat"
    );
    let snapshot = profile.accepted_home.join("model-catalog/snapshot.json");
    let valid = fs::read(&snapshot).unwrap();
    assert_preserved();
    let status = installed_command(&profile, &profile.accepted_home, "status", false);
    assert!(status.status.success());
    assert_eq!(fs::read(&snapshot).unwrap(), valid);
    assert_preserved();
    let offline = installed_command(&profile, &profile.accepted_home, "refresh", true);
    assert!(!offline.status.success());
    assert_eq!(fs::read(&snapshot).unwrap(), valid);
    let status = installed_command(&profile, &profile.accepted_home, "status", true);
    assert!(
        status.status.success(),
        "network namespace capability unavailable; not a skip"
    );
    let status: Value = serde_json::from_slice(&status.stdout).unwrap();
    assert_eq!(status["available"], true);
    assert_eq!(status["offline"], true);
    let offline = installed_command(&profile, &profile.empty_home, "refresh", true);
    assert!(!offline.status.success());
    let status = installed_command(&profile, &profile.empty_home, "status", true);
    assert!(status.status.success());
    assert_eq!(
        serde_json::from_slice::<Value>(&status.stdout).unwrap()["available"],
        false
    );
    assert_preserved();
}
