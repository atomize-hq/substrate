//! Dispatch routing tests covering PTY heuristics, registry helpers, telemetry, and world init flows.
use super::*;
use crate::execution::agent_events::{self, clear_agent_event_sender, init_event_channel};
use agent_api_types::ExecuteStreamFrame;
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use serde_json::Value as JsonValue;
use std::env;
use std::sync::Mutex;
use substrate_common::agent_events::AgentEventKind;
use substrate_common::FsDiff;
use tokio::runtime::Runtime;

#[cfg(target_os = "linux")]
use anyhow::anyhow;
#[cfg(target_os = "linux")]
use serial_test::serial;

// Global mutex to ensure tests that modify environment run sequentially
static TEST_ENV_MUTEX: Mutex<()> = Mutex::new(());

// Helper to run tests with TEST_MODE set
fn with_test_mode<F: FnOnce()>(f: F) {
    // Lock the mutex to ensure exclusive access to environment
    let _guard = TEST_ENV_MUTEX.lock().unwrap();

    // Save original value if it exists
    let original = env::var("TEST_MODE").ok();

    env::set_var("TEST_MODE", "1");
    f();

    // Restore original value or remove
    match original {
        Some(val) => env::set_var("TEST_MODE", val),
        None => env::remove_var("TEST_MODE"),
    }
}

// PTY heuristics and wrappers

#[test]
fn test_sudo_wants_pty() {
    // sudo without flags should want PTY
    assert!(sudo_wants_pty(
        "sudo",
        &["sudo".to_string(), "apt".to_string()]
    ));

    // sudo with -n should not want PTY
    assert!(!sudo_wants_pty(
        "sudo",
        &["sudo".to_string(), "-n".to_string(), "apt".to_string()]
    ));
    assert!(!sudo_wants_pty(
        "sudo",
        &["sudo".to_string(), "--non-interactive".to_string()]
    ));

    // sudo with -S should not want PTY
    assert!(!sudo_wants_pty(
        "sudo",
        &["sudo".to_string(), "-S".to_string()]
    ));
    assert!(!sudo_wants_pty(
        "sudo",
        &["sudo".to_string(), "--stdin".to_string()]
    ));

    // Not sudo
    assert!(!sudo_wants_pty(
        "apt",
        &["apt".to_string(), "update".to_string()]
    ));
}

#[test]
fn test_is_interactive_shell() {
    // Plain shell invocation is interactive
    assert!(is_interactive_shell("bash", &["bash".to_string()]));
    assert!(is_interactive_shell("zsh", &["zsh".to_string()]));
    assert!(is_interactive_shell("sh", &["sh".to_string()]));

    // Shell with -c is not interactive (unless -i is also present)
    assert!(!is_interactive_shell(
        "bash",
        &[
            "bash".to_string(),
            "-c".to_string(),
            "echo hello".to_string()
        ]
    ));
    assert!(is_interactive_shell(
        "bash",
        &[
            "bash".to_string(),
            "-i".to_string(),
            "-c".to_string(),
            "echo hello".to_string()
        ]
    ));

    // Explicit interactive flag
    assert!(is_interactive_shell(
        "bash",
        &["bash".to_string(), "-i".to_string()]
    ));
    assert!(is_interactive_shell(
        "bash",
        &["bash".to_string(), "--interactive".to_string()]
    ));

    // Not a shell
    assert!(!is_interactive_shell("python", &["python".to_string()]));
}

#[test]
fn test_looks_like_repl() {
    // Plain interpreter invocation is REPL
    assert!(looks_like_repl("python", &["python".to_string()]));
    assert!(looks_like_repl("python3", &["python3".to_string()]));
    assert!(looks_like_repl("node", &["node".to_string()]));
    assert!(looks_like_repl("irb", &["irb".to_string()]));

    // With script file is not REPL
    assert!(!looks_like_repl(
        "python",
        &["python".to_string(), "script.py".to_string()]
    ));
    assert!(!looks_like_repl(
        "node",
        &["node".to_string(), "app.js".to_string()]
    ));

    // With inline code is not REPL
    assert!(!looks_like_repl(
        "python",
        &[
            "python".to_string(),
            "-c".to_string(),
            "print('hello')".to_string()
        ]
    ));
    assert!(!looks_like_repl(
        "node",
        &[
            "node".to_string(),
            "-e".to_string(),
            "console.log('hello')".to_string()
        ]
    ));

    // Force interactive with -i is REPL even with script
    assert!(looks_like_repl(
        "python",
        &[
            "python".to_string(),
            "-i".to_string(),
            "script.py".to_string()
        ]
    ));
    assert!(looks_like_repl(
        "python",
        &[
            "python".to_string(),
            "--interactive".to_string(),
            "-c".to_string(),
            "print()".to_string()
        ]
    ));

    // Not an interpreter
    assert!(!looks_like_repl("bash", &["bash".to_string()]));
}

