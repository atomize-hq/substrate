use serde_json::json;
use substrate_broker::world_fs_policy;

pub(crate) fn host_doctor_main(
    json_mode: bool,
    world_enabled: bool,
    world_disable_attribution: Option<&crate::execution::config_model::DoctorDisableAttribution>,
) -> i32 {
    world_doctor_macos::run_host(
        json_mode,
        world_enabled,
        world_disable_attribution,
        &world_doctor_macos::SystemRunner,
    )
}

pub(crate) fn world_doctor_main(
    json_mode: bool,
    world_enabled: bool,
    world_disable_attribution: Option<&crate::execution::config_model::DoctorDisableAttribution>,
) -> i32 {
    // Preserve the world-enabled state for downstream consumers that still read the env var.
    std::env::set_var(
        "SUBSTRATE_WORLD_ENABLED",
        if world_enabled { "1" } else { "0" },
    );
    world_doctor_macos::run(
        json_mode,
        world_enabled,
        world_disable_attribution,
        &world_doctor_macos::SystemRunner,
    )
}

mod world_doctor_macos {
    use super::*;
    use chrono::SecondsFormat;
    use serde_json::Value;
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use std::os::unix::net::UnixStream;
    use std::path::Path;
    use std::path::PathBuf;
    use std::process::Command;
    use std::time::Duration;
    use transport_api_client::AgentClient;
    use world_mac_lima::transport::{
        managed_host_socket_path, Transport, COMPATIBILITY_TCP_HOST, COMPATIBILITY_TCP_PORT,
    };

    pub(super) trait CommandRunner {
        fn run(&self, program: &str, args: &[&str]) -> CommandOutput;
    }

    #[derive(Debug, Clone, Default)]
    pub(super) struct CommandOutput {
        pub success: bool,
        pub stdout: String,
        pub stderr: String,
    }

    pub(super) struct SystemRunner;

    impl CommandRunner for SystemRunner {
        fn run(&self, program: &str, args: &[&str]) -> CommandOutput {
            match Command::new(program).args(args).output() {
                Ok(output) => CommandOutput {
                    success: output.status.success(),
                    stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
                    stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
                },
                Err(_) => CommandOutput {
                    success: false,
                    stdout: String::new(),
                    stderr: String::new(),
                },
            }
        }
    }

    fn resolve_lima_vm_name() -> String {
        std::env::var("SUBSTRATE_LIMA_VM_NAME")
            .or_else(|_| std::env::var("LIMA_VM_NAME"))
            .unwrap_or_else(|_| "substrate".to_string())
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    enum HostVisibleTransport {
        Unix(PathBuf),
        Tcp { host: String, port: u16 },
    }

    fn host_visible_transports_for(selected_transport: Transport) -> Vec<HostVisibleTransport> {
        let managed_socket = HostVisibleTransport::Unix(managed_host_socket_path());
        let compatibility_tcp = HostVisibleTransport::Tcp {
            host: COMPATIBILITY_TCP_HOST.to_string(),
            port: COMPATIBILITY_TCP_PORT,
        };

        match selected_transport {
            Transport::UnixSocket => vec![managed_socket, compatibility_tcp],
            Transport::VSock | Transport::TCP => vec![compatibility_tcp, managed_socket],
        }
    }

    fn selected_host_visible_transports() -> Vec<HostVisibleTransport> {
        if let Some(socket_path) = std::env::var_os("SUBSTRATE_WORLD_SOCKET") {
            return vec![HostVisibleTransport::Unix(PathBuf::from(socket_path))];
        }

        host_visible_transports_for(Transport::auto_select().unwrap_or_default())
    }

    fn socket_override_in_effect() -> bool {
        std::env::var_os("SUBSTRATE_WORLD_SOCKET").is_some()
    }

    fn routed_proof_override_warning() -> &'static str {
        "SUBSTRATE_WORLD_SOCKET override in effect (advanced/test/breakglass); routed macOS proof requires unsetting it."
    }

