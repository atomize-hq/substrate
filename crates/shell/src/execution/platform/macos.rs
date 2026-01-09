use serde_json::json;
use substrate_broker::world_fs_policy;

pub(crate) fn host_doctor_main(json_mode: bool, world_enabled: bool) -> i32 {
    world_doctor_macos::run_host(json_mode, world_enabled, &world_doctor_macos::SystemRunner)
}

pub(crate) fn world_doctor_main(json_mode: bool, world_enabled: bool) -> i32 {
    // `world_doctor_macos::run` is used by unit tests; keep its signature stable and pass the
    // effective world-enabled state via an existing env flag.
    std::env::set_var(
        "SUBSTRATE_WORLD_ENABLED",
        if world_enabled { "1" } else { "0" },
    );
    world_doctor_macos::run(json_mode, &world_doctor_macos::SystemRunner)
}

mod world_doctor_macos {
    use super::*;
    use agent_api_client::AgentClient;
    use serde_json::Value;
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use std::os::unix::net::UnixStream;
    use std::path::Path;
    use std::path::PathBuf;
    use std::process::Command;
    use std::time::Duration;

    pub(super) trait CommandRunner {
        fn run(&self, program: &str, args: &[&str]) -> CommandOutput;
    }

    #[derive(Debug, Clone, Default)]
    pub(super) struct CommandOutput {
        pub success: bool,
        pub stdout: String,
    }

    pub(super) struct SystemRunner;

    impl CommandRunner for SystemRunner {
        fn run(&self, program: &str, args: &[&str]) -> CommandOutput {
            match Command::new(program).args(args).output() {
                Ok(output) => CommandOutput {
                    success: output.status.success(),
                    stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
                },
                Err(_) => CommandOutput {
                    success: false,
                    stdout: String::new(),
                },
            }
        }
    }

    fn probe_caps_uds(path: &Path) -> bool {
        let Ok(mut stream) = UnixStream::connect(path) else {
            return false;
        };
        let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
        let _ = stream.set_write_timeout(Some(Duration::from_secs(2)));
        let request =
            b"GET /v1/capabilities HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n";
        if stream.write_all(request).is_err() {
            return false;
        }
        let mut buf = [0u8; 512];
        match stream.read(&mut buf) {
            Ok(n) if n > 0 => std::str::from_utf8(&buf[..n])
                .unwrap_or("")
                .contains(" 200 "),
            _ => false,
        }
    }

    fn probe_caps_tcp(host: &str, port: u16) -> bool {
        let addr = format!("{host}:{port}");
        let Ok(mut stream) = TcpStream::connect(addr) else {
            return false;
        };
        let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
        let _ = stream.set_write_timeout(Some(Duration::from_secs(2)));
        let request =
            b"GET /v1/capabilities HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n";
        if stream.write_all(request).is_err() {
            return false;
        }
        let mut buf = [0u8; 512];
        match stream.read(&mut buf) {
            Ok(n) if n > 0 => std::str::from_utf8(&buf[..n])
                .unwrap_or("")
                .contains(" 200 "),
            _ => false,
        }
    }

