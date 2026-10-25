pub mod images;
pub mod response_transport;
pub mod search;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Anthropic Messages API request format (ingress-only).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AnthropicMessagesRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<ThinkingConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequences: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<SystemPrompt>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
}

/// Message in the conversation
#[derive(Clone, Deserialize, Serialize)]
pub struct Message {
    pub role: String,
    pub content: MessageContent,
}

/// Message content can be string or array of content blocks
#[derive(Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum MessageContent {
    Text(String),
    Blocks(Vec<ContentBlock>),
}

/// System prompt can be string or array of content blocks.
#[derive(Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum SystemPrompt {
    Text(String),
    Blocks(Vec<ContentBlock>),
}

impl SystemPrompt {
    /// Return the system prompt as plain text, dropping any non-text blocks.
    pub fn text_content(&self) -> String {
        match self {
            SystemPrompt::Text(text) => text.clone(),
            SystemPrompt::Blocks(blocks) => blocks
                .iter()
                .filter_map(|block| block.as_text().map(|text| text.to_string()))
                .collect::<Vec<_>>()
                .join("\n\n"),
        }
    }

    /// Build a system prompt from collected OpenAI-style content blocks.
    /// Text-only content stays in the text variant; multimodal content uses blocks.
    #[allow(dead_code)]
    pub fn from_content_blocks(blocks: Vec<ContentBlock>) -> Option<Self> {
        if blocks.is_empty() {
            return None;
        }
        Some(SystemPrompt::Blocks(blocks))
    }
}

/// Tool result content can be string or array of content blocks
#[derive(Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ToolResultContent {
    Text(String),
    Blocks(Vec<ToolResultBlock>),
    ResponseItems {
        items: Vec<response_transport::InputContent>,
    },
    DiscoveredTools {
        tools: Vec<response_transport::ToolDefinition>,
    },
}

impl fmt::Display for ToolResultContent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ToolResultContent::ResponseItems { .. } | ToolResultContent::DiscoveredTools { .. } => {
                f.write_str("[Responses result]")
            }
            ToolResultContent::Text(s) => f.write_str(s),
            ToolResultContent::Blocks(blocks) => f.write_str(
                &blocks
                    .iter()
                    .map(|block| match block {
                        ToolResultBlock::Known(KnownToolResultBlock::Text { text }) => text.clone(),
                        ToolResultBlock::Known(KnownToolResultBlock::Image { .. }) => {
                            "[Image]".to_string()
                        }
                        ToolResultBlock::Unknown(_) => "[Unknown]".to_string(),
                    })
                    .collect::<Vec<_>>()
                    .join("\n"),
            ),
        }
    }
}

/// Content blocks allowed in tool results.
/// Uses untagged enum to handle unknown types (like tool_reference) gracefully.
#[derive(Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ToolResultBlock {
    Known(KnownToolResultBlock),
    Unknown(serde_json::Value),
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum KnownToolResultBlock {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image")]
    Image { source: ImageSource },
}

/// Content block for multimodal messages.
///
/// Uses untagged deserialization with a two-level approach:
/// 1. First tries to parse as a KnownContentBlock (text, image, tool_use, etc.)
/// 2. If that fails, falls back to Unknown which captures the raw JSON
///
/// This allows the proxy to handle new content types (like "document" for PDFs,
/// or future types Anthropic may add) without failing to parse. Unknown types
/// are passed through unchanged to the backend provider.
#[derive(Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ContentBlock {
    /// Known content types with structured parsing
    Known(KnownContentBlock),
    /// Unknown content types - pass through as raw JSON
    Unknown(serde_json::Value),
}

