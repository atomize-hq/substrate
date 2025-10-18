//! Authentication service for the host proxy.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use agent_api_types::ApiError;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, warn};

use crate::AuthConfig;

/// Authentication token.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthToken {
    pub agent_id: String,
    pub token: String,
    pub scopes: Vec<String>,
    pub expires_at: Option<i64>,
}

/// Authentication service.
pub struct AuthService {
    config: AuthConfig,
    tokens: Arc<RwLock<HashMap<String, AuthToken>>>,
}

impl AuthService {
    /// Create a new authentication service.
    pub fn new(config: &AuthConfig) -> Result<Self> {
        let tokens = Arc::new(RwLock::new(HashMap::new()));

        let service = Self {
            config: config.clone(),
            tokens,
        };

        // Load tokens from file if configured
        if let Some(token_file) = &config.token_file {
            if token_file.exists() {
                let _ = futures::executor::block_on(service.load_tokens(token_file));
            }
        }

        Ok(service)
    }

    /// Load tokens from a file.
    async fn load_tokens(&self, path: &Path) -> Result<()> {
        let content = tokio::fs::read_to_string(path)
            .await
            .context("Failed to read token file")?;

        let tokens: Vec<AuthToken> =
            serde_json::from_str(&content).context("Failed to parse token file")?;

        let mut token_map = self.tokens.write().await;
        for token in tokens {
            token_map.insert(token.agent_id.clone(), token);
        }

        debug!("Loaded {} authentication tokens", token_map.len());
        Ok(())
    }

    /// Verify an agent's authentication.
    pub async fn verify_agent(&self, agent_id: &str) -> Result<(), ApiError> {
        // Allow anonymous if configured
        if self.config.allow_anonymous {
            return Ok(());
        }

        // Check if agent has valid token
        let tokens = self.tokens.read().await;
        if let Some(token) = tokens.get(agent_id) {
            // Check expiration
            if let Some(expires_at) = token.expires_at {
                let now = chrono::Utc::now().timestamp();
                if now > expires_at {
                    warn!("Token expired for agent: {}", agent_id);
                    return Err(ApiError::BadRequest("Token expired".to_string()));
                }
            }
            Ok(())
        } else {
            Err(ApiError::BadRequest(
                "Invalid or missing authentication".to_string(),
            ))
        }
    }

    /// Check if an agent has a specific scope.
    pub async fn has_scope(&self, agent_id: &str, scope: &str) -> bool {
        let tokens = self.tokens.read().await;
        if let Some(token) = tokens.get(agent_id) {
            token.scopes.contains(&scope.to_string())
        } else {
            false
        }
    }

    /// Add a new token.
    pub async fn add_token(&self, token: AuthToken) -> Result<()> {
        let mut tokens = self.tokens.write().await;
        tokens.insert(token.agent_id.clone(), token);
        Ok(())
    }

    /// Remove a token.
    pub async fn remove_token(&self, agent_id: &str) -> Result<()> {
        let mut tokens = self.tokens.write().await;
        tokens.remove(agent_id);
        Ok(())
    }
}