#[test]
fn test_container_wants_pty() {
    // docker run -it
    assert!(container_wants_pty(
        "docker",
        &[
            "docker".to_string(),
            "run".to_string(),
            "-it".to_string(),
            "ubuntu".to_string()
        ]
    ));
    assert!(container_wants_pty(
        "docker",
        &[
            "docker".to_string(),
            "run".to_string(),
            "-ti".to_string(),
            "ubuntu".to_string()
        ]
    ));

    // docker run with separate -i and -t
    assert!(container_wants_pty(
        "docker",
        &[
            "docker".to_string(),
            "run".to_string(),
            "-i".to_string(),
            "-t".to_string(),
            "ubuntu".to_string()
        ]
    ));

    // docker exec -it
    assert!(container_wants_pty(
        "docker",
        &[
            "docker".to_string(),
            "exec".to_string(),
            "-it".to_string(),
            "container1".to_string(),
            "bash".to_string()
        ]
    ));

    // Only -i or only -t is not enough
    assert!(!container_wants_pty(
        "docker",
        &[
            "docker".to_string(),
            "run".to_string(),
            "-i".to_string(),
            "ubuntu".to_string()
        ]
    ));
    assert!(!container_wants_pty(
        "docker",
        &[
            "docker".to_string(),
            "run".to_string(),
            "-t".to_string(),
            "ubuntu".to_string()
        ]
    ));

    // kubectl exec -it
    assert!(container_wants_pty(
        "kubectl",
        &[
            "kubectl".to_string(),
            "exec".to_string(),
            "-it".to_string(),
            "pod1".to_string(),
            "--".to_string(),
            "bash".to_string()
        ]
    ));

    // kubectl exec with separate flags
    assert!(container_wants_pty(
        "kubectl",
        &[
            "kubectl".to_string(),
            "exec".to_string(),
            "-i".to_string(),
            "-t".to_string(),
            "pod1".to_string()
        ]
    ));

    // docker-compose run/exec
    assert!(container_wants_pty(
        "docker-compose",
        &[
            "docker-compose".to_string(),
            "run".to_string(),
            "-it".to_string(),
            "service1".to_string()
        ]
    ));

    // docker compose (space form)
    assert!(container_wants_pty(
        "docker",
        &[
            "docker".to_string(),
            "compose".to_string(),
            "run".to_string(),
            "-it".to_string(),
            "service1".to_string()
        ]
    ));
}

#[test]
fn test_wants_debugger_pty() {
    // Python debuggers
    assert!(wants_debugger_pty(
        "python",
        &[
            "python".to_string(),
            "-m".to_string(),
            "pdb".to_string(),
            "script.py".to_string()
        ]
    ));
    assert!(wants_debugger_pty(
        "python3",
        &["python3".to_string(), "-m".to_string(), "ipdb".to_string()]
    ));

    // Node debuggers
    assert!(wants_debugger_pty(
        "node",
        &[
            "node".to_string(),
            "inspect".to_string(),
            "app.js".to_string()
        ]
    ));
    assert!(wants_debugger_pty(
        "node",
        &[
            "node".to_string(),
            "--inspect".to_string(),
            "app.js".to_string()
        ]
    ));
    assert!(wants_debugger_pty(
        "node",
        &[
            "node".to_string(),
            "--inspect-brk".to_string(),
            "app.js".to_string()
        ]
    ));

    // Not debuggers
    assert!(!wants_debugger_pty(
        "python",
        &["python".to_string(), "script.py".to_string()]
    ));
    assert!(!wants_debugger_pty(
        "node",
        &["node".to_string(), "app.js".to_string()]
    ));
}

#[test]
fn test_git_wants_pty() {
    // git add -p needs PTY
    assert!(git_wants_pty(&[
        "git".to_string(),
        "add".to_string(),
        "-p".to_string()
    ]));
    assert!(git_wants_pty(&[
        "git".to_string(),
        "add".to_string(),
        "-i".to_string()
    ]));

    // git rebase -i needs PTY
    assert!(git_wants_pty(&[
        "git".to_string(),
        "rebase".to_string(),
        "-i".to_string(),
        "HEAD~3".to_string()
    ]));

    // git commit without message needs PTY (opens editor)
    assert!(git_wants_pty(&["git".to_string(), "commit".to_string()]));

    // git commit with -e forces editor even with -m
    assert!(git_wants_pty(&[
        "git".to_string(),
        "commit".to_string(),
        "-m".to_string(),
        "msg".to_string(),
        "-e".to_string()
    ]));
    assert!(git_wants_pty(&[
        "git".to_string(),
        "commit".to_string(),
        "-m".to_string(),
        "msg".to_string(),
        "--edit".to_string()
    ]));

    // git commit with message doesn't need PTY
    assert!(!git_wants_pty(&[
        "git".to_string(),
        "commit".to_string(),
        "-m".to_string(),
        "message".to_string()
    ]));
    assert!(!git_wants_pty(&[
        "git".to_string(),
        "commit".to_string(),
        "--message=message".to_string()
    ]));

    // git commit with --no-edit doesn't need PTY
    assert!(!git_wants_pty(&[
        "git".to_string(),
        "commit".to_string(),
        "--no-edit".to_string()
    ]));

    // Regular git commands don't need PTY
    assert!(!git_wants_pty(&["git".to_string(), "status".to_string()]));
    assert!(!git_wants_pty(&["git".to_string(), "push".to_string()]));
    assert!(!git_wants_pty(&["git".to_string(), "pull".to_string()]));
}

