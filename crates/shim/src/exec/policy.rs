use anyhow::Result;
use std::{env, ffi::OsString, path::Path};
use substrate_broker::{policy_mode, quick_check, Decision, PolicyMode};
use substrate_trace::{ActiveSpan, PolicyDecision as TracePolicyDecision, TraceContext};

pub(crate) struct PolicyContext {
    pub(crate) span: Option<PolicySpan>,
}

pub(crate) struct PolicySpan {
    span: ActiveSpan,
    _parent_span_guard: ParentSpanGuard,
}

struct ParentSpanGuard {
    previous: Option<OsString>,
}

impl ParentSpanGuard {
    fn set_current(span_id: &str) -> Self {
        let previous = env::var_os("SHIM_PARENT_SPAN");
        env::set_var("SHIM_PARENT_SPAN", span_id);
        Self { previous }
    }
}

impl Drop for ParentSpanGuard {
    fn drop(&mut self) {
        match self.previous.take() {
            Some(value) => env::set_var("SHIM_PARENT_SPAN", value),
            None => env::remove_var("SHIM_PARENT_SPAN"),
        }
    }
}

impl PolicySpan {
    pub(crate) fn set_outcome(&mut self, outcome: &'static str) {
        self.span.set_outcome(outcome);
    }

    pub(crate) fn finish(
        self,
        exit_code: i32,
        scopes_used: Vec<String>,
        fs_diff: Option<substrate_common::FsDiff>,
    ) -> Result<()> {
        let PolicySpan {
            span,
            _parent_span_guard,
        } = self;
        span.finish(exit_code, scopes_used, fs_diff)
    }

    pub(crate) fn get_span_id(&self) -> &str {
        self.span.get_span_id()
    }
}

pub(crate) enum PolicyResult {
    Proceed(Box<PolicyContext>),
    Deny(i32),
}

pub(crate) fn evaluate_policy(
    trace_context: &TraceContext,
    command_str: &str,
    cwd: &Path,
    argv: &[String],
) -> Result<PolicyResult> {
    let _ = trace_context.init_trace(None);
    let mode = policy_mode();
    let mut policy_decision = None;

    if mode == PolicyMode::Disabled {
        return Ok(PolicyResult::Proceed(Box::new(PolicyContext {
            span: start_span(trace_context, command_str, cwd, None)?,
        })));
    }

    match quick_check(argv, cwd.to_str().unwrap_or(".")) {
        Ok(Decision::Allow) => {
            policy_decision = Some(TracePolicyDecision {
                action: "allow".to_string(),
                restrictions: None,
                reason: None,
            });
        }
        Ok(Decision::AllowWithRestrictions(restrictions)) => {
            policy_decision = Some(TracePolicyDecision {
                action: "allow_with_restrictions".to_string(),
                restrictions: Some(restrictions.iter().map(|r| format!("{:?}", r)).collect()),
                reason: None,
            });
        }
        Ok(Decision::Deny(reason)) => {
            if mode == PolicyMode::Enforce {
                eprintln!("substrate: command denied by policy: {}", reason);
                policy_decision = Some(TracePolicyDecision {
                    action: "deny".to_string(),
                    restrictions: None,
                    reason: Some(reason.clone()),
                });
                return Ok(deny_with_span(
                    trace_context,
                    command_str,
                    cwd,
                    policy_decision,
                ));
            }

            policy_decision = Some(TracePolicyDecision {
                action: "deny".to_string(),
                restrictions: None,
                reason: Some(format!("would deny (policy.mode=observe): {reason}")),
            });
        }
        Err(e) => {
            eprintln!("substrate: policy check failed: {}", e);
        }
    }

    Ok(PolicyResult::Proceed(Box::new(PolicyContext {
        span: start_span(trace_context, command_str, cwd, policy_decision)?,
    })))
}

fn deny_with_span(
    trace_context: &TraceContext,
    command_str: &str,
    cwd: &Path,
    policy_decision: Option<TracePolicyDecision>,
) -> PolicyResult {
    if let Ok(Some(mut span)) = start_span(trace_context, command_str, cwd, policy_decision) {
        span.set_outcome("denied");
        let _ = span.finish(126, vec![], None);
    }

    PolicyResult::Deny(126)
}

