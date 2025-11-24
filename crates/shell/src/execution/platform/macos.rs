use serde_json::json;

pub(crate) fn world_doctor_main(json_mode: bool) -> i32 {
    world_doctor_macos::run(json_mode, &world_doctor_macos::SystemRunner)
}

mod world_doctor_macos {
    use super::*;
    use serde_json::Value;
    use std::process::Command;

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

    pub(super) fn run(json_mode: bool, runner: &dyn CommandRunner) -> i32 {
        let mut ok = true;

        let mut lima_info = LimaInfo::default();

        let pass = |msg: &str| println!("PASS  | {}", msg);
        let warn = |msg: &str| println!("WARN  | {}", msg);
        let fail = |msg: &str| println!("FAIL  | {}", msg);
        let info = |msg: &str| println!("INFO  | {}", msg);

        if !json_mode {
            println!("== substrate world doctor (macOS) ==");
        }

        // Host checks
        lima_info.installed = runner.run("limactl", &["--version"]).success;
        if !json_mode {
            if lima_info.installed {
                pass("limactl: present");
            } else {
                fail("limactl: not found");
            }
        }
        ok &= lima_info.installed;

        let virtualization = runner.run("sysctl", &["-n", "kern.hv_support"]);
        lima_info.virtualization = virtualization.success && virtualization.stdout.trim() == "1";
        if !json_mode {
            if lima_info.virtualization {
                pass("Virtualization.framework available");
            } else {
                fail("Virtualization.framework unavailable (sysctl kern.hv_support != 1)");
            }
        }
        ok &= lima_info.virtualization;

        lima_info.vsock_proxy = which::which("vsock-proxy").is_ok();
        if !json_mode {
            if lima_info.vsock_proxy {
                pass("vsock-proxy: present");
            } else {
                warn("vsock-proxy: not found (SSH forwarding will be used)");
            }
        }

        // VM checks
        if lima_info.installed {
            let vm = runner.run("limactl", &["list", "substrate", "--json"]);
            if vm.success {
                match serde_json::from_str::<Value>(&vm.stdout) {
                    Ok(value) => {
                        lima_info.vm_status = value
                            .get("status")
                            .and_then(Value::as_str)
                            .unwrap_or("unknown")
                            .to_string();
                    }
                    Err(err) => {
                        lima_info.vm_status = format!("parse-error: {err}");
                    }
                }
            } else {
                lima_info.vm_status = "missing".into();
            }
        } else {
            lima_info.vm_status = "unknown".into();
        }

        if !json_mode {
            match lima_info.vm_status.as_str() {
                "Running" => pass("Lima VM 'substrate' running"),
                "missing" => {
                    warn("Lima VM 'substrate' not found");
                    ok = false;
                }
                status => {
                    warn(&format!(
                        "Lima VM 'substrate' not running (status: {status})"
                    ));
                    ok = false;
                }
            }
        } else if lima_info.vm_status != "Running" {
            ok = false;
        }

        if lima_info.vm_status == "Running" {
            // SSH connectivity
            let ssh = runner.run("limactl", &["shell", "substrate", "uname", "-a"]);
            lima_info.ssh_ok = ssh.success;
            if !json_mode {
                if lima_info.ssh_ok {
                    pass("SSH connectivity to Lima guest");
                } else {
                    warn("SSH connectivity failed (limactl shell substrate uname -a)");
                }
            }

            // systemd service status
            lima_info.service_active = runner
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
                .success;
            if !json_mode {
                if lima_info.service_active {
                    pass("substrate-world-agent service active");
                } else {
                    fail("substrate-world-agent service not active");
                }
            }
            ok &= lima_info.service_active;

            // Agent socket exists
            lima_info.agent_socket = runner
                .run(
                    "limactl",
                    &[
                        "shell",
                        "substrate",
                        "sudo",
                        "-n",
                        "test",
                        "-S",
                        "/run/substrate.sock",
                    ],
                )
                .success;
            if !json_mode {
                if lima_info.agent_socket {
                    pass("Agent socket present (/run/substrate.sock)");
                } else {
                    warn("Agent socket missing in guest");
                }
            }

            // Capabilities check
            lima_info.agent_caps_ok = runner
                .run(
                    "limactl",
                    &[
                        "shell",
                        "substrate",
                        "sudo",
                        "-n",
                        "timeout",
                        "5",
                        "curl",
                        "--fail",
                        "--unix-socket",
                        "/run/substrate.sock",
                        "http://localhost/v1/capabilities",
                    ],
                )
                .success;
            if !json_mode {
                if lima_info.agent_caps_ok {
                    pass("Agent responded to capabilities request");
                } else {
                    fail("Agent not responding to capabilities request");
                }
            }
            ok &= lima_info.agent_socket && lima_info.agent_caps_ok;

            // nftables (info)
            lima_info.nft_available = runner
                .run("limactl", &["shell", "substrate", "which", "nft"])
                .success;
            if !json_mode {
                if lima_info.nft_available {
                    info("nftables available in guest");
                } else {
                    warn("nftables not found in guest");
                }
            }

            // disk usage info
            let disk = runner.run("limactl", &["shell", "substrate", "df", "-h", "/"]);
            if !json_mode {
                if disk.success {
                    if let Some(line) = disk.stdout.lines().last() {
                        info(&format!("disk usage: {}", line));
                    }
                } else {
                    warn("Unable to query disk usage inside guest");
                }
            } else {
                lima_info.disk_usage = disk
                    .stdout
                    .lines()
                    .next_back()
                    .unwrap_or("")
                    .trim()
                    .to_string();
            }
        }

        if json_mode {
            let mut lima = json!({
                "installed": lima_info.installed,
                "virtualization": lima_info.virtualization,
                "vm_status": lima_info.vm_status,
                "service_active": lima_info.service_active,
                "agent_socket": lima_info.agent_socket,
                "agent_caps_ok": lima_info.agent_caps_ok,
                "vsock_proxy": lima_info.vsock_proxy,
                "ssh": lima_info.ssh_ok,
                "nft": lima_info.nft_available,
            });

            if !lima_info.disk_usage.is_empty() {
                lima["disk_usage"] = json!(lima_info.disk_usage);
            }

            let out = json!({
                "platform": "macos",
                "lima": lima,
                "ok": ok,
            });
            println!("{}", serde_json::to_string_pretty(&out).unwrap());
        }

        if ok {
            0
        } else {
            2
        }
    }

    #[derive(Default)]
    struct LimaInfo {
        installed: bool,
        virtualization: bool,
        vm_status: String,
        ssh_ok: bool,
        service_active: bool,
        agent_socket: bool,
        agent_caps_ok: bool,
        vsock_proxy: bool,
        nft_available: bool,
        disk_usage: String,
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
    use super::*;
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
        let keys = ["SUBSTRATE_WORLD", "SUBSTRATE_WORLD_ENABLED"];
        let prev = snapshot(&keys);

        update_world_env(false);

        assert_eq!(env::var("SUBSTRATE_WORLD").unwrap(), "enabled");
        assert_eq!(env::var("SUBSTRATE_WORLD_ENABLED").unwrap(), "1");

        restore(&keys, prev);
    }

    #[test]
    fn update_world_env_sets_disabled_flags() {
        let keys = ["SUBSTRATE_WORLD", "SUBSTRATE_WORLD_ENABLED"];
        let prev = snapshot(&keys);

        update_world_env(true);

        assert_eq!(env::var("SUBSTRATE_WORLD").unwrap(), "disabled");
        assert_eq!(env::var("SUBSTRATE_WORLD_ENABLED").unwrap(), "0");

        restore(&keys, prev);
    }
}