#[test]
fn test_has_top_level_shell_meta() {
    // Top-level metacharacters
    assert!(has_top_level_shell_meta("echo hello | grep h"));
    assert!(has_top_level_shell_meta("ls > file.txt"));
    assert!(has_top_level_shell_meta("cat < input.txt"));
    assert!(has_top_level_shell_meta("cmd1 && cmd2"));
    assert!(has_top_level_shell_meta("cmd1; cmd2"));

    // Metacharacters inside quotes don't count
    assert!(!has_top_level_shell_meta("echo 'hello | world'"));
    assert!(!has_top_level_shell_meta("echo \"hello > world\""));

    // Metacharacters inside subshells don't count
    assert!(!has_top_level_shell_meta("echo $(ls | grep txt)"));
    assert!(!has_top_level_shell_meta("echo `ls | grep txt`"));

    // No metacharacters
    assert!(!has_top_level_shell_meta("echo hello world"));
    assert!(!has_top_level_shell_meta("ls -la"));
}

#[test]
fn test_peel_wrappers() {
    // sshpass wrapper
    assert_eq!(
        peel_wrappers(&[
            "sshpass".to_string(),
            "-p".to_string(),
            "pass".to_string(),
            "ssh".to_string(),
            "host".to_string()
        ]),
        vec!["ssh".to_string(), "host".to_string()]
    );

    // timeout wrapper
    assert_eq!(
        peel_wrappers(&[
            "timeout".to_string(),
            "10".to_string(),
            "command".to_string()
        ]),
        vec!["command".to_string()]
    );
    assert_eq!(
        peel_wrappers(&[
            "timeout".to_string(),
            "-s".to_string(),
            "KILL".to_string(),
            "10".to_string(),
            "command".to_string()
        ]),
        vec!["command".to_string()]
    );

    // env wrapper
    assert_eq!(
        peel_wrappers(&[
            "env".to_string(),
            "VAR=val".to_string(),
            "command".to_string()
        ]),
        vec!["command".to_string()]
    );
    assert_eq!(
        peel_wrappers(&["env".to_string(), "-i".to_string(), "command".to_string()]),
        vec!["command".to_string()]
    );

    // stdbuf wrapper
    assert_eq!(
        peel_wrappers(&[
            "stdbuf".to_string(),
            "-oL".to_string(),
            "command".to_string()
        ]),
        vec!["command".to_string()]
    );

    // nice wrapper
    assert_eq!(
        peel_wrappers(&[
            "nice".to_string(),
            "-n".to_string(),
            "10".to_string(),
            "command".to_string()
        ]),
        vec!["command".to_string()]
    );

    // doas wrapper
    assert_eq!(
        peel_wrappers(&[
            "doas".to_string(),
            "-u".to_string(),
            "user".to_string(),
            "command".to_string()
        ]),
        vec!["command".to_string()]
    );

    // Not a wrapper
    assert_eq!(
        peel_wrappers(&["ls".to_string(), "-la".to_string()]),
        vec!["ls".to_string(), "-la".to_string()]
    );
}

#[test]
fn test_needs_pty_ssh() {
    with_test_mode(|| {
        // SSH without remote command needs PTY
        assert!(needs_pty("ssh host"), "ssh host should need PTY");

        // SSH with -t forces PTY
        assert!(needs_pty("ssh -t host"), "ssh -t host should need PTY");
        assert!(needs_pty("ssh -tt host"), "ssh -tt host should need PTY");
        assert!(
            needs_pty("ssh -t host ls"),
            "ssh -t host ls should need PTY"
        );

        // SSH with -T disables PTY
        assert!(!needs_pty("ssh -T host"), "ssh -T host should not need PTY");
        assert!(
            !needs_pty("ssh -T host ls"),
            "ssh -T host ls should not need PTY"
        );

        // SSH with remote command doesn't need PTY
        assert!(!needs_pty("ssh host ls"), "ssh host ls should not need PTY");
        assert!(!needs_pty("ssh host 'echo hello'"));

        // SSH with BatchMode=yes doesn't need PTY
        assert!(!needs_pty("ssh -o BatchMode=yes host"));
        assert!(!needs_pty("ssh -oBatchMode=yes host"));

        // SSH with RequestTTY options
        assert!(needs_pty("ssh -o RequestTTY=yes host"));
        assert!(needs_pty("ssh -oRequestTTY=force host"));
        assert!(!needs_pty("ssh -o RequestTTY=no host"));

        // SSH with -N (no remote command for port forwarding)
        assert!(!needs_pty("ssh -N -L 8080:localhost:80 host"));

        // SSH with -W (stdio forwarding)
        assert!(!needs_pty("ssh -W localhost:80 host"));
    });
}