    fn guest_service_active(runner: &dyn CommandRunner, vm_name: &str) -> bool {
        runner
            .run(
                "limactl",
                &[
                    "shell",
                    "--workdir=/",
                    vm_name,
                    "systemctl",
                    "is-active",
                    "substrate-world-service",
                ],
            )
            .success
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

    fn probe_caps_over_transport(transport: &HostVisibleTransport) -> bool {
        match transport {
            HostVisibleTransport::Unix(path) => probe_caps_uds(path),
            HostVisibleTransport::Tcp { host, port } => probe_caps_tcp(host, *port),
        }
    }

    fn probe_caps_via_vm(runner: &dyn CommandRunner, vm_name: &str) -> bool {
        runner
            .run(
                "limactl",
                &[
                    "shell",
                    "--workdir=/",
                    vm_name,
                    "sudo",
                    "-n",
                    "timeout",
                    "5",
                    "curl",
                    "-sS",
                    "--fail",
                    "--unix-socket",
                    "/run/substrate.sock",
                    "http://localhost/v1/capabilities",
                ],
            )
            .success
    }

    #[cfg(not(test))]
    async fn try_bootstrap_host_visible_transport_async() -> bool {
        let Ok(backend) = world_mac_lima::MacLimaBackend::new() else {
            return false;
        };
        if backend
            .ensure_persistent_session_ready_async()
            .await
            .is_err()
        {
            return false;
        }

        selected_host_visible_transports()
            .iter()
            .any(probe_caps_over_transport)
    }

    #[cfg(not(test))]
    fn try_bootstrap_host_visible_transport() -> bool {
        let Ok(rt) = tokio::runtime::Runtime::new() else {
            return false;
        };
        rt.block_on(try_bootstrap_host_visible_transport_async())
    }

    #[cfg(test)]
    fn try_bootstrap_host_visible_transport() -> bool {
        false
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum WorldDoctorReportPath {
        Routed,
        BootstrappedRouted,
        GuestDirectBreakglass,
        GuestDirectBreakglassFallbackV1,
    }

    impl WorldDoctorReportPath {
        fn as_str(self) -> &'static str {
            match self {
                Self::Routed => "routed",
                Self::BootstrappedRouted => "bootstrapped_routed",
                Self::GuestDirectBreakglass => "guest_direct_breakglass",
                Self::GuestDirectBreakglassFallbackV1 => "guest_direct_breakglass_fallback_v1",
            }
        }

        fn is_breakglass(self) -> bool {
            matches!(
                self,
                Self::GuestDirectBreakglass | Self::GuestDirectBreakglassFallbackV1
            )
        }
    }

    #[cfg(not(test))]
    async fn doctor_world_report_via_bootstrapped_transport() -> Option<Value> {
        let backend = world_mac_lima::MacLimaBackend::new().ok()?;
        backend.ensure_persistent_session_ready_async().await.ok()?;

        for transport in selected_host_visible_transports() {
            if let Some(report) = doctor_world_report_over_transport(&transport).await {
                return Some(report);
            }
        }

        None
    }

    #[cfg(test)]
    async fn doctor_world_report_via_bootstrapped_transport() -> Option<Value> {
        None
    }

    struct WorldServiceReachability {
        host_visible_transports: Vec<HostVisibleTransport>,
        service_active: bool,
        agent_caps_ok: bool,
        guest_direct_caps_ok: bool,
    }

    fn assess_world_service_reachability(
        vm_running: bool,
        vm_name: &str,
        runner: &dyn CommandRunner,
    ) -> WorldServiceReachability {
        let host_visible_transports = selected_host_visible_transports();
        if !vm_running {
            return WorldServiceReachability {
                host_visible_transports,
                service_active: false,
                agent_caps_ok: false,
                guest_direct_caps_ok: false,
            };
        }

        let service_active = guest_service_active(runner, vm_name);
        // Preserve guest service-status truth separately from routed reachability:
        // prove the selected host-visible transport contract first, and only
        // fall back to guest-direct breakglass access when those routed probes
        // fail.
        let mut agent_caps_ok = host_visible_transports
            .iter()
            .any(probe_caps_over_transport);
        if !agent_caps_ok
            && service_active
            && !socket_override_in_effect()
            // `ensure_persistent_session_ready_async()` proves routed host-visible
            // capabilities reachability while the backend-owned forwarding handle
            // is alive. Do not re-probe after the helper returns: dropping that
            // helper backend tears the temporary forwarding back down again.
            && try_bootstrap_host_visible_transport()
        {
            agent_caps_ok = true;
        }
        let guest_direct_caps_ok =
            !agent_caps_ok && service_active && probe_caps_via_vm(runner, vm_name);
        WorldServiceReachability {
            host_visible_transports,
            service_active,
            agent_caps_ok,
            guest_direct_caps_ok,
        }
    }

    async fn doctor_world_report_over_transport(transport: &HostVisibleTransport) -> Option<Value> {
        match transport {
            HostVisibleTransport::Unix(path) => {
                let client = AgentClient::unix_socket(path).ok()?;
                client
                    .doctor_world()
                    .await
                    .ok()
                    .and_then(|report| serde_json::to_value(report).ok())
            }
            HostVisibleTransport::Tcp { host, port } => {
                let client = AgentClient::tcp(host, *port).ok()?;
                client
                    .doctor_world()
                    .await
                    .ok()
                    .and_then(|report| serde_json::to_value(report).ok())
            }
        }
    }

    fn doctor_world_report_via_vm(runner: &dyn CommandRunner, vm_name: &str) -> Option<Value> {
        let output = runner.run(
            "limactl",
            &[
                "shell",
                "--workdir=/",
                vm_name,
                "sudo",
                "-n",
                "timeout",
                "5",
                "curl",
                "-sS",
                "--fail",
                "--unix-socket",
                "/run/substrate.sock",
                "http://localhost/v1/doctor/world",
            ],
        );
        if output.success {
            serde_json::from_str(&output.stdout).ok()
        } else {
            None
        }
    }

    fn fallback_world_report_v1_via_vm(runner: &dyn CommandRunner, vm_name: &str) -> Value {
        let collected_at_utc = chrono::Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);

        let landlock_output = runner.run(
            "limactl",
            &[
                "shell",
                "--workdir=/",
                vm_name,
                "sudo",
                "-n",
                "sh",
                "-c",
                r#"
set -eu
exec 2>&1
if ! grep -qs ' /sys/kernel/security ' /proc/mounts; then
  mount -t securityfs securityfs /sys/kernel/security || true
fi
cat /sys/kernel/security/landlock/abi_version
"#,
            ],
        );

        let (landlock_supported, landlock_abi, landlock_reason) = if landlock_output.success {
            let abi = landlock_output.stdout.trim().parse::<u64>().ok();
            match abi {
                Some(abi) => (true, Some(abi), Value::Null),
                None => (
                    false,
                    None,
                    Value::String(format!(
                        "invalid landlock abi_version: {}",
                        landlock_output.stdout.trim()
                    )),
                ),
            }
        } else {
            let combined = format!("{}{}", landlock_output.stdout, landlock_output.stderr);
            // Some Lima guests don't expose the Landlock ABI via securityfs, even when the
            // Landlock syscalls are present. Prefer a best-effort "supported" fallback so macOS
            // doctor scopes remain usable when the deployed world-service is behind this CLI.
            if combined.contains("landlock/abi_version")
                && (combined.contains("No such file") || combined.contains("not found"))
            {
                (true, Some(3), Value::Null)
            } else {
                (
                    false,
                    None,
                    Value::String(if combined.trim().is_empty() {
                        "landlock abi_version unavailable".to_string()
                    } else {
                        combined.trim().to_string()
                    }),
                )
            }
        };

        let probe_output = runner.run(
                "limactl",
                &[
                    "shell",
                    "--workdir=/",
                    vm_name,
                    "sudo",
                    "-n",
                    "timeout",
                    "10",
                    "sh",
                    "-c",
                    r#"
set -eu
exec 2>&1
modprobe overlay >/dev/null 2>&1 || true
dir="$(mktemp -d)"
cleanup() {
  umount "$dir/merged" >/dev/null 2>&1 || true
  rm -rf "$dir"
}
trap cleanup EXIT
mkdir -p "$dir/lower" "$dir/upper" "$dir/work" "$dir/merged"
mount -t overlay overlay -o "lowerdir=$dir/lower,upperdir=$dir/upper,workdir=$dir/work" "$dir/merged"
touch "$dir/merged/.substrate_enum_probe"
ls -a "$dir/merged" | grep -q '\.substrate_enum_probe'
echo pass
"#,
            ],
        );

        let probe_pass = probe_output.success && probe_output.stdout.trim().ends_with("pass");
        let probe_result = if probe_pass { "pass" } else { "fail" };
        let probe_failure_reason = if probe_pass {
            Value::Null
        } else {
            let combined = format!("{}{}", probe_output.stdout, probe_output.stderr);
            let details = combined.trim();
            Value::String(if details.is_empty() {
                "overlay enumeration probe failed".to_string()
            } else {
                details.to_string()
            })
        };

        let ok = landlock_supported && probe_pass;

        json!({
            "schema_version": 2,
            "ok": ok,
            "collected_at_utc": collected_at_utc,
            "policy_snapshot_v1_supported": true,
            "policy_resolution_mode": null,
            "landlock": {
                "supported": landlock_supported,
                "abi": landlock_abi,
                "reason": landlock_reason,
            },
            "world_fs_strategy": {
                "primary": "overlay",
                "fallback": "fuse",
                "probe": {
                    "id": "enumeration_v1",
                    "probe_file": ".substrate_enum_probe",
                    "result": probe_result,
                    "failure_reason": probe_failure_reason,
                }
            }
        })
    }

