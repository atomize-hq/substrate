//! Closed unary image contract for the pinned Codex image producer.
//! Correlation labels and remote references confer no local authority. Media
//! validation preserves original compressed bytes; it never fetches or transcodes.
use base64::{engine::general_purpose::STANDARD, Engine as _};
use image::{ImageDecoder, ImageFormat, ImageReader};
use serde::{Deserialize, Serialize};
use std::{fmt, io, io::Cursor};
use tokio::time::Instant;

pub const REQUEST_LIMIT: usize = 96 * 1024 * 1024;
pub const RESPONSE_LIMIT: usize = 48 * 1024 * 1024;
pub const FILE_LIMIT: usize = 32 * 1024 * 1024;
pub const INPUT_LIMIT: usize = 64 * 1024 * 1024;
pub const OUTPUT_LIMIT: usize = 32 * 1024 * 1024;
pub const RASTER_LIMIT: usize = 64 * 1024 * 1024;
pub const PIXEL_LIMIT: u64 = 16_777_216;
pub const AXIS_LIMIT: u32 = 16_384;
pub const WORKING_LIMIT: usize = 512 * 1024 * 1024;
pub const PROMPT_LIMIT: usize = 1024 * 1024;
pub const URL_LIMIT: usize = 16 * 1024;
pub const LABEL_LIMIT: usize = 1024;
pub const MODEL: &str = "gpt-image-2";
pub const REQUEST_ID_HEADER: &str = "x-codex-imagegen-request-id";

fn strict_some<'de, D: serde::Deserializer<'de>, T: Deserialize<'de>>(
    d: D,
) -> Result<Option<T>, D::Error> {
    T::deserialize(d).map(Some)
}
fn present<'de, D: serde::Deserializer<'de>, T: Deserialize<'de>>(
    d: D,
) -> Result<Option<Option<T>>, D::Error> {
    Option::<T>::deserialize(d).map(Some)
}

/// Native background syntax. This gateway admits only omitted or `auto` requests.
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImageBackground {
    Transparent,
    Opaque,
    Auto,
}
/// Native quality syntax. This gateway admits only omitted or `auto` requests.
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImageQuality {
    Low,
    Medium,
    High,
    Auto,
}

/// Generation uses an operation model independent of the inference catalogue.
/// Unknown fields and explicit null request options are errors.
///
/// ```
/// use substrate_gateway::models::images::ImageGenerationRequest;
/// let r: ImageGenerationRequest = serde_json::from_str(
///     r#"{"model":"gpt-image-2","prompt":"red square","background":"auto"}"#)?;
/// let wire = serde_json::to_value(r)?;
/// assert_eq!(wire["background"], "auto");
/// assert!(wire.get("n").is_none());
/// assert!(serde_json::from_str::<ImageGenerationRequest>(
///     r#"{"model":"gpt-image-2","prompt":"red","quality":null}"#).is_err());
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImageGenerationRequest {
    pub prompt: String,
    pub model: String,
    #[serde(
        default,
        deserialize_with = "strict_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub background: Option<ImageBackground>,
    #[serde(
        default,
        deserialize_with = "strict_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub n: Option<u64>,
    #[serde(
        default,
        deserialize_with = "strict_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub quality: Option<ImageQuality>,
    #[serde(
        default,
        deserialize_with = "strict_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub size: Option<String>,
}

/// Ordered edit references (one through five), with the same native controls.
/// HTTP(S) references are forwarded literally, never fetched by the gateway.
///
/// ```
/// use substrate_gateway::models::images::ImageEditRequest;
/// let r: ImageEditRequest = serde_json::from_str(
///     r#"{"prompt":"combine","model":"gpt-image-2","images":[{"image_url":"https://example.com/a.png"},{"image_url":"https://example.com/b.png"}]}"#)?;
/// assert_eq!(r.images[0].image_url, "https://example.com/a.png");
/// assert_eq!(r.images[1].image_url, "https://example.com/b.png");
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImageEditRequest {
    pub images: Vec<ImageUrl>,
    pub prompt: String,
    pub model: String,
    #[serde(
        default,
        deserialize_with = "strict_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub background: Option<ImageBackground>,
    #[serde(
        default,
        deserialize_with = "strict_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub n: Option<u64>,
    #[serde(
        default,
        deserialize_with = "strict_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub quality: Option<ImageQuality>,
    #[serde(
        default,
        deserialize_with = "strict_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub size: Option<String>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImageUrl {
    pub image_url: String,
}

/// Named native result projection. Missing and explicit-null options round trip
/// separately. Unconsumed upstream extensions (including usage) are ignored,
/// bounded by preparse limits, and never copied into the public response.
///
/// ```
/// use substrate_gateway::models::images::ImageResponse;
/// let r: ImageResponse = serde_json::from_str(
///     r#"{"created":3,"data":[],"quality":null,"usage":{"private":"ignored"}}"#)?;
/// let wire = serde_json::to_value(r)?;
/// assert!(wire["quality"].is_null());
/// assert!(wire.get("background").is_none());
/// assert!(wire.get("usage").is_none());
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Serialize, Deserialize)]
pub struct ImageResponse {
    pub created: u64,
    pub data: Vec<ImageData>,
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    pub background: Option<Option<ImageBackground>>,
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    pub quality: Option<Option<ImageQuality>>,
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    pub size: Option<Option<String>>,
}
#[derive(Serialize, Deserialize)]
pub struct ImageData {
    pub b64_json: String,
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    pub generation_id: Option<Option<String>>,
}