#[test]
fn test_needs_pty_known_tuis() {
    with_test_mode(|| {
        // Known TUI editors
        assert!(needs_pty("vim"));
        assert!(needs_pty("vi"));
        assert!(needs_pty("nano"));
        assert!(needs_pty("emacs"));

        // Known TUI pagers
        assert!(needs_pty("less"));
        assert!(needs_pty("more"));

        // Known TUI monitors
        assert!(needs_pty("top"));
        assert!(needs_pty("htop"));
        assert!(needs_pty("btop"));

        // AI tools
        assert!(needs_pty("claude"));

        // Not TUIs
        assert!(!needs_pty("ls"));
        assert!(!needs_pty("cat"));
        assert!(!needs_pty("echo hello"));
    });
}

#[test]
fn test_needs_pty_shell_meta() {
    with_test_mode(|| {
        // Commands with pipes don't need PTY by default
        assert!(!needs_pty("ls | grep txt"));
        assert!(!needs_pty("echo hello > file.txt"));

        // Commands with && or ; don't need PTY
        assert!(!needs_pty("cmd1 && cmd2"));
        assert!(!needs_pty("cmd1; cmd2"));
    });
}

#[test]
fn test_is_force_pty_command() {
    // Save and remove SUBSTRATE_FORCE_PTY if it exists
    let old_force = std::env::var("SUBSTRATE_FORCE_PTY").ok();
    std::env::remove_var("SUBSTRATE_FORCE_PTY");

    // :pty prefix forces PTY
    assert!(is_force_pty_command(":pty ls"));
    assert!(is_force_pty_command(":pty echo hello"));

    // Regular commands without SUBSTRATE_FORCE_PTY
    assert!(!is_force_pty_command("ls"));
    assert!(!is_force_pty_command("echo hello"));

    // Test with SUBSTRATE_FORCE_PTY set
    std::env::set_var("SUBSTRATE_FORCE_PTY", "1");
    assert!(is_force_pty_command("ls"));
    assert!(is_force_pty_command("echo hello"));

    // Restore original state
    match old_force {
        Some(val) => std::env::set_var("SUBSTRATE_FORCE_PTY", val),
        None => std::env::remove_var("SUBSTRATE_FORCE_PTY"),
    }
}

#[test]
fn test_is_pty_disabled() {
    // Test with env var not set
    env::remove_var("SUBSTRATE_DISABLE_PTY");
    assert!(!is_pty_disabled());

    // Test with env var set
    env::set_var("SUBSTRATE_DISABLE_PTY", "1");
    assert!(is_pty_disabled());
    env::remove_var("SUBSTRATE_DISABLE_PTY");
}

#[test]
#[cfg(unix)]
fn test_stdin_nonblock_roundtrip() {
    // Test that O_NONBLOCK can be set and restored correctly
    // This verifies the save/restore behavior that TerminalGuard relies on
    use std::io;
    use std::os::unix::io::AsRawFd;

    unsafe {
        let fd = io::stdin().as_raw_fd();

        // Get current flags
        let flags_before = libc::fcntl(fd, libc::F_GETFL);
        assert!(flags_before != -1, "Failed to get stdin flags");

        // Mimic TerminalGuard behavior: set O_NONBLOCK
        let result = libc::fcntl(fd, libc::F_SETFL, flags_before | libc::O_NONBLOCK);
        assert!(result != -1, "Failed to set O_NONBLOCK");

        // Verify O_NONBLOCK is set
        let flags_after = libc::fcntl(fd, libc::F_GETFL);
        assert!(
            flags_after != -1,
            "Failed to get flags after setting O_NONBLOCK"
        );
        assert!(
            flags_after & libc::O_NONBLOCK != 0,
            "O_NONBLOCK should be set"
        );

        // Restore original flags
        let result = libc::fcntl(fd, libc::F_SETFL, flags_before);
        assert!(result != -1, "Failed to restore original flags");

        // Verify restoration
        let flags_restored = libc::fcntl(fd, libc::F_GETFL);
        assert!(flags_restored != -1, "Failed to get restored flags");
        assert_eq!(
            flags_restored & libc::O_NONBLOCK,
            flags_before & libc::O_NONBLOCK,
            "O_NONBLOCK state should be restored to original"
        );
    }
}

