use crate::builtins::world_deps::AptSpecV1;
use crate::execution::build_agent_client_and_request;
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use std::collections::HashMap;
use tokio::runtime::Runtime;

const PROBE_PROFILE: &str = "world-deps-probe";
const PROVISION_PROFILE: &str = "world-deps-provision";
const PROVISION_COMMAND: &str = "substrate world enable --provision-deps";
const PROBE_PREFIX: &str = "__SUBSTRATE_WDAP0__";

pub(super) fn ensure_supported_backend_or_exit() {
    if cfg!(target_os = "windows") {
        eprintln!(
            "substrate: `{PROVISION_COMMAND}` is unsupported on Windows; unsupported on Windows."
        );
        std::process::exit(4);
    }
    if cfg!(target_os = "linux") {
        eprintln!(
            "substrate: `{PROVISION_COMMAND}` is unsupported on Linux host-native backends. Substrate will not mutate the host OS."
        );
        std::process::exit(4);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum WorldManager {
    Apt,
    Pacman,
    Unsupported,
}

impl WorldManager {
    fn as_str(self) -> &'static str {
        match self {
            Self::Apt => "apt",
            Self::Pacman => "pacman",
            Self::Unsupported => "unsupported",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct WorldManagerProbe {
    pub manager: WorldManager,
    pub reason: &'static str,
    pub pacman_present: bool,
}

impl WorldManagerProbe {
    fn render(self) -> String {
        format!(
            "World-manager probe result: {} (reason={}, pacman_present={})",
            self.manager.as_str(),
            self.reason,
            self.pacman_present
        )
    }
}

pub(super) fn render_requirement(requirement: &AptSpecV1) -> String {
    match &requirement.version {
        Some(version) if !version.trim().is_empty() => {
            format!("{}={}", requirement.name, version.trim())
        }
        _ => requirement.name.clone(),
    }
}

pub(super) fn print_verbose_requirements(requirements: &[AptSpecV1]) {
    println!("Provisioning request profile: {PROVISION_PROFILE}");
    for requirement in requirements {
        println!("{}", render_requirement(requirement));
    }
}

pub(super) fn render_pacman_requirement(requirement: &str) -> String {
    requirement.trim().to_string()
}

pub(super) fn print_pacman_requirements(requirements: &[String]) {
    for requirement in requirements {
        println!("{}", render_pacman_requirement(requirement));
    }
}

pub(super) fn print_verbose_pacman_requirements(requirements: &[String]) {
    println!("Provisioning request profile: {PROVISION_PROFILE}");
    let mut normalized = requirements
        .iter()
        .map(|requirement| render_pacman_requirement(requirement))
        .filter(|requirement| !requirement.trim().is_empty())
        .collect::<Vec<_>>();
    normalized.sort();
    normalized.dedup();

    for requirement in &normalized {
        println!("{requirement}");
    }

    if !normalized.is_empty() {
        println!("pacman -Sy --noconfirm --needed {}", normalized.join(" "));
    }
}

pub(super) fn print_probe_result(probe: WorldManagerProbe) {
    println!("{}", probe.render());
}

pub(super) fn exit_probe_result_not_supported(
    probe: WorldManagerProbe,
    required_manager: WorldManager,
) -> ! {
    match required_manager {
        WorldManager::Apt => match probe.manager {
            WorldManager::Pacman => eprintln!(
                "substrate: `{PROVISION_COMMAND}` detected a pacman-based world image (reason={}); this command currently provisions apt-backed packages only and exits 4 without mutating the world.",
                probe.reason
            ),
            WorldManager::Unsupported => eprintln!(
                "substrate: `{PROVISION_COMMAND}` probe returned unsupported (reason={}, pacman_present={}); this command cannot provision apt-backed packages for the detected world image and exits 4 without mutating the world.",
                probe.reason,
                probe.pacman_present
            ),
            WorldManager::Apt => unreachable!("apt probe results must not be routed here"),
        },
        WorldManager::Pacman => match probe.manager {
            WorldManager::Apt => eprintln!(
                "substrate: `{PROVISION_COMMAND}` detected an apt-based world image (reason={}); this command currently provisions pacman-backed packages only and exits 4 without mutating the world.",
                probe.reason
            ),
            WorldManager::Unsupported => eprintln!(
                "substrate: `{PROVISION_COMMAND}` probe returned unsupported (reason={}, pacman_present={}); this command cannot provision pacman-backed packages for the detected world image and exits 4 without mutating the world.",
                probe.reason,
                probe.pacman_present
            ),
            WorldManager::Pacman => unreachable!("pacman probe results must not be routed here"),
        },
        WorldManager::Unsupported => unreachable!("unsupported required manager is not valid"),
    }
    std::process::exit(4);
}

pub(super) fn probe_world_manager() -> Result<WorldManagerProbe, String> {
    let response = execute_with_profile(&build_manager_probe_command(), PROBE_PROFILE)?;
    if response.exit != 0 {
        return Err(format!(
            "world-manager probe failed (exit={}): {}",
            response.exit,
            response_snippet(&response)
        ));
    }

    let stdout = decode_output(&response.stdout_b64);
    let mut os_release_readable = false;
    let mut id = None::<String>;
    let mut id_like = None::<String>;
    let mut pacman_present = None::<bool>;

    for line in stdout.lines() {
        let Some((key, value)) = parse_probe_line(line) else {
            continue;
        };
        match key {
            "os_release_readable" => os_release_readable = value == "1",
            "id" if !value.trim().is_empty() => id = Some(value.trim().to_string()),
            "id_like" if !value.trim().is_empty() => id_like = Some(value.trim().to_string()),
            "pacman_present" => pacman_present = Some(value == "1"),
            _ => {}
        }
    }

    if !os_release_readable {
        return Ok(classify_world_manager_probe(
            false,
            id.as_deref(),
            id_like.as_deref(),
            false,
        ));
    }

    let Some(pacman_present) = pacman_present else {
        return Err("world-manager probe output missing pacman_present".to_string());
    };

    Ok(classify_world_manager_probe(
        os_release_readable,
        id.as_deref(),
        id_like.as_deref(),
        pacman_present,
    ))
}

pub(super) fn provision_apt_requirements(requirements: &[AptSpecV1]) {
    if requirements.is_empty() {
        return;
    }

    let present = match probe_requirements(requirements) {
        Ok(present) => present,
        Err(detail) => exit_backend_unavailable(&detail),
    };
    let unsatisfied = requirements
        .iter()
        .filter(|requirement| !present.get(&requirement.name).copied().unwrap_or(false))
        .cloned()
        .collect::<Vec<_>>();
    if unsatisfied.is_empty() {
        return;
    }

    let response =
        match execute_with_profile(&build_install_command(&unsatisfied), PROVISION_PROFILE) {
            Ok(response) => response,
            Err(detail) => exit_backend_unavailable(&detail),
        };
    if response.exit == 0 {
        return;
    }

    let snippet = response_snippet(&response);
    if response.exit == 5 {
        eprintln!(
            "substrate: `{PROVISION_COMMAND}` failed: in-world APT provisioning was blocked (exit=5): {snippet}"
        );
        std::process::exit(5);
    }

    eprintln!(
        "substrate: `{PROVISION_COMMAND}` failed during in-world APT provisioning (exit={}): {snippet}",
        response.exit
    );
    std::process::exit(4);
}

pub(super) fn provision_pacman_requirements(requirements: &[String]) {
    if requirements.is_empty() {
        return;
    }

    let response = match execute_with_profile(
        &build_pacman_install_command(requirements),
        PROVISION_PROFILE,
    ) {
        Ok(response) => response,
        Err(detail) => exit_backend_unavailable(&detail),
    };
    if response.exit == 0 {
        return;
    }

    let snippet = response_snippet(&response);
    eprintln!(
        "substrate: `{PROVISION_COMMAND}` failed during in-world pacman provisioning (exit={}): {snippet}",
        response.exit
    );
    std::process::exit(4);
}

pub(super) fn exit_backend_unavailable(detail: &str) -> ! {
    eprintln!(
        "substrate: world backend unavailable for `{PROVISION_COMMAND}`. Run `substrate world doctor --json`, then retry `{PROVISION_COMMAND}`."
    );
    if !detail.trim().is_empty() {
        eprintln!("Underlying error: {}", detail.trim());
    }
    std::process::exit(3);
}

pub(super) fn classify_world_manager_probe(
    os_release_readable: bool,
    id: Option<&str>,
    id_like: Option<&str>,
    pacman_present: bool,
) -> WorldManagerProbe {
    if !os_release_readable {
        return WorldManagerProbe {
            manager: WorldManager::Unsupported,
            reason: "os_release_unreadable",
            pacman_present,
        };
    }

    let normalized_id = id.unwrap_or("").trim().to_ascii_lowercase();
    let normalized_id_like = id_like.unwrap_or("").trim().to_ascii_lowercase();

    if normalized_id.is_empty() && normalized_id_like.is_empty() {
        return WorldManagerProbe {
            manager: WorldManager::Unsupported,
            reason: "os_release_missing_identity",
            pacman_present,
        };
    }

    let mut is_debian_family = matches!(normalized_id.as_str(), "debian" | "ubuntu");
    let mut is_arch_family = matches!(normalized_id.as_str(), "arch" | "archlinux");

    for token in normalized_id_like.split_whitespace() {
        match token {
            "debian" | "ubuntu" => is_debian_family = true,
            "arch" | "archlinux" => is_arch_family = true,
            _ => {}
        }
    }

    if is_debian_family && is_arch_family {
        return WorldManagerProbe {
            manager: WorldManager::Unsupported,
            reason: "ambiguous_family_mapping",
            pacman_present,
        };
    }

    if is_debian_family {
        return WorldManagerProbe {
            manager: WorldManager::Apt,
            reason: "debian_family",
            pacman_present,
        };
    }

    if is_arch_family {
        if pacman_present {
            return WorldManagerProbe {
                manager: WorldManager::Pacman,
                reason: "arch_family_pacman_present",
                pacman_present,
            };
        }

        return WorldManagerProbe {
            manager: WorldManager::Unsupported,
            reason: "arch_family_pacman_missing",
            pacman_present,
        };
    }

    WorldManagerProbe {
        manager: WorldManager::Unsupported,
        reason: "unmapped_family",
        pacman_present,
    }
}

fn parse_probe_line(line: &str) -> Option<(&str, &str)> {
    let rest = line.strip_prefix(PROBE_PREFIX)?.trim_start();
    let (key, value) = rest.split_once('=')?;
    Some((key.trim(), value.trim()))
}

fn probe_requirements(requirements: &[AptSpecV1]) -> Result<HashMap<String, bool>, String> {
    let response = execute_with_profile(&build_apt_probe_command(requirements), PROVISION_PROFILE)?;
    if response.exit != 0 {
        return Err(format!(
            "world-deps provisioning probe failed (exit={}): {}",
            response.exit,
            response_snippet(&response)
        ));
    }

    let stdout = decode_output(&response.stdout_b64);
    let mut present = HashMap::new();
    for line in stdout.lines() {
        let Some(rest) = line.strip_prefix(&format!("{PROBE_PREFIX} ")) else {
            continue;
        };
        let mut parts = rest.split_whitespace();
        let Some(name) = parts.next() else {
            continue;
        };
        let is_present = matches!(parts.next(), Some("1"));
        present.insert(name.to_string(), is_present);
    }
    Ok(present)
}

fn execute_with_profile(
    cmd: &str,
    profile: &str,
) -> Result<transport_api_types::ExecuteResponse, String> {
    let (client, mut request, _) =
        build_agent_client_and_request(cmd).map_err(|err| format!("{err:#}"))?;
    request.profile = Some(profile.to_string());
    if cfg!(target_os = "macos") {
        request.cwd = Some("/tmp".to_string());
    }

    let runtime = Runtime::new().map_err(|err| err.to_string())?;
    runtime.block_on(async move {
        client
            .execute(request)
            .await
            .map_err(|err| format!("world-service /v1/execute request failed: {err}"))
    })
}

fn build_manager_probe_command() -> String {
    let mut script = String::from("set -eu\n");
    script.push_str("os_release='/etc/os-release'\n");
    script.push_str("if [ ! -r \"$os_release\" ]; then\n");
    script.push_str("  printf '");
    script.push_str(PROBE_PREFIX);
    script.push_str(" os_release_readable=%s\\n' 0\n");
    script.push_str("  exit 0\n");
    script.push_str("fi\n");
    script.push_str(". \"$os_release\"\n");
    script.push_str("printf '");
    script.push_str(PROBE_PREFIX);
    script.push_str(" os_release_readable=%s\\n' 1\n");
    script.push_str("printf '");
    script.push_str(PROBE_PREFIX);
    script.push_str(" id=%s\\n' \"${ID-}\"\n");
    script.push_str("printf '");
    script.push_str(PROBE_PREFIX);
    script.push_str(" id_like=%s\\n' \"${ID_LIKE-}\"\n");
    script.push_str("if command -v pacman >/dev/null 2>&1; then\n");
    script.push_str("  printf '");
    script.push_str(PROBE_PREFIX);
    script.push_str(" pacman_present=%s\\n' 1\n");
    script.push_str("else\n");
    script.push_str("  printf '");
    script.push_str(PROBE_PREFIX);
    script.push_str(" pacman_present=%s\\n' 0\n");
    script.push_str("fi\n");
    script.push_str("exit 0\n");
    format!("sh -lc {}", sh_quote(&script))
}

fn build_apt_probe_command(requirements: &[AptSpecV1]) -> String {
    let mut script = String::from("set +e\n");
    script.push_str("check_pkg() {\n");
    script.push_str("  pkg_name=\"$1\"\n");
    script.push_str("  want_version=\"$2\"\n");
    script.push_str("  output=\"$(dpkg-query -W -f='${Status} ${Version}\\n' \"$pkg_name\" 2>/dev/null || true)\"\n");
    script.push_str("  present=0\n");
    script.push_str("  case \"$output\" in\n");
    script.push_str("    \"install ok installed \"*)\n");
    script.push_str("      installed_version=\"${output#install ok installed }\"\n");
    script.push_str("      if [ -z \"$want_version\" ] || [ \"$installed_version\" = \"$want_version\" ]; then\n");
    script.push_str("        present=1\n");
    script.push_str("      fi\n");
    script.push_str("      ;;\n");
    script.push_str("  esac\n");
    script.push_str("  printf '");
    script.push_str(PROBE_PREFIX);
    script.push_str(" %s %s\\n' \"$pkg_name\" \"$present\"\n");
    script.push_str("}\n");
    for requirement in requirements {
        script.push_str("check_pkg ");
        script.push_str(&sh_quote(&requirement.name));
        script.push(' ');
        script.push_str(&sh_quote(
            requirement.version.as_deref().unwrap_or_default().trim(),
        ));
        script.push('\n');
    }
    script.push_str("exit 0\n");
    format!("sh -lc {}", sh_quote(&script))
}

fn build_install_command(requirements: &[AptSpecV1]) -> String {
    let rendered = requirements
        .iter()
        .map(render_requirement)
        .map(|requirement| sh_quote(&requirement))
        .collect::<Vec<_>>()
        .join(" ");

    let mut script = String::from("set -eu\n");
    script.push_str("if ! command -v apt-get >/dev/null 2>&1; then\n");
    script.push_str("  echo 'apt-get not found in world; install.method=apt requires an apt-based world image' >&2\n");
    script.push_str("  exit 127\n");
    script.push_str("fi\n");
    script.push_str("SUDO=''\n");
    script.push_str("if [ \"$(id -u)\" -ne 0 ]; then\n");
    script.push_str("  if command -v sudo >/dev/null 2>&1; then\n");
    script.push_str("    SUDO='sudo -n'\n");
    script.push_str("  else\n");
    script.push_str(
        "    echo 'not running as root and sudo is unavailable; cannot run apt-get' >&2\n",
    );
    script.push_str("    exit 126\n");
    script.push_str("  fi\n");
    script.push_str("fi\n");
    script.push('\n');
    script.push_str("# DNS preflight: Lima guests can drift into a broken stub resolver state\n");
    script.push_str("# (/etc/resolv.conf -> 127.0.0.53 while systemd-resolved is inactive).\n");
    script.push_str("APT_DNS_TEST_HOST='ports.ubuntu.com'\n");
    script.push_str("dns_lookup() {\n");
    script.push_str("  host=\"$1\"\n");
    script.push_str("  if command -v getent >/dev/null 2>&1; then\n");
    script.push_str("    getent hosts \"$host\" >/dev/null 2>&1 && return 0\n");
    script.push_str("  fi\n");
    script.push_str("  if command -v nslookup >/dev/null 2>&1; then\n");
    script.push_str("    nslookup \"$host\" >/dev/null 2>&1 && return 0\n");
    script.push_str("  fi\n");
    script.push_str("  return 1\n");
    script.push_str("}\n");
    script.push_str("dns_remediate() {\n");
    script.push_str("  # Prefer resolving via systemd-resolved without relying on the stub.\n");
    script.push_str("  if command -v systemctl >/dev/null 2>&1; then\n");
    script.push_str("    $SUDO systemctl disable --now dnsmasq >/dev/null 2>&1 || true\n");
    script.push_str("    $SUDO systemctl enable --now systemd-resolved >/dev/null 2>&1 || true\n");
    script.push_str("    if [ -e /run/systemd/resolve/resolv.conf ]; then\n");
    script.push_str("      $SUDO ln -sf /run/systemd/resolve/resolv.conf /etc/resolv.conf >/dev/null 2>&1 || true\n");
    script.push_str("    fi\n");
    script.push_str("  fi\n");
    script.push_str("  # Last resort: static resolv.conf.\n");
    script.push_str("  if ! dns_lookup \"$APT_DNS_TEST_HOST\"; then\n");
    script.push_str("    $SUDO sh -c 'printf \"nameserver 1.1.1.1\\nnameserver 8.8.8.8\\noptions timeout:1 attempts:3\\n\" > /etc/resolv.conf' >/dev/null 2>&1 || true\n");
    script.push_str("  fi\n");
    script.push_str("}\n");
    script.push_str("if ! dns_lookup \"$APT_DNS_TEST_HOST\"; then\n");
    script.push_str("  echo \"substrate: DNS lookup failed for ${APT_DNS_TEST_HOST}; attempting remediation\" >&2\n");
    script.push_str("  dns_remediate\n");
    script.push_str("  if ! dns_lookup \"$APT_DNS_TEST_HOST\"; then\n");
    script.push_str("    echo \"substrate: DNS remediation failed for ${APT_DNS_TEST_HOST}; /etc/resolv.conf:\" >&2\n");
    script.push_str("    $SUDO sh -c 'cat /etc/resolv.conf 2>/dev/null || true' >&2 || true\n");
    script.push_str("    exit 100\n");
    script.push_str("  fi\n");
    script.push_str("fi\n");
    script.push_str("$SUDO env DEBIAN_FRONTEND=noninteractive apt-get update\n");
    script.push_str("$SUDO env DEBIAN_FRONTEND=noninteractive apt-get install -y ");
    script.push_str(&rendered);
    script.push('\n');
    format!("sh -lc {}", sh_quote(&script))
}

fn build_pacman_install_command(requirements: &[String]) -> String {
    let rendered = requirements
        .iter()
        .map(|requirement| sh_quote(&render_pacman_requirement(requirement)))
        .collect::<Vec<_>>()
        .join(" ");

    let mut script = String::from("set -eu\n");
    script.push_str("if ! command -v pacman >/dev/null 2>&1; then\n");
    script.push_str("  echo 'pacman not found in world; install.method=pacman requires a pacman-based world image' >&2\n");
    script.push_str("  exit 127\n");
    script.push_str("fi\n");
    script.push_str("pacman -Sy --noconfirm --needed ");
    script.push_str(&rendered);
    script.push('\n');
    format!("sh -lc {}", sh_quote(&script))
}

fn response_snippet(response: &transport_api_types::ExecuteResponse) -> String {
    let stderr = decode_output(&response.stderr_b64);
    if !stderr.trim().is_empty() {
        return stderr.trim().to_string();
    }
    let stdout = decode_output(&response.stdout_b64);
    if stdout.trim().is_empty() {
        "unknown error".to_string()
    } else {
        stdout.trim().to_string()
    }
}

fn decode_output(raw: &str) -> String {
    let bytes = BASE64.decode(raw.as_bytes()).unwrap_or_default();
    String::from_utf8_lossy(&bytes).to_string()
}

fn sh_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_debian_family_as_apt() {
        let probe = classify_world_manager_probe(true, Some("ubuntu"), Some("debian"), false);
        assert_eq!(probe.manager, WorldManager::Apt);
        assert_eq!(probe.reason, "debian_family");
    }

    #[test]
    fn classifies_arch_family_with_pacman_present_as_pacman() {
        let probe = classify_world_manager_probe(true, Some("arch"), Some("archlinux"), true);
        assert_eq!(probe.manager, WorldManager::Pacman);
        assert_eq!(probe.reason, "arch_family_pacman_present");
    }

    #[test]
    fn classifies_arch_family_without_pacman_as_unsupported() {
        let probe = classify_world_manager_probe(true, Some("arch"), Some("archlinux"), false);
        assert_eq!(probe.manager, WorldManager::Unsupported);
        assert_eq!(probe.reason, "arch_family_pacman_missing");
    }

    #[test]
    fn classifies_unreadable_os_release_without_pacman_as_unsupported() {
        let probe = classify_world_manager_probe(false, None, None, false);
        assert_eq!(probe.manager, WorldManager::Unsupported);
        assert_eq!(probe.reason, "os_release_unreadable");
    }

    #[test]
    fn classifies_ambiguous_and_unmapped_as_unsupported() {
        let ambiguous =
            classify_world_manager_probe(true, Some("debian"), Some("arch ubuntu"), true);
        assert_eq!(ambiguous.manager, WorldManager::Unsupported);
        assert_eq!(ambiguous.reason, "ambiguous_family_mapping");

        let unmapped = classify_world_manager_probe(true, Some("fedora"), Some("rhel"), true);
        assert_eq!(unmapped.manager, WorldManager::Unsupported);
        assert_eq!(unmapped.reason, "unmapped_family");
    }

    #[test]
    fn builds_pacman_install_command_with_expected_flags_and_order() {
        let cmd = build_pacman_install_command(&[
            "alpm".to_string(),
            "curl".to_string(),
            "zlib".to_string(),
        ]);

        assert!(cmd.contains("pacman -Sy --noconfirm --needed"));
        assert!(cmd.contains("alpm"));
        assert!(cmd.contains("curl"));
        assert!(cmd.contains("zlib"));
        assert!(cmd.find("alpm").unwrap() < cmd.find("curl").unwrap());
        assert!(cmd.find("curl").unwrap() < cmd.find("zlib").unwrap());
    }
}