/// Only these two untrusted caller correlation labels may be forwarded.
/// The deadline covers collection, validation, one send, and public serialization.
///
/// ```
/// use substrate_gateway::models::images::ImageContext;
/// let context = ImageContext::new(tokio::time::Instant::now() + std::time::Duration::from_secs(180));
/// assert!(context.turn_id.is_none());
/// assert!(context.originator.is_none());
/// ```
pub struct ImageContext {
    pub turn_id: Option<String>,
    pub originator: Option<String>,
    pub deadline: Instant,
}
impl ImageContext {
    pub fn new(deadline: Instant) -> Self {
        Self {
            turn_id: None,
            originator: None,
            deadline,
        }
    }
    pub(crate) fn validate(&self) -> Result<(), ImageError> {
        check_deadline(self.deadline)?;
        for s in [&self.turn_id, &self.originator].into_iter().flatten() {
            validate_label(s)?;
            reqwest::header::HeaderValue::from_str(s).map_err(|_| ImageError::InvalidRequest)?;
        }
        Ok(())
    }
}
/// Validated media result and its sole optional named response header.
/// No arbitrary HTTP header or body passthrough is exposed.
pub struct ImageResult {
    pub response: ImageResponse,
    pub request_id: Option<String>,
}

macro_rules! safe_debug {
    ($($t:ty),* $(,)?) => {$ (
        impl fmt::Debug for $t {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(concat!(stringify!($t), "([private image transport])"))
            }
        }
    )*};
}
safe_debug!(
    ImageGenerationRequest,
    ImageEditRequest,
    ImageUrl,
    ImageResponse,
    ImageData,
    ImageContext,
    ImageResult,
    ImageBackground,
    ImageQuality
);

/// Safe operation errors never include caller text, credentials or remote bodies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ImageError {
    #[error("image operation not admitted")]
    Unsupported,
    #[error("held image authentication unavailable")]
    Authentication,
    #[error("invalid image request")]
    InvalidRequest,
    #[error("image resource limit exceeded")]
    Limit,
    #[error("image operation capacity occupied")]
    Busy,
    #[error("image deadline exceeded; remote outcome is unknown if sent")]
    Deadline,
    #[error("image transport failed; remote outcome is unknown")]
    Transport,
    #[error("upstream image operation denied")]
    UpstreamDenied,
    #[error("upstream image rate limited")]
    RateLimited,
    #[error("upstream image operation failed")]
    Upstream,
    #[error("invalid upstream image response")]
    InvalidResponse,
}

pub(crate) fn check_deadline(deadline: Instant) -> Result<(), ImageError> {
    if Instant::now() >= deadline {
        Err(ImageError::Deadline)
    } else {
        Ok(())
    }
}
pub(crate) fn validate_label(s: &str) -> Result<(), ImageError> {
    if s.len() > LABEL_LIMIT {
        Err(ImageError::Limit)
    } else {
        Ok(())
    }
}
fn controls(
    prompt: &str,
    model: &str,
    background: Option<ImageBackground>,
    n: Option<u64>,
    quality: Option<ImageQuality>,
    size: Option<&str>,
) -> Result<(), ImageError> {
    if prompt.len() > PROMPT_LIMIT {
        return Err(ImageError::Limit);
    }
    if model != MODEL
        || background.is_some_and(|b| b != ImageBackground::Auto)
        || n.is_some()
        || quality.is_some_and(|q| q != ImageQuality::Auto)
        || size.is_some_and(|s| s != "auto")
    {
        return Err(ImageError::Unsupported);
    }
    Ok(())
}
impl ImageGenerationRequest {
    pub(crate) fn validate(&self) -> Result<(), ImageError> {
        controls(
            &self.prompt,
            &self.model,
            self.background,
            self.n,
            self.quality,
            self.size.as_deref(),
        )
    }
}
impl ImageEditRequest {
    pub(crate) async fn validate(&self, deadline: Instant) -> Result<(), ImageError> {
        controls(
            &self.prompt,
            &self.model,
            self.background,
            self.n,
            self.quality,
            self.size.as_deref(),
        )?;
        if !(1..=5).contains(&self.images.len()) {
            return Err(ImageError::InvalidRequest);
        }
        let mut aggregate = 0usize;
        for reference in &self.images {
            check_deadline(deadline)?;
            if reference.image_url.starts_with("data:") {
                let (prefix, data) = reference
                    .image_url
                    .split_once(',')
                    .ok_or(ImageError::InvalidRequest)?;
                let format = match prefix {
                    "data:image/png;base64" => ImageFormat::Png,
                    "data:image/jpeg;base64" => ImageFormat::Jpeg,
                    "data:image/webp;base64" => ImageFormat::WebP,
                    _ => return Err(ImageError::InvalidRequest),
                };
                let size = base64_size(data)?;
                aggregate = checked_add(aggregate, size, INPUT_LIMIT)?;
                validate_media(data, format, deadline).await?;
            } else {
                if reference.image_url.len() > URL_LIMIT {
                    return Err(ImageError::Limit);
                }
                // Parse only the bounded remote URL. The original bytes are sent.
                let url = url::Url::parse(&reference.image_url)
                    .map_err(|_| ImageError::InvalidRequest)?;
                if !matches!(url.scheme(), "https" | "http")
                    || url.host_str().is_none()
                    || !url.username().is_empty()
                    || url.password().is_some()
                {
                    return Err(ImageError::InvalidRequest);
                }
            }
            // No detached blocking decoder: bounded synchronous work stays owned.
            tokio::task::yield_now().await;
            check_deadline(deadline)?;
        }
        Ok(())
    }
}
impl ImageResponse {
    pub(crate) async fn validate(&self, deadline: Instant) -> Result<(), ImageError> {
        if self.data.is_empty() {
            return Err(ImageError::InvalidResponse);
        }
        if let Some(Some(size)) = &self.size {
            validate_label(size)?;
        }
        let mut aggregate = 0usize;
        for data in &self.data {
            if let Some(Some(id)) = &data.generation_id {
                validate_label(id)?;
            }
            let size = base64_size(&data.b64_json).map_err(response_error)?;
            aggregate = checked_add(aggregate, size, OUTPUT_LIMIT)?;
            validate_media(&data.b64_json, ImageFormat::Png, deadline)
                .await
                .map_err(response_error)?;
            tokio::task::yield_now().await;
            check_deadline(deadline)?;
        }
        Ok(())
    }
}
fn response_error(e: ImageError) -> ImageError {
    match e {
        ImageError::InvalidRequest => ImageError::InvalidResponse,
        other => other,
    }
}
pub(crate) fn checked_add(a: usize, b: usize, limit: usize) -> Result<usize, ImageError> {
    a.checked_add(b)
        .filter(|n| *n <= limit)
        .ok_or(ImageError::Limit)
}

