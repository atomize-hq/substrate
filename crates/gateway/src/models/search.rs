//! Closed standalone-search wire contract pinned to Codex b412ff32c417f855c2b2d1581b77058eed87c84b.
//! Search IDs and references are body data, never local authorization or isolation claims.
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{fmt, io};

/// Maximum admitted UTF-8 request wire bytes.
pub const REQUEST_LIMIT: usize = 2 * 1024 * 1024;
/// Maximum cumulative upstream UTF-8 response wire bytes.
pub const RESPONSE_LIMIT: usize = 16 * 1024 * 1024;
/// Conservative logical payload accounting, not allocator capacity or RSS.
pub const WORKING_LIMIT: usize = 64 * 1024 * 1024;

/// Native search envelope; unknown fields and explicit null request options fail closed.
///
/// ```
/// use substrate_gateway::models::search::SearchRequest;
/// let request: SearchRequest = serde_json::from_str(r#"{"id":"session","model":"slug","commands":{}}"#)?;
/// assert_eq!(request.model, "slug");
/// assert!(!format!("{request:?}").contains("session"));
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SearchRequest {
    pub id: String,
    pub model: String,
    #[serde(
        default,
        deserialize_with = "strict_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub reasoning: Option<SearchReasoning>,
    #[serde(
        default,
        deserialize_with = "strict_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub input: Option<SearchInput>,
    #[serde(
        default,
        deserialize_with = "strict_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub commands: Option<SearchCommands>,
    #[serde(
        default,
        deserialize_with = "strict_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub settings: Option<SearchSettings>,
    #[serde(
        default,
        deserialize_with = "strict_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub max_output_tokens: Option<u64>,
}

/// Text or the message population emitted by pinned web-search `recent_input`.
/// Other general Responses item variants are rejected explicitly.
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum SearchInput {
    Text(String),
    Items(Vec<SearchMessage>),
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum SearchMessage {
    Message {
        #[serde(
            default,
            deserialize_with = "strict_some",
            skip_serializing_if = "Option::is_none"
        )]
        id: Option<String>,
        role: String,
        content: Vec<SearchContent>,
        #[serde(
            default,
            deserialize_with = "strict_some",
            skip_serializing_if = "Option::is_none"
        )]
        phase: Option<MessagePhase>,
        #[serde(
            default,
            deserialize_with = "strict_some",
            skip_serializing_if = "Option::is_none"
        )]
        internal_chat_message_metadata_passthrough: Option<SearchMessageMetadata>,
    },
}

/// Untrusted pinned message metadata, forwarded only inside the search body.
/// Receiving these fields verifies no host record, execution, identity or completion.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SearchMessageMetadata {
    #[serde(
        default,
        deserialize_with = "strict_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub turn_id: Option<String>,
    #[serde(
        default,
        deserialize_with = "strict_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub create_time: Option<serde_json::Number>,
    #[serde(
        default,
        deserialize_with = "strict_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub content_item_kinds: Option<Vec<String>>,
    #[serde(
        default,
        deserialize_with = "strict_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub cell_id: Option<String>,
    #[serde(
        default,
        deserialize_with = "strict_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub executed_tool_calls: Option<Vec<SearchExecutedCall>>,
    #[serde(
        default,
        deserialize_with = "strict_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub tool_calls_complete: Option<bool>,
}

/// Transport-only call metadata. The two named Value leaves remain inert;
/// truncation/omission markers are neither generated nor interpreted here.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SearchExecutedCall {
    pub name: String,
    pub arguments: Value,
    #[serde(
        default,
        deserialize_with = "strict_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub tool_result_sources: Option<Vec<SearchResultSource>>,
    // Value accepts explicit null; Some(Null) is distinct from an absent field.
    #[serde(
        default,
        deserialize_with = "strict_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub tool_result_metadata: Option<Value>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SearchResultSource {
    pub r#type: String,
    pub id: String,
}