    pub(super) fn run_host(
        json_mode: bool,
        world_enabled: bool,
        runner: &dyn CommandRunner,
    ) -> i32 {
        let fs_policy = world_fs_policy();

        let pass = |msg: &str| println!("PASS  | {}", msg);
        let warn = |msg: &str| println!("WARN  | {}", msg);
        let fail = |msg: &str| println!("FAIL  | {}", msg);
        let info = |msg: &str| println!("INFO  | {}", msg);

        if !json_mode {
            println!("== substrate host doctor ==");
        }

        if !world_enabled {
            if !json_mode {
                fail("world isolation disabled by effective config (--no-world)");
            }
            // Continue gathering best-effort host facts.
        }

        let lima_installed = runner.run("limactl", &["--version"]).success;
        if !json_mode {
            if lima_installed {
                pass("limactl: present");
            } else {
                fail("limactl: not found");
            }
        }

        let virtualization = runner.run("sysctl", &["-n", "kern.hv_support"]);
        let lima_virtualization = virtualization.success && virtualization.stdout.trim() == "1";
        if !json_mode {
            if lima_virtualization {
                pass("Virtualization.framework available");
            } else {
                fail("Virtualization.framework unavailable (sysctl kern.hv_support != 1)");
            }
        }

        let vsock_proxy = which::which("vsock-proxy").is_ok();
        if !json_mode {
            if vsock_proxy {
                pass("vsock-proxy: present");
            } else {
                warn("vsock-proxy: not found (SSH forwarding may be used)");
            }
            info(&format!(
                "world_fs: mode={} isolation={} require_world={}",
                fs_policy.mode.as_str(),
                fs_policy.isolation.as_str(),
                fs_policy.require_world
            ));
        }

        // VM status
        let vm_status = if lima_installed {
            let vm = runner.run("limactl", &["list", "substrate", "--json"]);
            if vm.success {
                match serde_json::from_str::<Value>(&vm.stdout) {
                    Ok(value) => value
                        .get("status")
                        .and_then(Value::as_str)
                        .unwrap_or("unknown")
                        .to_string(),
                    Err(err) => format!("parse-error: {err}"),
                }
            } else {
                "missing".into()
            }
        } else {
            "unknown".into()
        };

        if !json_mode {
            match vm_status.as_str() {
                "Running" => pass("Lima VM 'substrate' running"),
                "missing" => warn("Lima VM 'substrate' not found"),
                status => warn(&format!(
                    "Lima VM 'substrate' not running (status: {status})"
                )),
            }
        }

        // If the VM isn't running, do not attempt to exec inside it (avoids accidental VM start).
        let can_probe_vm = vm_status == "Running";

        // systemd service status (inside guest, but only when already running)
        let service_active = if can_probe_vm {
            runner
                .run(
                    "limactl",
                    &[
                        "shell",
                        "substrate",
                        "systemctl",
                        "is-active",
                        "substrate-world-agent",
                    ],
                )
                .success
        } else {
            false
        };

        if !json_mode {
            if can_probe_vm {
                if service_active {
                    pass("substrate-world-agent service active");
                } else {
                    fail("substrate-world-agent service not active");
                }
            }
        }

        // Capabilities probe (host-side; no forwarding spawns)
        let agent_caps_ok = if !world_enabled || !can_probe_vm || !service_active {
            false
        } else {
            let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
            let sock = home.join(".substrate/sock/agent.sock");
            if sock.exists() && probe_caps_uds(&sock) {
                true
            } else {
                probe_caps_tcp("127.0.0.1", 17788)
            }
        };

        if !json_mode {
            if can_probe_vm && service_active {
                if agent_caps_ok {
                    pass("world-agent reachable (capabilities probe)");
                } else {
                    fail("world-agent unreachable (capabilities probe)");
                }
            }
        }

        let host_ok = world_enabled
            && lima_installed
            && lima_virtualization
            && vm_status == "Running"
            && service_active
            && agent_caps_ok;

        if json_mode {
            let out = json!({
                "schema_version": 1,
                "platform": "macos",
                "world_enabled": world_enabled,
                "ok": host_ok,
                "host": {
                    "platform": "macos",
                    "ok": host_ok,
                    "world_fs_mode": fs_policy.mode.as_str(),
                    "world_fs_isolation": fs_policy.isolation.as_str(),
                    "world_fs_require_world": fs_policy.require_world,
                    "lima": {
                        "installed": lima_installed,
                        "virtualization": lima_virtualization,
                        "vm_status": vm_status,
                        "service_active": service_active,
                        "agent_caps_ok": agent_caps_ok,
                        "vsock_proxy": vsock_proxy,
                    }
                }
            });
            println!("{}", serde_json::to_string_pretty(&out).unwrap());
        }

        if !world_enabled {
            4
        } else if !lima_installed {
            3
        } else if host_ok {
            0
        } else {
            4
        }
    }