/// Strict standard base64 length/padding/alphabet checks precede any allocation.
fn base64_size(data: &str) -> Result<usize, ImageError> {
    let raw = data.as_bytes();
    if raw.is_empty() || raw.len() % 4 != 0 {
        return Err(ImageError::InvalidRequest);
    }
    // Check the maximum encoded size before walking attacker-controlled input.
    let encoded_limit = FILE_LIMIT
        .div_ceil(3)
        .checked_mul(4)
        .ok_or(ImageError::Limit)?;
    if raw.len() > encoded_limit {
        return Err(ImageError::Limit);
    }
    let padding = if raw.ends_with(b"==") {
        2
    } else {
        usize::from(raw.ends_with(b"="))
    };
    let payload = &raw[..raw.len() - padding];
    if !payload
        .iter()
        .all(|c| c.is_ascii_alphanumeric() || *c == b'+' || *c == b'/')
    {
        return Err(ImageError::InvalidRequest);
    }
    let size = raw
        .len()
        .checked_div(4)
        .and_then(|n| n.checked_mul(3))
        .and_then(|n| n.checked_sub(padding))
        .ok_or(ImageError::Limit)?;
    if size > FILE_LIMIT {
        return Err(ImageError::Limit);
    }
    Ok(size)
}

// IMG-R1: this is a bounded geometry/container safety check, not a decoder.
// The mature decoder still validates pixels. In particular VP8X reports canvas
// dimensions while the VP8 decoder allocates planes using its inner header.
// References: developers.google.com/speed/webp/docs/riff_container and
// webp_lossless_bitstream_specification; image-webp 0.2.4 decoder/vp8/lossless.
fn webp_word(bytes: &[u8]) -> Result<u32, ImageError> {
    let b = bytes.get(..4).ok_or(ImageError::InvalidRequest)?;
    Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
}
fn webp_u24(bytes: &[u8]) -> Result<u32, ImageError> {
    let b = bytes.get(..3).ok_or(ImageError::InvalidRequest)?;
    Ok(u32::from(b[0]) | (u32::from(b[1]) << 8) | (u32::from(b[2]) << 16))
}
// One level of chunk iteration, with no allocation or recursive descent. The
// caller passes an exact RIFF or ANMF range; chunks cannot escape that range.
type WebPChunk<'a> = (&'a [u8], &'a [u8]);
fn webp_chunk<'a>(
    bytes: &'a [u8],
    position: &mut usize,
) -> Result<Option<WebPChunk<'a>>, ImageError> {
    if *position == bytes.len() {
        return Ok(None);
    }
    let end = position.checked_add(8).ok_or(ImageError::InvalidRequest)?;
    let header = bytes
        .get(*position..end)
        .ok_or(ImageError::InvalidRequest)?;
    let size = usize::try_from(webp_word(&header[4..])?).map_err(|_| ImageError::Limit)?;
    let payload_end = end.checked_add(size).ok_or(ImageError::InvalidRequest)?;
    let padded_end = payload_end
        .checked_add(size & 1)
        .ok_or(ImageError::InvalidRequest)?;
    let payload = bytes
        .get(end..payload_end)
        .ok_or(ImageError::InvalidRequest)?;
    if padded_end > bytes.len() || (size & 1 != 0 && bytes[payload_end] != 0) {
        return Err(ImageError::InvalidRequest);
    }
    *position = padded_end;
    Ok(Some((&header[..4], payload)))
}
fn webp_geometry(w: u32, h: u32) -> Result<(), ImageError> {
    let pixels = u64::from(w)
        .checked_mul(u64::from(h))
        .ok_or(ImageError::Limit)?;
    if w == 0
        || h == 0
        || w > AXIS_LIMIT
        || h > AXIS_LIMIT
        || pixels > PIXEL_LIMIT
        || pixels
            .checked_mul(4)
            .is_none_or(|bytes| bytes > RASTER_LIMIT as u64)
    {
        return Err(ImageError::Limit);
    }
    Ok(())
}
fn webp_frame(tag: &[u8], bytes: &[u8], expected: Option<(u32, u32)>) -> Result<(), ImageError> {
    let (w, h) = if tag == b"VP8 " {
        let b = bytes.get(..10).ok_or(ImageError::InvalidRequest)?;
        if b[0] & 1 != 0 || b[3..6] != [0x9d, 0x01, 0x2a] {
            return Err(ImageError::InvalidRequest);
        }
        let w = u32::from(u16::from_le_bytes([b[6], b[7]]) & 0x3fff);
        let h = u32::from(u16::from_le_bytes([b[8], b[9]]) & 0x3fff);
        // Bound the actual rounded Y/U/V plane geometry as well as the pixels.
        let rounded = u64::from(w)
            .div_ceil(16)
            .checked_mul(16)
            .and_then(|w| w.checked_mul(u64::from(h).div_ceil(16) * 16))
            .and_then(|pixels| pixels.checked_mul(3))
            .map(|bytes| bytes / 2)
            .ok_or(ImageError::Limit)?;
        if rounded > RASTER_LIMIT as u64 {
            return Err(ImageError::Limit);
        }
        (w, h)
    } else if tag == b"VP8L" {
        let b = bytes.get(..5).ok_or(ImageError::InvalidRequest)?;
        if b[0] != 0x2f {
            return Err(ImageError::InvalidRequest);
        }
        let bits = webp_word(&b[1..])?;
        if bits >> 29 != 0 {
            return Err(ImageError::InvalidRequest);
        }
        ((bits & 0x3fff) + 1, ((bits >> 14) & 0x3fff) + 1)
    } else {
        return Err(ImageError::InvalidRequest);
    };
    webp_geometry(w, h)?;
    if expected.is_some_and(|dimensions| dimensions != (w, h)) {
        return Err(ImageError::InvalidRequest);
    }
    Ok(())
}
// A still image or an ANMF image group has exactly one image bitstream, with an
// optional preceding ALPH for VP8. ALPH has implicit dimensions: the already
// bounded matching canvas/frame supplies them, including compressed alpha.
fn webp_still_chunks(
    bytes: &[u8],
    expected: Option<(u32, u32)>,
    deadline: Instant,
) -> Result<(), ImageError> {
    let (mut position, mut image, mut alpha) = (0, false, false);
    while let Some((tag, payload)) = webp_chunk(bytes, &mut position)? {
        check_deadline(deadline)?;
        match tag {
            b"VP8 " | b"VP8L" => {
                if image || (alpha && tag != b"VP8 ") {
                    return Err(ImageError::InvalidRequest);
                }
                webp_frame(tag, payload, expected)?;
                image = true;
            }
            b"ALPH" => {
                if alpha || image || payload.is_empty() {
                    return Err(ImageError::InvalidRequest);
                }
                alpha = true;
            }
            b"VP8X" | b"ANIM" | b"ANMF" => return Err(ImageError::InvalidRequest),
            _ => {
                // ANMF's ALPH branch decodes the immediately following chunk
                // as VP8 without checking its tag. Do not let unknown bytes
                // become an unchecked alternative bitstream at that boundary.
                if alpha && !image {
                    return Err(ImageError::InvalidRequest);
                }
            }
        }
    }
    if !image {
        return Err(ImageError::InvalidRequest);
    }
    Ok(())
}
fn webp_safety_preflight(bytes: &[u8], deadline: Instant) -> Result<(), ImageError> {
    let header = bytes.get(..12).ok_or(ImageError::InvalidRequest)?;
    if &header[..4] != b"RIFF" || &header[8..] != b"WEBP" {
        return Err(ImageError::InvalidRequest);
    }
    let size = usize::try_from(webp_word(&header[4..])?).map_err(|_| ImageError::Limit)?;
    if size.checked_add(8) != Some(bytes.len()) {
        return Err(ImageError::InvalidRequest);
    }
    let chunks = &bytes[12..];
    let mut position = 0;
    let (first, extended) = webp_chunk(chunks, &mut position)?.ok_or(ImageError::InvalidRequest)?;
    if first == b"VP8 " || first == b"VP8L" {
        return webp_still_chunks(chunks, None, deadline);
    }
    if first != b"VP8X" || extended.len() != 10 {
        return Err(ImageError::InvalidRequest);
    }
    let canvas = (
        webp_u24(&extended[4..7])? + 1,
        webp_u24(&extended[7..10])? + 1,
    );
    webp_geometry(canvas.0, canvas.1)?;
    if extended[0] & 2 == 0 {
        return webp_still_chunks(&chunks[position..], Some(canvas), deadline);
    }
    let (mut animation, mut frame) = (false, false);
    while let Some((tag, payload)) = webp_chunk(chunks, &mut position)? {
        check_deadline(deadline)?;
        match tag {
            b"ANIM" => {
                if animation || frame || payload.len() != 6 {
                    return Err(ImageError::InvalidRequest);
                }
                animation = true;
            }
            b"ANMF" => {
                let header = payload.get(..16).ok_or(ImageError::InvalidRequest)?;
                if !animation || header[15] & 0xfc != 0 {
                    return Err(ImageError::InvalidRequest);
                }
                let x = webp_u24(&header[..3])?
                    .checked_mul(2)
                    .ok_or(ImageError::Limit)?;
                let y = webp_u24(&header[3..6])?
                    .checked_mul(2)
                    .ok_or(ImageError::Limit)?;
                let w = webp_u24(&header[6..9])? + 1;
                let h = webp_u24(&header[9..12])? + 1;
                webp_geometry(w, h)?;
                if x.checked_add(w).is_none_or(|end| end > canvas.0)
                    || y.checked_add(h).is_none_or(|end| end > canvas.1)
                {
                    return Err(ImageError::InvalidRequest);
                }
                webp_still_chunks(&payload[16..], Some((w, h)), deadline)?;
                frame = true;
            }
            b"VP8X" | b"VP8 " | b"VP8L" | b"ALPH" => return Err(ImageError::InvalidRequest),
            _ => {}
        }
    }
    if !frame {
        return Err(ImageError::InvalidRequest);
    }
    Ok(())
}

