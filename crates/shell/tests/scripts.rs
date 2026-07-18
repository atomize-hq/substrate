use substrate_shell::scripts::{write_bash_preexec_script, BASH_PREEXEC_SCRIPT};
#[cfg(unix)]
use transport_api_types::{
    InstallBootstrapContextCarrierV1, InstallBootstrapContextV1, PlatformPrincipalV1,
};

#[cfg(unix)]
fn current_install_context(prefix: &std::path::Path) -> InstallBootstrapContextCarrierV1 {
    let output = std::process::Command::new("id")
        .args(["-u"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let uid = String::from_utf8(output.stdout)
        .unwrap()
        .trim()
        .parse::<u32>()
        .unwrap();
    let output = std::process::Command::new("id")
        .args(["-nu", &uid.to_string()])
        .output()
        .unwrap();
    assert!(output.status.success());
    let account = String::from_utf8(output.stdout).unwrap().trim().to_string();
    let principal = PlatformPrincipalV1::Unix { account, uid };
    let PlatformPrincipalV1::Unix { account, uid } = principal else {
        unreachable!();
    };
    let context =
        InstallBootstrapContextV1::new_unix(prefix.to_str().unwrap(), &account, uid).unwrap();
    InstallBootstrapContextCarrierV1::from_context(context).unwrap()
}

#[test]
fn bash_preexec_script_contains_hooks() {
    assert!(BASH_PREEXEC_SCRIPT.contains("__substrate_preexec"));
    assert!(BASH_PREEXEC_SCRIPT.contains("SHIM_TRACE_LOG"));
    assert!(BASH_PREEXEC_SCRIPT.contains("trap '__substrate_preexec' DEBUG"));
}

#[test]
fn write_bash_preexec_script_writes_constant() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(".substrate_preexec");
    #[cfg(unix)]
    let context = current_install_context(dir.path());

    #[cfg(unix)]
    write_bash_preexec_script(&path, &context).unwrap();
    #[cfg(not(unix))]
    write_bash_preexec_script(&path).unwrap();

    let written = std::fs::read_to_string(&path).unwrap();
    #[cfg(unix)]
    {
        assert!(written.ends_with(BASH_PREEXEC_SCRIPT));
        assert!(written.contains(&context.host_context_commitment));
        assert!(written.contains("SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1"));
        assert!(!written.contains("$HOME/.substrate"));
        assert!(!written.contains("~/.bashrc"));
    }
    #[cfg(not(unix))]
    assert_eq!(written, BASH_PREEXEC_SCRIPT);
}

#[cfg(unix)]
#[test]
fn write_bash_preexec_script_rejects_final_symlink_without_following_it() {
    let selected = tempfile::tempdir().unwrap();
    let ambient = tempfile::tempdir().unwrap();
    let path = selected.path().join(".substrate_preexec");
    let victim = ambient.path().join("victim");
    std::fs::write(&victim, "ambient-b-sentinel\n").unwrap();
    std::os::unix::fs::symlink(&victim, &path).unwrap();
    let context = current_install_context(selected.path());

    assert!(write_bash_preexec_script(&path, &context).is_err());

    assert_eq!(
        std::fs::read_to_string(&victim).unwrap(),
        "ambient-b-sentinel\n"
    );
    assert!(std::fs::symlink_metadata(&path)
        .unwrap()
        .file_type()
        .is_symlink());
}
