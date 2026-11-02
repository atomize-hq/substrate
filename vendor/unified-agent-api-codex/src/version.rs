use std::collections::{BTreeMap, HashSet};

use semver::{Prerelease, Version};
use serde_json::Value;

use crate::{
    CodexCapabilities, CodexFeature, CodexFeatureFlags, CodexFeatureStage, CodexLatestReleases,
    CodexRelease, CodexReleaseChannel, CodexUpdateAdvisory, CodexUpdateStatus, CodexVersionInfo,
    FeaturesListFormat,
};

fn parse_semver_from_raw(raw: &str) -> Option<Version> {
    for token in raw.split_whitespace() {
        let candidate = token
            .trim_matches(|c: char| matches!(c, '(' | ')' | ',' | ';'))
            .trim_start_matches('v');
        if let Ok(version) = Version::parse(candidate) {
            return Some(version);
        }
    }
    None
}

pub(super) fn parse_version_output(output: &str) -> CodexVersionInfo {
    let raw = output.trim().to_string();
    let parsed_version = parse_semver_from_raw(&raw);
    let semantic = parsed_version
        .as_ref()
        .map(|version| (version.major, version.minor, version.patch));
    let mut commit = extract_commit_hash(&raw);
    if commit.is_none() {
        for token in raw.split_whitespace() {
            let candidate = token
                .trim_matches(|c: char| matches!(c, '(' | ')' | ',' | ';'))
                .trim_start_matches('v');
            if let Some(cleaned) = cleaned_hex(candidate) {
                commit = Some(cleaned);
                break;
            }
        }
    }
    let channel = parsed_version
        .as_ref()
        .map(release_channel_for_version)
        .unwrap_or_else(|| infer_release_channel(&raw));

    CodexVersionInfo {
        raw,
        semantic,
        commit,
        channel,
    }
}

fn release_channel_for_version(version: &Version) -> CodexReleaseChannel {
    if version.pre.is_empty() {
        CodexReleaseChannel::Stable
    } else {
        let prerelease = version.pre.as_str().to_ascii_lowercase();
        if prerelease.contains("beta") {
            CodexReleaseChannel::Beta
        } else if prerelease.contains("nightly") {
            CodexReleaseChannel::Nightly
        } else {
            CodexReleaseChannel::Custom
        }
    }
}

fn infer_release_channel(raw: &str) -> CodexReleaseChannel {
    let lower = raw.to_ascii_lowercase();
    if lower.contains("beta") {
        CodexReleaseChannel::Beta
    } else if lower.contains("nightly") {
        CodexReleaseChannel::Nightly
    } else {
        CodexReleaseChannel::Custom
    }
}

fn codex_semver(info: &CodexVersionInfo) -> Option<Version> {
    if let Some(parsed) = parse_semver_from_raw(&info.raw) {
        return Some(parsed);
    }
    let (major, minor, patch) = info.semantic?;
    let mut version = Version::new(major, minor, patch);
    if version.pre.is_empty() {
        match info.channel {
            CodexReleaseChannel::Beta => {
                version.pre = Prerelease::new("beta").ok()?;
            }
            CodexReleaseChannel::Nightly => {
                version.pre = Prerelease::new("nightly").ok()?;
            }
            CodexReleaseChannel::Stable | CodexReleaseChannel::Custom => {}
        }
    }
    Some(version)
}

fn codex_release_from_info(info: &CodexVersionInfo) -> Option<CodexRelease> {
    let version = codex_semver(info)?;
    Some(CodexRelease {
        channel: info.channel,
        version,
    })
}

fn extract_commit_hash(raw: &str) -> Option<String> {
    let tokens: Vec<&str> = raw.split_whitespace().collect();
    for window in tokens.windows(2) {
        if window[0].eq_ignore_ascii_case("commit") {
            if let Some(cleaned) = cleaned_hex(window[1]) {
                return Some(cleaned);
            }
        }
    }

    for token in tokens {
        if let Some(cleaned) = cleaned_hex(token) {
            return Some(cleaned);
        }
    }
    None
}

fn cleaned_hex(token: &str) -> Option<String> {
    let trimmed = token
        .trim_matches(|c: char| matches!(c, '(' | ')' | ',' | ';'))
        .trim_start_matches("commit")
        .trim_start_matches(':')
        .trim_start_matches('g');
    if trimmed.len() >= 7 && trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
        Some(trimmed.to_string())
    } else {
        None
    }
}