#[test]
fn test_needs_pty_integration() {
    with_test_mode(|| {
        // Interactive shells need PTY
        assert!(needs_pty("bash"));
        assert!(needs_pty("zsh"));

        // Shell with command doesn't need PTY
        assert!(!needs_pty("bash -c 'echo hello'"));

        // Python REPL needs PTY
        assert!(needs_pty("python"));
        assert!(needs_pty("python3"));

        // Python with script doesn't need PTY
        assert!(!needs_pty("python script.py"));

        // Docker run -it needs PTY
        assert!(needs_pty("docker run -it ubuntu"));

        // Git interactive commands need PTY
        assert!(needs_pty("git add -p"));
        assert!(needs_pty("git commit"));

        // Sudo needs PTY for password
        assert!(needs_pty("sudo apt update"));
        assert!(!needs_pty("sudo -n apt update"));
    });
}

#[test]
fn test_needs_pty_ssh_variations() {
    with_test_mode(|| {
        // SSH with -T flag should not get PTY
        assert!(!needs_pty("ssh -T host 'cmd'"));

        // SSH with -t flag should get PTY
        assert!(needs_pty("ssh -t host"));
        assert!(needs_pty("ssh -tt host"));

        // SSH with remote command (no -t) should not get PTY
        assert!(!needs_pty("ssh host ls"));
        assert!(!needs_pty("ssh host 'echo hello'"));

        // SSH with -l user form
        assert!(needs_pty("ssh -l user host"));
        assert!(!needs_pty("ssh -l user host ls"));

        // SSH with -- delimiter
        assert!(needs_pty("ssh -o SomeOption -- host"));
        assert!(!needs_pty("ssh -o SomeOption -- host ls"));

        // SSH with BatchMode should not get PTY
        assert!(!needs_pty("ssh -o BatchMode=yes host"));

        // SSH with RequestTTY option
        assert!(needs_pty("ssh -o RequestTTY=yes host"));
        assert!(needs_pty("ssh -o RequestTTY=force host"));
        assert!(!needs_pty("ssh -o RequestTTY=no host"));

        // SSH RequestTTY=auto behavior
        assert!(needs_pty("ssh -o RequestTTY=auto host")); // interactive login
        assert!(!needs_pty("ssh -o RequestTTY=auto host id")); // remote cmd, no -t

        // Test case-insensitive options
        assert!(needs_pty("ssh -o RequestTTY=YES host"));
        assert!(needs_pty("ssh -o RequestTTY=Force host"));
        assert!(!needs_pty("ssh -o RequestTTY=NO host"));
        assert!(!needs_pty("ssh -o BatchMode=YES host"));

        // Test inline -o format
        assert!(needs_pty("ssh -oRequestTTY=yes host"));
        assert!(needs_pty("ssh -oRequestTTY=force host"));
        assert!(!needs_pty("ssh -oRequestTTY=no host"));
        assert!(!needs_pty("ssh -oBatchMode=yes host"));

        // Test case-insensitive inline options
        assert!(needs_pty("ssh -oRequestTTY=Yes host"));
        assert!(!needs_pty("ssh -oRequestTTY=No host"));
        assert!(!needs_pty("ssh -oBatchMode=YES host"));

        // SSH with -W should not get PTY unless -t is explicit
        assert!(!needs_pty("ssh -W host:port jumphost"));
        assert!(needs_pty("ssh -t -W host:port jumphost"));

        // SSH with 2-arg options that could confuse host detection
        assert!(needs_pty("ssh -p 2222 host"));
        assert!(needs_pty("ssh -o StrictHostKeyChecking=no host"));
        assert!(!needs_pty("ssh -p 2222 host echo ok"));
        assert!(needs_pty("ssh -J jumphost host"));
        assert!(!needs_pty("ssh -J jumphost host 'id'"));

        // Plain SSH interactive login
        assert!(needs_pty("ssh host"));
        assert!(needs_pty("ssh -l user host"));
        assert!(needs_pty("ssh user@host"));

        // SSH -N flag (no remote command, typically for port forwarding)
        assert!(!needs_pty("ssh -N host"));
        assert!(!needs_pty("ssh -N -L 8080:localhost:80 host"));
        assert!(needs_pty("ssh -t -N host")); // -t forces PTY

        // SSH -O control operations
        assert!(!needs_pty("ssh -O check host"));
        assert!(!needs_pty("ssh -O exit host"));
        assert!(!needs_pty("ssh -O stop host"));
        assert!(needs_pty("ssh -t -O check host")); // -t forces PTY
    });
}