fn decode_media(data: &str, format: ImageFormat, deadline: Instant) -> Result<(), ImageError> {
    check_deadline(deadline)?;
    let size = base64_size(data)?;
    let mut compressed = Vec::new();
    compressed
        .try_reserve_exact(size)
        .map_err(|_| ImageError::Limit)?;
    // decode_vec can reserve padding space; use an exact pre-sized output slice.
    compressed.resize(size, 0);
    let n = STANDARD
        .decode_slice(data, &mut compressed)
        .map_err(|_| ImageError::InvalidRequest)?;
    if n != size {
        return Err(ImageError::InvalidRequest);
    }
    check_deadline(deadline)?;
    if image::guess_format(&compressed).map_err(|_| ImageError::InvalidRequest)? != format {
        return Err(ImageError::InvalidRequest);
    }
    if format == ImageFormat::WebP {
        webp_safety_preflight(&compressed, deadline)?;
    }
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(AXIS_LIMIT);
    limits.max_image_height = Some(AXIS_LIMIT);
    // Best effort codec-internal allocation limit, NOT a hard RSS guarantee.
    limits.max_alloc = Some(RASTER_LIMIT as u64);
    let mut reader = ImageReader::with_format(Cursor::new(&compressed), format);
    reader.limits(limits);
    #[cfg(test)]
    DECODER_ENTRY_OBSERVER.with(|hook| {
        if let Some(hook) = hook.get() {
            hook();
        }
    });
    let decoder = reader.into_decoder().map_err(decoder_error)?;
    let (w, h) = decoder.dimensions();
    let pixels = u64::from(w)
        .checked_mul(u64::from(h))
        .ok_or(ImageError::Limit)?;
    let total = decoder.total_bytes();
    if w == 0
        || h == 0
        || w > AXIS_LIMIT
        || h > AXIS_LIMIT
        || pixels > PIXEL_LIMIT
        || total == 0
        || total > RASTER_LIMIT as u64
    {
        return Err(ImageError::Limit);
    }
    check_deadline(deadline)?;
    let size = usize::try_from(total).map_err(|_| ImageError::Limit)?;
    let mut raster = Vec::new();
    raster
        .try_reserve_exact(size)
        .map_err(|_| ImageError::Limit)?;
    raster.resize(size, 0);
    decoder.read_image(&mut raster).map_err(decoder_error)?;
    #[cfg(test)]
    DECODE_OBSERVER.with(|hook| {
        if let Some(hook) = hook.get() {
            hook();
        }
    });
    check_deadline(deadline)?;
    drop(raster);
    drop(compressed);
    check_deadline(deadline)
}