    pub(super) fn run(json_mode: bool, runner: &dyn CommandRunner) -> i32 {
        let fs_policy = world_fs_policy();
        let world_enabled = std::env::var("SUBSTRATE_WORLD_ENABLED")
            .ok()
            .map(|raw| {
                matches!(
                    raw.trim(),
                    "1" | "true" | "TRUE" | "yes" | "YES" | "on" | "ON"
                )
            })
            .unwrap_or(true);

        let pass = |msg: &str| println!("PASS  | {}", msg);
        let warn = |msg: &str| println!("WARN  | {}", msg);
        let fail = |msg: &str| println!("FAIL  | {}", msg);
        let info = |msg: &str| println!("INFO  | {}", msg);

        if !json_mode {
            println!("== substrate world doctor ==");
            println!("== Host ==");
        }

        if !world_enabled && !json_mode {
            fail("world isolation disabled by effective config (--no-world)");
        }

        let lima_installed = runner.run("limactl", &["--version"]).success;
        let virtualization = runner.run("sysctl", &["-n", "kern.hv_support"]);
        let lima_virtualization = virtualization.success && virtualization.stdout.trim() == "1";
        let vsock_proxy = which::which("vsock-proxy").is_ok();

        if !json_mode {
            if lima_installed {
                pass("limactl: present");
            } else {
                fail("limactl: not found");
            }
            if lima_virtualization {
                pass("Virtualization.framework available");
            } else {
                fail("Virtualization.framework unavailable (sysctl kern.hv_support != 1)");
            }
            if vsock_proxy {
                pass("vsock-proxy: present");
            } else {
                warn("vsock-proxy: not found (SSH forwarding may be used)");
            }
            info(&format!(
                "world_fs: mode={} isolation={} require_world={}",
                fs_policy.mode.as_str(),
                fs_policy.isolation.as_str(),
                fs_policy.require_world
            ));
        }

        let vm_status = if lima_installed {
            let vm = runner.run("limactl", &["list", "substrate", "--json"]);
            if vm.success {
                match serde_json::from_str::<Value>(&vm.stdout) {
                    Ok(value) => value
                        .get("status")
                        .and_then(Value::as_str)
                        .unwrap_or("unknown")
                        .to_string(),
                    Err(err) => format!("parse-error: {err}"),
                }
            } else {
                "missing".into()
            }
        } else {
            "unknown".into()
        };

        if !json_mode {
            match vm_status.as_str() {
                "Running" => pass("Lima VM 'substrate' running"),
                "missing" => warn("Lima VM 'substrate' not found"),
                status => warn(&format!(
                    "Lima VM 'substrate' not running (status: {status})"
                )),
            }
        }

        let vm_running = vm_status == "Running";

        // World doctor short-circuit: do not exec inside the guest and do not probe the agent.
        let can_probe_vm = world_enabled && vm_running;
        let service_active = if can_probe_vm {
            runner
                .run(
                    "limactl",
                    &[
                        "shell",
                        "substrate",
                        "systemctl",
                        "is-active",
                        "substrate-world-agent",
                    ],
                )
                .success
        } else {
            false
        };

        if !json_mode && can_probe_vm {
            if service_active {
                pass("substrate-world-agent service active");
            } else {
                fail("substrate-world-agent service not active");
            }
        }

        let agent_caps_ok = if !can_probe_vm || !service_active {
            false
        } else {
            let sock = dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".substrate/sock/agent.sock");
            if sock.exists() && probe_caps_uds(&sock) {
                true
            } else {
                probe_caps_tcp("127.0.0.1", 17788)
            }
        };

        if !json_mode && can_probe_vm && service_active {
            if agent_caps_ok {
                pass("world-agent reachable (capabilities probe)");
            } else {
                fail("world-agent unreachable (capabilities probe)");
            }
        }

        let host_ok = world_enabled
            && lima_installed
            && lima_virtualization
            && vm_status == "Running"
            && service_active
            && agent_caps_ok;

        let host_value = json!({
            "platform": "macos",
            "ok": host_ok,
            "world_fs_mode": fs_policy.mode.as_str(),
            "world_fs_isolation": fs_policy.isolation.as_str(),
            "world_fs_require_world": fs_policy.require_world,
            "lima": {
                "installed": lima_installed,
                "virtualization": lima_virtualization,
                "vm_status": vm_status,
                "service_active": service_active,
                "agent_caps_ok": agent_caps_ok,
                "vsock_proxy": vsock_proxy,
            }
        });

        let mut exit_code = 4;
        let world_value = if !world_enabled {
            json!({"status": "disabled", "ok": false})
        } else if !(lima_installed && lima_virtualization && vm_running && service_active) {
            json!({"status": "not_provisioned", "ok": false})
        } else if !agent_caps_ok {
            exit_code = 3;
            json!({"status": "unreachable", "ok": false})
        } else {
            let sock = dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".substrate/sock/agent.sock");
            let report = tokio::runtime::Runtime::new().ok().and_then(|rt| {
                let sock_clone = sock.clone();
                rt.block_on(async {
                    if sock_clone.exists() {
                        if let Ok(client) = AgentClient::unix_socket(&sock_clone) {
                            if let Ok(report) = client.doctor_world().await {
                                return Some(report);
                            }
                        }
                    }
                    if let Ok(client) = AgentClient::tcp("127.0.0.1", 17788) {
                        if let Ok(report) = client.doctor_world().await {
                            return Some(report);
                        }
                    }
                    None
                })
            });

