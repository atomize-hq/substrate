use mockito::{Matcher, Server};
use serde::Deserialize;
use serde_json::json;
use substrate_gateway::cli::{AppConfig, ModelConfig};
use substrate_gateway::core::GatewayRequest;
use substrate_gateway::models::Message;
use substrate_gateway::models::MessageContent;
use substrate_gateway::providers::{AuthType, ProviderConfig, ProviderRegistry};
use tempfile::NamedTempFile;

fn sample_request(model: &str) -> GatewayRequest {
    GatewayRequest {
        model: model.to_string(),
        messages: vec![Message {
            role: "user".to_string(),
            content: MessageContent::Text("ping".to_string()),
        }],
        max_output_tokens: 64,
        reasoning: None,
        temperature: None,
        top_p: None,
        top_k: None,
        stop_sequences: None,
        stream: None,
        metadata: None,
        system: None,
        tools: None,
    }
}

fn sample_response(model: &str) -> String {
    format!(
        r#"{{
            "id":"chatcmpl-test",
            "object":"chat.completion",
            "model":"{model}",
            "choices":[{{
                "message":{{"role":"assistant","content":"ok"}},
                "finish_reason":"stop"
            }}],
            "usage":{{"prompt_tokens":1,"completion_tokens":1,"total_tokens":2}}
        }}"#
    )
}

#[tokio::test]
async fn azure_openai_transport_uses_api_key_header_and_v1_path() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("POST", "/openai/v1/chat/completions")
        .match_header("api-key", "azure-secret")
        .match_body(Matcher::Regex(
            "\"model\":\"kimi-k2-thinking-deployment\"".to_string(),
        ))
        .with_status(200)
        .with_body(sample_response("kimi-k2-thinking-deployment"))
        .create_async()
        .await;

    let registry = ProviderRegistry::from_configs_with_models(
        &[ProviderConfig {
            name: "azure-kimi".to_string(),
            provider_type: "azure-openai".to_string(),
            auth_type: AuthType::ApiKey,
            api_key: Some("azure-secret".to_string()),
            oauth_provider: None,
            project_id: None,
            location: None,
            base_url: Some(format!("{}/openai/v1", server.url())),
            headers: None,
            models: vec![],
            enabled: Some(true),
        }],
        None,
        &[ModelConfig {
            name: "Kimi-K2-Thinking".to_string(),
            mappings: vec![],
        }],
    )
    .unwrap();

    let provider = registry.get_provider("azure-kimi").unwrap();
    let response = provider
        .send_message(sample_request("kimi-k2-thinking-deployment"))
        .await
        .unwrap();

    assert_eq!(response.model, "kimi-k2-thinking-deployment");
    mock.assert_async().await;
}

#[tokio::test]
async fn azure_gpt_5_chat_requests_use_max_completion_tokens_instead_of_max_tokens() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("POST", "/openai/v1/chat/completions")
        .match_header("api-key", "azure-secret")
        .match_body(Matcher::Json(json!({
            "model": "gpt-5.4-mini",
            "messages": [
                {
                    "role": "user",
                    "content": "ping"
                }
            ],
            "max_completion_tokens": 64
        })))
        .with_status(200)
        .with_body(sample_response("gpt-5.4-mini"))
        .create_async()
        .await;

    let registry = ProviderRegistry::from_configs_with_models(
        &[ProviderConfig {
            name: "azure-kimi".to_string(),
            provider_type: "azure-openai".to_string(),
            auth_type: AuthType::ApiKey,
            api_key: Some("azure-secret".to_string()),
            oauth_provider: None,
            project_id: None,
            location: None,
            base_url: Some(format!("{}/openai/v1", server.url())),
            headers: None,
            models: vec![],
            enabled: Some(true),
        }],
        None,
        &[ModelConfig {
            name: "gpt-5.4-mini".to_string(),
            mappings: vec![],
        }],
    )
    .unwrap();

    let provider = registry.get_provider("azure-kimi").unwrap();
    let response = provider
        .send_message(sample_request("gpt-5.4-mini"))
        .await
        .unwrap();

    assert_eq!(response.model, "gpt-5.4-mini");
    mock.assert_async().await;
}