async fn validate_media(
    data: &str,
    format: ImageFormat,
    deadline: Instant,
) -> Result<(), ImageError> {
    decode_media(data, format, deadline)?;
    tokio::task::yield_now().await;
    check_deadline(deadline)
}

fn decoder_error(error: image::ImageError) -> ImageError {
    if matches!(error, image::ImageError::Limits(_)) {
        ImageError::Limit
    } else {
        ImageError::InvalidRequest
    }
}
#[cfg(test)]
pub(crate) fn set_decode_observer(hook: Option<fn()>) {
    DECODE_OBSERVER.with(|h| h.set(hook));
}
#[cfg(test)]
pub(crate) fn set_decoder_entry_observer(hook: Option<fn()>) {
    DECODER_ENTRY_OBSERVER.with(|h| h.set(hook));
}
#[cfg(test)]
thread_local! {
    static DECODER_ENTRY_OBSERVER: std::cell::Cell<Option<fn()>> = const { std::cell::Cell::new(None) };
    static DECODE_OBSERVER: std::cell::Cell<Option<fn()>> = const { std::cell::Cell::new(None) };
}

/// Reserve overlapping logical payload before typed allocation. Every outside-
/// string byte costs 256 bytes of potential node/vector/enum overhead; depth is
/// capped at 32. Raw/typed/serialized copies cost 3*wire. Scratch reserves two
/// compressed image copies (JPEG wrapper) plus one raster (128MiB). Response
/// parsing additionally reserves 2*maximum request wire for still-owned outbound
/// HTTP representations. Excludes allocator, TLS and decoder internal buffers.
struct JsonCost {
    wire: usize,
    logical: usize,
    limit: usize,
    string: bool,
    escaped: bool,
    depth: usize,
}
impl JsonCost {
    fn new(response: bool) -> Self {
        Self {
            wire: 0,
            logical: 2 * FILE_LIMIT + RASTER_LIMIT + if response { 2 * REQUEST_LIMIT } else { 0 },
            limit: if response {
                RESPONSE_LIMIT
            } else {
                REQUEST_LIMIT
            },
            string: false,
            escaped: false,
            depth: 0,
        }
    }
    fn feed(&mut self, raw: &[u8]) -> Result<(), ImageError> {
        self.wire = checked_add(self.wire, raw.len(), self.limit)?;
        let copies = raw.len().checked_mul(3).ok_or(ImageError::Limit)?;
        self.logical = checked_add(self.logical, copies, WORKING_LIMIT)?;
        for &b in raw {
            if self.string {
                if self.escaped {
                    self.escaped = false;
                } else if b == b'\\' {
                    self.escaped = true;
                } else if b == b'"' {
                    self.string = false;
                }
            } else if !b.is_ascii_whitespace() {
                self.logical = checked_add(self.logical, 256, WORKING_LIMIT)?;
                match b {
                    b'"' => self.string = true,
                    b'[' | b'{' => {
                        self.depth = checked_add(self.depth, 1, 32)
                            .map_err(|_| ImageError::InvalidRequest)?;
                    }
                    b']' | b'}' => {
                        self.depth = self
                            .depth
                            .checked_sub(1)
                            .ok_or(ImageError::InvalidRequest)?;
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }
}
pub(crate) fn preflight_json(raw: &[u8], response: bool) -> Result<(), ImageError> {
    JsonCost::new(response).feed(raw)
}

/// A dry serialization charges logical overlap/nodes before reserving the exact
/// wire size. The second pass checks each copy against that precharged size.
pub(crate) fn bounded_json<T: Serialize>(value: &T, response: bool) -> Result<Vec<u8>, ImageError> {
    struct Counter(JsonCost);
    impl io::Write for Counter {
        fn write(&mut self, b: &[u8]) -> io::Result<usize> {
            self.0
                .feed(b)
                .map_err(|_| io::Error::other("image limit"))?;
            Ok(b.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    let mut counter = Counter(JsonCost::new(response));
    serde_json::to_writer(&mut counter, value).map_err(|_| ImageError::Limit)?;
    let size = counter.0.wire;
    struct Writer {
        bytes: Vec<u8>,
        limit: usize,
    }
    impl io::Write for Writer {
        fn write(&mut self, b: &[u8]) -> io::Result<usize> {
            checked_add(self.bytes.len(), b.len(), self.limit)
                .map_err(|_| io::Error::other("image limit"))?;
            self.bytes.extend_from_slice(b);
            Ok(b.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(size)
        .map_err(|_| ImageError::Limit)?;
    let mut writer = Writer { bytes, limit: size };
    serde_json::to_writer(&mut writer, value).map_err(|_| ImageError::Limit)?;
    Ok(writer.bytes)
}

// Independent small RIFF/header fixtures. No production serializer or safety
// helper constructs expected input, and the decoder-entry sentinel prevents
// allocating raster work if any rejected case accidentally reaches the decoder.
#[cfg(test)]
pub(crate) mod webp_guard_test_data {
    fn chunk(tag: &[u8; 4], bytes: &[u8]) -> Vec<u8> {
        let mut out = tag.to_vec();
        out.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
        out.extend_from_slice(bytes);
        if bytes.len() & 1 != 0 {
            out.push(0);
        }
        out
    }
    fn riff(chunks: &[Vec<u8>]) -> Vec<u8> {
        let mut body = b"WEBP".to_vec();
        for c in chunks {
            body.extend(c);
        }
        let mut out = b"RIFF".to_vec();
        out.extend_from_slice(&(body.len() as u32).to_le_bytes());
        out.extend(body);
        out
    }
    fn extended(w: u32, h: u32, flags: u8) -> Vec<u8> {
        let mut b = vec![flags, 0, 0, 0];
        b.extend_from_slice(&(w - 1).to_le_bytes()[..3]);
        b.extend_from_slice(&(h - 1).to_le_bytes()[..3]);
        chunk(b"VP8X", &b)
    }
    fn vp8(w: u16, h: u16) -> Vec<u8> {
        let mut b = vec![0x10, 0, 0, 0x9d, 1, 0x2a];
        b.extend_from_slice(&w.to_le_bytes());
        b.extend_from_slice(&h.to_le_bytes());
        chunk(b"VP8 ", &b)
    }
    fn vp8l(w: u32, h: u32) -> Vec<u8> {
        let mut b = vec![0x2f];
        b.extend_from_slice(&((w - 1) | ((h - 1) << 14)).to_le_bytes());
        chunk(b"VP8L", &b)
    }
    fn anmf(w: u32, h: u32, chunks: &[Vec<u8>]) -> Vec<u8> {
        let mut b = vec![0; 6];
        b.extend_from_slice(&(w - 1).to_le_bytes()[..3]);
        b.extend_from_slice(&(h - 1).to_le_bytes()[..3]);
        b.extend_from_slice(&[0, 0, 0, 2]);
        for c in chunks {
            b.extend(c);
        }
        chunk(b"ANMF", &b)
    }
    pub(crate) fn invalid_cases() -> Vec<Vec<u8>> {
        let alpha = chunk(b"ALPH", &[0, 255, 255, 255, 255]);
        let animation = chunk(b"ANIM", &[0; 6]);
        let mut cases = vec![
            include_bytes!("../../tests/fixtures/codex_images/inner-vp8-bomb.webp").to_vec(),
            riff(&[extended(2, 2, 0), vp8(3, 2)]),
            riff(&[extended(2, 2, 0), vp8l(16384, 16384)]),
            riff(&[vp8(16383, 16383)]),
            riff(&[vp8l(16384, 16384)]),
            riff(&[extended(2, 2, 0), vp8l(3, 2)]),
            riff(&[
                extended(2, 2, 2),
                animation.clone(),
                anmf(2, 2, &[vp8(16383, 16383)]),
            ]),
            riff(&[
                extended(2, 2, 0x12),
                animation.clone(),
                anmf(2, 2, &[alpha.clone(), vp8(16383, 16383)]),
            ]),
            riff(&[
                extended(2, 2, 2),
                animation.clone(),
                anmf(2, 2, &[vp8l(16384, 16384)]),
            ]),
            riff(&[
                extended(2, 2, 2),
                animation.clone(),
                anmf(3, 2, &[vp8(3, 2)]),
            ]),
            riff(&[
                extended(2, 2, 2),
                animation.clone(),
                anmf(16384, 16384, &[vp8l(16384, 16384)]),
            ]),
            riff(&[
                extended(2, 2, 0x12),
                animation.clone(),
                anmf(
                    2,
                    2,
                    &[
                        alpha.clone(),
                        chunk(
                            b"JUNK",
                            &[0x10, 0, 0, 0x9d, 1, 0x2a, 0xff, 0x3f, 0xff, 0x3f],
                        ),
                        vp8(2, 2),
                    ],
                ),
            ]),
            riff(&[
                extended(2, 2, 0),
                animation.clone(),
                anmf(2, 2, &[vp8(2, 2)]),
            ]),
            riff(&[
                extended(2, 2, 2),
                animation.clone(),
                vp8(16383, 16383),
                anmf(2, 2, &[vp8(2, 2)]),
            ]),
            riff(&[
                extended(2, 2, 2),
                animation.clone(),
                anmf(2, 2, &[vp8(2, 2)]),
                anmf(2, 2, &[vp8(16383, 16383)]),
            ]),
            riff(&[extended(2, 2, 0), vp8(2, 2), vp8l(2, 2)]),
            riff(&[extended(2, 2, 0x10), alpha.clone(), alpha, vp8(2, 2)]),
            riff(&[extended(2, 2, 0), extended(2, 2, 0), vp8(2, 2)]),
            riff(&[extended(2, 2, 0), b"VP8 \xff\xff\xff\xff".to_vec()]),
        ];
        let mut bad = include_bytes!("../../tests/fixtures/codex_images/red-alpha.webp").to_vec();
        bad[4..8].copy_from_slice(&u32::MAX.to_le_bytes());
        cases.push(bad);
        let mut trailing = include_bytes!("../../tests/fixtures/codex_images/red.webp").to_vec();
        trailing.push(0);
        cases.push(trailing);
        let mut padding = include_bytes!("../../tests/fixtures/codex_images/red.webp").to_vec();
        *padding.last_mut().unwrap() = 1;
        cases.push(padding);
        let mut short = vp8(2, 2);
        short.truncate(17);
        cases.push(riff(&[extended(2, 2, 0), short]));
        let mut offset = riff(&[extended(2, 2, 2), animation, anmf(2, 2, &[vp8(2, 2)])]);
        let frame = offset.windows(4).position(|b| b == b"ANMF").unwrap();
        offset[frame + 8..frame + 11].copy_from_slice(&[0xff; 3]);
        cases.push(offset);
        cases
    }
    pub(crate) fn valid_cases() -> [&'static [u8]; 6] {
        [
            include_bytes!("../../tests/fixtures/codex_images/red-lossy.webp"),
            include_bytes!("../../tests/fixtures/codex_images/red-extended.webp"),
            include_bytes!("../../tests/fixtures/codex_images/red-alpha.webp"),
            include_bytes!("../../tests/fixtures/codex_images/red-animation.webp"),
            include_bytes!("../../tests/fixtures/codex_images/red-compressed-alpha.webp"),
            include_bytes!("../../tests/fixtures/codex_images/red-alpha-lossless.webp"),
        ]
    }
    pub(crate) fn invalid_entropy() -> Vec<u8> {
        riff(&[extended(2, 2, 0), vp8(2, 2)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn deadline() -> Instant {
        Instant::now() + std::time::Duration::from_secs(180)
    }
    #[test]
    fn checked_arithmetic_and_base64_preallocation_boundaries() {
        assert_eq!(
            checked_add(usize::MAX, 1, usize::MAX),
            Err(ImageError::Limit)
        );
        assert_eq!(
            checked_add(FILE_LIMIT, 1, FILE_LIMIT),
            Err(ImageError::Limit)
        );
        assert_eq!(
            checked_add(FILE_LIMIT, FILE_LIMIT, INPUT_LIMIT),
            Ok(INPUT_LIMIT)
        );
        assert_eq!(
            checked_add(FILE_LIMIT, 1, OUTPUT_LIMIT),
            Err(ImageError::Limit)
        );
        assert_eq!(base64_size("Zg=="), Ok(1));
        assert_eq!(base64_size("Zm8="), Ok(2));
        assert_eq!(base64_size("Zm9v"), Ok(3));
        for s in ["", "====", "Zg=", "Zg===", "Z g=", "A===", "AA=A", "____"] {
            assert_eq!(base64_size(s), Err(ImageError::InvalidRequest));
        }
        // Noncanonical padding bits cannot bypass the actual base64 decoder.
        assert_eq!(
            decode_media("Zh==", ImageFormat::Png, deadline()),
            Err(ImageError::InvalidRequest)
        );
        assert_eq!(
            base64_size(&"A".repeat(FILE_LIMIT.div_ceil(3) * 4 + 4)),
            Err(ImageError::Limit)
        );
        let mut cost = JsonCost::new(false);
        cost.wire = usize::MAX;
        assert_eq!(cost.feed(b"x"), Err(ImageError::Limit));
        let mut cost = JsonCost::new(true);
        cost.logical = usize::MAX;
        assert_eq!(cost.feed(b"x"), Err(ImageError::Limit));
    }
    #[test]
    fn preparse_and_preserialize_charge_depth_dense_nodes_and_copies() {
        assert!(preflight_json(br#"{"quoted":"[[[\\\"still a string"}"#, false).is_ok());
        assert_eq!(
            preflight_json(&[b'['; 33], false),
            Err(ImageError::InvalidRequest)
        );
        assert_eq!(
            preflight_json(&vec![b'0'; 1_600_000], false),
            Err(ImageError::Limit)
        );
        assert_eq!(
            bounded_json(&vec![0; 900_000], false),
            Err(ImageError::Limit)
        );
        let mut cost = JsonCost::new(false);
        // Exact request wire boundary consumes a bounded reserve before parsing.
        assert!(cost.feed(&vec![b' '; REQUEST_LIMIT]).is_ok());
        assert_eq!(cost.feed(b" "), Err(ImageError::Limit));
        let mut cost = JsonCost::new(true);
        assert!(cost.feed(&vec![b' '; RESPONSE_LIMIT]).is_ok());
        assert_eq!(cost.feed(b" "), Err(ImageError::Limit));
    }
    #[tokio::test]
    async fn real_decode_all_formats_bombs_and_no_late_success_at_decode_boundary() {
        for (bytes, format) in [
            (
                include_bytes!("../../tests/fixtures/codex_images/red.png").as_slice(),
                ImageFormat::Png,
            ),
            (
                include_bytes!("../../tests/fixtures/codex_images/red.jpg").as_slice(),
                ImageFormat::Jpeg,
            ),
            (
                include_bytes!("../../tests/fixtures/codex_images/red.webp").as_slice(),
                ImageFormat::WebP,
            ),
        ] {
            validate_media(&STANDARD.encode(bytes), format, deadline())
                .await
                .unwrap();
            assert_eq!(
                validate_media(&STANDARD.encode(bytes), format, Instant::now()).await,
                Err(ImageError::Deadline)
            );
        }
        for bytes in [
            include_bytes!("../../tests/fixtures/codex_images/pixel-bomb.png"),
            include_bytes!("../../tests/fixtures/codex_images/axis-bomb.png"),
            include_bytes!("../../tests/fixtures/codex_images/raster-bomb.png"),
        ] {
            assert_eq!(
                validate_media(&STANDARD.encode(bytes), ImageFormat::Png, deadline()).await,
                Err(ImageError::Limit)
            );
        }
        set_decode_observer(Some(|| {
            std::thread::sleep(std::time::Duration::from_millis(30))
        }));
        let result = validate_media(
            &STANDARD.encode(include_bytes!("../../tests/fixtures/codex_images/red.png")),
            ImageFormat::Png,
            Instant::now() + std::time::Duration::from_millis(10),
        )
        .await;
        set_decode_observer(None);
        assert_eq!(result, Err(ImageError::Deadline));
    }
    #[test]
    fn img_r1_inner_geometry_rejected_before_mature_decoder_entry() {
        let data = STANDARD.encode(include_bytes!(
            "../../tests/fixtures/codex_images/inner-vp8-bomb.webp"
        ));
        set_decoder_entry_observer(Some(|| {
            panic!("IMG-R1 reached allocating mature decoder boundary")
        }));
        let result =
            std::panic::catch_unwind(|| decode_media(&data, ImageFormat::WebP, deadline()));
        set_decoder_entry_observer(None);
        assert!(
            matches!(
                result,
                Ok(Err(ImageError::InvalidRequest | ImageError::Limit))
            ),
            "IMG-R1 must reject before allocating decoder, observed {result:?}"
        );
    }

    #[test]
    fn img_r1_related_container_and_nested_geometry_fail_before_decoder() {
        let mut position = usize::MAX;
        assert_eq!(
            webp_chunk(&[], &mut position),
            Err(ImageError::InvalidRequest)
        );
        for bytes in webp_guard_test_data::invalid_cases() {
            set_decoder_entry_observer(Some(|| {
                panic!("IMG-R1 invalid header reached mature decoder")
            }));
            let result = std::panic::catch_unwind(|| {
                decode_media(&STANDARD.encode(&bytes), ImageFormat::WebP, deadline())
            });
            set_decoder_entry_observer(None);
            assert!(
                matches!(
                    result,
                    Ok(Err(ImageError::InvalidRequest | ImageError::Limit))
                ),
                "{bytes:?}: {result:?}"
            );
        }
        for bytes in webp_guard_test_data::valid_cases() {
            for end in 0..bytes.len() {
                set_decoder_entry_observer(Some(|| {
                    panic!("IMG-R1 truncated container reached mature decoder")
                }));
                let result = std::panic::catch_unwind(|| {
                    decode_media(
                        &STANDARD.encode(&bytes[..end]),
                        ImageFormat::WebP,
                        deadline(),
                    )
                });
                set_decoder_entry_observer(None);
                assert!(
                    matches!(
                        result,
                        Ok(Err(ImageError::InvalidRequest | ImageError::Limit))
                    ),
                    "truncation{end}: {result:?}"
                );
            }
        }
    }
    #[test]
    fn img_r1_valid_webp_forms_still_require_actual_decoder() {
        thread_local! {static ENTERED:std::cell::Cell<usize>=const{std::cell::Cell::new(0)};}
        set_decoder_entry_observer(Some(|| ENTERED.with(|c| c.set(c.get() + 1))));
        for bytes in webp_guard_test_data::valid_cases() {
            decode_media(&STANDARD.encode(bytes), ImageFormat::WebP, deadline()).unwrap();
        }
        assert_eq!(
            decode_media(
                &STANDARD.encode(webp_guard_test_data::invalid_entropy()),
                ImageFormat::WebP,
                deadline()
            ),
            Err(ImageError::InvalidRequest)
        );
        set_decoder_entry_observer(None);
        assert_eq!(
            ENTERED.with(|c| c.get()),
            7,
            "header admission must not replace genuine pixel decoding"
        );
    }
}