/// Known content block types that we parse specifically
#[derive(Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum KnownContentBlock {
    #[serde(rename = "text")]
    Text {
        text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<serde_json::Value>,
    },
    #[serde(rename = "image")]
    Image {
        #[serde(with = "core_image_source")]
        source: ImageSource,
    },
    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        transport: Option<response_transport::ToolTransport>,
    },
    #[serde(rename = "tool_result")]
    ToolResult {
        tool_use_id: String,
        content: ToolResultContent,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        transport: Option<response_transport::ToolTransport>,
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        is_error: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<serde_json::Value>,
    },
    #[serde(rename = "gateway_response_item")]
    ResponseItem {
        item: Box<response_transport::ResponseItem>,
    },
    /// Thinking block - stored as raw JSON to preserve exact signature.
    #[serde(rename = "thinking")]
    Thinking {
        #[serde(flatten)]
        raw: serde_json::Value,
    },
}

// Convenience constructors for ContentBlock
impl ContentBlock {
    pub fn text(text: String, cache_control: Option<serde_json::Value>) -> Self {
        ContentBlock::Known(KnownContentBlock::Text {
            text,
            cache_control,
        })
    }

    pub fn image(source: ImageSource) -> Self {
        ContentBlock::Known(KnownContentBlock::Image { source })
    }

    pub fn tool_use(id: String, name: String, input: serde_json::Value) -> Self {
        ContentBlock::Known(KnownContentBlock::ToolUse {
            id,
            name,
            input,
            transport: None,
        })
    }

    pub fn thinking(raw: serde_json::Value) -> Self {
        ContentBlock::Known(KnownContentBlock::Thinking { raw })
    }

    /// Check if this is a tool result block
    pub fn is_tool_result(&self) -> bool {
        matches!(
            self,
            ContentBlock::Known(KnownContentBlock::ToolResult { .. })
        )
    }

    /// Get text content if this is a text block
    pub fn as_text(&self) -> Option<&str> {
        match self {
            ContentBlock::Known(KnownContentBlock::Text { text, .. }) => Some(text),
            _ => None,
        }
    }

    /// Get mutable reference to text content if this is a text block
    pub fn as_text_mut(&mut self) -> Option<&mut String> {
        match self {
            ContentBlock::Known(KnownContentBlock::Text { text, .. }) => Some(text),
            _ => None,
        }
    }
}

/// OpenAI image input detail retained by the gateway's typed request path.
///
/// ```
/// use substrate_gateway::models::ImageDetail;
/// assert_eq!(serde_json::to_value(ImageDetail::Auto)?, "auto");
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ImageDetail {
    Low,
    High,
    Auto,
    Original,
}

/// Image source for vision API.
///
/// `detail` is an in-memory OpenAI input annotation: clones retain it, while
/// canonical Anthropic-shaped serde omits it on serialization and ignores it on
/// deserialization.
/// Consequently, serializing and deserializing this type does not retain detail.
/// Core content-block serde retains the annotation; endpoint-owned Anthropic
/// serialization uses this external DTO projection and omits it.
/// Rust struct literals must specify `detail: None` when no control was supplied.
///
/// ```
/// use substrate_gateway::models::{ImageDetail, ImageSource};
/// let source = ImageSource {
///     r#type: "url".into(),
///     media_type: None,
///     data: None,
///     url: Some("https://example.com/image.png".into()),
///     detail: Some(ImageDetail::High),
/// };
/// assert_eq!(source.clone().detail, Some(ImageDetail::High));
/// assert_eq!(serde_json::to_value(&source)?, serde_json::json!({
///     "type": "url", "url": "https://example.com/image.png"
/// }));
/// let block = substrate_gateway::models::ContentBlock::image(source.clone());
/// let block: substrate_gateway::models::ContentBlock =
///     serde_json::from_value(serde_json::to_value(block)?)?;
/// assert!(matches!(block, substrate_gateway::models::ContentBlock::Known(
///     substrate_gateway::models::KnownContentBlock::Image { source }
/// ) if source.detail == Some(ImageDetail::High)));
/// let restored: ImageSource = serde_json::from_value(serde_json::to_value(source)?)?;
/// assert_eq!(restored.detail, None);
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ImageSource {
    pub r#type: String, // "base64" or "url"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Optional OpenAI image input control; absent from canonical Anthropic serde.
    #[serde(skip)]
    pub detail: Option<ImageDetail>,
}