pub(super) fn parse_features_from_json(output: &str) -> Option<CodexFeatureFlags> {
    let parsed: Value = serde_json::from_str(output).ok()?;
    let mut tokens = HashSet::new();
    collect_feature_tokens(&parsed, &mut tokens);
    if tokens.is_empty() {
        return None;
    }

    let mut flags = CodexFeatureFlags::default();
    for token in tokens {
        apply_feature_token(&mut flags, &token);
    }
    Some(flags)
}

fn collect_feature_tokens(value: &Value, tokens: &mut HashSet<String>) {
    match value {
        Value::String(value) if !value.trim().is_empty() => {
            tokens.insert(value.clone());
        }
        Value::Array(items) => {
            for item in items {
                collect_feature_tokens(item, tokens);
            }
        }
        Value::Object(map) => {
            for (key, value) in map {
                if let Value::Bool(true) = value {
                    tokens.insert(key.clone());
                }
                collect_feature_tokens(value, tokens);
            }
        }
        _ => {}
    }
}

pub(super) fn parse_features_from_text(output: &str) -> CodexFeatureFlags {
    let mut flags = CodexFeatureFlags::default();
    let lower = output.to_ascii_lowercase();
    if lower.contains("features list") {
        flags.supports_features_list = true;
    }
    if lower.contains("--output-schema") || lower.contains("output schema") {
        flags.supports_output_schema = true;
    }
    if lower.contains("add-dir") || lower.contains("add dir") {
        flags.supports_add_dir = true;
    }
    if lower.contains("login --mcp") || lower.contains("mcp login") {
        flags.supports_mcp_login = true;
    }
    if lower.contains("login") && lower.contains("mcp") {
        flags.supports_mcp_login = true;
    }

    for token in lower
        .split(|c: char| c.is_ascii_whitespace() || c == ',' || c == ';' || c == '|')
        .filter(|token| !token.is_empty())
    {
        apply_feature_token(&mut flags, token);
    }
    flags
}

pub(super) fn parse_help_output(output: &str) -> CodexFeatureFlags {
    let mut flags = parse_features_from_text(output);
    let lower = output.to_ascii_lowercase();
    if lower.contains("features list") {
        flags.supports_features_list = true;
    }
    flags
}

pub(super) fn merge_feature_flags(target: &mut CodexFeatureFlags, update: CodexFeatureFlags) {
    target.supports_features_list |= update.supports_features_list;
    target.supports_output_schema |= update.supports_output_schema;
    target.supports_add_dir |= update.supports_add_dir;
    target.supports_mcp_login |= update.supports_mcp_login;
}

pub(super) fn detected_feature_flags(flags: &CodexFeatureFlags) -> bool {
    flags.supports_output_schema || flags.supports_add_dir || flags.supports_mcp_login
}

pub(super) fn should_run_help_fallback(flags: &CodexFeatureFlags) -> bool {
    !flags.supports_features_list
        || !flags.supports_output_schema
        || !flags.supports_add_dir
        || !flags.supports_mcp_login
}

fn normalize_feature_token(token: &str) -> String {
    token
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect()
}

fn apply_feature_token(flags: &mut CodexFeatureFlags, token: &str) {
    let normalized = normalize_feature_token(token);
    let compact = normalized.replace('_', "");
    if normalized.contains("features_list") || compact.contains("featureslist") {
        flags.supports_features_list = true;
    }
    if normalized.contains("output_schema") || compact.contains("outputschema") {
        flags.supports_output_schema = true;
    }
    if normalized.contains("add_dir") || compact.contains("adddir") {
        flags.supports_add_dir = true;
    }
    if normalized.contains("mcp_login")
        || (normalized.contains("login") && normalized.contains("mcp"))
    {
        flags.supports_mcp_login = true;
    }
}