#[derive(Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "snake_case",
    try_from = "SearchContentWire"
)]
pub enum SearchContent {
    InputText {
        text: String,
    },
    OutputText {
        text: String,
    },
    InputImage {
        #[serde(skip_serializing_if = "Option::is_none")]
        image_url: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        file_id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        detail: Option<ImageDetail>,
    },
    InputAudio {
        audio_url: String,
    },
}
// Avoid flatten+deny_unknown_fields: enforce the native untagged image choice
// explicitly so dual image_url/file_id or ignored unknown fields cannot survive.
#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
enum SearchContentWire {
    InputText {
        text: String,
    },
    OutputText {
        text: String,
    },
    InputImage {
        #[serde(default, deserialize_with = "strict_some")]
        image_url: Option<String>,
        #[serde(default, deserialize_with = "strict_some")]
        file_id: Option<String>,
        #[serde(default, deserialize_with = "strict_some")]
        detail: Option<ImageDetail>,
    },
    InputAudio {
        audio_url: String,
    },
}
impl TryFrom<SearchContentWire> for SearchContent {
    type Error = &'static str;
    fn try_from(value: SearchContentWire) -> Result<Self, Self::Error> {
        Ok(match value {
            SearchContentWire::InputText { text } => Self::InputText { text },
            SearchContentWire::OutputText { text } => Self::OutputText { text },
            SearchContentWire::InputAudio { audio_url } => Self::InputAudio { audio_url },
            SearchContentWire::InputImage {
                image_url,
                file_id,
                detail,
            } => {
                if image_url.is_some() == file_id.is_some() {
                    return Err("exactly one image reference is required");
                }
                Self::InputImage {
                    image_url,
                    file_id,
                    detail,
                }
            }
        })
    }
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessagePhase {
    Commentary,
    FinalAnswer,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImageDetail {
    Auto,
    Low,
    High,
    Original,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SearchReasoning {
    #[serde(
        default,
        deserialize_with = "nonempty_effort",
        skip_serializing_if = "Option::is_none"
    )]
    pub effort: Option<String>,
    #[serde(
        default,
        deserialize_with = "strict_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub summary: Option<ReasoningSummary>,
    #[serde(
        default,
        deserialize_with = "strict_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub context: Option<ReasoningContext>,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReasoningSummary {
    Auto,
    Concise,
    Detailed,
    None,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReasoningContext {
    Auto,
    CurrentTurn,
    AllTurns,
}
fn nonempty_effort<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    let value = String::deserialize(d)?;
    if value.is_empty() {
        return Err(serde::de::Error::custom("empty effort"));
    }
    Ok(Some(value))
}
fn strict_some<'de, D: serde::Deserializer<'de>, T: Deserialize<'de>>(
    d: D,
) -> Result<Option<T>, D::Error> {
    T::deserialize(d).map(Some)
}
fn nullable_some<'de, D: serde::Deserializer<'de>, T: Deserialize<'de>>(
    d: D,
) -> Result<Option<Option<T>>, D::Error> {
    Option::<T>::deserialize(d).map(Some)
}

/// Only these named response fields carry opaque service data; no field drives local I/O.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SearchResponse {
    pub output: String,
    #[serde(
        default,
        deserialize_with = "nullable_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub results: Option<Option<Vec<Value>>>,
    #[serde(
        default,
        deserialize_with = "nullable_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub encrypted_output: Option<Option<String>>,
}

