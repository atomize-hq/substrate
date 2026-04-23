use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(try_from = "IdentityTupleDef")]
pub struct IdentityTuple {
    pub client: String,
    pub router: String,
    pub protocol: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth_authority: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct IdentityTupleDef {
    client: String,
    router: String,
    protocol: String,
    #[serde(default)]
    provider: Option<String>,
    #[serde(default)]
    auth_authority: Option<String>,
}

impl IdentityTuple {
    pub fn validate(&self) -> Result<(), String> {
        validate_required_snake_case_id("client", &self.client)?;
        validate_required_snake_case_id("router", &self.router)?;
        validate_required_dotted_id("protocol", &self.protocol)?;
        validate_optional_snake_case_id("provider", self.provider.as_deref())?;
        validate_optional_snake_case_id("auth_authority", self.auth_authority.as_deref())?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PlacementExecution {
    InWorld,
    HostOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(try_from = "PlacementPostureDef")]
pub struct PlacementPosture {
    pub execution: PlacementExecution,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host_to_world_bridge: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
struct PlacementPostureDef {
    execution: PlacementExecution,
    #[serde(default)]
    host_to_world_bridge: Option<bool>,
}

impl PlacementPosture {
    pub fn validate(&self) -> Result<(), String> {
        if self.host_to_world_bridge == Some(false) {
            return Err(
                "placement_posture.host_to_world_bridge must be omitted unless it is true"
                    .to_string(),
            );
        }

        if self.execution == PlacementExecution::HostOnly && self.host_to_world_bridge == Some(true)
        {
            return Err(
                "placement_posture.execution=\"host_only\" is invalid with host_to_world_bridge=true"
                    .to_string(),
            );
        }

        Ok(())
    }
}

impl TryFrom<IdentityTupleDef> for IdentityTuple {
    type Error = String;

    fn try_from(value: IdentityTupleDef) -> Result<Self, Self::Error> {
        let tuple = Self {
            client: value.client,
            router: value.router,
            protocol: value.protocol,
            provider: value.provider,
            auth_authority: value.auth_authority,
        };
        tuple.validate()?;
        Ok(tuple)
    }
}

impl TryFrom<PlacementPostureDef> for PlacementPosture {
    type Error = String;

    fn try_from(value: PlacementPostureDef) -> Result<Self, Self::Error> {
        let posture = Self {
            execution: value.execution,
            host_to_world_bridge: value.host_to_world_bridge,
        };
        posture.validate()?;
        Ok(posture)
    }
}

pub fn validate_identity_tuple_and_placement_posture(
    identity_tuple: Option<&IdentityTuple>,
    placement_posture: Option<&PlacementPosture>,
) -> Result<(), String> {
    if let Some(identity_tuple) = identity_tuple {
        identity_tuple.validate()?;
    }
    if let Some(placement_posture) = placement_posture {
        placement_posture.validate()?;
    }

    if let (Some(identity_tuple), Some(placement_posture)) = (identity_tuple, placement_posture) {
        if identity_tuple.router == "direct_provider_path"
            && placement_posture.execution != PlacementExecution::HostOnly
        {
            return Err(
                "identity_tuple.router=\"direct_provider_path\" requires placement_posture.execution=\"host_only\""
                    .to_string(),
            );
        }

        if identity_tuple.router == "direct_provider_path"
            && placement_posture.host_to_world_bridge == Some(true)
        {
            return Err(
                "identity_tuple.router=\"direct_provider_path\" is invalid with placement_posture.host_to_world_bridge=true"
                    .to_string(),
            );
        }
    }

    Ok(())
}

fn validate_required_snake_case_id(field: &str, value: &str) -> Result<(), String> {
    if value.is_empty() {
        return Err(format!("identity_tuple.{field} must not be empty"));
    }
    if !snake_case_id_pattern().is_match(value) {
        return Err(format!(
            "identity_tuple.{field} must use lowercase snake_case ids"
        ));
    }
    Ok(())
}

fn validate_optional_snake_case_id(field: &str, value: Option<&str>) -> Result<(), String> {
    let Some(value) = value else {
        return Ok(());
    };

    if value.is_empty() {
        return Err(format!(
            "identity_tuple.{field} must be omitted when unresolved or not applicable"
        ));
    }
    if !snake_case_id_pattern().is_match(value) {
        return Err(format!(
            "identity_tuple.{field} must use lowercase snake_case ids"
        ));
    }
    Ok(())
}

fn validate_required_dotted_id(field: &str, value: &str) -> Result<(), String> {
    if value.is_empty() {
        return Err(format!("identity_tuple.{field} must not be empty"));
    }
    if !dotted_id_pattern().is_match(value) {
        return Err(format!(
            "identity_tuple.{field} must use lowercase dotted ids"
        ));
    }
    Ok(())
}

fn snake_case_id_pattern() -> &'static Regex {
    static SNAKE_CASE_ID_RE: OnceLock<Regex> = OnceLock::new();
    SNAKE_CASE_ID_RE.get_or_init(|| {
        Regex::new(r"^[a-z][a-z0-9]*(?:_[a-z0-9]+)*$").expect("snake_case id regex is valid")
    })
}

fn dotted_id_pattern() -> &'static Regex {
    static DOTTED_ID_RE: OnceLock<Regex> = OnceLock::new();
    DOTTED_ID_RE.get_or_init(|| {
        Regex::new(r"^[a-z][a-z0-9]*(?:\.[a-z0-9]+)+$").expect("dotted id regex is valid")
    })
}