pub(super) fn parse_feature_list_output(
    stdout: &str,
    prefer_json: bool,
) -> Result<(Vec<CodexFeature>, FeaturesListFormat), String> {
    let trimmed = stdout.trim();
    if trimmed.is_empty() {
        return Err("features list output was empty".to_string());
    }

    if prefer_json {
        if let Some(features) = parse_feature_list_json(trimmed) {
            if !features.is_empty() {
                return Ok((features, FeaturesListFormat::Json));
            }
        }
        if let Some(features) = parse_feature_list_text(trimmed) {
            if !features.is_empty() {
                return Ok((features, FeaturesListFormat::Text));
            }
        }
    } else {
        if let Some(features) = parse_feature_list_text(trimmed) {
            if !features.is_empty() {
                return Ok((features, FeaturesListFormat::Text));
            }
        }
        if let Some(features) = parse_feature_list_json(trimmed) {
            if !features.is_empty() {
                return Ok((features, FeaturesListFormat::Json));
            }
        }
    }

    Err("could not parse JSON or text feature rows".to_string())
}

fn parse_feature_list_json(output: &str) -> Option<Vec<CodexFeature>> {
    let parsed: Value = serde_json::from_str(output).ok()?;
    parse_feature_list_json_value(&parsed)
}

fn parse_feature_list_json_value(value: &Value) -> Option<Vec<CodexFeature>> {
    match value {
        Value::Array(items) => Some(
            items
                .iter()
                .filter_map(|item| match item {
                    Value::Object(map) => feature_from_json_fields(None, map),
                    Value::String(name) => Some(CodexFeature {
                        name: name.clone(),
                        stage: None,
                        enabled: true,
                        extra: BTreeMap::new(),
                    }),
                    _ => None,
                })
                .collect(),
        ),
        Value::Object(map) => {
            if let Some(features) = map.get("features") {
                return parse_feature_list_json_value(features);
            }
            if map.contains_key("name") || map.contains_key("enabled") || map.contains_key("stage")
            {
                return feature_from_json_fields(None, map).map(|feature| vec![feature]);
            }
            Some(
                map.iter()
                    .filter_map(|(name, value)| match value {
                        Value::Object(inner) => {
                            feature_from_json_fields(Some(name.as_str()), inner)
                        }
                        Value::Bool(flag) => Some(CodexFeature {
                            name: name.clone(),
                            stage: None,
                            enabled: *flag,
                            extra: BTreeMap::new(),
                        }),
                        Value::String(flag) => parse_feature_enabled_str(flag)
                            .map(|enabled| CodexFeature {
                                name: name.clone(),
                                stage: None,
                                enabled,
                                extra: BTreeMap::new(),
                            })
                            .or_else(|| {
                                Some(CodexFeature {
                                    name: name.clone(),
                                    stage: Some(CodexFeatureStage::parse(flag)),
                                    enabled: true,
                                    extra: BTreeMap::new(),
                                })
                            }),
                        _ => None,
                    })
                    .collect(),
            )
        }
        _ => None,
    }
}

fn parse_feature_list_text(output: &str) -> Option<Vec<CodexFeature>> {
    let mut features = Vec::new();
    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed
            .chars()
            .all(|c| matches!(c, '-' | '=' | '+' | '*' | '|'))
        {
            continue;
        }

        let tokens: Vec<&str> = trimmed.split_whitespace().collect();
        if tokens.len() < 3 {
            continue;
        }
        if tokens[0].eq_ignore_ascii_case("feature")
            && tokens[1].eq_ignore_ascii_case("stage")
            && tokens[2].eq_ignore_ascii_case("enabled")
        {
            continue;
        }

        let enabled_token = tokens.last().copied().unwrap_or_default();
        let enabled = match parse_feature_enabled_str(enabled_token) {
            Some(value) => value,
            None => continue,
        };
        let stage_token = tokens.get(tokens.len() - 2).copied().unwrap_or_default();
        let name = tokens[..tokens.len() - 2].join(" ");
        if name.is_empty() {
            continue;
        }
        let stage = (!stage_token.is_empty()).then(|| CodexFeatureStage::parse(stage_token));
        features.push(CodexFeature {
            name,
            stage,
            enabled,
            extra: BTreeMap::new(),
        });
    }

    if features.is_empty() {
        None
    } else {
        Some(features)
    }
}

fn parse_feature_enabled_value(value: &Value) -> Option<bool> {
    match value {
        Value::Bool(flag) => Some(*flag),
        Value::String(raw) => parse_feature_enabled_str(raw),
        _ => None,
    }
}

fn parse_feature_enabled_str(raw: &str) -> Option<bool> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "true" | "yes" | "y" | "on" | "1" | "enabled" => Some(true),
        "false" | "no" | "n" | "off" | "0" | "disabled" => Some(false),
        _ => None,
    }
}

