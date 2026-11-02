use super::*;

#[test]
fn parses_features_from_json_and_text() {
    let json = r#"{"features":["output_schema","add_dir"],"mcp_login":true}"#;
    let parsed_json = version::parse_features_from_json(json).unwrap();
    assert!(parsed_json.supports_output_schema);
    assert!(parsed_json.supports_add_dir);
    assert!(parsed_json.supports_mcp_login);

    let text = "Features: output-schema add-dir login --mcp";
    let parsed_text = version::parse_features_from_text(text);
    assert!(parsed_text.supports_output_schema);
    assert!(parsed_text.supports_add_dir);
    assert!(parsed_text.supports_mcp_login);
}

#[test]
fn parses_feature_list_json_and_text_tables() {
    let json = r#"{"features":[{"name":"json-stream","stage":"stable","enabled":true,"notes":"keep"},{"name":"cloud-exec","stage":"experimental","enabled":false}]}"#;
    let (json_features, json_format) = version::parse_feature_list_output(json, true).unwrap();
    assert_eq!(json_format, FeaturesListFormat::Json);
    assert_eq!(json_features.len(), 2);
    assert_eq!(json_features[0].name, "json-stream");
    assert_eq!(json_features[0].stage, Some(CodexFeatureStage::Stable));
    assert!(json_features[0].enabled);
    assert!(json_features[0].extra.contains_key("notes"));
    assert_eq!(
        json_features[1].stage,
        Some(CodexFeatureStage::Experimental)
    );
    assert!(!json_features[1].enabled);

    let text = r#"
Feature   Stage         Enabled
json-stream stable      true
	cloud-exec experimental false
	"#;
    let (text_features, text_format) = version::parse_feature_list_output(text, false).unwrap();
    assert_eq!(text_format, FeaturesListFormat::Text);
    assert_eq!(text_features.len(), 2);
    assert_eq!(
        text_features[1].stage,
        Some(CodexFeatureStage::Experimental)
    );
    assert!(!text_features[1].enabled);

    let (fallback_features, fallback_format) =
        version::parse_feature_list_output(text, true).unwrap();
    assert_eq!(fallback_format, FeaturesListFormat::Text);
    assert_eq!(fallback_features.len(), 2);
}

#[test]
fn parses_help_output_flags() {
    let help =
        "Usage: codex --output-schema ... add-dir ... login --mcp. See `codex features list`.";
    let parsed = version::parse_help_output(help);
    assert!(parsed.supports_output_schema);
    assert!(parsed.supports_add_dir);
    assert!(parsed.supports_mcp_login);
    assert!(parsed.supports_features_list);
}

#[test]
fn capability_guard_reports_detected_support() {
    let flags = CodexFeatureFlags {
        supports_features_list: true,
        supports_output_schema: true,
        supports_add_dir: true,
        supports_mcp_login: true,
    };
    let capabilities = capabilities_with_feature_flags(flags);

    let output_schema = capabilities.guard_output_schema();
    assert_eq!(output_schema.support, CapabilitySupport::Supported);
    assert!(output_schema.is_supported());

    let add_dir = capabilities.guard_add_dir();
    assert_eq!(add_dir.support, CapabilitySupport::Supported);
    assert!(add_dir.is_supported());

    let mcp_login = capabilities.guard_mcp_login();
    assert_eq!(mcp_login.support, CapabilitySupport::Supported);

    let features_list = capabilities.guard_features_list();
    assert_eq!(features_list.support, CapabilitySupport::Supported);
}

#[test]
fn capability_guard_marks_absent_feature_as_unsupported() {
    let flags = CodexFeatureFlags {
        supports_features_list: true,
        supports_output_schema: false,
        supports_add_dir: false,
        supports_mcp_login: false,
    };
    let capabilities = capabilities_with_feature_flags(flags);

    let output_schema = capabilities.guard_output_schema();
    assert_eq!(output_schema.support, CapabilitySupport::Unsupported);
    assert!(!output_schema.is_supported());
    assert!(output_schema
        .notes
        .iter()
        .any(|note| note.contains("features list")));

    let mcp_login = capabilities.guard_mcp_login();
    assert_eq!(mcp_login.support, CapabilitySupport::Unsupported);
}

#[test]
fn capability_guard_returns_unknown_without_feature_list() {
    let capabilities = capabilities_with_feature_flags(CodexFeatureFlags::default());

    let add_dir = capabilities.guard_add_dir();
    assert_eq!(add_dir.support, CapabilitySupport::Unknown);
    assert!(add_dir.is_unknown());
    assert!(add_dir
        .notes
        .iter()
        .any(|note| note.contains("unknown") || note.contains("unavailable")));

    let features_list = capabilities.guard_features_list();
    assert_eq!(features_list.support, CapabilitySupport::Unknown);
}