fn start_span(
    trace_context: &TraceContext,
    command_str: &str,
    cwd: &Path,
    policy_decision: Option<TracePolicyDecision>,
) -> Result<Option<PolicySpan>> {
    let mut builder = trace_context
        .create_span_builder()
        .with_command(command_str)
        .with_cwd(cwd.to_str().unwrap_or("."));

    if let Some(pd) = policy_decision {
        builder = builder.with_policy_decision(pd);
    }

    let span = match builder.start() {
        Ok(span) => Some(PolicySpan {
            _parent_span_guard: ParentSpanGuard::set_current(span.get_span_id()),
            span,
        }),
        Err(e) => {
            eprintln!("substrate: failed to create span: {}", e);
            None
        }
    };

    Ok(span)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use serial_test::serial;
    use std::{env, fs, path::Path, process::Command};
    use substrate_broker::{detect_profile, set_global_broker, set_observe_only, BrokerHandle};
    use tempfile::TempDir;

    fn new_trace_context(prefix: &Path) -> substrate_trace::TraceContext {
        substrate_trace::TraceContext::explicit_product(prefix).expect("trace context")
    }

    fn initialize_test_git_repository(path: &Path, policy_yaml: &str) -> String {
        fs::create_dir_all(path).unwrap();
        let run = |args: &[&str]| {
            let output = Command::new("git")
                .arg("-C")
                .arg(path)
                .args(args)
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "git {:?} failed: {}",
                args,
                String::from_utf8_lossy(&output.stderr)
            );
            output
        };
        run(&["init", "--quiet"]);
        run(&["config", "user.name", "Substrate Shim Test"]);
        run(&["config", "user.email", "shim-test@substrate.invalid"]);
        fs::write(path.join("policy.yaml"), policy_yaml).unwrap();
        run(&["add", "policy.yaml"]);
        run(&["commit", "--quiet", "-m", "test policy"]);
        String::from_utf8(run(&["rev-parse", "HEAD"]).stdout)
            .unwrap()
            .trim()
            .to_string()
    }

    fn allow_policy_yaml(policy_id: &str, cmd: &str) -> String {
        format!("id: \"{policy_id}\"\nname: \"{policy_id}\"\ncmd_allowed:\n  - \"{cmd}\"\n")
    }

    fn deny_policy_yaml(policy_id: &str, cmd: &str) -> String {
        format!("id: \"{policy_id}\"\nname: \"{policy_id}\"\ncmd_denied:\n  - \"{cmd}\"\n")
    }

    fn write_workspace_policy(root: &Path, policy_yaml: &str) {
        let substrate_dir = root.join(".substrate");
        fs::create_dir_all(&substrate_dir).unwrap();
        fs::write(substrate_dir.join("workspace.yaml"), "version: 1\n").unwrap();
        fs::write(substrate_dir.join("policy.yaml"), policy_yaml).unwrap();
    }

    fn latest_command_complete(trace_path: &Path) -> Value {
        fs::read_to_string(trace_path)
            .unwrap()
            .lines()
            .filter_map(|line| serde_json::from_str::<Value>(line).ok())
            .find(|value| value["event_type"] == "command_complete")
            .expect("command_complete entry")
    }

    #[test]
    #[serial]
    fn evaluate_policy_allows_when_command_not_denied() {
        let _ = set_global_broker(BrokerHandle::new());
        set_observe_only(true);
        let temp = TempDir::new().unwrap();
        let trace_context = new_trace_context(temp.path());

        let argv = vec!["echo".to_string(), "ok".to_string()];
        let result = evaluate_policy(&trace_context, "echo ok", Path::new("."), &argv)
            .expect("policy evaluation should succeed");
        assert!(matches!(result, PolicyResult::Proceed(_)));
    }

    #[test]
    #[serial]
    fn evaluate_policy_denies_blocked_command_when_enforced() {
        let _ = set_global_broker(BrokerHandle::new());
        set_observe_only(false);
        let temp = TempDir::new().unwrap();
        let trace_context = new_trace_context(temp.path());

        let argv = vec!["rm".to_string(), "-rf".to_string(), "/tmp".to_string()];
        match evaluate_policy(&trace_context, "rm -rf /tmp", Path::new("/tmp"), &argv)
            .expect("policy evaluation should succeed")
        {
            PolicyResult::Deny(code) => assert_eq!(code, 126),
            PolicyResult::Proceed(_) => panic!("expected deny result"),
        }

        set_observe_only(true);
    }

    #[test]
    #[serial]
    fn start_span_restores_previous_parent_span_on_finish() {
        let previous = env::var_os("SHIM_PARENT_SPAN");
        env::set_var("SHIM_PARENT_SPAN", "spn_previous");
        let temp = TempDir::new().unwrap();
        let trace_context = new_trace_context(temp.path());

        let span = start_span(&trace_context, "echo ok", Path::new("."), None)
            .expect("span setup should succeed")
            .expect("span should be created");
        let current_span_id = span.get_span_id().to_string();
        assert_eq!(
            env::var("SHIM_PARENT_SPAN").ok(),
            Some(current_span_id),
            "expected active shim span to be published while command is running"
        );

        span.finish(0, vec![], None)
            .expect("span finish should succeed");
        assert_eq!(
            env::var("SHIM_PARENT_SPAN").ok(),
            Some("spn_previous".to_string()),
            "expected previous parent span to be restored after finish"
        );

        match previous {
            Some(value) => env::set_var("SHIM_PARENT_SPAN", value),
            None => env::remove_var("SHIM_PARENT_SPAN"),
        }
    }

    #[test]
    #[serial]
    fn start_span_unsets_parent_span_when_none_existed() {
        let previous = env::var_os("SHIM_PARENT_SPAN");
        env::remove_var("SHIM_PARENT_SPAN");
        let temp = TempDir::new().unwrap();
        let trace_context = new_trace_context(temp.path());

        let span = start_span(&trace_context, "echo ok", Path::new("."), None)
            .expect("span setup should succeed")
            .expect("span should be created");
        assert!(
            env::var_os("SHIM_PARENT_SPAN").is_some(),
            "expected shim span to set SHIM_PARENT_SPAN while active"
        );

        drop(span);
        assert!(
            env::var_os("SHIM_PARENT_SPAN").is_none(),
            "expected SHIM_PARENT_SPAN to be unset when no previous parent existed"
        );

        match previous {
            Some(value) => env::set_var("SHIM_PARENT_SPAN", value),
            None => env::remove_var("SHIM_PARENT_SPAN"),
        }
    }

    #[test]
    #[serial]
    fn evaluate_policy_uses_bound_prefix_for_trace_and_policy_git() {
        let _ = set_global_broker(BrokerHandle::new());
        set_observe_only(false);

        let previous_home = env::var_os("HOME");
        let previous_trace = env::var_os("SHIM_TRACE_LOG");
        let previous_substrate_home = env::var_os("SUBSTRATE_HOME");
        let previous_substrate_root = env::var_os("SUBSTRATE_ROOT");

        let temp = TempDir::new().unwrap();
        let prefix_a = temp.path().join("selected-a");
        let prefix_b = temp.path().join("ambient-b");
        let commit_a =
            initialize_test_git_repository(&prefix_a, &allow_policy_yaml("selected-a", "echo ok"));
        let commit_b =
            initialize_test_git_repository(&prefix_b, &deny_policy_yaml("ambient-b", "echo ok"));
        write_workspace_policy(&prefix_b, &deny_policy_yaml("ambient-workspace", "echo ok"));
        env::set_var("HOME", &prefix_b);
        env::set_var("SHIM_TRACE_LOG", prefix_b.join("trace.jsonl"));
        env::set_var("SUBSTRATE_HOME", &prefix_b);
        env::set_var("SUBSTRATE_ROOT", &prefix_b);

        let trace_context = new_trace_context(&prefix_a);
        trace_context.set_policy_id("selected-a");
        env::set_var("SUBSTRATE_HOME", &prefix_a);
        detect_profile(&prefix_a).unwrap();
        env::set_var("SUBSTRATE_HOME", &prefix_b);
        let argv = vec!["echo".to_string(), "ok".to_string()];

        for _ in 0..2 {
            match evaluate_policy(&trace_context, "echo ok", Path::new("."), &argv).unwrap() {
                PolicyResult::Proceed(context) => {
                    let span = context.span.expect("span should exist");
                    span.finish(0, vec![], None).unwrap();
                }
                PolicyResult::Deny(code) => panic!("unexpected deny result: {code}"),
            }
        }

        let trace_path = prefix_a.join("trace.jsonl");
        assert!(trace_path.is_file());
        assert!(!prefix_b.join("trace.jsonl").exists());

        let entries = fs::read_to_string(&trace_path).unwrap();
        let complete_entries = entries
            .lines()
            .filter_map(|line| serde_json::from_str::<Value>(line).ok())
            .filter(|value| value["event_type"] == "command_complete")
            .collect::<Vec<_>>();
        assert_eq!(complete_entries.len(), 2);
        for value in complete_entries {
            assert_eq!(value["policy_id"].as_str(), Some("selected-a"));
            assert_eq!(
                value["replay_context"]["policy_id"].as_str(),
                Some("selected-a")
            );
            assert_eq!(
                value["replay_context"]["policy_commit"].as_str(),
                Some(commit_a.as_str())
            );
            assert_ne!(
                value["replay_context"]["policy_commit"].as_str(),
                Some(commit_b.as_str())
            );
        }

        match previous_home {
            Some(value) => env::set_var("HOME", value),
            None => env::remove_var("HOME"),
        }
        match previous_trace {
            Some(value) => env::set_var("SHIM_TRACE_LOG", value),
            None => env::remove_var("SHIM_TRACE_LOG"),
        }
        match previous_substrate_home {
            Some(value) => env::set_var("SUBSTRATE_HOME", value),
            None => env::remove_var("SUBSTRATE_HOME"),
        }
        match previous_substrate_root {
            Some(value) => env::set_var("SUBSTRATE_ROOT", value),
            None => env::remove_var("SUBSTRATE_ROOT"),
        }
    }

    #[test]
    #[serial]
    fn evaluate_policy_missing_bound_git_has_no_ambient_fallback() {
        let _ = set_global_broker(BrokerHandle::new());
        set_observe_only(true);

        let previous_home = env::var_os("HOME");
        let previous_trace = env::var_os("SHIM_TRACE_LOG");
        let temp = TempDir::new().unwrap();
        let prefix_a = temp.path().join("selected-a");
        let prefix_b = temp.path().join("ambient-b");
        fs::create_dir_all(&prefix_a).unwrap();
        let commit_b =
            initialize_test_git_repository(&prefix_b, &deny_policy_yaml("ambient-b", "echo ok"));
        env::set_var("HOME", &prefix_b);
        env::set_var("SHIM_TRACE_LOG", prefix_b.join("trace.jsonl"));

        let trace_context = new_trace_context(&prefix_a);
        let argv = vec!["echo".to_string(), "ok".to_string()];
        match evaluate_policy(&trace_context, "echo ok", Path::new("."), &argv).unwrap() {
            PolicyResult::Proceed(context) => {
                let span = context.span.expect("span should exist");
                span.finish(0, vec![], None).unwrap();
            }
            PolicyResult::Deny(code) => panic!("unexpected deny result: {code}"),
        }

        let value = latest_command_complete(&prefix_a.join("trace.jsonl"));
        assert!(value["replay_context"]["policy_commit"].is_null());
        assert_ne!(
            value["replay_context"]["policy_commit"].as_str(),
            Some(commit_b.as_str())
        );
        assert!(!prefix_b.join("trace.jsonl").exists());

        match previous_home {
            Some(value) => env::set_var("HOME", value),
            None => env::remove_var("HOME"),
        }
        match previous_trace {
            Some(value) => env::set_var("SHIM_TRACE_LOG", value),
            None => env::remove_var("SHIM_TRACE_LOG"),
        }
    }
}
