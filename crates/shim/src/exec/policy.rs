use anyhow::Result;
use std::path::Path;
use substrate_broker::{quick_check, Decision};
use substrate_trace::{
    create_span_builder, init_trace, ActiveSpan, PolicyDecision as TracePolicyDecision,
};

pub(crate) struct PolicyContext {
    pub(crate) span: Option<ActiveSpan>,
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
    let mut policy_decision = None;

    match quick_check(argv, cwd.to_str().unwrap_or(".")) {
        Ok(Decision::Allow) => {
            policy_decision = Some(TracePolicyDecision {
                action: "allow".to_string(),
                restrictions: None,
                reason: None,
            });
        }
        Ok(Decision::AllowWithRestrictions(restrictions)) => {
            eprintln!(
                "substrate: command requires restrictions: {:?}",
                restrictions
            );
            policy_decision = Some(TracePolicyDecision {
                action: "allow_with_restrictions".to_string(),
                restrictions: Some(restrictions.iter().map(|r| format!("{:?}", r)).collect()),
                reason: None,
            });
        }
        Ok(Decision::Deny(reason)) => {
            eprintln!("substrate: command denied by policy: {}", reason);
            policy_decision = Some(TracePolicyDecision {
                action: "deny".to_string(),
                restrictions: None,
                reason: Some(reason.clone()),
            });
            return Ok(deny_with_span(command_str, cwd, policy_decision));
        }
        Err(e) => {
            eprintln!("substrate: policy check failed: {}", e);
        }
    }

    let mut builder = match create_span_builder() {
        Ok(builder) => builder
            .with_command(command_str)
            .with_cwd(cwd.to_str().unwrap_or(".")),
        Err(err) => {
            eprintln!("substrate: failed to create span builder: {}", err);
            return Ok(PolicyResult::Deny(126));
        }
    };

    if let Some(pd) = policy_decision.clone() {
        builder = builder.with_policy_decision(pd);
    }

    let span = match builder.start() {
        Ok(span) => {
            std::env::set_var("SHIM_PARENT_SPAN", span.get_span_id());
            Some(span)
        }
        Err(e) => {
            eprintln!("substrate: failed to create span: {}", e);
            None
        }
    };

    Ok(PolicyResult::Proceed(Box::new(PolicyContext { span })))
}

fn deny_with_span(
    command_str: &str,
    cwd: &Path,
    policy_decision: Option<TracePolicyDecision>,
) -> PolicyResult {
    let mut builder = match create_span_builder() {
        Ok(builder) => builder
            .with_command(command_str)
            .with_cwd(cwd.to_str().unwrap_or(".")),
        Err(err) => {
            eprintln!("substrate: failed to create span: {}", err);
            return PolicyResult::Deny(126);
        }
    };

    if let Some(pd) = policy_decision.clone() {
        builder = builder.with_policy_decision(pd);
    }

    if let Ok(span) = builder.start() {
        let _ = span.finish(126, vec![], None);
    }

    PolicyResult::Deny(126)
}
