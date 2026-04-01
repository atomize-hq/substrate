use anyhow::Result;
use std::{env, ffi::OsString, path::Path};
use substrate_broker::{policy_mode, quick_check, Decision, PolicyMode};
use substrate_trace::{
    create_span_builder, init_trace, ActiveSpan, PolicyDecision as TracePolicyDecision,
};

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
    command_str: &str,
    cwd: &Path,
    argv: &[String],
) -> Result<PolicyResult> {
    let _ = init_trace(None);
    let mode = policy_mode();
    let mut policy_decision = None;

    if mode == PolicyMode::Disabled {
        return Ok(PolicyResult::Proceed(Box::new(PolicyContext {
            span: start_span(command_str, cwd, None)?,
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
                return Ok(deny_with_span(command_str, cwd, policy_decision));
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
        span: start_span(command_str, cwd, policy_decision)?,
    })))
}

fn deny_with_span(
    command_str: &str,
    cwd: &Path,
    policy_decision: Option<TracePolicyDecision>,
) -> PolicyResult {
    if let Ok(Some(mut span)) = start_span(command_str, cwd, policy_decision) {
        span.set_outcome("denied");
        let _ = span.finish(126, vec![], None);
    }

    PolicyResult::Deny(126)
}

fn start_span(
    command_str: &str,
    cwd: &Path,
    policy_decision: Option<TracePolicyDecision>,
) -> Result<Option<PolicySpan>> {
    let mut builder = match create_span_builder() {
        Ok(builder) => builder
            .with_command(command_str)
            .with_cwd(cwd.to_str().unwrap_or(".")),
        Err(err) => {
            eprintln!("substrate: failed to create span builder: {}", err);
            return Ok(None);
        }
    };

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
    use serial_test::serial;
    use std::{env, path::Path};
    use substrate_broker::{set_global_broker, set_observe_only, BrokerHandle};

    #[test]
    #[serial]
    fn evaluate_policy_allows_when_command_not_denied() {
        let _ = set_global_broker(BrokerHandle::new());
        set_observe_only(true);
        let _ = substrate_trace::set_global_trace_context(substrate_trace::TraceContext::default());

        let argv = vec!["echo".to_string(), "ok".to_string()];
        let result = evaluate_policy("echo ok", Path::new("."), &argv)
            .expect("policy evaluation should succeed");
        assert!(matches!(result, PolicyResult::Proceed(_)));
    }

    #[test]
    #[serial]
    fn evaluate_policy_denies_blocked_command_when_enforced() {
        let _ = set_global_broker(BrokerHandle::new());
        set_observe_only(false);

        let argv = vec!["rm".to_string(), "-rf".to_string(), "/tmp".to_string()];
        match evaluate_policy("rm -rf /tmp", Path::new("/tmp"), &argv)
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
        let _ = substrate_trace::set_global_trace_context(substrate_trace::TraceContext::default());

        let span = start_span("echo ok", Path::new("."), None)
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
        let _ = substrate_trace::set_global_trace_context(substrate_trace::TraceContext::default());

        let span = start_span("echo ok", Path::new("."), None)
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
}
