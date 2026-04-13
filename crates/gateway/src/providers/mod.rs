pub mod anthropic_compatible;
pub mod error;
pub mod gemini;
pub mod openai;
pub mod registry;
pub mod streaming;

use crate::core::{GatewayRequest, GatewayResponse, GatewayStreamResponse};
use crate::models::{CountTokensRequest, CountTokensResponse};
use async_trait::async_trait;
use error::ProviderError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Transport mode for OpenAI-compatible providers.
///
/// `AzureOpenAI` keeps Azure Foundry auth separate from the generic OpenAI path
/// without changing the upstream Anthropic or router contracts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OpenAITransport {
    OpenAI,
    AzureOpenAI,
}

/// Main provider trait - all providers must implement this.
#[async_trait]
pub trait GatewayProvider: Send + Sync {
    /// Send a message request to the provider
    /// Must transform to/from provider format as needed
    async fn send_message(&self, request: GatewayRequest)
        -> Result<GatewayResponse, ProviderError>;

    /// Send a streaming message request to the provider
    /// Returns a stream of raw bytes (currently Anthropic SSE) along with headers to forward
    async fn send_message_stream(
        &self,
        request: GatewayRequest,
    ) -> Result<GatewayStreamResponse, ProviderError>;

    /// Count tokens for a request
    /// Provider-specific implementation (tiktoken for OpenAI, etc.)
    async fn count_tokens(
        &self,
        request: CountTokensRequest,
    ) -> Result<CountTokensResponse, ProviderError>;

    /// Check if provider supports a specific model
    fn supports_model(&self, model: &str) -> bool;
}

/// Authentication type for providers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum AuthType {
    /// API key authentication
    #[default]
    ApiKey,
    /// OAuth 2.0 authentication
    OAuth,
}

/// Provider configuration from TOML
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub name: String,
    /// Provider implementation label. OpenAI-compatible providers support
    /// `openai` and `azure-openai` here.
    pub provider_type: String,

    /// Authentication type (default: api_key)
    #[serde(default)]
    pub auth_type: AuthType,

    /// API key (required for auth_type = "apikey")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,

    /// OAuth provider ID (required for auth_type = "oauth")
    /// References a token stored in TokenStore
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oauth_provider: Option<String>,

    /// Google Cloud Project ID (for Vertex AI provider)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,

    /// Location/Region (for Vertex AI provider)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,

    pub base_url: Option<String>,

    /// Custom HTTP headers (e.g., {"X-Novita-Source": "substrate-gateway"})
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,

    pub models: Vec<String>,
    pub enabled: Option<bool>,
}

impl ProviderConfig {
    pub fn is_enabled(&self) -> bool {
        self.enabled.unwrap_or(true)
    }

    /// Get the API key or OAuth provider ID
    #[allow(dead_code)]
    pub fn get_auth_credential(&self) -> Option<String> {
        match self.auth_type {
            AuthType::ApiKey => self.api_key.clone(),
            AuthType::OAuth => self.oauth_provider.clone(),
        }
    }
}

// Re-export provider implementations
pub use anthropic_compatible::AnthropicCompatibleProvider;
pub use openai::OpenAIProvider;
pub(crate) use openai::OpenAIProviderConfig;
pub use registry::ProviderRegistry;
