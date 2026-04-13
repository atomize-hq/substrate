//! Reference-only source for the retired gateway admin/config UI.
//!
//! Extracted from `crates/gateway/src/server/mod.rs` when the active runtime
//! stopped serving the browser admin surface. This file is intentionally not
//! compiled; it preserves the source needed to understand or revive the UI.

use std::sync::Arc;

use axum::{
    extract::State,
    response::{Html, IntoResponse, Response},
    Json,
};

use crate::cli::AppConfig;
use crate::providers::ProviderRegistry;
use crate::router::Router;
use crate::server::{AppError, AppState, ReloadableState};

/// Former route wiring in the active server:
///
/// ```rust,ignore
/// let app = AxumRouter::new()
///     .route("/", get(serve_admin))
///     .route("/api/config/json", get(get_config_json))
///     .route("/api/config/json", post(update_config_json))
///     .route("/api/reload", post(reload_config));
/// ```

/// Former `GET /` handler for the browser admin UI.
async fn serve_admin() -> impl IntoResponse {
    Html(include_str!("admin.html"))
}

/// Former config inspection endpoint used by the admin UI.
async fn get_config_json(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let inner = state.snapshot();
    Json(serde_json::json!({
        "server": {
            "host": inner.config.server.host,
            "port": inner.config.server.port,
        },
        "router": {
            "default": inner.config.router.default,
            "background": inner.config.router.background,
            "think": inner.config.router.think,
            "websearch": inner.config.router.websearch,
            "auto_map_regex": inner.config.router.auto_map_regex,
            "background_regex": inner.config.router.background_regex,
            "prompt_rules": inner.config.router.prompt_rules,
        },
        "providers": inner.config.providers,
        "models": inner.config.models,
    }))
}

/// Former null-pruning helper used before TOML serialization.
fn remove_null_values(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            map.retain(|_, v| !v.is_null());
            for (_, v) in map.iter_mut() {
                remove_null_values(v);
            }
        }
        serde_json::Value::Array(arr) => {
            for item in arr.iter_mut() {
                remove_null_values(item);
            }
        }
        _ => {}
    }
}

/// Former config mutation endpoint used by the admin UI.
async fn update_config_json(
    State(state): State<Arc<AppState>>,
    Json(mut new_config): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    remove_null_values(&mut new_config);

    let config_path = &state.config_path;
    let config_str = std::fs::read_to_string(config_path)
        .map_err(|e| AppError::Parse(format!("Failed to read config: {}", e)))?;

    let mut config: toml::Value = toml::from_str(&config_str)
        .map_err(|e| AppError::Parse(format!("Failed to parse config: {}", e)))?;

    if let Some(providers) = new_config.get("providers") {
        let providers_toml: toml::Value = serde_json::from_str(&providers.to_string())
            .map_err(|e| AppError::Parse(format!("Failed to convert providers: {}", e)))?;

        if let Some(table) = config.as_table_mut() {
            table.insert("providers".to_string(), providers_toml);
        }
    }

    if let Some(models) = new_config.get("models") {
        let models_toml: toml::Value = serde_json::from_str(&models.to_string())
            .map_err(|e| AppError::Parse(format!("Failed to convert models: {}", e)))?;

        if let Some(table) = config.as_table_mut() {
            table.insert("models".to_string(), models_toml);
        }
    }

    if let Some(router) = new_config.get("router") {
        if let Some(router_table) = config.get_mut("router").and_then(|v| v.as_table_mut()) {
            let update_field = |table: &mut toml::map::Map<String, toml::Value>,
                                key: &str,
                                value: Option<&serde_json::Value>| {
                if let Some(val) = value {
                    if let Some(s) = val.as_str() {
                        table.insert(key.to_string(), toml::Value::String(s.to_string()));
                    }
                } else {
                    table.remove(key);
                }
            };

            if let Some(default) = router.get("default") {
                if let Some(s) = default.as_str() {
                    router_table.insert("default".to_string(), toml::Value::String(s.to_string()));
                }
            }

            update_field(router_table, "think", router.get("think"));
            update_field(router_table, "websearch", router.get("websearch"));
            update_field(router_table, "background", router.get("background"));
            update_field(router_table, "auto_map_regex", router.get("auto_map_regex"));
            update_field(
                router_table,
                "background_regex",
                router.get("background_regex"),
            );
        }
    }

    let new_config_str = toml::to_string_pretty(&config)
        .map_err(|e| AppError::Parse(format!("Failed to serialize config: {}", e)))?;

    std::fs::write(config_path, new_config_str)
        .map_err(|e| AppError::Parse(format!("Failed to write config: {}", e)))?;

    Ok(Json(serde_json::json!({
        "status": "success",
        "message": "Configuration saved successfully"
    })))
}

/// Former config reload endpoint used by the admin UI.
async fn reload_config(State(state): State<Arc<AppState>>) -> Response {
    let config_str = match std::fs::read_to_string(&state.config_path) {
        Ok(s) => s,
        Err(e) => {
            return Html(format!("<div class='px-4 py-3 rounded-xl bg-red-500/20 border border-red-500/50 text-foreground text-sm'><strong>❌ Reload failed</strong><br/>Failed to read config: {}</div>", e)).into_response();
        }
    };

    let new_config: AppConfig = match toml::from_str(&config_str) {
        Ok(c) => c,
        Err(e) => {
            return Html(format!("<div class='px-4 py-3 rounded-xl bg-red-500/20 border border-red-500/50 text-foreground text-sm'><strong>❌ Reload failed</strong><br/>Failed to parse config: {}</div>", e)).into_response();
        }
    };

    let new_router = Router::new(new_config.clone());
    let new_registry = match ProviderRegistry::from_configs_with_models(
        &new_config.providers,
        Some(state.token_store.clone()),
        &new_config.models,
    ) {
        Ok(r) => Arc::new(r),
        Err(e) => {
            return Html(format!("<div class='px-4 py-3 rounded-xl bg-red-500/20 border border-red-500/50 text-foreground text-sm'><strong>❌ Reload failed</strong><br/>Failed to init providers: {}</div>", e)).into_response();
        }
    };

    let new_inner = Arc::new(ReloadableState {
        config: new_config,
        router: new_router,
        provider_registry: new_registry,
    });

    *state.inner.write().unwrap() = new_inner;

    Html("<div class='px-4 py-3 rounded-xl bg-green-500/20 border border-green-500/50 text-foreground text-sm'><strong>✅ Configuration reloaded</strong><br/>New settings are now active.</div>").into_response()
}
