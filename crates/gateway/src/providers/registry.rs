use super::gemini::{GeminiAuthConfig, GeminiProvider};
use super::{
    error::ProviderError, AnthropicCompatibleProvider, GatewayProvider, OpenAIProvider,
    OpenAIProviderConfig, OpenAITransport, ProviderConfig,
};
use crate::auth::{CodexAuthSource, CodexAuthState, TokenStore};
use crate::cli::ModelConfig;
use crate::launch::GatewayMode;
use std::collections::HashMap;
use std::sync::Arc;

/// Default base URL for OpenAI-compatible API
const DEFAULT_OPENAI_BASE_URL: &str = "https://api.openai.com/v1";

/// GitHub repository URL (used in HTTP-Referer headers)
const REPO_URL: &str = "https://github.com/elidickinson/claude-code-mux";

fn codex_auth_source_for_openai_oauth(
    config: &ProviderConfig,
    gateway_mode: GatewayMode,
) -> Result<Option<CodexAuthSource>, ProviderError> {
    if config.provider_type != "openai" || config.auth_type != super::AuthType::OAuth {
        return Ok(None);
    }

    let source = match gateway_mode {
        GatewayMode::InWorld => CodexAuthSource::Integrated,
        GatewayMode::HostOnly => CodexAuthSource::StandaloneLocal {
            path: CodexAuthState::default_path().map_err(|e| {
                ProviderError::ConfigError(format!(
                    "Failed to resolve standalone Codex auth path for provider '{}': {}",
                    config.name, e
                ))
            })?,
        },
    };

    Ok(Some(source))
}

/// Provider registry that manages all configured providers
pub struct ProviderRegistry {
    /// Map of provider name -> provider instance
    providers: HashMap<String, Arc<Box<dyn GatewayProvider>>>,
    /// Map of model name -> provider name for fast lookup
    model_to_provider: HashMap<String, String>,
}

