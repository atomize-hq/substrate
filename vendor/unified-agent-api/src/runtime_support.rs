use crate::AgentWrapperError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeSupportRecord {
    pub runtime_family: String,
    pub target_triple: String,
    pub version: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct EmbeddedRuntimeSupportRecord {
    target_triple: &'static str,
    latest_validated: Option<&'static str>,
}

include!("runtime_support_data.rs");

fn runtime_family_records(runtime_family: &str) -> Option<&'static [EmbeddedRuntimeSupportRecord]> {
    match runtime_family {
        "codex" => Some(CODEX_RUNTIME_SUPPORT),
        _ => None,
    }
}

pub fn resolve_runtime_support(
    runtime_family: &str,
    target_triple: &str,
) -> Result<RuntimeSupportRecord, AgentWrapperError> {
    let records = runtime_family_records(runtime_family).ok_or_else(|| {
        AgentWrapperError::UnknownRuntimeFamily {
            runtime_family: runtime_family.to_string(),
        }
    })?;
    let record = records
        .iter()
        .find(|record| record.target_triple == target_triple);
    let Some(record) = record else {
        return Err(AgentWrapperError::UnsupportedTargetTriple {
            runtime_family: runtime_family.to_string(),
            target_triple: target_triple.to_string(),
        });
    };

    let Some(version) = record.latest_validated else {
        return Err(AgentWrapperError::MissingValidatedRuntime {
            runtime_family: runtime_family.to_string(),
            target_triple: target_triple.to_string(),
        });
    };

    Ok(RuntimeSupportRecord {
        runtime_family: runtime_family.to_string(),
        target_triple: target_triple.to_string(),
        version: version.to_string(),
    })
}

pub fn list_runtime_support(
    runtime_family: &str,
) -> Result<Vec<RuntimeSupportRecord>, AgentWrapperError> {
    let records = runtime_family_records(runtime_family).ok_or_else(|| {
        AgentWrapperError::UnknownRuntimeFamily {
            runtime_family: runtime_family.to_string(),
        }
    })?;

    let mut resolved = records
        .iter()
        .filter_map(|record| {
            record.latest_validated.map(|version| RuntimeSupportRecord {
                runtime_family: runtime_family.to_string(),
                target_triple: record.target_triple.to_string(),
                version: version.to_string(),
            })
        })
        .collect::<Vec<_>>();
    resolved.sort_by(|left, right| left.target_triple.cmp(&right.target_triple));
    Ok(resolved)
}
