use substrate_shell::scripts::{write_bash_preexec_script, BASH_PREEXEC_SCRIPT};

#[test]
fn bash_preexec_script_contains_hooks() {
    assert!(BASH_PREEXEC_SCRIPT.contains("__substrate_preexec"));
    assert!(BASH_PREEXEC_SCRIPT.contains("SHIM_TRACE_LOG"));
    assert!(BASH_PREEXEC_SCRIPT.contains("trap '__substrate_preexec' DEBUG"));
}

#[test]
fn write_bash_preexec_script_writes_constant() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("preexec.sh");

    write_bash_preexec_script(&path).unwrap();

    let written = std::fs::read_to_string(&path).unwrap();
    assert_eq!(written, BASH_PREEXEC_SCRIPT);
}