#[test]
fn test_needs_pty_quoted_args() {
    with_test_mode(|| {
        // Quoted filename with spaces
        assert!(needs_pty("vim 'a b.txt'"));
        assert!(needs_pty("vim \"file with spaces.txt\""));

        // Complex quoted arguments
        assert!(needs_pty("vim 'file (1).txt'"));
    });
}

#[test]
fn test_needs_pty_pipes_redirects() {
    with_test_mode(|| {
        // Pipes should prevent PTY
        assert!(!needs_pty("ls | less"));
        assert!(!needs_pty("vim file.txt | grep pattern"));

        // Redirects should prevent PTY
        assert!(!needs_pty("vim > output.txt"));
        assert!(!needs_pty("less < input.txt"));

        // Background jobs should prevent PTY
        assert!(!needs_pty("vim &"));

        // Command sequences should prevent PTY
        assert!(!needs_pty("vim; ls"));
    });
}

#[test]
fn test_repl_heuristic() {
    with_test_mode(|| {
        // Interactive REPL (no args) should get PTY
        assert!(needs_pty("python"));
        assert!(needs_pty("python3"));
        assert!(needs_pty("node"));

        // Script execution should NOT get PTY
        assert!(!needs_pty("python script.py"));
        assert!(!needs_pty("python3 /path/to/script.py"));
        assert!(!needs_pty("node app.js"));

        // Inline code should NOT get PTY
        assert!(!needs_pty("python -c 'print(1)'"));
        assert!(!needs_pty("node -e 'console.log(1)'"));
        assert!(!needs_pty("node -p '1+1'"));
        assert!(!needs_pty("node --print '1+1'"));
        assert!(!needs_pty("node --eval 'console.log(1)'"));

        // Explicit interactive flag should get PTY even with script
        assert!(needs_pty("python -i script.py"));
        assert!(needs_pty("python -i -c 'print(1)'"));
    });
}

#[test]
fn test_debugger_pty() {
    with_test_mode(|| {
        // Debuggers should get PTY
        assert!(needs_pty("python -m pdb script.py"));
        assert!(needs_pty("python3 -m ipdb script.py"));
        assert!(needs_pty("node inspect app.js"));
        assert!(needs_pty("node --inspect-brk app.js"));
        assert!(needs_pty("node --inspect script.js"));
    });
}