            match report {
                Some(report) => {
                    let status = if report.ok { "ok" } else { "missing_prereqs" };
                    let mut value = serde_json::to_value(report).unwrap_or_else(|_| json!({}));
                    if let Some(obj) = value.as_object_mut() {
                        obj.insert("status".to_string(), json!(status));
                    }
                    if host_ok && value.get("ok").and_then(Value::as_bool) == Some(true) {
                        exit_code = 0;
                    } else {
                        exit_code = 4;
                    }
                    value
                }
                None => {
                    exit_code = 3;
                    json!({"status": "unreachable", "ok": false})
                }
            }
        };

        let ok = host_ok && world_value.get("ok").and_then(Value::as_bool) == Some(true);

        if json_mode {
            let out = json!({
                "schema_version": 1,
                "platform": "macos",
                "world_enabled": world_enabled,
                "ok": ok,
                "host": host_value,
                "world": world_value,
            });
            println!("{}", serde_json::to_string_pretty(&out).unwrap());
        } else {
            println!("== World ==");
            match world_value.get("status").and_then(Value::as_str) {
                Some("disabled") => fail("world doctor disabled (world isolation is off)"),
                Some("not_provisioned") => {
                    fail("world backend not provisioned (VM/service not running)")
                }
                Some("unreachable") => fail("world backend unreachable (agent did not respond)"),
                Some("missing_prereqs") | Some("ok") => {
                    let landlock_supported = world_value
                        .get("landlock")
                        .and_then(|l| l.get("supported"))
                        .and_then(Value::as_bool)
                        .unwrap_or(false);
                    let landlock_abi = world_value
                        .get("landlock")
                        .and_then(|l| l.get("abi"))
                        .and_then(Value::as_u64);
                    if landlock_supported {
                        pass(&format!(
                            "landlock: supported{}",
                            landlock_abi
                                .map(|abi| format!(" (abi {abi})"))
                                .unwrap_or_default()
                        ));
                    } else {
                        fail("landlock: unsupported");
                    }
                    let probe_result = world_value
                        .get("world_fs_strategy")
                        .and_then(|w| w.get("probe"))
                        .and_then(|p| p.get("result"))
                        .and_then(Value::as_str)
                        .unwrap_or("fail");
                    if probe_result == "pass" {
                        pass("world fs strategy probe: pass");
                    } else {
                        fail("world fs strategy probe: fail");
                    }
                    if ok {
                        pass("world doctor: ok");
                    } else {
                        fail("world doctor: ok=false");
                    }
                }
                _ => fail("world doctor: unknown status"),
            }
        }

        exit_code
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use std::collections::VecDeque;

        use std::cell::RefCell;

        struct MockRunner {
            responses: RefCell<VecDeque<(String, Vec<String>, CommandOutput)>>,
        }

        impl MockRunner {
            fn new(responses: Vec<(String, Vec<String>, CommandOutput)>) -> Self {
                Self {
                    responses: RefCell::new(VecDeque::from(responses)),
                }
            }
        }

        impl CommandRunner for MockRunner {
            fn run(&self, program: &str, args: &[&str]) -> CommandOutput {
                if let Some((expected_prog, expected_args, output)) =
                    self.responses.borrow_mut().pop_front()
                {
                    assert_eq!(expected_prog, program);
                    assert_eq!(
                        expected_args,
                        args.iter().map(|s| s.to_string()).collect::<Vec<_>>()
                    );
                    output
                } else {
                    panic!("unexpected command: {} {:?}", program, args);
                }
            }
        }

        fn success_out(stdout: &str) -> CommandOutput {
            CommandOutput {
                success: true,
                stdout: stdout.into(),
            }
        }

        fn failure_out() -> CommandOutput {
            CommandOutput {
                success: false,
                stdout: String::new(),
            }
        }

        #[test]
        fn doctor_ok_json() {
            let vm_json = r#"{"status":"Running"}"#;
            let responses = vec![
                (
                    "limactl".into(),
                    vec!["--version".into()],
                    success_out("Lima v1"),
                ),
                (
                    "sysctl".into(),
                    vec!["-n".into(), "kern.hv_support".into()],
                    success_out("1\n"),
                ),
                (
                    "limactl".into(),
                    vec!["list".into(), "substrate".into(), "--json".into()],
                    success_out(vm_json),
                ),
                (
                    "limactl".into(),
                    vec![
                        "shell".into(),
                        "substrate".into(),
                        "uname".into(),
                        "-a".into(),
                    ],
                    success_out("Linux"),
                ),
                (
                    "limactl".into(),
                    vec![
                        "shell".into(),
                        "substrate".into(),
                        "systemctl".into(),
                        "is-active".into(),
                        "substrate-world-agent".into(),
                    ],
                    success_out("active\n"),
                ),
                (
                    "limactl".into(),
                    vec![
                        "shell".into(),
                        "substrate".into(),
                        "sudo".into(),
                        "-n".into(),
                        "test".into(),
                        "-S".into(),
                        "/run/substrate.sock".into(),
                    ],
                    success_out(""),
                ),
                (
                    "limactl".into(),
                    vec![
                        "shell".into(),
                        "substrate".into(),
                        "sudo".into(),
                        "-n".into(),
                        "timeout".into(),
                        "5".into(),
                        "curl".into(),
                        "--fail".into(),
                        "--unix-socket".into(),
                        "/run/substrate.sock".into(),
                        "http://localhost/v1/capabilities".into(),
                    ],
                    success_out("{}"),
                ),
                (
                    "limactl".into(),
                    vec![
                        "shell".into(),
                        "substrate".into(),
                        "which".into(),
                        "nft".into(),
                    ],
                    success_out("/usr/sbin/nft\n"),
                ),
                (
                    "limactl".into(),
                    vec![
                        "shell".into(),
                        "substrate".into(),
                        "df".into(),
                        "-h".into(),
                        "/".into(),
                    ],
                    success_out(
                        "Filesystem size used avail use% mounted\n/dev/root 10G 5G 5G 50% /\n",
                    ),
                ),
            ];
            let runner = MockRunner::new(responses);
            let exit = run(true, &runner);
            assert_eq!(exit, 0);
        }

        #[test]
        fn doctor_missing_vm_human() {
            let responses = vec![
                ("limactl".into(), vec!["--version".into()], failure_out()),
                (
                    "sysctl".into(),
                    vec!["-n".into(), "kern.hv_support".into()],
                    success_out("1\n"),
                ),
            ];
            let runner = MockRunner::new(responses);
            let exit = run(false, &runner);
            assert_eq!(exit, 2);
        }
    }
}