#[tokio::test]
async fn generic_openai_transport_keeps_bearer_header_and_generic_v1_path() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/chat/completions")
        .match_header("authorization", "Bearer openai-secret")
        .match_body(Matcher::Regex("\"model\":\"gpt-4.1\"".to_string()))
        .with_status(200)
        .with_body(sample_response("gpt-4.1"))
        .create_async()
        .await;

    let registry = ProviderRegistry::from_configs_with_models(
        &[ProviderConfig {
            name: "openai".to_string(),
            provider_type: "openai".to_string(),
            auth_type: AuthType::ApiKey,
            api_key: Some("openai-secret".to_string()),
            oauth_provider: None,
            project_id: None,
            location: None,
            base_url: Some(format!("{}/v1", server.url())),
            headers: None,
            models: vec![],
            enabled: Some(true),
        }],
        None,
        &[ModelConfig {
            name: "gpt-4.1".to_string(),
            mappings: vec![],
        }],
    )
    .unwrap();

    let provider = registry.get_provider("openai").unwrap();
    let response = provider
        .send_message(sample_request("gpt-4.1"))
        .await
        .unwrap();

    assert_eq!(response.model, "gpt-4.1");
    mock.assert_async().await;
}

#[test]
fn default_example_config_uses_capability_oriented_azure_routing_labels() {
    let path = format!("{}/config/default.example.toml", env!("CARGO_MANIFEST_DIR"));
    let source = std::fs::read_to_string(path).unwrap();
    let config: AppConfig = toml::from_str(&source).unwrap();

    let azure_provider = config
        .providers
        .iter()
        .find(|provider| provider.provider_type == "azure-openai")
        .unwrap();

    assert_eq!(azure_provider.name, "azure-kimi");
    assert_eq!(azure_provider.auth_type, AuthType::ApiKey);
    assert_eq!(
        azure_provider.base_url.as_deref(),
        Some("$AZURE_KIMI_ENDPOINT")
    );
    assert_eq!(config.router.think.as_deref(), Some("substrate-think"));
    assert_eq!(config.router.default, "substrate-default");
}

#[test]
fn app_config_resolves_azure_endpoint_and_api_key_from_env() {
    let mut temp_file = NamedTempFile::new().unwrap();
    std::io::Write::write_all(
        &mut temp_file,
        br#"
[server]
host = "127.0.0.1"
port = 13456

[router]
default = "substrate-default"

[[providers]]
name = "azure-kimi"
provider_type = "azure-openai"
auth_type = "apikey"
api_key = "$AZURE_KIMI_API_KEY"
base_url = "$AZURE_KIMI_ENDPOINT"
enabled = true
models = []
"#,
    )
    .unwrap();

    std::env::set_var("AZURE_KIMI_API_KEY", "azure-secret");
    std::env::set_var(
        "AZURE_KIMI_ENDPOINT",
        "https://example.azure.com/openai/v1/chat/completions",
    );

    let config = AppConfig::from_file(&temp_file.path().to_path_buf()).unwrap();
    let provider = &config.providers[0];

    assert_eq!(provider.api_key.as_deref(), Some("azure-secret"));
    assert_eq!(
        provider.base_url.as_deref(),
        Some("https://example.azure.com/openai/v1/chat/completions")
    );

    std::env::remove_var("AZURE_KIMI_API_KEY");
    std::env::remove_var("AZURE_KIMI_ENDPOINT");
}

#[derive(Debug, Deserialize)]
struct ModelsOnly {
    models: Vec<ModelConfig>,
}

#[test]
fn models_example_keeps_capability_labels_and_azure_deployment_targets() {
    let path = format!("{}/config/models.example.toml", env!("CARGO_MANIFEST_DIR"));
    let source = std::fs::read_to_string(path).unwrap();
    let models: ModelsOnly = toml::from_str(&source).unwrap();

    let thinking = models
        .models
        .iter()
        .find(|model| model.name == "substrate-think")
        .unwrap();
    let execution = models
        .models
        .iter()
        .find(|model| model.name == "substrate-default")
        .unwrap();

    assert_eq!(thinking.mappings[0].provider, "azure-kimi");
    assert_eq!(
        thinking.mappings[0].actual_model,
        "kimi-k2-thinking-deployment"
    );
    assert_eq!(execution.mappings[0].provider, "azure-kimi");
    assert_eq!(execution.mappings[0].actual_model, "kimi-k2-5-deployment");
}