fn feature_from_json_fields(
    name_hint: Option<&str>,
    map: &serde_json::Map<String, Value>,
) -> Option<CodexFeature> {
    let name = map
        .get("name")
        .and_then(Value::as_str)
        .map(str::to_string)
        .or_else(|| name_hint.map(str::to_string))?;
    let enabled = map
        .get("enabled")
        .and_then(parse_feature_enabled_value)
        .or_else(|| map.get("value").and_then(parse_feature_enabled_value))?;
    let stage = map
        .get("stage")
        .or_else(|| map.get("status"))
        .and_then(Value::as_str)
        .map(CodexFeatureStage::parse);

    let mut extra = BTreeMap::new();
    for (key, value) in map {
        if matches!(
            key.as_str(),
            "name" | "stage" | "status" | "enabled" | "value"
        ) {
            continue;
        }
        extra.insert(key.clone(), value.clone());
    }

    Some(CodexFeature {
        name,
        stage,
        enabled,
        extra,
    })
}

/// Computes an update advisory for a previously probed binary.
///
/// Callers that already have a [`CodexCapabilities`] snapshot can use this
/// helper to avoid re-running `codex --version`. Provide a [`CodexLatestReleases`]
/// table sourced from your preferred distribution channel.
pub fn update_advisory_from_capabilities(
    capabilities: &CodexCapabilities,
    latest_releases: &CodexLatestReleases,
) -> CodexUpdateAdvisory {
    let local_release = capabilities
        .version
        .as_ref()
        .and_then(codex_release_from_info);
    let preferred_channel = local_release
        .as_ref()
        .map(|release| release.channel)
        .unwrap_or(CodexReleaseChannel::Stable);
    let (latest_release, comparison_channel, fell_back) =
        latest_releases.select_for_channel(preferred_channel);
    let mut notes = Vec::new();

    if fell_back {
        notes.push(format!(
            "No latest {preferred_channel} release provided; comparing against {comparison_channel}."
        ));
    }

    let status = match (local_release.as_ref(), latest_release.as_ref()) {
        (None, None) => CodexUpdateStatus::UnknownLatestVersion,
        (None, Some(_)) => CodexUpdateStatus::UnknownLocalVersion,
        (Some(_), None) => CodexUpdateStatus::UnknownLatestVersion,
        (Some(local), Some(latest)) => {
            if local.version < latest.version {
                CodexUpdateStatus::UpdateRecommended
            } else if local.version > latest.version {
                CodexUpdateStatus::LocalNewerThanKnown
            } else {
                CodexUpdateStatus::UpToDate
            }
        }
    };

    match status {
        CodexUpdateStatus::UpdateRecommended => {
            if let (Some(local), Some(latest)) = (local_release.as_ref(), latest_release.as_ref()) {
                notes.push(format!(
                    "Local codex {local_version} is behind latest {comparison_channel} {latest_version}.",
                    local_version = local.version,
                    latest_version = latest.version
                ));
            }
        }
        CodexUpdateStatus::LocalNewerThanKnown => {
            if let Some(local) = local_release.as_ref() {
                let known = latest_release
                    .as_ref()
                    .map(|release| release.version.to_string())
                    .unwrap_or_else(|| "unknown".to_string());
                notes.push(format!(
                    "Local codex {local_version} is newer than provided {comparison_channel} metadata (latest table: {known}).",
                    local_version = local.version
                ));
            }
        }
        CodexUpdateStatus::UnknownLocalVersion => {
            if let Some(latest) = latest_release.as_ref() {
                notes.push(format!(
                    "Latest known {comparison_channel} release is {latest_version}; local version could not be parsed.",
                    latest_version = latest.version
                ));
            } else {
                notes.push(
                    "Local version could not be parsed and no latest release was provided."
                        .to_string(),
                );
            }
        }
        CodexUpdateStatus::UnknownLatestVersion => notes.push(
            "No latest Codex release information provided; update advisory unavailable."
                .to_string(),
        ),
        CodexUpdateStatus::UpToDate => {
            if let Some(latest) = latest_release.as_ref() {
                notes.push(format!(
                    "Local codex matches latest {comparison_channel} release {latest_version}.",
                    latest_version = latest.version
                ));
            }
        }
    }

    CodexUpdateAdvisory {
        local_release,
        latest_release,
        comparison_channel,
        status,
        notes,
    }
}