// The core owns history serialization; the external ImageSource DTO retains its
// canonical Anthropic shape. Both projections use the same authoritative detail.
mod core_image_source {
    use super::{ImageDetail, ImageSource};
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    pub fn serialize<S: Serializer>(
        source: &ImageSource,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        struct View<'a> {
            #[serde(flatten)]
            source: &'a ImageSource,
            #[serde(skip_serializing_if = "Option::is_none")]
            detail: Option<ImageDetail>,
        }
        View {
            source,
            detail: source.detail,
        }
        .serialize(serializer)
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<ImageSource, D::Error> {
        #[derive(Deserialize)]
        struct View {
            #[serde(flatten)]
            source: ImageSource,
            #[serde(default)]
            detail: Option<ImageDetail>,
        }
        let mut view = View::deserialize(deserializer)?;
        view.source.detail = view.detail;
        Ok(view.source)
    }
}

/// Tool definition for function calling
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Tool {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_schema: Option<serde_json::Value>,
}

/// Thinking/reasoning configuration for Plan Mode
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ThinkingConfig {
    pub r#type: String, // "enabled" or "disabled"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub budget_tokens: Option<u32>,
}

/// Token usage information
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

/// Request for counting tokens
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CountTokensRequest {
    pub model: String,
    pub messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<SystemPrompt>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
}

/// Response for token counting
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CountTokensResponse {
    pub input_tokens: u32,
}

/// Router decision result
#[derive(Debug, Clone)]
pub struct RouteDecision {
    pub model_name: String,
    pub route_type: RouteType,
    pub matched_prompt: Option<String>,
}

/// Type of routing decision
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteType {
    WebSearch,
    PromptRule,
    Think,
    Background,
    Default,
}

impl std::fmt::Display for RouteType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RouteType::WebSearch => write!(f, "web-search"),
            RouteType::PromptRule => write!(f, "prompt-rule"),
            RouteType::Think => write!(f, "think"),
            RouteType::Background => write!(f, "background"),
            RouteType::Default => write!(f, "default"),
        }
    }
}

impl std::fmt::Debug for ContentBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ContentBlock([payload omitted])")
    }
}

impl std::fmt::Debug for KnownContentBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("KnownContentBlock([payload omitted])")
    }
}

impl std::fmt::Debug for ToolResultContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ToolResultContent([payload omitted])")
    }
}

impl std::fmt::Debug for ToolResultBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ToolResultBlock([payload omitted])")
    }
}

impl std::fmt::Debug for KnownToolResultBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("KnownToolResultBlock([payload omitted])")
    }
}

impl std::fmt::Debug for MessageContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("MessageContent([payload omitted])")
    }
}

impl std::fmt::Debug for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Message([payload omitted])")
    }
}

impl std::fmt::Debug for SystemPrompt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SystemPrompt([payload omitted])")
    }
}

#[cfg(test)]
mod image_detail_tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn gwa06_canonical_image_serde_remains_unchanged() {
        for detail in [
            Some(ImageDetail::Low),
            Some(ImageDetail::High),
            Some(ImageDetail::Auto),
            None,
        ] {
            let source = ImageSource {
                r#type: "url".into(),
                media_type: None,
                data: None,
                url: Some("https://example.com/image.png".into()),
                detail,
            };
            assert_eq!(source.clone().detail, detail);
            let canonical = json!({"type":"url", "url":"https://example.com/image.png"});
            assert_eq!(serde_json::to_value(&source).unwrap(), canonical);
            let restored: ImageSource = serde_json::from_value(canonical).unwrap();
            assert_eq!(restored.detail, None);
        }
        // Unknown canonical fields remain ignored; this is not new Messages ingress vocabulary.
        for wire in [json!("high"), json!("invalid"), json!({"unknown":true})] {
            let source: ImageSource = serde_json::from_value(json!({
                "type":"url", "url":"https://example.com/image.png", "detail":wire
            }))
            .unwrap();
            assert_eq!(source.detail, None);
        }
    }
}