    fn lima_json_value(
        vm_name: &str,
        lima_installed: bool,
        lima_virtualization: bool,
        vm_status: &str,
        service_active: bool,
        agent_caps_ok: bool,
        vsock_proxy: bool,
    ) -> Value {
        json!({
            "vm_name": vm_name,
            "installed": lima_installed,
            "virtualization": lima_virtualization,
            "vm_status": vm_status,
            "service_active": service_active,
            "agent_caps_ok": agent_caps_ok,
            "vsock_proxy": vsock_proxy,
        })
    }

    fn annotate_lima_json_with_socket_override(mut lima: Value, socket_override: bool) -> Value {
        if socket_override {
            if let Some(obj) = lima.as_object_mut() {
                obj.insert("socket_override_in_effect".into(), json!(true));
            }
        }
        lima
    }

    struct WorldDoctorAssessment {
        exit_code: i32,
        world_fs_mode: String,
        world_fs_isolation: String,
        world_fs_require_world: bool,
        vm_name: String,
        lima_installed: bool,
        lima_virtualization: bool,
        vsock_proxy: bool,
        socket_override_in_effect: bool,
        vm_status: String,
        service_active: bool,
        agent_caps_ok: bool,
        world_value: Value,
        out: Value,
    }

    fn collect_world_doctor_assessment(
        report_internal_errors: bool,
        world_enabled: bool,
        world_disable_attribution: Option<
            &crate::execution::config_model::DoctorDisableAttribution,
        >,
        runner: &dyn CommandRunner,
    ) -> WorldDoctorAssessment {
        let fs_policy = world_fs_policy();
        let vm_name = resolve_lima_vm_name();
        let world_fs_mode = fs_policy.mode.as_str().to_string();
        let world_fs_isolation = fs_policy.isolation.as_str().to_string();
        let world_fs_require_world = fs_policy.require_world;
        let socket_override = socket_override_in_effect();

        let lima_installed = runner.run("limactl", &["--version"]).success;
        let virtualization = runner.run("sysctl", &["-n", "kern.hv_support"]);
        let lima_virtualization = virtualization.success && virtualization.stdout.trim() == "1";
        let vsock_proxy = which::which("vsock-proxy").is_ok();

        let vm_status = if lima_installed {
            let vm = runner.run("limactl", &["list", &vm_name, "--json"]);
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

        let vm_running = vm_status == "Running";
        let WorldServiceReachability {
            host_visible_transports,
            service_active,
            agent_caps_ok,
            guest_direct_caps_ok,
        } = assess_world_service_reachability(vm_running, &vm_name, runner);

        let host_ok = world_enabled
            && lima_installed
            && lima_virtualization
            && vm_status == "Running"
            && service_active
            && agent_caps_ok;

        let host_value = json!({
            "platform": "macos",
            "ok": host_ok,
            "world_fs_mode": world_fs_mode,
            "world_fs_isolation": world_fs_isolation,
            "world_fs_require_world": world_fs_require_world,
            "lima": annotate_lima_json_with_socket_override(
                lima_json_value(
                    &vm_name,
                    lima_installed,
                    lima_virtualization,
                    &vm_status,
                    service_active,
                    agent_caps_ok,
                    vsock_proxy,
                ),
                socket_override,
            )
        });

        let mut exit_code = 4;
        let world_value = if !world_enabled {
            json!({"status": "disabled", "ok": false})
        } else if !(lima_installed && lima_virtualization && vm_running && service_active) {
            json!({"status": "not_provisioned", "ok": false})
        } else if !(agent_caps_ok || guest_direct_caps_ok) {
            exit_code = 3;
            json!({"status": "unreachable", "ok": false})
        } else {
            let report = match tokio::runtime::Runtime::new() {
                Ok(rt) => {
                    let host_visible_transports = host_visible_transports.clone();
                    Some(rt.block_on(async {
                        for transport in &host_visible_transports {
                            if let Some(report) = doctor_world_report_over_transport(transport).await
                            {
                                return Some((report, WorldDoctorReportPath::Routed));
                            }
                        }

                        if let Some(report) = doctor_world_report_via_bootstrapped_transport().await
                        {
                            return Some((report, WorldDoctorReportPath::BootstrappedRouted));
                        }

                        doctor_world_report_via_vm(runner, &vm_name).map(|report| {
                            (report, WorldDoctorReportPath::GuestDirectBreakglass)
                        })
                    }))
                }
                Err(err) => {
                    if report_internal_errors {
                        eprintln!(
                            "substrate world doctor: internal error: failed to create tokio runtime: {err}"
                        );
                    }
                    None
                }
            }
            .flatten();

            let (report, report_path) = match report {
                Some((report, path)) => (Some(report), Some(path)),
                None => (None, None),
            };

            let mut value = match report {
                Some(report) => serde_json::to_value(report).unwrap_or_else(|_| json!({})),
                None => fallback_world_report_v1_via_vm(runner, &vm_name),
            };

            let report_path =
                report_path.unwrap_or(WorldDoctorReportPath::GuestDirectBreakglassFallbackV1);
            let used_guest_direct_breakglass = report_path.is_breakglass();

            let status = if used_guest_direct_breakglass {
                "breakglass_only"
            } else if value.get("ok").and_then(Value::as_bool) == Some(true) {
                "ok"
            } else {
                "missing_prereqs"
            };
            if let Some(obj) = value.as_object_mut() {
                obj.insert("status".to_string(), json!(status));
                obj.insert("report_path".to_string(), json!(report_path.as_str()));
            }

            if host_ok
                && value.get("ok").and_then(Value::as_bool) == Some(true)
                && !used_guest_direct_breakglass
            {
                exit_code = 0;
            } else {
                exit_code = 4;
            }
            value
        };

        let ok = host_ok && world_value.get("ok").and_then(Value::as_bool) == Some(true);

        let mut out = json!({
            "schema_version": 1,
            "platform": "macos",
            "world_enabled": world_enabled,
            "ok": ok,
            "host": host_value,
            "world": world_value.clone(),
        });
        if let Some(attribution) = world_disable_attribution {
            out["world_disable_reason"] = json!(attribution.reason);
            out["world_disable_source"] = json!(attribution.source);
        }

        WorldDoctorAssessment {
            exit_code,
            world_fs_mode,
            world_fs_isolation,
            world_fs_require_world,
            vm_name,
            lima_installed,
            lima_virtualization,
            vsock_proxy,
            socket_override_in_effect: socket_override,
            vm_status,
            service_active,
            agent_caps_ok,
            world_value,
            out,
        }
    }

    pub(super) fn run_host(
        json_mode: bool,
        world_enabled: bool,
        world_disable_attribution: Option<
            &crate::execution::config_model::DoctorDisableAttribution,
        >,
        runner: &dyn CommandRunner,
    ) -> i32 {
        let fs_policy = world_fs_policy();
        let vm_name = resolve_lima_vm_name();
        let socket_override = socket_override_in_effect();

        let pass = |msg: &str| println!("PASS  | {}", msg);
        let warn = |msg: &str| println!("WARN  | {}", msg);
        let fail = |msg: &str| println!("FAIL  | {}", msg);
        let info = |msg: &str| println!("INFO  | {}", msg);

        if !json_mode {
            println!("== substrate host doctor ==");
        }

        if !world_enabled && !json_mode {
            if let Some(attribution) = world_disable_attribution {
                fail(attribution.reason);
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
            let vm = runner.run("limactl", &["list", &vm_name, "--json"]);
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
                "Running" => pass(&format!("Lima VM '{vm_name}' running")),
                "missing" => warn(&format!("Lima VM '{vm_name}' not found")),
                status => warn(&format!(
                    "Lima VM '{vm_name}' not running (status: {status})"
                )),
            }
        }

        // If the VM isn't running, do not attempt to exec inside it (avoids accidental VM start).
        let can_probe_vm = vm_status == "Running";
        let WorldServiceReachability {
            service_active,
            agent_caps_ok,
            ..
        } = assess_world_service_reachability(can_probe_vm, &vm_name, runner);

        if !json_mode && can_probe_vm {
            if service_active {
                pass("substrate-world-service service active");
            } else {
                fail("substrate-world-service service not active");
            }
        }

        if !json_mode && can_probe_vm && service_active {
            if agent_caps_ok {
                pass("world-service reachable (capabilities probe)");
            } else {
                fail("world-service unreachable (capabilities probe)");
            }
        }

        if !json_mode && socket_override {
            warn(routed_proof_override_warning());
        }

        let host_ok = world_enabled
            && lima_installed
            && lima_virtualization
            && vm_status == "Running"
            && service_active
            && agent_caps_ok;

        if json_mode {
            let mut out = json!({
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
                    "lima": annotate_lima_json_with_socket_override(
                        lima_json_value(
                            &vm_name,
                            lima_installed,
                            lima_virtualization,
                            &vm_status,
                            service_active,
                            agent_caps_ok,
                            vsock_proxy,
                        ),
                        socket_override,
                    )
                }
            });
            if let Some(attribution) = world_disable_attribution {
                out["world_disable_reason"] = json!(attribution.reason);
                out["world_disable_source"] = json!(attribution.source);
            }
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

    pub(super) fn run(
        json_mode: bool,
        world_enabled: bool,
        world_disable_attribution: Option<
            &crate::execution::config_model::DoctorDisableAttribution,
        >,
        runner: &dyn CommandRunner,
    ) -> i32 {
        let pass = |msg: &str| println!("PASS  | {}", msg);
        let warn = |msg: &str| println!("WARN  | {}", msg);
        let fail = |msg: &str| println!("FAIL  | {}", msg);
        let info = |msg: &str| println!("INFO  | {}", msg);

        let assessment = collect_world_doctor_assessment(
            json_mode,
            world_enabled,
            world_disable_attribution,
            runner,
        );

        if !json_mode {
            println!("== substrate world doctor ==");
            println!("== Host ==");
        }

        if !world_enabled && !json_mode {
            if let Some(attribution) = world_disable_attribution {
                fail(attribution.reason);
            }
        }

        if !json_mode {
            if assessment.lima_installed {
                pass("limactl: present");
            } else {
                fail("limactl: not found");
            }
            if assessment.lima_virtualization {
                pass("Virtualization.framework available");
            } else {
                fail("Virtualization.framework unavailable (sysctl kern.hv_support != 1)");
            }
            if assessment.vsock_proxy {
                pass("vsock-proxy: present");
            } else {
                warn("vsock-proxy: not found (SSH forwarding may be used)");
            }
            info(&format!(
                "world_fs: mode={} isolation={} require_world={}",
                assessment.world_fs_mode,
                assessment.world_fs_isolation,
                assessment.world_fs_require_world
            ));
        }

        if !json_mode {
            match assessment.vm_status.as_str() {
                "Running" => pass(&format!("Lima VM '{}' running", assessment.vm_name)),
                "missing" => warn(&format!("Lima VM '{}' not found", assessment.vm_name)),
                status => warn(&format!(
                    "Lima VM '{}' not running (status: {status})",
                    assessment.vm_name
                )),
            }
        }

        let can_probe_vm = assessment.vm_status == "Running";

        if !json_mode && can_probe_vm {
            if assessment.service_active {
                pass("substrate-world-service service active");
            } else {
                fail("substrate-world-service service not active");
            }
        }

        if !json_mode && can_probe_vm && assessment.service_active {
            if assessment.agent_caps_ok {
                pass("world-service reachable (capabilities probe)");
            } else {
                fail("world-service unreachable (capabilities probe)");
            }
        }

        if !json_mode && assessment.socket_override_in_effect {
            warn(routed_proof_override_warning());
        }

        if json_mode {
            println!("{}", serde_json::to_string_pretty(&assessment.out).unwrap());
        } else {
            println!("== World ==");
            match assessment.world_value.get("status").and_then(Value::as_str) {
                Some("disabled") => fail("world doctor disabled (world isolation is off)"),
                Some("not_provisioned") => {
                    fail("world backend not provisioned (VM/service not running)")
                }
                Some("unreachable") => fail("world backend unreachable (agent did not respond)"),
                Some("breakglass_only") => {
                    fail("world doctor report available only via guest-direct breakglass fallback");
                    let landlock_supported = assessment
                        .world_value
                        .get("landlock")
                        .and_then(|l| l.get("supported"))
                        .and_then(Value::as_bool)
                        .unwrap_or(false);
                    let landlock_abi = assessment
                        .world_value
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
                    let probe_result = assessment
                        .world_value
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
                    fail("world doctor: ok=false");
                }
                Some("missing_prereqs") | Some("ok") => {
                    let landlock_supported = assessment
                        .world_value
                        .get("landlock")
                        .and_then(|l| l.get("supported"))
                        .and_then(Value::as_bool)
                        .unwrap_or(false);
                    let landlock_abi = assessment
                        .world_value
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
                    let probe_result = assessment
                        .world_value
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
                    if assessment.out.get("ok").and_then(Value::as_bool) == Some(true) {
                        pass("world doctor: ok");
                    } else {
                        fail("world doctor: ok=false");
                    }
                }
                _ => fail("world doctor: unknown status"),
            }
        }

        assessment.exit_code
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use serial_test::serial;
        use std::collections::VecDeque;

        use std::cell::RefCell;
        use std::io::{Read, Write};
        use std::os::unix::net::{UnixListener, UnixStream};
        use std::path::{Path, PathBuf};
        use std::sync::{
            atomic::{AtomicBool, Ordering},
            Arc,
        };
        use std::thread;

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
                stderr: String::new(),
            }
        }

        fn failure_out() -> CommandOutput {
            CommandOutput {
                success: false,
                stdout: String::new(),
                stderr: String::new(),
            }
        }

        struct AgentSocketGuard {
            path: PathBuf,
            shutdown: Arc<AtomicBool>,
            handle: Option<thread::JoinHandle<()>>,
        }

        impl AgentSocketGuard {
            fn start(path: &Path) -> Self {
                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent).expect("create socket parent");
                }
                let _ = std::fs::remove_file(path);
                let listener = UnixListener::bind(path).expect("bind stub socket");

                let socket_path = path.to_path_buf();
                let cleanup_path = socket_path.clone();
                let shutdown = Arc::new(AtomicBool::new(false));
                let shutdown_flag = shutdown.clone();

                let handle = thread::spawn(move || {
                    while !shutdown_flag.load(Ordering::SeqCst) {
                        let (mut stream, _) = match listener.accept() {
                            Ok(pair) => pair,
                            Err(_) => continue,
                        };
                        let mut buf = [0u8; 4096];
                        let read = stream.read(&mut buf).unwrap_or(0);
                        let request = String::from_utf8_lossy(&buf[..read]);
                        let first_line = request.lines().next().unwrap_or("");
                        if first_line.starts_with("GET /v1/capabilities") {
                            write_response(
                                &mut stream,
                                r#"{"version":"v1","features":["execute"],"backend":"world-service","platform":"linux"}"#,
                            );
                        } else if first_line.starts_with("GET /v1/doctor/world") {
                            write_response(
                                &mut stream,
                                r#"{"schema_version":2,"ok":true,"collected_at_utc":"2026-01-08T00:00:00Z","policy_snapshot_v1_supported":true,"policy_resolution_mode":null,"landlock":{"supported":true,"abi":3,"reason":null},"world_fs_strategy":{"primary":"overlay","fallback":"fuse","probe":{"id":"enumeration_v1","probe_file":".substrate_enum_probe","result":"pass","failure_reason":null}}}"#,
                            );
                        } else {
                            let _ = stream.write_all(b"HTTP/1.1 404 Not Found\r\n\r\n");
                        }
                    }

                    let _ = std::fs::remove_file(&cleanup_path);
                });

                Self {
                    path: socket_path,
                    shutdown,
                    handle: Some(handle),
                }
            }
        }

        impl Drop for AgentSocketGuard {
            fn drop(&mut self) {
                self.shutdown.store(true, Ordering::SeqCst);
                let _ = UnixStream::connect(&self.path);
                if let Some(handle) = self.handle.take() {
                    let _ = handle.join();
                }
            }
        }

        fn write_response(stream: &mut UnixStream, body: &str) {
            let reply = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = stream.write_all(reply.as_bytes());
        }

        fn with_env_var<T>(key: &str, value: Option<&str>, f: impl FnOnce() -> T) -> T {
            let prev = std::env::var_os(key);
            match value {
                Some(value) => std::env::set_var(key, value),
                None => std::env::remove_var(key),
            }
            let result = f();
            match prev {
                Some(value) => std::env::set_var(key, value),
                None => std::env::remove_var(key),
            }
            result
        }

        #[test]
        #[serial]
        fn doctor_ok_json() {
            let vm_json = r#"{"status":"Running"}"#;
            let temp = tempfile::tempdir().expect("tempdir");
            let home = temp.path();
            let sock = home.join(".substrate/sock/agent.sock");
            let _sock_guard = AgentSocketGuard::start(&sock);

            let prev_home = std::env::var_os("HOME");
            std::env::set_var("HOME", home);
            let prev_enabled = std::env::var_os("SUBSTRATE_WORLD_ENABLED");
            std::env::set_var("SUBSTRATE_WORLD_ENABLED", "1");

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
                        "--workdir=/".into(),
                        "substrate".into(),
                        "systemctl".into(),
                        "is-active".into(),
                        "substrate-world-service".into(),
                    ],
                    success_out("active\n"),
                ),
                (
                    "limactl".into(),
                    vec![
                        "shell".into(),
                        "--workdir=/".into(),
                        "substrate".into(),
                        "sudo".into(),
                        "-n".into(),
                        "timeout".into(),
                        "5".into(),
                        "curl".into(),
                        "-sS".into(),
                        "--fail".into(),
                        "--unix-socket".into(),
                        "/run/substrate.sock".into(),
                        "http://localhost/v1/capabilities".into(),
                    ],
                    failure_out(),
                ),
            ];
            let runner = MockRunner::new(responses);
            let exit = run(true, true, None, &runner);
            assert_eq!(exit, 0);

            match prev_home {
                Some(value) => std::env::set_var("HOME", value),
                None => std::env::remove_var("HOME"),
            }
            match prev_enabled {
                Some(value) => std::env::set_var("SUBSTRATE_WORLD_ENABLED", value),
                None => std::env::remove_var("SUBSTRATE_WORLD_ENABLED"),
            }
        }

        #[test]
        #[serial]
        fn doctor_resolves_override_vm_name_and_reports_it() {
            with_env_var("LIMA_VM_NAME", Some("substrate-fallback"), || {
                with_env_var("SUBSTRATE_LIMA_VM_NAME", Some("substrate-arch"), || {
                    assert_eq!(resolve_lima_vm_name(), "substrate-arch");
                })
            });
        }

        #[test]
        #[serial]
        fn host_visible_transports_follow_selected_transport_order() {
            let temp = tempfile::tempdir().expect("tempdir");
            with_env_var(
                "HOME",
                Some(temp.path().to_str().expect("home path")),
                || {
                    let expected_socket = managed_host_socket_path();

                    let unix_first = host_visible_transports_for(Transport::UnixSocket);
                    assert_eq!(
                        unix_first,
                        vec![
                            HostVisibleTransport::Unix(expected_socket.clone()),
                            HostVisibleTransport::Tcp {
                                host: COMPATIBILITY_TCP_HOST.to_string(),
                                port: COMPATIBILITY_TCP_PORT,
                            },
                        ]
                    );

                    let tcp_first = host_visible_transports_for(Transport::VSock);
                    assert_eq!(
                        tcp_first,
                        vec![
                            HostVisibleTransport::Tcp {
                                host: COMPATIBILITY_TCP_HOST.to_string(),
                                port: COMPATIBILITY_TCP_PORT,
                            },
                            HostVisibleTransport::Unix(expected_socket),
                        ]
                    );
                },
            );
        }

        #[test]
        #[serial]
        fn selected_host_visible_transports_honor_socket_override() {
            let override_path = PathBuf::from("/tmp/substrate-override.sock");
            let override_path_string = override_path.to_str().expect("override path").to_string();
            with_env_var(
                "SUBSTRATE_WORLD_SOCKET",
                Some(&override_path_string),
                || {
                    assert_eq!(
                        selected_host_visible_transports(),
                        vec![HostVisibleTransport::Unix(override_path.clone())]
                    );
                },
            );
        }

        #[test]
        #[serial]
        fn world_doctor_assessment_marks_socket_override_as_breakglass() {
            let override_path = PathBuf::from("/tmp/substrate-override.sock");
            let override_path_string = override_path.to_str().expect("override path").to_string();
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
                    success_out(r#"{"status":"Running"}"#),
                ),
                (
                    "limactl".into(),
                    vec![
                        "shell".into(),
                        "--workdir=/".into(),
                        "substrate".into(),
                        "systemctl".into(),
                        "is-active".into(),
                        "substrate-world-service".into(),
                    ],
                    success_out("active\n"),
                ),
                (
                    "limactl".into(),
                    vec![
                        "shell".into(),
                        "--workdir=/".into(),
                        "substrate".into(),
                        "sudo".into(),
                        "-n".into(),
                        "timeout".into(),
                        "5".into(),
                        "curl".into(),
                        "-sS".into(),
                        "--fail".into(),
                        "--unix-socket".into(),
                        "/run/substrate.sock".into(),
                        "http://localhost/v1/capabilities".into(),
                    ],
                    failure_out(),
                ),
            ];
            let runner = MockRunner::new(responses);

            with_env_var(
                "SUBSTRATE_WORLD_SOCKET",
                Some(&override_path_string),
                || {
                    let assessment = collect_world_doctor_assessment(true, true, None, &runner);
                    assert!(assessment.socket_override_in_effect);
                    assert_eq!(
                        assessment
                            .out
                            .pointer("/host/lima/socket_override_in_effect")
                            .and_then(Value::as_bool),
                        Some(true)
                    );
                },
            );
        }

        #[test]
        #[serial]
        fn world_doctor_json_uses_override_vm_name() {
            let vm_name = "substrate-arch";
            let vm_json = r#"{"status":"Running"}"#;
            let temp = tempfile::tempdir().expect("tempdir");
            let home = temp.path();
            let sock = home.join(".substrate/sock/agent.sock");
            let _sock_guard = AgentSocketGuard::start(&sock);

            with_env_var("HOME", Some(home.to_str().expect("home path")), || {
                with_env_var("SUBSTRATE_WORLD_ENABLED", Some("1"), || {
                    with_env_var("LIMA_VM_NAME", Some("substrate-fallback"), || {
                        with_env_var("SUBSTRATE_LIMA_VM_NAME", Some(vm_name), || {
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
                                    vec!["list".into(), vm_name.into(), "--json".into()],
                                    success_out(vm_json),
                                ),
                                (
                                    "limactl".into(),
                                    vec![
                                        "shell".into(),
                                        "--workdir=/".into(),
                                        vm_name.into(),
                                        "systemctl".into(),
                                        "is-active".into(),
                                        "substrate-world-service".into(),
                                    ],
                                    success_out("active\n"),
                                ),
                            ];
                            let runner = MockRunner::new(responses);
                            let exit = run(true, true, None, &runner);
                            assert_eq!(exit, 0);

                            let lima =
                                lima_json_value(vm_name, true, true, "Running", true, true, false);
                            assert_eq!(lima["vm_name"], vm_name);
                        })
                    })
                })
            });
        }

        #[test]
        #[serial]
        fn host_doctor_json_uses_override_vm_name() {
            let vm_name = "substrate-arch";
            let vm_json = r#"{"status":"Running"}"#;
            let temp = tempfile::tempdir().expect("tempdir");
            let home = temp.path();
            let sock = home.join(".substrate/sock/agent.sock");
            let _sock_guard = AgentSocketGuard::start(&sock);

            with_env_var("LIMA_VM_NAME", Some("substrate-fallback"), || {
                with_env_var("SUBSTRATE_LIMA_VM_NAME", Some(vm_name), || {
                    with_env_var("HOME", Some(home.to_str().expect("home path")), || {
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
                                vec!["list".into(), vm_name.into(), "--json".into()],
                                success_out(vm_json),
                            ),
                            (
                                "limactl".into(),
                                vec![
                                    "shell".into(),
                                    "--workdir=/".into(),
                                    vm_name.into(),
                                    "systemctl".into(),
                                    "is-active".into(),
                                    "substrate-world-service".into(),
                                ],
                                success_out("active\n"),
                            ),
                        ];
                        let runner = MockRunner::new(responses);
                        let exit = run_host(true, true, None, &runner);
                        assert_eq!(exit, 0);

                        let lima =
                            lima_json_value(vm_name, true, true, "Running", true, true, false);
                        assert_eq!(lima["vm_name"], vm_name);
                    })
                })
            });
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
            let exit = run(false, true, None, &runner);
            assert_eq!(exit, 4);
        }

        #[test]
        #[serial]
        fn reachability_preserves_guest_service_status_when_host_transport_is_reachable() {
            let vm_json = r#"{"status":"Running"}"#;
            let temp = tempfile::tempdir().expect("tempdir");
            let home = temp.path();
            let sock = home.join(".substrate/sock/agent.sock");
            let _sock_guard = AgentSocketGuard::start(&sock);

            with_env_var("HOME", Some(home.to_str().expect("home path")), || {
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
                            "--workdir=/".into(),
                            "substrate".into(),
                            "systemctl".into(),
                            "is-active".into(),
                            "substrate-world-service".into(),
                        ],
                        failure_out(),
                    ),
                ];
                let runner = MockRunner::new(responses);
                let assessment = collect_world_doctor_assessment(true, true, None, &runner);
                assert_eq!(assessment.exit_code, 4);
                assert!(!assessment.service_active);
                assert!(assessment.agent_caps_ok);
                assert_eq!(
                    assessment
                        .out
                        .pointer("/world/status")
                        .and_then(Value::as_str),
                    Some("not_provisioned")
                );
            });
        }

        #[test]
        #[serial]
        fn reachability_uses_guest_direct_fallback_after_host_transport_failure() {
            let vm_json = r#"{"status":"Running"}"#;
            let world_report = r#"{"schema_version":2,"ok":true,"collected_at_utc":"2026-01-08T00:00:00Z","policy_snapshot_v1_supported":true,"policy_resolution_mode":null,"landlock":{"supported":true,"abi":3,"reason":null},"world_fs_strategy":{"primary":"overlay","fallback":"fuse","probe":{"id":"enumeration_v1","probe_file":".substrate_enum_probe","result":"pass","failure_reason":null}}}"#;

            with_env_var(
                "SUBSTRATE_WORLD_SOCKET",
                Some("/tmp/substrate-missing.sock"),
                || {
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
                                "--workdir=/".into(),
                                "substrate".into(),
                                "systemctl".into(),
                                "is-active".into(),
                                "substrate-world-service".into(),
                            ],
                            success_out("active\n"),
                        ),
                        (
                            "limactl".into(),
                            vec![
                                "shell".into(),
                                "--workdir=/".into(),
                                "substrate".into(),
                                "sudo".into(),
                                "-n".into(),
                                "timeout".into(),
                                "5".into(),
                                "curl".into(),
                                "-sS".into(),
                                "--fail".into(),
                                "--unix-socket".into(),
                                "/run/substrate.sock".into(),
                                "http://localhost/v1/capabilities".into(),
                            ],
                            success_out(r#"{"version":"v1","features":["execute"]}"#),
                        ),
                        (
                            "limactl".into(),
                            vec![
                                "shell".into(),
                                "--workdir=/".into(),
                                "substrate".into(),
                                "sudo".into(),
                                "-n".into(),
                                "timeout".into(),
                                "5".into(),
                                "curl".into(),
                                "-sS".into(),
                                "--fail".into(),
                                "--unix-socket".into(),
                                "/run/substrate.sock".into(),
                                "http://localhost/v1/doctor/world".into(),
                            ],
                            success_out(world_report),
                        ),
                    ];
                    let runner = MockRunner::new(responses);
                    let assessment = collect_world_doctor_assessment(true, true, None, &runner);
                    assert_eq!(assessment.exit_code, 4);
                    assert!(assessment.service_active);
                    assert!(!assessment.agent_caps_ok);
                    assert_eq!(
                        assessment
                            .out
                            .pointer("/host/lima/agent_caps_ok")
                            .and_then(Value::as_bool),
                        Some(false)
                    );
                    assert_eq!(
                        assessment
                            .out
                            .pointer("/world/status")
                            .and_then(Value::as_str),
                        Some("breakglass_only")
                    );
                    assert_eq!(
                        assessment.out.pointer("/world/ok").and_then(Value::as_bool),
                        Some(true)
                    );
                    assert_eq!(
                        assessment
                            .out
                            .pointer("/world/report_path")
                            .and_then(Value::as_str),
                        Some("guest_direct_breakglass")
                    );
                    assert_eq!(
                        assessment.out.pointer("/ok").and_then(Value::as_bool),
                        Some(false)
                    );
                },
            );
        }

        #[test]
        #[serial]
        fn reachability_marks_v1_guest_direct_fallback_as_breakglass_only() {
            let vm_json = r#"{"status":"Running"}"#;

            with_env_var(
                "SUBSTRATE_WORLD_SOCKET",
                Some("/tmp/substrate-missing.sock"),
                || {
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
                            "--workdir=/".into(),
                            "substrate".into(),
                            "systemctl".into(),
                            "is-active".into(),
                            "substrate-world-service".into(),
                        ],
                        success_out("active\n"),
                    ),
                    (
                        "limactl".into(),
                        vec![
                            "shell".into(),
                            "--workdir=/".into(),
                            "substrate".into(),
                            "sudo".into(),
                            "-n".into(),
                            "timeout".into(),
                            "5".into(),
                            "curl".into(),
                            "-sS".into(),
                            "--fail".into(),
                            "--unix-socket".into(),
                            "/run/substrate.sock".into(),
                            "http://localhost/v1/capabilities".into(),
                        ],
                        success_out(r#"{"version":"v1","features":["execute"]}"#),
                    ),
                    (
                        "limactl".into(),
                        vec![
                            "shell".into(),
                            "--workdir=/".into(),
                            "substrate".into(),
                            "sudo".into(),
                            "-n".into(),
                            "timeout".into(),
                            "5".into(),
                            "curl".into(),
                            "-sS".into(),
                            "--fail".into(),
                            "--unix-socket".into(),
                            "/run/substrate.sock".into(),
                            "http://localhost/v1/doctor/world".into(),
                        ],
                        failure_out(),
                    ),
                    (
                        "limactl".into(),
                        vec![
                            "shell".into(),
                            "--workdir=/".into(),
                            "substrate".into(),
                            "sudo".into(),
                            "-n".into(),
                            "sh".into(),
                            "-c".into(),
                            r#"
set -eu
exec 2>&1
if ! grep -qs ' /sys/kernel/security ' /proc/mounts; then
  mount -t securityfs securityfs /sys/kernel/security || true
fi
cat /sys/kernel/security/landlock/abi_version
"#
                            .into(),
                        ],
                        success_out("3\n"),
                    ),
                    (
                        "limactl".into(),
                        vec![
                            "shell".into(),
                            "--workdir=/".into(),
                            "substrate".into(),
                            "sudo".into(),
                            "-n".into(),
                            "timeout".into(),
                            "10".into(),
                            "sh".into(),
                            "-c".into(),
                            r#"
set -eu
exec 2>&1
modprobe overlay >/dev/null 2>&1 || true
dir="$(mktemp -d)"
cleanup() {
  umount "$dir/merged" >/dev/null 2>&1 || true
  rm -rf "$dir"
}
trap cleanup EXIT
mkdir -p "$dir/lower" "$dir/upper" "$dir/work" "$dir/merged"
mount -t overlay overlay -o "lowerdir=$dir/lower,upperdir=$dir/upper,workdir=$dir/work" "$dir/merged"
touch "$dir/merged/.substrate_enum_probe"
ls -a "$dir/merged" | grep -q '\.substrate_enum_probe'
echo pass
"#
                            .into(),
                        ],
                        success_out("pass\n"),
                    ),
                ];
                    let runner = MockRunner::new(responses);
                    let assessment = collect_world_doctor_assessment(true, true, None, &runner);
                    assert_eq!(assessment.exit_code, 4);
                    assert_eq!(
                        assessment
                            .out
                            .pointer("/world/status")
                            .and_then(Value::as_str),
                        Some("breakglass_only")
                    );
                    assert_eq!(
                        assessment
                            .out
                            .pointer("/world/report_path")
                            .and_then(Value::as_str),
                        Some("guest_direct_breakglass_fallback_v1")
                    );
                    assert_eq!(
                        assessment.out.pointer("/world/ok").and_then(Value::as_bool),
                        Some(true)
                    );
                    assert_eq!(
                        assessment.out.pointer("/ok").and_then(Value::as_bool),
                        Some(false)
                    );
                },
            );
        }

        #[test]
        #[serial]
        fn reachability_reports_best_effort_host_facts_when_world_is_disabled() {
            let vm_json = r#"{"status":"Running"}"#;
            let temp = tempfile::tempdir().expect("tempdir");
            let home = temp.path();
            let sock = home.join(".substrate/sock/agent.sock");
            let _sock_guard = AgentSocketGuard::start(&sock);

            with_env_var("HOME", Some(home.to_str().expect("home path")), || {
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
                            "--workdir=/".into(),
                            "substrate".into(),
                            "systemctl".into(),
                            "is-active".into(),
                            "substrate-world-service".into(),
                        ],
                        success_out("active\n"),
                    ),
                ];
                let runner = MockRunner::new(responses);
                let assessment = collect_world_doctor_assessment(true, false, None, &runner);
                assert_eq!(assessment.exit_code, 4);
                assert!(assessment.service_active);
                assert!(assessment.agent_caps_ok);
                assert_eq!(
                    assessment
                        .out
                        .pointer("/world/status")
                        .and_then(Value::as_str),
                    Some("disabled")
                );
                assert_eq!(
                    assessment
                        .out
                        .pointer("/host/lima/service_active")
                        .and_then(Value::as_bool),
                    Some(true)
                );
                assert_eq!(
                    assessment
                        .out
                        .pointer("/host/lima/agent_caps_ok")
                        .and_then(Value::as_bool),
                    Some(true)
                );
            });
        }

        #[test]
        #[serial]
        fn world_doctor_json_reports_running_vm_with_inactive_service_as_not_provisioned() {
            let vm_json = r#"{"status":"Running"}"#;
            let temp = tempfile::tempdir().expect("tempdir");
            let home = temp.path();

            with_env_var("HOME", Some(home.to_str().expect("home path")), || {
                with_env_var("SUBSTRATE_WORLD_ENABLED", Some("1"), || {
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
                                "--workdir=/".into(),
                                "substrate".into(),
                                "systemctl".into(),
                                "is-active".into(),
                                "substrate-world-service".into(),
                            ],
                            failure_out(),
                        ),
                    ];
                    let runner = MockRunner::new(responses);
                    let assessment = collect_world_doctor_assessment(true, true, None, &runner);
                    assert_eq!(assessment.exit_code, 4);

                    let json = assessment.out;
                    assert_eq!(json.pointer("/ok").and_then(Value::as_bool), Some(false));
                    assert_eq!(
                        json.pointer("/host/lima/vm_status").and_then(Value::as_str),
                        Some("Running")
                    );
                    assert_eq!(
                        json.pointer("/host/lima/service_active")
                            .and_then(Value::as_bool),
                        Some(false)
                    );
                    assert_eq!(
                        json.pointer("/host/lima/agent_caps_ok")
                            .and_then(Value::as_bool),
                        Some(false)
                    );
                    assert_eq!(
                        json.pointer("/world/status").and_then(Value::as_str),
                        Some("not_provisioned")
                    );
                    assert_eq!(
                        json.pointer("/world/ok").and_then(Value::as_bool),
                        Some(false)
                    );
                });
            });
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