#[test]
fn test_windows_exe_handling() {
    with_test_mode(|| {
        // Windows-style paths with .exe should work
        if cfg!(windows) {
            assert!(needs_pty(r#"C:\Python\python.exe"#));
            assert!(needs_pty(r#"C:\Program Files\Git\usr\bin\ssh.exe"#));
            assert!(needs_pty(r#"vim.exe file.txt"#));
        }
    });
}

#[test]
fn test_container_k8s_pty() {
    with_test_mode(|| {
        // Docker/Podman commands with -it should get PTY
        assert!(needs_pty("docker run -it ubuntu bash"));
        assert!(needs_pty("docker exec -it container bash"));
        assert!(needs_pty("podman run -it alpine sh"));
        assert!(!needs_pty("docker run ubuntu echo hello"));

        // Only -t without -i should NOT get PTY (not fully interactive)
        assert!(!needs_pty("podman run -t alpine sh"));
        assert!(!needs_pty("docker run -t ubuntu bash"));

        // kubectl exec with -it should get PTY
        assert!(needs_pty("kubectl exec -it pod -- sh"));
        assert!(needs_pty("kubectl exec --stdin --tty pod -- bash"));
        assert!(!needs_pty("kubectl exec pod -- ls"));
        assert!(!needs_pty("kubectl exec --tty pod -- bash")); // Only -t, not -i

        // Container false positives and split flags
        assert!(!needs_pty("docker run --timeout=5s ubuntu echo hi"));
        assert!(needs_pty("docker exec -ti c bash"));
        assert!(needs_pty("kubectl exec --stdin --tty pod -- sh"));
        assert!(needs_pty("docker exec -i -t c bash"));
        assert!(needs_pty("docker exec -t -i c bash"));

        // Docker/Podman should NOT detect flags in the in-container command
        assert!(!needs_pty("docker run ubuntu bash -lc \"echo -t\""));
        assert!(!needs_pty("docker exec c sh -c 'echo -it'"));

        // Docker -- separator sanity test
        assert!(needs_pty("docker run -it -- ubuntu bash"));

        // docker-compose support (both forms)
        assert!(needs_pty("docker-compose exec -it web sh"));
        assert!(needs_pty("docker compose exec -it web sh"));
    });
}

#[test]
fn test_wrapper_commands() {
    with_test_mode(|| {
        // sshpass wrapper
        assert!(needs_pty("sshpass -p x ssh host"));
        assert!(!needs_pty("sshpass -p x ssh host ls"));

        // env wrapper with -i and -u flags
        assert!(needs_pty("env -i vim file"));
        assert!(needs_pty("env -u PATH bash"));
        assert!(needs_pty("env FOO=1 -i bash"));
        assert!(needs_pty("env FOO=1 ssh -t host"));
        assert!(needs_pty("env FOO=1 BAR=2 vim file.txt"));

        // timeout wrapper
        assert!(needs_pty("timeout 10s ssh host"));
        assert!(!needs_pty("timeout 10s ssh host ls"));

        // stdbuf wrapper
        assert!(needs_pty("stdbuf -oL less README.md"));
        assert!(needs_pty("stdbuf -oL vim file.txt"));

        // nice/ionice wrappers
        assert!(needs_pty("nice -n 10 vim file.txt"));
        assert!(needs_pty("ionice -n 5 less README.md"));

        // doas wrapper (sudo alternative)
        assert!(needs_pty("doas vim /etc/hosts"));
        assert!(needs_pty("doas -u user ssh host"));
    });
}

#[test]
fn test_pipeline_last_tui() {
    with_test_mode(|| {
        // This test requires SUBSTRATE_PTY_PIPELINE_LAST to be set
        let old_pipeline = std::env::var("SUBSTRATE_PTY_PIPELINE_LAST").ok();
        std::env::set_var("SUBSTRATE_PTY_PIPELINE_LAST", "1");

        // Pipeline with TUI at the end should get PTY
        assert!(needs_pty("ls | less"));
        assert!(needs_pty("git ls-files | fzf"));

        // Pipeline with redirect should NOT get PTY
        assert!(!needs_pty("ls | less > out.txt"));
        assert!(!needs_pty("git diff | head > changes.txt"));
        assert!(!needs_pty("ls | less 2>err.log"));
        assert!(!needs_pty("cmd | less < input.txt"));
        assert!(!needs_pty("ls | less >> append.txt"));
        assert!(!needs_pty("ls | less 2>&1"));

        // Restore SUBSTRATE_PTY_PIPELINE_LAST
        match old_pipeline {
            Some(val) => std::env::set_var("SUBSTRATE_PTY_PIPELINE_LAST", val),
            None => std::env::remove_var("SUBSTRATE_PTY_PIPELINE_LAST"),
        }
    });
}

#[test]
fn test_ssh_spacing_edge_cases() {
    with_test_mode(|| {
        // SSH with spaces around = in options (OpenSSH accepts this)
        assert!(needs_pty("ssh -o RequestTTY = yes host"));
        assert!(needs_pty("ssh -o RequestTTY = force host"));
        assert!(!needs_pty("ssh -o RequestTTY = no host"));
        assert!(!needs_pty("ssh -o BatchMode = yes host"));

        // -E and -B options with 2 args
        assert!(needs_pty("ssh -E logfile.txt host"));
        assert!(needs_pty("ssh -B 192.168.1.1 host"));
        assert!(!needs_pty("ssh -E log.txt host ls"));
    });
}

#[test]
fn test_force_vs_disable_precedence() {
    // Test that force overrides disable at the execute_command level
    let old_disable = std::env::var("SUBSTRATE_DISABLE_PTY").ok();
    let old_force = std::env::var("SUBSTRATE_FORCE_PTY").ok();

    // Test with both set - force should override
    std::env::set_var("SUBSTRATE_DISABLE_PTY", "1");
    std::env::set_var("SUBSTRATE_FORCE_PTY", "1");

    // is_force_pty_command should return true when SUBSTRATE_FORCE_PTY is set
    assert!(is_force_pty_command("echo hello"));
    assert!(is_force_pty_command("ls -l"));

    // :pty prefix should also force
    assert!(is_force_pty_command(":pty echo hello"));

    // is_pty_disabled should still return true
    assert!(is_pty_disabled());

    // Restore environment variables
    match old_disable {
        Some(val) => std::env::set_var("SUBSTRATE_DISABLE_PTY", val),
        None => std::env::remove_var("SUBSTRATE_DISABLE_PTY"),
    }
    match old_force {
        Some(val) => std::env::set_var("SUBSTRATE_FORCE_PTY", val),
        None => std::env::remove_var("SUBSTRATE_FORCE_PTY"),
    }
}

#[test]
fn test_git_commit_edit_flag() {
    with_test_mode(|| {
        // git commit -e can override -m to open editor
        assert!(needs_pty("git commit -m 'msg' -e"));
        assert!(needs_pty("git commit -m 'msg' --edit"));

        // --no-edit overrides -e
        assert!(!needs_pty("git commit -e --no-edit"));
        assert!(!needs_pty("git commit --edit --no-edit"));
    });
}

// Registry helpers
#[test]
fn parse_demo_burst_command_defaults() {
    assert_eq!(parse_demo_burst_command(":demo-burst"), Some((4, 400, 0)));
    assert_eq!(
        parse_demo_burst_command(":demo-burst 3 10 5"),
        Some((3, 10, 5))
    );
    assert!(parse_demo_burst_command(":other").is_none());
}

// Telemetry stream handling
#[test]
fn consume_agent_stream_buffer_emits_agent_events() {
    let _guard = agent_events::acquire_event_test_guard();
    let rt = Runtime::new().expect("runtime");
    rt.block_on(async {
        let mut rx = init_event_channel();

        let frames = [
            ExecuteStreamFrame::Stdout {
                chunk_b64: BASE64.encode("hello"),
            },
            ExecuteStreamFrame::Stderr {
                chunk_b64: BASE64.encode("oops"),
            },
            ExecuteStreamFrame::Exit {
                exit: 0,
                span_id: "spn_test".into(),
                scopes_used: vec!["scope:a".into()],
                fs_diff: None,
            },
        ];

        let mut buffer = Vec::new();
        for frame in frames {
            let mut line = serde_json::to_vec(&frame).expect("serialize frame");
            line.push(b'\n');
            buffer.extend(line);
        }

        let mut exit_code = None;
        let mut scopes_used = Vec::new();
        let mut fs_diff = None;

        consume_agent_stream_buffer(
            "tester",
            &mut buffer,
            &mut exit_code,
            &mut scopes_used,
            &mut fs_diff,
        )
        .expect("consume stream");

        let stdout_event = rx.recv().await.expect("stdout event");
        assert_eq!(stdout_event.kind, AgentEventKind::PtyData);
        assert_eq!(stdout_event.data["chunk"], "hello");
        assert_eq!(stdout_event.data["stream"], "stdout");

        let stderr_event = rx.recv().await.expect("stderr event");
        assert_eq!(stderr_event.kind, AgentEventKind::PtyData);
        assert_eq!(stderr_event.data["chunk"], "oops");
        assert_eq!(stderr_event.data["stream"], "stderr");

        assert_eq!(exit_code, Some(0));
        assert_eq!(scopes_used, vec!["scope:a".to_string()]);
        assert!(fs_diff.is_none());
    });
    clear_agent_event_sender();
}

#[test]
fn parse_fs_diff_from_agent_json() {
    let sample = r#"{
        "exit":0,
        "span_id":"spn_x",
        "stdout_b64":"",
        "stderr_b64":"",
        "scopes_used":["tcp:example.com:443"],
        "fs_diff":{
            "writes":["/tmp/t/a.txt"],
            "mods":[],
            "deletes":[],
            "truncated":false
        }
    }"#;
    let v: JsonValue = serde_json::from_str(sample).unwrap();
    let fd_val = v.get("fs_diff").cloned().unwrap();
    let diff: FsDiff = serde_json::from_value(fd_val).unwrap();
    assert_eq!(diff.writes.len(), 1);
    assert_eq!(diff.writes[0], std::path::PathBuf::from("/tmp/t/a.txt"));
    assert!(diff.mods.is_empty());
    assert!(diff.deletes.is_empty());
    assert!(!diff.truncated);
}

// Linux world initialization paths
#[cfg(target_os = "linux")]
mod linux_world_tests {
    use super::*;
    use std::env;

    fn clear_env() {
        env::remove_var("SUBSTRATE_WORLD");
        env::remove_var("SUBSTRATE_WORLD_ID");
        env::remove_var("SUBSTRATE_TEST_LOCAL_WORLD_ID");
    }

    #[test]
    #[serial]
    fn agent_probe_enables_world() {
        clear_env();
        let outcome = init_linux_world_with_probe(false, || Ok(()));
        assert_eq!(outcome, LinuxWorldInit::Agent);
        assert_eq!(env::var("SUBSTRATE_WORLD").unwrap(), "enabled");
        assert!(env::var("SUBSTRATE_WORLD_ID").is_err());
    }

    #[test]
    #[serial]
    fn fallback_uses_local_backend_stub() {
        clear_env();
        env::set_var("SUBSTRATE_TEST_LOCAL_WORLD_ID", "wld_test_stub");
        let outcome = init_linux_world_with_probe(false, || Err(anyhow!("no agent")));
        assert_eq!(outcome, LinuxWorldInit::LocalBackend);
        assert_eq!(env::var("SUBSTRATE_WORLD").unwrap(), "enabled");
        assert_eq!(env::var("SUBSTRATE_WORLD_ID").unwrap(), "wld_test_stub");
    }

    #[test]
    #[serial]
    fn disabled_skips_initialization() {
        clear_env();
        let outcome = init_linux_world_with_probe(true, || Ok(()));
        assert_eq!(outcome, LinuxWorldInit::Disabled);
        assert!(env::var("SUBSTRATE_WORLD").is_err());
        assert!(env::var("SUBSTRATE_WORLD_ID").is_err());
    }
}
