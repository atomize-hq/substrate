use crate::builtins::world_deps::AptSpecV1;
use crate::execution::build_agent_client_and_request;
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use std::collections::HashMap;
use tokio::runtime::Runtime;

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

    let response = match execute_with_profile(&build_install_command(&unsatisfied)) {
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

pub(super) fn exit_backend_unavailable(detail: &str) -> ! {
    eprintln!(
        "substrate: world backend unavailable for `{PROVISION_COMMAND}`. Run `substrate world doctor --json`, then retry `{PROVISION_COMMAND}`."
    );
    if !detail.trim().is_empty() {
        eprintln!("Underlying error: {}", detail.trim());
    }
    std::process::exit(3);
}

fn probe_requirements(requirements: &[AptSpecV1]) -> Result<HashMap<String, bool>, String> {
    let response = execute_with_profile(&build_probe_command(requirements))?;
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

fn execute_with_profile(cmd: &str) -> Result<agent_api_types::ExecuteResponse, String> {
    let (client, mut request, _) =
        build_agent_client_and_request(cmd).map_err(|err| format!("{err:#}"))?;
    request.profile = Some(PROVISION_PROFILE.to_string());
    if cfg!(target_os = "macos") {
        request.cwd = Some("/tmp".to_string());
    }

    let runtime = Runtime::new().map_err(|err| err.to_string())?;
    runtime.block_on(async move {
        client
            .execute(request)
            .await
            .map_err(|err| format!("world-agent /v1/execute request failed: {err}"))
    })
}

fn build_probe_command(requirements: &[AptSpecV1]) -> String {
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
    script.push_str("$SUDO env DEBIAN_FRONTEND=noninteractive apt-get update\n");
    script.push_str("$SUDO env DEBIAN_FRONTEND=noninteractive apt-get install -y ");
    script.push_str(&rendered);
    script.push('\n');
    format!("sh -lc {}", sh_quote(&script))
}

fn response_snippet(response: &agent_api_types::ExecuteResponse) -> String {
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