impl ProviderRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
            model_to_provider: HashMap::new(),
        }
    }

    /// Load providers from configuration
    #[allow(dead_code)]
    pub fn from_configs(
        configs: &[ProviderConfig],
        token_store: Option<TokenStore>,
    ) -> Result<Self, ProviderError> {
        Self::from_configs_with_models(configs, token_store, &[])
    }

    /// Load providers from configuration with model mappings
    pub fn from_configs_with_models(
        configs: &[ProviderConfig],
        token_store: Option<TokenStore>,
        models: &[ModelConfig],
    ) -> Result<Self, ProviderError> {
        let gateway_mode = GatewayMode::from_env_or_default().map_err(|e| {
            ProviderError::ConfigError(format!("Failed to resolve gateway launch mode: {}", e))
        })?;
        Self::from_configs_with_models_and_mode(configs, token_store, models, gateway_mode)
    }

    pub(crate) fn from_configs_with_models_and_mode(
        configs: &[ProviderConfig],
        token_store: Option<TokenStore>,
        models: &[ModelConfig],
        gateway_mode: GatewayMode,
    ) -> Result<Self, ProviderError> {
        let mut registry = Self::new();

        for config in configs {
            // Skip disabled providers
            if !config.is_enabled() {
                continue;
            }

            // Get API key - required for API key auth, skipped for OAuth
            let api_key = match &config.auth_type {
                super::AuthType::ApiKey => config.api_key.clone().ok_or_else(|| {
                    ProviderError::ConfigError(format!(
                        "Provider '{}' requires api_key for ApiKey auth",
                        config.name
                    ))
                })?,
                super::AuthType::OAuth => {
                    // OAuth providers will handle authentication differently
                    // For now, use a placeholder - will be replaced with token
                    config
                        .oauth_provider
                        .clone()
                        .unwrap_or_else(|| config.name.clone())
                }
            };

            let codex_auth_source = codex_auth_source_for_openai_oauth(config, gateway_mode)?;

            // Create provider instance based on type
            let provider: Box<dyn GatewayProvider> = match config.provider_type.as_str() {
                // OpenAI-compatible providers (unified with custom headers support)
                "openai" => {
                    let base_url = config
                        .base_url
                        .clone()
                        .unwrap_or_else(|| DEFAULT_OPENAI_BASE_URL.to_string());
                    let custom_headers: Vec<(String, String)> = config
                        .headers
                        .clone()
                        .unwrap_or_default()
                        .into_iter()
                        .collect();

                    Box::new(OpenAIProvider::with_headers(OpenAIProviderConfig {
                        name: config.name.clone(),
                        api_key,
                        base_url,
                        models: config.models.clone(),
                        custom_headers,
                        oauth_provider: config.oauth_provider.clone(),
                        token_store: token_store.clone(),
                        codex_auth_source,
                    }))
                }

                // Azure Foundry / Azure OpenAI v1 transport.
                // Uses the same request shape as OpenAI-compatible chat/responses,
                // but switches API-key auth to `api-key` and keeps the base URL
                // rooted at `/openai/v1`.
                "azure-openai" => {
                    let base_url = config.base_url.clone().ok_or_else(|| {
                        ProviderError::ConfigError(format!(
                            "Provider '{}' requires base_url for Azure OpenAI v1 transport",
                            config.name
                        ))
                    })?;
                    let custom_headers: Vec<(String, String)> = config
                        .headers
                        .clone()
                        .unwrap_or_default()
                        .into_iter()
                        .collect();

                    Box::new(OpenAIProvider::with_transport(
                        OpenAITransport::AzureOpenAI,
                        OpenAIProviderConfig {
                            name: config.name.clone(),
                            api_key,
                            base_url,
                            models: config.models.clone(),
                            custom_headers,
                            oauth_provider: config.oauth_provider.clone(),
                            token_store: token_store.clone(),
                            codex_auth_source: None,
                        },
                    ))
                }

                // OpenRouter (OpenAI-compatible)
                // Note: OpenRouter's Anthropic-compatible endpoint only supports Claude models,
                // so we use the OpenAI endpoint to support all models (Kimi, DeepSeek, etc.)
                "openrouter" => Box::new(OpenAIProvider::with_headers(OpenAIProviderConfig {
                    name: config.name.clone(),
                    api_key,
                    base_url: config
                        .base_url
                        .clone()
                        .unwrap_or_else(|| "https://openrouter.ai/api/v1".to_string()),
                    models: config.models.clone(),
                    custom_headers: vec![
                        ("HTTP-Referer".to_string(), REPO_URL.to_string()),
                        ("X-Title".to_string(), "Substrate Gateway".to_string()),
                    ],
                    oauth_provider: config.oauth_provider.clone(),
                    token_store: token_store.clone(),
                    codex_auth_source: None,
                })),

                // Deprecated aliases for OpenAI-compatible providers
                // These will be removed in a future version
                // NOTE: Keep these preset URLs/headers aligned with the archived admin UI
                // reference in crates/gateway/archived/admin_ui/admin.html.
                provider @ ("deepinfra" | "novita" | "baseten" | "together" | "fireworks"
                | "groq" | "nebius" | "cerebras" | "moonshot") => {
                    tracing::warn!(
                        "Provider type '{}' is deprecated. Migrate to: provider_type = \"openai\", base_url = \"<url>\"[, headers = {{ \"X-Header\" = \"value\" }}]",
                        provider
                    );

                    let (base_url, headers) = match provider {
                        "deepinfra" => (
                            Some("https://api.deepinfra.com/v1/openai".to_string()),
                            None,
                        ),
                        "novita" => (
                            Some("https://api.novita.ai/v3/openai".to_string()),
                            Some(
                                vec![(
                                    "X-Novita-Source".to_string(),
                                    "substrate-gateway".to_string(),
                                )]
                                .into_iter()
                                .collect(),
                            ),
                        ),
                        "baseten" => (Some("https://inference.baseten.co/v1".to_string()), None),
                        "together" => (Some("https://api.together.xyz/v1".to_string()), None),
                        "fireworks" => (
                            Some("https://api.fireworks.ai/inference/v1".to_string()),
                            None,
                        ),
                        "groq" => (Some("https://api.groq.com/openai/v1".to_string()), None),
                        "nebius" => (Some("https://api.studio.nebius.ai/v1".to_string()), None),
                        "cerebras" => (Some("https://api.cerebras.ai/v1".to_string()), None),
                        "moonshot" => (Some("https://api.moonshot.cn/v1".to_string()), None),
                        _ => unreachable!(),
                    };

                    // Use config headers if provided, otherwise use preset headers
                    let headers = config.headers.as_ref().or(headers.as_ref());
                    let headers_vec: Vec<(String, String)> =
                        headers.cloned().unwrap_or_default().into_iter().collect();

                    Box::new(OpenAIProvider::with_headers(OpenAIProviderConfig {
                        name: config.name.clone(),
                        api_key,
                        base_url: base_url.unwrap_or_else(|| DEFAULT_OPENAI_BASE_URL.to_string()),
                        models: config.models.clone(),
                        custom_headers: headers_vec,
                        oauth_provider: config.oauth_provider.clone(),
                        token_store: token_store.clone(),
                        codex_auth_source: None,
                    }))
                }

                // Anthropic-compatible providers
                "anthropic" => Box::new(AnthropicCompatibleProvider::new(
                    config.name.clone(),
                    api_key,
                    config
                        .base_url
                        .clone()
                        .unwrap_or_else(|| "https://api.anthropic.com".to_string()),
                    config.models.clone(),
                    config.oauth_provider.clone(),
                    token_store.clone(),
                )),
                "z.ai" => Box::new(AnthropicCompatibleProvider::zai(
                    api_key,
                    config.models.clone(),
                    token_store.clone(),
                )),
                "minimax" => Box::new(AnthropicCompatibleProvider::minimax(
                    api_key,
                    config.models.clone(),
                    token_store.clone(),
                )),
                "zenmux" => Box::new(AnthropicCompatibleProvider::zenmux(
                    api_key,
                    config.models.clone(),
                    token_store.clone(),
                )),
                "kimi-coding" => Box::new(AnthropicCompatibleProvider::kimi_coding(
                    api_key,
                    config.models.clone(),
                    token_store.clone(),
                )),

                // Google Gemini (supports OAuth, API Key, Vertex AI)
                "gemini" => {
                    let api_key_opt = if config.auth_type == super::AuthType::ApiKey {
                        Some(api_key.clone())
                    } else {
                        None
                    };

                    Box::new(GeminiProvider::new(
                        config.name.clone(),
                        config.base_url.clone(),
                        config.models.clone(),
                        HashMap::new(), // custom headers
                        GeminiAuthConfig {
                            api_key: api_key_opt,
                            oauth_provider_id: config.oauth_provider.clone(),
                            token_store: token_store.clone(),
                            project_id: None, // No project_id/location for Gemini (AI Studio/OAuth only)
                            location: None,
                        },
                    ))
                }

                "vertex-ai" => {
                    // Vertex AI provider (separate from Gemini)
                    // Uses Google Cloud Vertex AI with ADC authentication
                    Box::new(GeminiProvider::new(
                        config.name.clone(),
                        config.base_url.clone(),
                        config.models.clone(),
                        HashMap::new(), // custom headers
                        GeminiAuthConfig {
                            api_key: None,           // No API key for Vertex AI (uses ADC)
                            oauth_provider_id: None, // No OAuth for Vertex AI
                            token_store: token_store.clone(),
                            project_id: config.project_id.clone(), // GCP project ID
                            location: config.location.clone(),     // GCP location
                        },
                    ))
                }

                other => {
                    return Err(ProviderError::ConfigError(format!(
                        "Unknown provider type: {}",
                        other
                    )));
                }
            };

            // NOTE: models field in provider config is deprecated
            // Model mappings are now defined in [[models]] section
            // We only register the provider by name

            // Add provider to registry
            registry
                .providers
                .insert(config.name.clone(), Arc::new(provider));
        }

        // Populate model_to_provider mappings from model configurations
        for model in models {
            // Map each model name to its first (highest priority) provider
            if let Some(first_mapping) = model.mappings.first() {
                registry
                    .model_to_provider
                    .insert(model.name.clone(), first_mapping.provider.clone());
            }
        }

        Ok(registry)
    }

    /// Get a provider by name
    pub fn get_provider(&self, name: &str) -> Option<Arc<Box<dyn GatewayProvider>>> {
        self.providers.get(name).cloned()
    }

    /// Get a provider for a specific model
    pub fn get_provider_for_model(
        &self,
        model: &str,
    ) -> Result<Arc<Box<dyn GatewayProvider>>, ProviderError> {
        // First, check if we have a direct model → provider mapping
        if let Some(provider_name) = self.model_to_provider.get(model) {
            if let Some(provider) = self.providers.get(provider_name) {
                return Ok(provider.clone());
            }
        }

        // If no direct mapping, search through all providers
        for provider in self.providers.values() {
            if provider.supports_model(model) {
                return Ok(provider.clone());
            }
        }

        Err(ProviderError::ModelNotSupported(model.to_string()))
    }

    /// List all available models
    pub fn list_models(&self) -> Vec<String> {
        self.model_to_provider.keys().cloned().collect()
    }

    /// List all providers
    pub fn list_providers(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }

    #[allow(dead_code)]
    pub fn insert_provider_for_tests(
        &mut self,
        name: impl Into<String>,
        provider: Box<dyn GatewayProvider>,
    ) {
        self.providers.insert(name.into(), Arc::new(provider));
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_registry() {
        let registry = ProviderRegistry::new();
        assert!(registry.list_models().is_empty());
        assert!(registry.list_providers().is_empty());
    }

    #[test]
    fn test_get_provider_for_model_not_found() {
        let registry = ProviderRegistry::new();
        let result = registry.get_provider_for_model("gpt-4");
        assert!(result.is_err());
    }

    #[test]
    fn test_model_counting_with_configs() {
        use crate::providers::{AuthType, ProviderConfig};

        let providers = vec![
            ProviderConfig {
                name: "provider-a".to_string(),
                provider_type: "anthropic".to_string(),
                auth_type: AuthType::ApiKey,
                api_key: Some("test-key-1".to_string()),
                base_url: None,
                models: vec![],
                enabled: Some(true),
                oauth_provider: None,
                project_id: None,
                location: None,
                headers: None,
            },
            ProviderConfig {
                name: "provider-b".to_string(),
                provider_type: "anthropic".to_string(),
                auth_type: AuthType::ApiKey,
                api_key: Some("test-key-2".to_string()),
                base_url: None,
                models: vec![],
                enabled: Some(true),
                oauth_provider: None,
                project_id: None,
                location: None,
                headers: None,
            },
        ];

        let models = vec![
            crate::cli::ModelConfig {
                name: "model-1".to_string(),
                mappings: vec![crate::cli::ModelMapping {
                    priority: 1,
                    provider: "provider-a".to_string(),
                    actual_model: "actual-model-1".to_string(),
                    inject_continuation_prompt: false,
                }],
            },
            crate::cli::ModelConfig {
                name: "model-2".to_string(),
                mappings: vec![crate::cli::ModelMapping {
                    priority: 1,
                    provider: "provider-b".to_string(),
                    actual_model: "actual-model-2".to_string(),
                    inject_continuation_prompt: false,
                }],
            },
        ];

        // Actually test the method we implemented
        let registry = ProviderRegistry::from_configs_with_models(
            &providers, None, // token_store
            &models,
        )
        .unwrap();

        assert_eq!(registry.list_models().len(), 2);
        assert!(registry.list_models().contains(&"model-1".to_string()));
        assert!(registry.list_models().contains(&"model-2".to_string()));
        assert_eq!(registry.list_providers().len(), 2);
    }

    #[test]
    fn test_azure_openai_provider_type_is_registered() {
        use crate::providers::{AuthType, ProviderConfig};

        let providers = vec![ProviderConfig {
            name: "azure-kimi".to_string(),
            provider_type: "azure-openai".to_string(),
            auth_type: AuthType::ApiKey,
            api_key: Some("test-azure-key".to_string()),
            oauth_provider: None,
            project_id: None,
            location: None,
            base_url: Some("https://example.azure.com/openai/v1".to_string()),
            headers: None,
            models: vec!["kimi-k2-thinking".to_string()],
            enabled: Some(true),
        }];

        let models = vec![crate::cli::ModelConfig {
            name: "kimi-k2-thinking".to_string(),
            mappings: vec![crate::cli::ModelMapping {
                priority: 1,
                provider: "azure-kimi".to_string(),
                actual_model: "kimi-k2-thinking".to_string(),
                inject_continuation_prompt: false,
            }],
        }];

        let registry =
            ProviderRegistry::from_configs_with_models(&providers, None, &models).unwrap();

        assert!(registry.get_provider("azure-kimi").is_some());
        assert!(registry.get_provider_for_model("kimi-k2-thinking").is_ok());
    }
}