/// Fixed search failure classifications, safe for public and diagnostic views.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum SearchError {
    #[error("search operation is not admitted")]
    Unsupported,
    #[error("search credential is unavailable")]
    Authentication,
    #[error("search request is invalid")]
    InvalidRequest,
    #[error("search payload limit exceeded")]
    Limit,
    #[error("search capacity is busy")]
    Busy,
    #[error("search operation deadline exceeded; remote outcome is unknown")]
    Deadline,
    #[error("search transport failed; remote outcome is unknown")]
    Transport,
    #[error("search upstream denied the operation")]
    UpstreamDenied,
    #[error("search upstream is rate limited")]
    RateLimited,
    #[error("search upstream failed")]
    Upstream,
    #[error("search upstream returned an invalid response")]
    InvalidResponse,
}
#[derive(Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SearchCommands {
    /// Query the internet search engine for a given list of queries.
    #[serde(
        default,
        deserialize_with = "strict_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub search_query: Option<Vec<SearchQuery>>,
    /// Query the image search engine for a given list of queries.
    #[serde(
        default,
        deserialize_with = "strict_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub image_query: Option<Vec<SearchQuery>>,
    /// Open pages by reference id or URL.
    #[serde(
        default,
        deserialize_with = "strict_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub open: Option<Vec<OpenOperation>>,
    /// Open links from previously opened pages.
    #[serde(
        default,
        deserialize_with = "strict_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub click: Option<Vec<ClickOperation>>,
    /// Find text patterns in pages.
    #[serde(
        default,
        deserialize_with = "strict_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub find: Option<Vec<FindOperation>>,
    /// Take screenshots of PDF pages.
    #[serde(
        default,
        deserialize_with = "strict_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub screenshot: Option<Vec<ScreenshotOperation>>,
    /// Look up prices for the given stock symbols.
    #[serde(
        default,
        deserialize_with = "strict_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub finance: Option<Vec<FinanceOperation>>,
    /// Look up weather forecasts.
    #[serde(
        default,
        deserialize_with = "strict_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub weather: Option<Vec<WeatherOperation>>,
    /// Look up sports schedules and standings.
    #[serde(
        default,
        deserialize_with = "strict_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub sports: Option<Vec<SportsOperation>>,
    /// Get time for the given UTC offsets.
    #[serde(
        default,
        deserialize_with = "strict_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub time: Option<Vec<TimeOperation>>,
    /// Set the length of the response to be returned.
    #[serde(
        default,
        deserialize_with = "strict_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub response_length: Option<SearchResponseLength>,
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SearchQuery {
    /// Search query.
    pub q: String,
    /// Whether to filter by recency, as a number of recent days.
    #[serde(
        default,
        deserialize_with = "strict_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub recency: Option<u64>,
    /// Whether to filter by a specific list of domains.
    #[serde(
        default,
        deserialize_with = "strict_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub domains: Option<Vec<String>>,
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct OpenOperation {
    /// Reference id or URL to open.
    pub ref_id: String,
    /// Line number to position the page at.
    #[serde(
        default,
        deserialize_with = "strict_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub lineno: Option<u64>,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ClickOperation {
    /// Reference id containing the numbered link.
    pub ref_id: String,
    /// Numbered link id to open.
    pub id: u64,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct FindOperation {
    /// Reference id or URL to search within.
    pub ref_id: String,
    /// Text pattern to find.
    pub pattern: String,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ScreenshotOperation {
    /// Reference id or URL to screenshot.
    pub ref_id: String,
    /// Zero-indexed PDF page number.
    pub pageno: u64,
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct FinanceOperation {
    /// Ticker symbol to look up.
    pub ticker: String,
    /// Asset type to look up.
    pub r#type: FinanceAssetType,
    /// ISO 3166-1 alpha-3 country code, "OTC", or "" for cryptocurrency.
    #[serde(
        default,
        deserialize_with = "strict_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub market: Option<String>,
}

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FinanceAssetType {
    Equity,
    Fund,
    Crypto,
    Index,
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct WeatherOperation {
    /// Location in "Country, Area, City" format.
    pub location: String,
    /// Start date in YYYY-MM-DD format. Defaults to today.
    #[serde(
        default,
        deserialize_with = "strict_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub start: Option<String>,
    /// Number of days to return. Defaults to 7.
    #[serde(
        default,
        deserialize_with = "strict_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub duration: Option<u64>,
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SportsOperation {
    /// Tool name for sports requests.
    #[serde(
        default,
        deserialize_with = "strict_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub tool: Option<SportsToolName>,
    /// Sports function to call.
    pub r#fn: SportsFunction,
    /// League to look up.
    pub league: SportsLeague,
    /// Team to look up, using the common 3 or 4 letter alias used in broadcasts.
    #[serde(
        default,
        deserialize_with = "strict_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub team: Option<String>,
    /// Opponent to use with `team` when narrowing the lookup.
    #[serde(
        default,
        deserialize_with = "strict_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub opponent: Option<String>,
    /// Start date in YYYY-MM-DD format.
    #[serde(
        default,
        deserialize_with = "strict_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub date_from: Option<String>,
    /// End date in YYYY-MM-DD format.
    #[serde(
        default,
        deserialize_with = "strict_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub date_to: Option<String>,
    /// Number of games to return.
    #[serde(
        default,
        deserialize_with = "strict_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub num_games: Option<u64>,
    /// Locale for the lookup.
    #[serde(
        default,
        deserialize_with = "strict_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub locale: Option<String>,
}

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SportsToolName {
    Sports,
}

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SportsFunction {
    Schedule,
    Standings,
}

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SportsLeague {
    Nba,
    Wnba,
    Nfl,
    Nhl,
    Mlb,
    Epl,
    Ncaamb,
    Ncaawb,
    Ipl,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TimeOperation {
    /// UTC offset formatted like "+03:00".
    pub utc_offset: String,
}

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SearchResponseLength {
    Short,
    Medium,
    Long,
}

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExternalWebAccessMode {
    Cached,
    Indexed,
    Live,
}

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum ExternalWebAccess {
    Boolean(bool),
    Mode(ExternalWebAccessMode),
}

#[derive(Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SearchSettings {
    #[serde(
        default,
        deserialize_with = "strict_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub user_location: Option<ApproximateLocation>,
    #[serde(
        default,
        deserialize_with = "strict_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub search_context_size: Option<SearchContextSize>,
    #[serde(
        default,
        deserialize_with = "strict_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub filters: Option<SearchFilters>,
    #[serde(
        default,
        deserialize_with = "strict_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub image_settings: Option<SearchImageSettings>,
    #[serde(
        default,
        deserialize_with = "strict_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub allowed_callers: Option<Vec<AllowedCaller>>,
    #[serde(
        default,
        deserialize_with = "strict_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub external_web_access: Option<ExternalWebAccess>,
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ApproximateLocation {
    pub r#type: LocationType,
    #[serde(
        default,
        deserialize_with = "strict_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub country: Option<String>,
    #[serde(
        default,
        deserialize_with = "strict_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub region: Option<String>,
    #[serde(
        default,
        deserialize_with = "strict_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub city: Option<String>,
    #[serde(
        default,
        deserialize_with = "strict_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub timezone: Option<String>,
}

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LocationType {
    Approximate,
}

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SearchContextSize {
    Low,
    Medium,
    High,
}

#[derive(Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SearchFilters {
    #[serde(
        default,
        deserialize_with = "strict_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub allowed_domains: Option<Vec<String>>,
    #[serde(
        default,
        deserialize_with = "strict_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub blocked_domains: Option<Vec<String>>,
}

#[derive(Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SearchImageSettings {
    #[serde(
        default,
        deserialize_with = "strict_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub max_results: Option<u64>,
    #[serde(
        default,
        deserialize_with = "strict_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub caption: Option<bool>,
}

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AllowedCaller {
    Direct,
    Shell,
    CodeInterpreter,
}

// No raw transport data appears in Debug, including nested commands and settings.
macro_rules! safe_debug {
    ($($name:ty),* $(,)?) => {$ (
        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(concat!(stringify!($name), "([private search transport])"))
            }
        }
    )*};
}
safe_debug!(
    SearchMessageMetadata,
    SearchExecutedCall,
    SearchResultSource,
    SearchRequest,
    SearchInput,
    SearchMessage,
    SearchContent,
    MessagePhase,
    ImageDetail,
    SearchReasoning,
    ReasoningSummary,
    ReasoningContext,
    SearchResponse,
    SearchCommands,
    SearchQuery,
    OpenOperation,
    ClickOperation,
    FindOperation,
    ScreenshotOperation,
    FinanceOperation,
    FinanceAssetType,
    WeatherOperation,
    SportsOperation,
    SportsToolName,
    SportsFunction,
    SportsLeague,
    TimeOperation,
    SearchResponseLength,
    ExternalWebAccessMode,
    ExternalWebAccess,
    SearchSettings,
    ApproximateLocation,
    LocationType,
    SearchContextSize,
    SearchFilters,
    SearchImageSettings,
    AllowedCaller
);

/// Preflight application-owned JSON representations before allocating typed nodes.
/// Counts every outside-string byte conservatively as a node, and every string
/// once. Three wire copies cover raw input, decoded strings/scratch, and wire
/// serialization at their actual (partly disjoint) lifetimes. 256 bytes per node
/// covers typed fields, Value/map entries and vector growth. This is a logical
/// payload bound, not a bound on reqwest/TLS/allocator memory or process RSS.
pub(crate) fn preflight_json(raw: &[u8], wire_limit: usize) -> Result<(), SearchError> {
    if raw.len() > wire_limit {
        return Err(SearchError::Limit);
    }
    let mut logical = raw.len().checked_mul(3).ok_or(SearchError::Limit)?;
    // Retained outbound request and bounded fixed operation/model/header metadata.
    logical = logical
        .checked_add(4 * REQUEST_LIMIT)
        .ok_or(SearchError::Limit)?;
    let mut in_string = false;
    let mut escaped = false;
    for &byte in raw {
        if in_string {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                in_string = false;
            }
        } else if !byte.is_ascii_whitespace() {
            logical = logical.checked_add(256).ok_or(SearchError::Limit)?;
            if logical > WORKING_LIMIT {
                return Err(SearchError::Limit);
            }
            in_string = byte == b'"';
        }
    }
    if logical > WORKING_LIMIT {
        return Err(SearchError::Limit);
    }
    Ok(())
}

/// Serialize without allowing an unbounded temporary allocation first.
pub(crate) fn bounded_json<T: Serialize>(value: &T, limit: usize) -> Result<Vec<u8>, SearchError> {
    struct Writer {
        bytes: Vec<u8>,
        limit: usize,
    }
    impl io::Write for Writer {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            if self
                .bytes
                .len()
                .checked_add(bytes.len())
                .is_none_or(|n| n > self.limit)
            {
                return Err(io::Error::other("search limit"));
            }
            self.bytes
                .try_reserve_exact(bytes.len())
                .map_err(|_| io::Error::other("search limit"))?;
            self.bytes.extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    // First count without retaining a serialized buffer, then reserve that exact
    // bounded capacity. Avoid repeated growth for an opaque tree with many small
    // serialized fields. The writer still checks every subsequent copy.
    let size =
        super::response_transport::payload_size(value, limit).map_err(|_| SearchError::Limit)?;
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(size)
        .map_err(|_| SearchError::Limit)?;
    let mut writer = Writer { bytes, limit };
    serde_json::to_writer(&mut writer, value).map_err(|_| SearchError::Limit)?;
    Ok(writer.bytes)
}
