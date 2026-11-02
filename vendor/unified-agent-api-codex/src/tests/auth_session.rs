use super::*;

#[tokio::test]
async fn auth_helper_uses_app_scoped_home_without_mutating_env() {
    let _guard = env_guard_async().await;
    let temp = tempfile::tempdir().unwrap();
    let log_path = temp.path().join("auth.log");
    let app_home = temp.path().join("app-home");
    let caller_home = temp.path().join("caller-home");
    let previous_home = env::var("CODEX_HOME").ok();
    env::set_var("CODEX_HOME", &caller_home);
    env::set_var("AUTH_HELPER_LOG", &log_path);

    let script = r#"#!/usr/bin/env bash
set -e
echo "args:$*" >> "$AUTH_HELPER_LOG"
echo "CODEX_HOME=${CODEX_HOME:-missing}" >> "$AUTH_HELPER_LOG"
if [[ "$1" == "login" && "$2" == "status" ]]; then
  echo "Logged in using ChatGPT"
  exit 0
fi
echo "Not logged in" >&2
exit 1
"#;
    let binary = write_fake_codex(temp.path(), script);
    let helper = AuthSessionHelper::with_client(
        CodexClient::builder()
            .binary(&binary)
            .codex_home(&app_home)
            .build(),
    );

    let status = helper.status().await.unwrap();
    assert!(matches!(
        status,
        CodexAuthStatus::LoggedIn(CodexAuthMethod::ChatGpt)
    ));

    let logged = std_fs::read_to_string(&log_path).unwrap();
    assert!(logged.contains("args:login status"));
    assert!(logged.contains(&format!("CODEX_HOME={}", app_home.display())));

    assert_eq!(
        env::var("CODEX_HOME").unwrap(),
        caller_home.display().to_string()
    );

    env::remove_var("AUTH_HELPER_LOG");
    if let Some(previous) = previous_home {
        env::set_var("CODEX_HOME", previous);
    } else {
        env::remove_var("CODEX_HOME");
    }
}

#[tokio::test]
async fn ensure_api_key_login_runs_when_logged_out() {
    let _guard = env_guard_async().await;
    let temp = tempfile::tempdir().unwrap();
    let log_path = temp.path().join("login.log");
    let state_path = temp.path().join("api-key-state");
    let script = format!(
        r#"#!/usr/bin/env bash
set -e
echo "$@" >> "{log}"
if [[ "$1" == "login" && "$2" == "status" ]]; then
  if [[ -f "{state}" ]]; then
echo "Logged in using an API key - sk-already"
exit 0
  fi
  echo "Not logged in" >&2
  exit 1
fi
if [[ "$1" == "login" && "$2" == "--api-key" ]]; then
  echo "Logged in using an API key - $3" > "{state}"
  echo "Logged in using an API key - $3"
  exit 0
fi
echo "unexpected args: $*" >&2
exit 2
"#,
        log = log_path.display(),
        state = state_path.display()
    );
    let binary = write_fake_codex(temp.path(), &script);
    let helper = AuthSessionHelper::with_client(
        CodexClient::builder()
            .binary(&binary)
            .codex_home(temp.path().join("app-home"))
            .build(),
    );

    let status = helper.ensure_api_key_login("sk-test-key").await.unwrap();
    match status {
        CodexAuthStatus::LoggedIn(CodexAuthMethod::ApiKey { masked_key }) => {
            assert_eq!(masked_key.as_deref(), Some("sk-test-key"));
        }
        other => panic!("unexpected status: {other:?}"),
    }

    let second = helper.ensure_api_key_login("sk-other").await.unwrap();
    assert!(matches!(
        second,
        CodexAuthStatus::LoggedIn(CodexAuthMethod::ApiKey { .. })
    ));

    let log = std_fs::read_to_string(&log_path).unwrap();
    assert!(log.contains("login status"));
    assert!(log.contains("login --api-key sk-test-key"));
    assert_eq!(
        log.lines()
            .filter(|line| line.contains("--api-key"))
            .count(),
        1
    );
}

#[tokio::test]
async fn ensure_chatgpt_login_launches_when_needed() {
    let _guard = env_guard_async().await;
    let temp = tempfile::tempdir().unwrap();
    let log_path = temp.path().join("chatgpt.log");
    let state_path = temp.path().join("chatgpt-state");
    let script = format!(
        r#"#!/usr/bin/env bash
set -e
echo "$@" >> "{log}"
if [[ "$1" == "login" && "$2" == "status" ]]; then
  if [[ -f "{state}" ]]; then
echo "Logged in using ChatGPT"
exit 0
  fi
  echo "Not logged in" >&2
  exit 1
fi
if [[ "$1" == "login" && -z "$2" ]]; then
  echo "Logged in using ChatGPT" > "{state}"
  echo "Logged in using ChatGPT"
  exit 0
fi
echo "unknown args: $*" >&2
exit 2
"#,
        log = log_path.display(),
        state = state_path.display()
    );
    let binary = write_fake_codex(temp.path(), &script);
    let helper = AuthSessionHelper::with_client(
        CodexClient::builder()
            .binary(&binary)
            .codex_home(temp.path().join("app-home"))
            .build(),
    );

    let child = helper.ensure_chatgpt_login().await.unwrap();
    let child = child.expect("expected ChatGPT login child");
    let output = child.wait_with_output().await.unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Logged in using ChatGPT"));

    let second = helper.ensure_chatgpt_login().await.unwrap();
    assert!(second.is_none());

    let log = std_fs::read_to_string(&log_path).unwrap();
    assert!(log.lines().any(|line| line == "login"));
    assert_eq!(log.lines().filter(|line| line == &"login").count(), 1);
}

#[test]
fn parses_chatgpt_login() {
    let message = "Logged in using ChatGPT";
    let parsed = parse_login_success(message);
    assert!(matches!(
        parsed,
        Some(CodexAuthStatus::LoggedIn(CodexAuthMethod::ChatGpt))
    ));
}

#[test]
fn parses_api_key_login() {
    let message = "Logged in using an API key - sk-1234***abcd";
    let parsed = parse_login_success(message);
    match parsed {
        Some(CodexAuthStatus::LoggedIn(CodexAuthMethod::ApiKey { masked_key })) => {
            assert_eq!(masked_key.as_deref(), Some("sk-1234***abcd"));
        }
        other => panic!("unexpected status: {other:?}"),
    }
}

#[test]
fn parse_login_accepts_unknown_on_success() {
    let message = "Authenticated";
    assert!(parse_login_success(message).is_none());
    let status = CodexAuthStatus::LoggedIn(CodexAuthMethod::Unknown {
        raw: message.to_string(),
    });
    assert!(matches!(
        status,
        CodexAuthStatus::LoggedIn(CodexAuthMethod::Unknown { .. })
    ));
}
