#[cfg(unix)]
mod unix {
    use std::{fs, process::Command, time::Duration};

    use claude_code::{ClaudeClient, ClaudeCodeError};
    use tempfile::TempDir;
    use tokio::time;

    fn pid_exists(pid: i32) -> bool {
        Command::new("kill")
            .arg("-0")
            .arg(pid.to_string())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
    }

    async fn assert_pid_gone(pid: i32) {
        let deadline = time::Instant::now() + Duration::from_millis(500);
        loop {
            if !pid_exists(pid) {
                return;
            }

            if time::Instant::now() >= deadline {
                panic!("expected pid {pid} to be gone, but it still exists");
            }

            time::sleep(Duration::from_millis(25)).await;
        }
    }

    #[tokio::test]
    async fn run_command_timeout_reaps_child() {
        use std::os::unix::fs::PermissionsExt;

        let dir = TempDir::new().expect("temp dir");
        let pid_file = dir.path().join("pid.txt");
        let script_path = dir.path().join("fake-claude");

        let script = r#"#!/bin/sh
set -eu
: "${PID_FILE:?missing PID_FILE}"
echo $$ > "$PID_FILE"
exec sleep 1000000
"#;

        fs::write(&script_path, script).expect("write script");
        let mut perms = fs::metadata(&script_path).expect("metadata").permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms).expect("chmod");

        let client = ClaudeClient::builder()
            .binary(&script_path)
            .env("PID_FILE", pid_file.to_string_lossy().to_string())
            .timeout(Some(Duration::from_millis(750)))
            .build();

        let err = client.version().await.unwrap_err();
        assert!(
            matches!(err, ClaudeCodeError::Timeout { .. }),
            "expected timeout, got: {err:?}"
        );

        let pid: i32 = {
            let deadline = time::Instant::now() + Duration::from_secs(1);
            loop {
                match fs::read_to_string(&pid_file) {
                    Ok(contents) => {
                        let trimmed = contents.trim();
                        if let Ok(pid) = trimmed.parse() {
                            break pid;
                        }
                    }
                    Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
                    Err(err) => panic!("failed to read pid file: {err}"),
                }

                if time::Instant::now() >= deadline {
                    panic!("pid file was not populated before timeout");
                }
                time::sleep(Duration::from_millis(25)).await;
            }
        };
        assert_pid_gone(pid).await;
    }
}