#[cfg(test)]
mod platform_tests {
    use crate::execution::{update_world_env, world_env_guard};
    use std::env;

    fn snapshot(keys: &[&str]) -> Vec<Option<String>> {
        keys.iter().map(|key| env::var(key).ok()).collect()
    }

    fn restore(keys: &[&str], values: Vec<Option<String>>) {
        for (key, value) in keys.iter().zip(values.into_iter()) {
            match value {
                Some(v) => env::set_var(key, v),
                None => env::remove_var(key),
            }
        }
    }

    #[test]
    fn update_world_env_sets_enabled_flags() {
        let _guard = world_env_guard();
        let keys = ["SUBSTRATE_WORLD", "SUBSTRATE_WORLD_ENABLED"];
        let prev = snapshot(&keys);

        update_world_env(false);

        assert_eq!(env::var("SUBSTRATE_WORLD").unwrap(), "enabled");
        assert_eq!(env::var("SUBSTRATE_WORLD_ENABLED").unwrap(), "1");

        restore(&keys, prev);
    }

    #[test]
    fn update_world_env_sets_disabled_flags() {
        let _guard = world_env_guard();
        let keys = ["SUBSTRATE_WORLD", "SUBSTRATE_WORLD_ENABLED"];
        let prev = snapshot(&keys);

        update_world_env(true);

        assert_eq!(env::var("SUBSTRATE_WORLD").unwrap(), "disabled");
        assert_eq!(env::var("SUBSTRATE_WORLD_ENABLED").unwrap(), "0");

        restore(&keys, prev);
    }
}
