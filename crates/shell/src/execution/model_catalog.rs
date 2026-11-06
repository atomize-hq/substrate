//! Opt-in descriptive metadata. This module never resolves or changes an agent selection.

use anyhow::{bail, ensure, Context, Result};
use serde::de::{DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fmt;
use std::io::Read;
use std::net::ToSocketAddrs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use super::agent_runtime::host_session_authority::trusted_fs::{
    EntryKind, TrustedAuthorityRoot, TrustedDirectory,
};
use chrono::{DateTime, Utc};
use std::fs::{File, Metadata};
use std::os::fd::{AsRawFd, FromRawFd};
use std::os::unix::fs::MetadataExt;

const ORIGIN: &str = "https://models.dev/catalog.json?type=chat";
const BODY_LIMIT: usize = 32 * 1024 * 1024;
const TEXT_LIMIT: usize = 4096;
const PROVIDER_LIMIT: usize = 4096;
const MODEL_LIMIT: usize = 131_072;
const DEADLINE: Duration = Duration::from_secs(15);
const SNAPSHOT: &str = "snapshot.json";
const FAILURE: &str = "refresh-failure.json";
const STAGE: &str = "publication.stage";
const FAILURE_LIMIT: usize = 4096;

// Deserialize every value, including ignored descriptive fields, with duplicate,
// depth and decoded text bounds. serde_json::Value alone silently accepts duplicates.
struct StrictValue(usize);

impl<'de> DeserializeSeed<'de> for StrictValue {
    type Value = Value;

    fn deserialize<D: serde::Deserializer<'de>>(self, deserializer: D) -> Result<Value, D::Error> {
        if self.0 > 32 {
            return Err(serde::de::Error::custom("catalog nesting exceeds 32"));
        }
        deserializer.deserialize_any(self)
    }
}

impl<'de> Visitor<'de> for StrictValue {
    type Value = Value;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("bounded JSON without duplicate fields")
    }

    fn visit_map<A: MapAccess<'de>>(self, mut input: A) -> Result<Value, A::Error> {
        let mut output = Map::new();
        while let Some(key) = input.next_key::<String>()? {
            if key.len() > TEXT_LIMIT || output.contains_key(&key) {
                return Err(serde::de::Error::custom(
                    "oversize or duplicate catalog key",
                ));
            }
            output.insert(key, input.next_value_seed(StrictValue(self.0 + 1))?);
        }
        Ok(Value::Object(output))
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut input: A) -> Result<Value, A::Error> {
        let mut output = Vec::new();
        while let Some(value) = input.next_element_seed(StrictValue(self.0 + 1))? {
            output.push(value);
        }
        Ok(Value::Array(output))
    }

    fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Value, E> {
        if value.len() > TEXT_LIMIT {
            return Err(E::custom("catalog text exceeds 4096 bytes"));
        }
        Ok(Value::String(value.to_owned()))
    }

    fn visit_string<E: serde::de::Error>(self, value: String) -> Result<Value, E> {
        self.visit_str(&value)
    }

    fn visit_bool<E: serde::de::Error>(self, value: bool) -> Result<Value, E> {
        Ok(Value::Bool(value))
    }

    fn visit_u64<E: serde::de::Error>(self, value: u64) -> Result<Value, E> {
        Ok(Value::Number(value.into()))
    }

    fn visit_i64<E: serde::de::Error>(self, value: i64) -> Result<Value, E> {
        Ok(Value::Number(value.into()))
    }

    fn visit_f64<E: serde::de::Error>(self, value: f64) -> Result<Value, E> {
        serde_json::Number::from_f64(value)
            .map(Value::Number)
            .ok_or_else(|| E::custom("nonfinite catalog number"))
    }

    fn visit_unit<E: serde::de::Error>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }
}

fn strict_json(bytes: &[u8]) -> Result<Value> {
    ensure!(bytes.len() <= BODY_LIMIT, "catalog exceeds 32 MiB");
    let mut decoder = serde_json::Deserializer::from_slice(bytes);
    let value = StrictValue(1).deserialize(&mut decoder)?;
    decoder.end()?;
    Ok(value)
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Model {
    id: String,
    name: String,
    description: String,
    #[serde(rename = "type", default = "chat_type")]
    kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    canonical_model_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    status: Option<String>,
}

fn chat_type() -> String {
    "chat".to_owned()
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Provider {
    id: String,
    name: String,
    models: BTreeMap<String, Model>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Catalog {
    models: BTreeMap<String, Model>,
    providers: BTreeMap<String, Provider>,
}

fn identity(value: &str) -> Result<()> {
    ensure!(
        !value.is_empty()
            && value.len() <= TEXT_LIMIT
            && !value.chars().any(|c| c.is_control() || c.is_whitespace()),
        "invalid catalog identity"
    );
    Ok(())
}

impl Catalog {
    fn parse(bytes: &[u8]) -> Result<Self> {
        let value = strict_json(bytes)?;
        validate_upstream_shape(&value)?;
        let catalog: Self = serde_json::from_value(value)?;
        catalog.validate()?;
        Ok(catalog)
    }

    fn validate(&self) -> Result<()> {
        ensure!(self.providers.len() <= PROVIDER_LIMIT, "too many providers");
        ensure!(
            self.models.len() <= MODEL_LIMIT,
            "too many canonical models"
        );
        let mut offerings = 0usize;
        for (key, model) in &self.models {
            model.validate(key)?;
            ensure!(
                model.canonical_model_id.is_none(),
                "canonical model cannot be an offering"
            );
        }
        for (key, provider) in &self.providers {
            identity(key)?;
            ensure!(
                key == &provider.id && !provider.name.is_empty(),
                "provider identity mismatch"
            );
            ensure!(!provider.models.is_empty(), "provider has no offerings");
            offerings = offerings
                .checked_add(provider.models.len())
                .context("offering count overflow")?;
            ensure!(offerings <= MODEL_LIMIT, "too many offerings");
            for (key, model) in &provider.models {
                model.validate(key)?;
                if let Some(canonical) = &model.canonical_model_id {
                    identity(canonical)?;
                    ensure!(
                        self.models.contains_key(canonical),
                        "dangling canonical model attribution"
                    );
                }
            }
        }
        Ok(())
    }
}

fn validate_upstream_shape(value: &Value) -> Result<()> {
    let root = value.as_object().context("catalog must be an object")?;
    let models = root
        .get("models")
        .and_then(Value::as_object)
        .context("missing model map")?;
    let providers = root
        .get("providers")
        .and_then(Value::as_object)
        .context("missing provider map")?;
    ensure!(
        models.len() <= MODEL_LIMIT && providers.len() <= PROVIDER_LIMIT,
        "catalog map count exceeds bound"
    );
    for model in models.values() {
        validate_model_shape(model, false)?;
    }
    let mut count = 0usize;
    for provider in providers.values() {
        let provider = provider.as_object().context("provider must be an object")?;
        for field in ["id", "name", "npm", "doc"] {
            ensure!(
                provider
                    .get(field)
                    .and_then(Value::as_str)
                    .is_some_and(|s| !s.is_empty()),
                "missing/string provider field {field}"
            );
        }
        string_array(provider.get("env").context("missing provider env")?)?;
        if let Some(api) = provider.get("api") {
            ensure!(api.is_string(), "provider api must be a string");
        }
        let offerings = provider
            .get("models")
            .and_then(Value::as_object)
            .context("missing offering map")?;
        count += offerings.len();
        ensure!(count <= MODEL_LIMIT, "too many offerings");
        for model in offerings.values() {
            validate_model_shape(model, true)?;
        }
    }
    Ok(())
}

fn string_array(value: &Value) -> Result<()> {
    ensure!(
        value
            .as_array()
            .is_some_and(|values| values.iter().all(Value::is_string)),
        "expected string array"
    );
    Ok(())
}

fn validate_model_shape(value: &Value, offering: bool) -> Result<()> {
    let model = value.as_object().context("model must be an object")?;
    for field in ["id", "name", "description"] {
        ensure!(
            model
                .get(field)
                .and_then(Value::as_str)
                .is_some_and(|s| !s.is_empty()),
            "missing/string model field {field}"
        );
    }
    for field in [
        "canonical_model_id",
        "type",
        "family",
        "status",
        "knowledge",
        "release_date",
        "last_updated",
        "license",
    ] {
        if let Some(value) = model.get(field) {
            ensure!(value.is_string(), "model field {field} must be a string");
        }
    }
    for field in [
        "attachment",
        "reasoning",
        "tool_call",
        "open_weights",
        "structured_output",
        "temperature",
    ] {
        if let Some(value) = model.get(field) {
            ensure!(value.is_boolean(), "model field {field} must be boolean");
        } else if offering
            && ["attachment", "reasoning", "tool_call", "open_weights"].contains(&field)
        {
            bail!("missing offering field {field}");
        }
    }
    for field in ["release_date", "last_updated"] {
        ensure!(
            !offering || model.contains_key(field),
            "missing offering date {field}"
        );
    }
    if let Some(modalities) = model.get("modalities") {
        let modalities = modalities.as_object().context("invalid modalities")?;
        string_array(
            modalities
                .get("input")
                .context("missing input modalities")?,
        )?;
        string_array(
            modalities
                .get("output")
                .context("missing output modalities")?,
        )?;
    } else {
        ensure!(!offering, "missing offering modalities");
    }
    if let Some(limit) = model.get("limit") {
        let limit = limit.as_object().context("invalid token limits")?;
        for field in ["context", "input", "output"] {
            if let Some(value) = limit.get(field) {
                ensure!(
                    value.as_f64().is_some_and(|n| n.is_finite() && n >= 0.0),
                    "invalid numeric token limit"
                );
            } else {
                ensure!(
                    !offering || field == "input",
                    "missing offering token limit"
                );
            }
        }
    } else {
        ensure!(!offering, "missing offering token limits");
    }
    Ok(())
}

impl Model {
    fn validate(&self, key: &str) -> Result<()> {
        identity(key)?;
        ensure!(key == self.id, "model map key/id mismatch");
        ensure!(
            !self.name.is_empty() && !self.description.is_empty(),
            "empty model description/name"
        );
        ensure!(self.kind == "chat", "catalog violates chat type filter");
        Ok(())
    }
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Snapshot {
    version: u32,
    origin: String,
    type_filter: String,
    normalizer: String,
    upstream_schema: String,
    fetched_utc: DateTime<Utc>,
    http_status: u16,
    decoded_bytes: usize,
    source_sha256: String,
    normalized_sha256: String,
    etag: Option<String>,
    last_modified: Option<String>,
    catalog: Catalog,
}

impl Snapshot {
    fn validate(&self) -> Result<()> {
        ensure!(
            self.version == 1
                && self.origin == ORIGIN
                && self.type_filter == "chat"
                && self.normalizer == "substrate-models-dev-v1"
                && self.upstream_schema == "models.dev/catalog-v1",
            "unknown catalog provenance"
        );
        ensure!(
            self.source_sha256.len() == 64
                && self.source_sha256.bytes().all(|b| b.is_ascii_hexdigit()),
            "invalid source digest"
        );
        ensure!(
            self.http_status == 200 && self.decoded_bytes <= BODY_LIMIT,
            "invalid source response bounds/status"
        );
        self.catalog.validate()?;
        ensure!(
            self.normalized_sha256 == digest(&serde_json::to_vec(&self.catalog)?),
            "catalog content digest mismatch"
        );
        Ok(())
    }

    fn decode(bytes: &[u8]) -> Result<Self> {
        let snapshot: Self = serde_json::from_value(strict_json(bytes)?)?;
        snapshot.validate()?;
        Ok(snapshot)
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct RefreshFailure {
    version: u32,
    attempted_utc: DateTime<Utc>,
    base_snapshot_sha256: Option<String>,
    reason: String,
}

/// The response is never interpreted as executable endpoints, packages or headers.
fn fetch(origin: &str, timeout: Duration) -> Result<Snapshot> {
    let deadline = Instant::now() + timeout;
    // ureq's default resolver cannot interrupt getaddrinfo. Bound its wait here;
    // the resolver thread can only resolve a name, never continue the HTTP request.
    let agent = ureq::AgentBuilder::new()
        .redirects(0)
        .try_proxy_from_env(false)
        .max_idle_connections(0)
        .timeout(timeout)
        .resolver(move |address: &str| {
            let address = address.to_owned();
            let (sender, receiver) = std::sync::mpsc::sync_channel(1);
            std::thread::spawn(move || {
                let result = address
                    .to_socket_addrs()
                    .map(|addresses| addresses.collect::<Vec<_>>());
                let _ = sender.send(result);
            });
            receiver
                .recv_timeout(deadline.saturating_duration_since(Instant::now()))
                .map_err(|_| {
                    std::io::Error::new(std::io::ErrorKind::TimedOut, "catalog DNS deadline")
                })?
        })
        .build();
    let response = agent
        .get(origin)
        .set("Accept", "application/json")
        .call()
        .context("catalog metadata request failed")?;
    ensure!(
        response.status() == 200,
        "catalog server refused refresh (status {})",
        response.status()
    );
    let validator = |name| -> Result<Option<String>> {
        let values = response.all(name);
        ensure!(values.len() <= 1, "duplicate server validator");
        values
            .first()
            .map(|value| {
                ensure!(
                    value.len() <= TEXT_LIMIT && !value.chars().any(char::is_control),
                    "invalid server validator"
                );
                Ok((*value).to_owned())
            })
            .transpose()
    };
    let etag = validator("ETag")?;
    let last_modified = validator("Last-Modified")?;
    let mut body = Vec::new();
    response
        .into_reader()
        .take((BODY_LIMIT + 1) as u64)
        .read_to_end(&mut body)
        .context("catalog response truncated, unreadable or timed out")?;
    ensure!(
        Instant::now() < deadline,
        "catalog refresh deadline exceeded"
    );
    let catalog = Catalog::parse(&body)?;
    ensure!(
        Instant::now() < deadline,
        "catalog refresh deadline exceeded during validation"
    );
    let normalized_sha256 = digest(&serde_json::to_vec(&catalog)?);
    Ok(Snapshot {
        version: 1,
        origin: ORIGIN.to_owned(),
        type_filter: "chat".to_owned(),
        normalizer: "substrate-models-dev-v1".to_owned(),
        upstream_schema: "models.dev/catalog-v1".to_owned(),
        fetched_utc: Utc::now(),
        http_status: 200,
        decoded_bytes: body.len(),
        source_sha256: digest(&body),
        normalized_sha256,
        etag,
        last_modified,
        catalog,
    })
}

struct Cache {
    root: TrustedAuthorityRoot,
    directory: TrustedDirectory,
    home: PathBuf,
    owner: u32,
    directory_identity: (u64, u64),
}

#[derive(Debug, Eq, PartialEq)]
struct Preimage {
    device: u64,
    inode: u64,
    size: u64,
    modified: (i64, i64),
    changed: (i64, i64),
    sha256: String,
}

impl Preimage {
    fn new(meta: &Metadata, bytes: &[u8]) -> Self {
        Self {
            device: meta.dev(),
            inode: meta.ino(),
            size: meta.len(),
            modified: (meta.mtime(), meta.mtime_nsec()),
            changed: (meta.ctime(), meta.ctime_nsec()),
            sha256: digest(bytes),
        }
    }
}

impl Cache {
    fn open(home: &Path, owner: u32, create: bool) -> Result<Option<Self>> {
        let root = TrustedAuthorityRoot::open_for_owner(home, owner)
            .context("catalog requires an accepted private product home")?;
        let directory = match root.directory().entry_kind("model-catalog")? {
            None if !create => return Ok(None),
            None => root.directory().create_directory("model-catalog")?,
            Some(EntryKind::Directory) => root.directory().open_directory("model-catalog")?,
            Some(_) => bail!("catalog directory is not a private physical directory"),
        };
        let meta = directory
            .borrow_fd()
            .try_clone_to_owned()
            .map(File::from)?
            .metadata()?;
        ensure!(
            meta.mode() & 0o7777 == 0o700,
            "catalog directory must have mode 0700"
        );
        no_acl(directory.borrow_fd().as_raw_fd())?;
        let cache = Self {
            root,
            directory,
            home: home.to_path_buf(),
            owner,
            directory_identity: (meta.dev(), meta.ino()),
        };
        cache.revalidate()?;
        Ok(Some(cache))
    }

    fn revalidate(&self) -> Result<()> {
        self.root.revalidate()?;
        // Reopen the requested home as well as the held physical root: a renamed
        // home must not redirect publication into an abandoned handle.
        let reopened = TrustedAuthorityRoot::open_for_owner(&self.home, self.owner)?;
        ensure!(
            reopened.identity() == self.root.identity(),
            "accepted catalog home was replaced"
        );
        let directory = reopened.directory().open_directory("model-catalog")?;
        no_acl(directory.borrow_fd().as_raw_fd())?;
        let meta = directory
            .borrow_fd()
            .try_clone_to_owned()
            .map(File::from)?
            .metadata()?;
        ensure!(
            (meta.dev(), meta.ino()) == self.directory_identity,
            "catalog directory was replaced"
        );
        Ok(())
    }

    fn read(&self, name: &str, limit: usize) -> Result<Option<(Vec<u8>, Preimage)>> {
        let component = std::ffi::CString::new(name)?;
        // SAFETY: held descriptor and the fixed single component remain live.
        // NONBLOCK prevents a hostile FIFO substitution from blocking a reader.
        let fd = unsafe {
            libc::openat(
                self.directory.borrow_fd().as_raw_fd(),
                component.as_ptr(),
                libc::O_RDONLY | libc::O_CLOEXEC | libc::O_NOFOLLOW | libc::O_NONBLOCK,
            )
        };
        if fd < 0 {
            let error = std::io::Error::last_os_error();
            if error.kind() == std::io::ErrorKind::NotFound {
                return Ok(None);
            }
            return Err(error).context("open catalog file without following symlinks");
        }
        // SAFETY: successful openat returned unique ownership of fd.
        let mut file = unsafe { File::from_raw_fd(fd) };
        let before = file.metadata()?;
        ensure!(
            before.is_file()
                && before.uid() == self.owner
                && before.mode() & 0o7777 == 0o600
                && before.nlink() == 1
                && before.dev() == self.directory_identity.0,
            "catalog file type/owner/mode/link/device mismatch"
        );
        ensure!(
            before.len() <= limit as u64,
            "catalog file exceeds storage bound"
        );
        no_acl(file.as_raw_fd())?;
        let mut bytes = Vec::new();
        (&mut file)
            .take((limit + 1) as u64)
            .read_to_end(&mut bytes)?;
        ensure!(bytes.len() <= limit, "catalog file exceeds read bound");
        let preimage = Preimage::new(&before, &bytes);
        ensure!(
            preimage == Preimage::new(&file.metadata()?, &bytes),
            "catalog file changed during read"
        );
        let mut stat = std::mem::MaybeUninit::<libc::stat>::uninit();
        // SAFETY: descriptor/name are live and stat points to writable storage.
        ensure!(
            unsafe {
                libc::fstatat(
                    self.directory.borrow_fd().as_raw_fd(),
                    component.as_ptr(),
                    stat.as_mut_ptr(),
                    libc::AT_SYMLINK_NOFOLLOW,
                )
            } == 0,
            "catalog file disappeared during read"
        );
        // SAFETY: successful fstatat initialized stat.
        let stat = unsafe { stat.assume_init() };
        ensure!(
            stat.st_dev == before.dev() && stat.st_ino == before.ino(),
            "catalog file name was replaced during read"
        );
        Ok(Some((bytes, preimage)))
    }

    fn check_preimage(&self, name: &str, limit: usize, expected: Option<&Preimage>) -> Result<()> {
        ensure!(
            self.read(name, limit)?
                .as_ref()
                .map(|(_, identity)| identity)
                == expected,
            "catalog publication preimage changed"
        );
        Ok(())
    }

    fn inventory(&self) -> Result<()> {
        // This directory is dedicated to the importer, never config or session data.
        // Refuse unknown entries rather than deleting or charging someone else's data.
        for entry in self.directory.entries()? {
            let limit = match entry.name.as_str() {
                SNAPSHOT | STAGE => BODY_LIMIT,
                FAILURE => FAILURE_LIMIT,
                _ => bail!("unknown entry in managed catalog storage"),
            };
            self.read(&entry.name, limit)?
                .context("catalog entry disappeared")?;
        }
        Ok(())
    }

    fn publish(
        &self,
        name: &str,
        bytes: &[u8],
        expected: Option<&Preimage>,
        limit: usize,
    ) -> Result<()> {
        ensure!(
            bytes.len() <= limit && bytes.len() <= BODY_LIMIT,
            "catalog publication exceeds storage bound"
        );
        ensure!(
            self.read(STAGE, BODY_LIMIT)?.is_none(),
            "catalog staging already exists"
        );
        self.revalidate()?;
        self.check_preimage(name, limit, expected)?;
        let mut staged = self.directory.create_file(STAGE)?;
        staged.write_all(bytes)?;
        staged.sync()?;
        self.revalidate()?;
        self.check_preimage(name, limit, expected)?;
        self.directory
            .rename_replace(STAGE, staged, &self.directory, name)?;
        Ok(())
    }

    fn recover_stage(&self) -> Result<()> {
        if let Some((_, preimage)) = self.read(STAGE, BODY_LIMIT)? {
            self.revalidate()?;
            self.check_preimage(STAGE, BODY_LIMIT, Some(&preimage))?;
            self.directory.unlink_file(STAGE)?;
        }
        Ok(())
    }

    fn refresh(&self, origin: &str, timeout: Duration) -> Result<Value> {
        let fd = self.directory.borrow_fd();
        // Lock the held directory, not a replaceable lock file. All refresh paths
        // share this one nonblocking lock through fetch, validation and publication.
        // SAFETY: fd is a live directory descriptor; flock does not take ownership.
        ensure!(
            unsafe { libc::flock(fd.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } == 0,
            "another catalog refresh is in flight"
        );
        struct Unlock(i32);
        impl Drop for Unlock {
            fn drop(&mut self) {
                // SAFETY: Cache keeps this descriptor live until after the guard.
                unsafe { libc::flock(self.0, libc::LOCK_UN) };
            }
        }
        let _lock = Unlock(fd.as_raw_fd());
        self.revalidate()?;
        self.inventory()?;
        self.recover_stage()?;
        let previous = self.read(SNAPSHOT, BODY_LIMIT)?;
        if let Some((bytes, _)) = &previous {
            Snapshot::decode(bytes)?;
        }
        let failure = self.read(FAILURE, FAILURE_LIMIT)?;
        let result = (|| {
            let snapshot = fetch(origin, timeout)?;
            let bytes = serde_json::to_vec(&snapshot)?;
            self.publish(
                SNAPSHOT,
                &bytes,
                previous.as_ref().map(|(_, preimage)| preimage),
                BODY_LIMIT,
            )?;
            Ok(())
        })();
        if let Err(error) = result {
            // Never store upstream text, URLs or transport diagnostics in durable metadata.
            self.recover_stage()?;
            let recorded = RefreshFailure {
                version: 1,
                attempted_utc: Utc::now(),
                base_snapshot_sha256: previous.as_ref().map(|(bytes, _)| digest(bytes)),
                reason: "refresh_failed".to_owned(),
            };
            self.publish(
                FAILURE,
                &serde_json::to_vec(&recorded)?,
                failure.as_ref().map(|(_, preimage)| preimage),
                FAILURE_LIMIT,
            )?;
            return Err(error);
        }
        if let Some((_, preimage)) = &failure {
            self.revalidate()?;
            self.check_preimage(FAILURE, FAILURE_LIMIT, Some(preimage))?;
            self.directory.unlink_file(FAILURE)?;
        }
        self.status(Utc::now())
    }

    fn status(&self, now: DateTime<Utc>) -> Result<Value> {
        self.revalidate()?;
        let current = self.read(SNAPSHOT, BODY_LIMIT)?;
        let snapshot = current
            .as_ref()
            .map(|(bytes, _)| Snapshot::decode(bytes))
            .transpose()?;
        let failure = self
            .read(FAILURE, FAILURE_LIMIT)?
            .map(|(bytes, _)| -> Result<RefreshFailure> {
                let failure: RefreshFailure = serde_json::from_value(strict_json(&bytes)?)?;
                ensure!(
                    failure.version == 1 && failure.reason == "refresh_failed",
                    "unknown refresh failure record"
                );
                Ok(failure)
            })
            .transpose()?;
        let current_digest = current.as_ref().map(|(bytes, _)| digest(bytes));
        let failure = failure.filter(|failure| failure.base_snapshot_sha256 == current_digest);
        self.revalidate()?;
        let mut status = empty_status();
        status["refresh_failure"] = serde_json::to_value(&failure)?;
        status["offline"] = Value::Bool(failure.is_some());
        let failure_clock_uncertain = failure
            .as_ref()
            .is_some_and(|failure| failure.attempted_utc > now);
        status["clock_uncertain"] = Value::Bool(failure_clock_uncertain);
        if let Some(snapshot) = snapshot {
            let age = now.signed_duration_since(snapshot.fetched_utc);
            let uncertain = age.num_milliseconds() < 0 || failure_clock_uncertain;
            status["available"] = Value::Bool(true);
            status["fetched_utc"] = serde_json::to_value(snapshot.fetched_utc)?;
            status["age_seconds"] = if uncertain {
                Value::Null
            } else {
                Value::from(age.num_seconds())
            };
            status["clock_uncertain"] = Value::Bool(uncertain);
            status["stale"] =
                Value::Bool(uncertain || failure.is_some() || age.num_seconds() >= 24 * 60 * 60);
            status["http_status"] = Value::from(snapshot.http_status);
            status["decoded_bytes"] = Value::from(snapshot.decoded_bytes);
            status["source_sha256"] = Value::String(snapshot.source_sha256);
            status["normalized_sha256"] = Value::String(snapshot.normalized_sha256);
            status["etag"] = serde_json::to_value(snapshot.etag)?;
            status["last_modified"] = serde_json::to_value(snapshot.last_modified)?;
            status["provider_count"] = Value::from(snapshot.catalog.providers.len());
            status["model_count"] = Value::from(snapshot.catalog.models.len());
            status["offering_count"] = Value::from(
                snapshot
                    .catalog
                    .providers
                    .values()
                    .map(|provider| provider.models.len())
                    .sum::<usize>(),
            );
        }
        Ok(status)
    }
}

fn empty_status() -> Value {
    serde_json::json!({"version":1,"origin":ORIGIN,"type_filter":"chat","available":false,"stale":true,"age_seconds":null,"clock_uncertain":false,"offline":false,"refresh_failure":null,"authority":"descriptive_metadata_only"})
}

fn no_acl(fd: i32) -> Result<()> {
    for name in [c"system.posix_acl_access", c"system.posix_acl_default"] {
        // SAFETY: fd/name are valid; zero-length null buffer queries xattr size.
        let size = unsafe { libc::fgetxattr(fd, name.as_ptr(), std::ptr::null_mut(), 0) };
        ensure!(
            size < 0 && std::io::Error::last_os_error().raw_os_error() == Some(libc::ENODATA),
            "catalog ACL present or validation unavailable"
        );
    }
    Ok(())
}

pub(crate) fn try_handle_catalog_command(
    cli: &super::cli::Cli,
    carrier: &transport_api_types::InstallBootstrapContextCarrierV1,
) -> Option<i32> {
    use super::cli::{AgentCatalogAction, AgentsAction, SubCommands};
    let Some(SubCommands::Agents(command)) = &cli.sub else {
        return None;
    };
    let AgentsAction::Catalog(command) = &command.action else {
        return None;
    };
    let result = (|| -> Result<Value> {
        super::install_bootstrap::bind_unix_install_bootstrap_context(carrier)?;
        let transport_api_types::PlatformPrincipalV1::Unix { uid, .. } =
            carrier.context.intended_host_principal
        else {
            bail!("catalog requires a Unix accepted-home principal");
        };
        let refresh = matches!(command.action, AgentCatalogAction::Refresh(_));
        let cache = Cache::open(
            Path::new(&carrier.context.host_substrate_home),
            uid,
            refresh,
        )?;
        match (cache, refresh) {
            (Some(cache), true) => {
                #[cfg(not(substrate_catalog_test_support))]
                let origin = ORIGIN;
                #[cfg(substrate_catalog_test_support)]
                let fixture_origin = test_support::dispatch_origin()?;
                #[cfg(substrate_catalog_test_support)]
                let origin = fixture_origin.as_str();
                cache.refresh(origin, DEADLINE)
            }
            (Some(cache), false) => cache.status(Utc::now()),
            (None, _) => Ok(empty_status()),
        }
    })();
    let json = match &command.action {
        AgentCatalogAction::Refresh(args) | AgentCatalogAction::Status(args) => args.json,
    };
    Some(match result {
        Ok(status) => {
            if json {
                println!("{status}");
            } else {
                println!(
                    "Catalog: available={} stale={} offline={}",
                    status["available"], status["stale"], status["offline"]
                );
            }
            0
        }
        Err(error) => {
            // Errors are diagnostics, never a signal to change a selected model.
            if json {
                println!(
                    "{}",
                    serde_json::json!({"version":1,"ok":false,"error":"catalog_operation_failed"})
                );
            }
            eprintln!("substrate: catalog operation failed: {error:#}");
            1
        }
    })
}

/// Fixture access and loopback CLI origin injection, compiled only in dedicated test builds.
#[cfg(substrate_catalog_test_support)]
#[doc(hidden)]
pub mod test_support {
    use super::*;

    pub(super) fn dispatch_origin() -> Result<String> {
        // A dedicated test binary must never fall through to a live metadata request.
        let origin = std::env::var("SUBSTRATE_CATALOG_TEST_ORIGIN")
            .context("test catalog CLI requires an explicit loopback origin")?;
        let url = url::Url::parse(&origin)?;
        ensure!(
            url.scheme() == "http"
                && url.host_str() == Some("127.0.0.1")
                && url.port().is_some()
                && url.username().is_empty()
                && url.password().is_none()
                && url.fragment().is_none(),
            "test catalog CLI origin must be loopback HTTP without credentials or fragment"
        );
        Ok(origin)
    }

    pub fn refresh(home: &Path, origin: &str, timeout: Duration) -> Result<Value> {
        let url = url::Url::parse(origin)?;
        ensure!(
            url.scheme() == "http"
                && url.host_str() == Some("127.0.0.1")
                && url.username().is_empty()
                && url.password().is_none(),
            "fixture origin must be loopback HTTP without credentials"
        );
        // SAFETY: geteuid has no pointer or ownership preconditions.
        Cache::open(home, unsafe { libc::geteuid() }, true)?
            .context("missing fixture home")?
            .refresh(origin, timeout)
    }

    pub fn status(home: &Path) -> Result<Value> {
        // SAFETY: geteuid has no pointer or ownership preconditions.
        Cache::open(home, unsafe { libc::geteuid() }, false)?
            .map(|cache| cache.status(Utc::now()))
            .unwrap_or_else(|| Ok(empty_status()))
    }

    pub fn stage_until_terminated(home: &Path) -> Result<()> {
        // A subprocess-only interruption hook. It uses the same directory lock,
        // trusted exclusive stage creation and fsync as publication, then stops
        // before the atomic rename so SIGKILL can exercise real recovery.
        let cache =
            Cache::open(home, unsafe { libc::geteuid() }, true)?.context("missing fixture home")?;
        let fd = cache.directory.borrow_fd().as_raw_fd();
        ensure!(
            unsafe { libc::flock(fd, libc::LOCK_EX | libc::LOCK_NB) } == 0,
            "refresh already in flight"
        );
        let mut stage = cache.directory.create_file(STAGE)?;
        stage.write_all(b"partial snapshot")?;
        stage.sync()?;
        std::fs::write(home.join("interruption-ready"), b"ready")?;
        loop {
            std::thread::park();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub(super) fn fixture() -> Vec<u8> {
        br#"{"models":{"lab/base":{"id":"lab/base","name":"Base","description":"Metadata"}},"providers":{"host":{"id":"host","name":"Host","npm":"never-executed","doc":"https://example.invalid","env":["NEVER_READ"],"models":{"route/model":{"id":"route/model","name":"Offering","description":"Metadata","canonical_model_id":"lab/base","attachment":false,"reasoning":false,"tool_call":true,"open_weights":false,"release_date":"2026-01-01","last_updated":"2026-01-01","modalities":{"input":["text"],"output":["text"]},"limit":{"context":1000,"output":100}}}}}}"#.to_vec()
    }

    #[test]
    fn model_catalog_normalizes_distinct_offering_and_canonical_identity() {
        let catalog = Catalog::parse(&fixture()).unwrap();
        assert_eq!(
            catalog.providers["host"].models["route/model"]
                .canonical_model_id
                .as_deref(),
            Some("lab/base")
        );
        assert!(!catalog.providers["host"].models.contains_key("lab/base"));
    }

    #[test]
    fn model_catalog_rejects_duplicates_even_in_ignored_fields() {
        for input in [
            br#"{"models":{},"models":{},"providers":{}}"#.as_slice(),
            br#"{"models":{},"providers":{},"unknown":{"x":1,"\u0078":2}}"#,
        ] {
            assert!(Catalog::parse(input).is_err());
        }
    }

    #[test]
    fn model_catalog_rejects_types_utf8_truncation_identity_and_coherence() {
        let good = String::from_utf8(fixture()).unwrap();
        for bad in [
            good.replace("\"id\":\"host\"", "\"id\":12"),
            good.replace("\"id\":\"host\"", "\"id\":\"other\""),
            good.replace(
                "\"canonical_model_id\":\"lab/base\"",
                "\"canonical_model_id\":\"missing\"",
            ),
            good.replace("\"description\":\"Metadata\"", "\"description\":null"),
            good.replace(
                "\"description\":\"Metadata\"",
                "\"description\":\"\",\"type\":\"image\"",
            ),
        ] {
            assert!(Catalog::parse(bad.as_bytes()).is_err(), "{bad}");
        }
        assert!(Catalog::parse(&[0xff]).is_err());
        assert!(Catalog::parse(&fixture()[..30]).is_err());
    }

    #[test]
    fn model_catalog_bounds_body_text_and_depth() {
        assert!(strict_json(&vec![b' '; BODY_LIMIT + 1]).is_err());
        assert!(strict_json(
            serde_json::to_string(&"a".repeat(TEXT_LIMIT + 1))
                .unwrap()
                .as_bytes()
        )
        .is_err());
        let nested = format!("{}0{}", "[".repeat(32), "]".repeat(32));
        assert!(strict_json(nested.as_bytes()).is_err());
        let nested = format!("{}0{}", "[".repeat(31), "]".repeat(31));
        assert!(strict_json(nested.as_bytes()).is_ok());
    }

    #[test]
    fn model_catalog_provider_offering_counts_and_optional_attribution_are_bounded() {
        let mut catalog = Catalog::parse(&fixture()).unwrap();
        let offering = catalog.providers["host"].models["route/model"].clone();
        catalog.providers.get_mut("host").unwrap().models = (0..=MODEL_LIMIT)
            .map(|i| (format!("offering-{i}"), offering.clone()))
            .collect();
        assert!(catalog
            .validate()
            .unwrap_err()
            .to_string()
            .contains("too many offerings"));
        let mut value: Value = serde_json::from_slice(&fixture()).unwrap();
        value["providers"]["host"]["models"]["route/model"]
            .as_object_mut()
            .unwrap()
            .remove("canonical_model_id");
        assert!(Catalog::parse(&serde_json::to_vec(&value).unwrap()).is_ok());
        value["providers"] = Value::Object(
            (0..=PROVIDER_LIMIT)
                .map(|i| (format!("provider-{i}"), serde_json::json!({})))
                .collect(),
        );
        assert!(Catalog::parse(&serde_json::to_vec(&value).unwrap())
            .unwrap_err()
            .to_string()
            .contains("count exceeds"));
    }

    fn home() -> tempfile::TempDir {
        use std::os::unix::fs::PermissionsExt;
        let parent = std::env::var_os("SUBSTRATE_A1_TEST_PARENT")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")));
        let home = tempfile::tempdir_in(parent).unwrap();
        std::fs::set_permissions(home.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        home
    }

    fn cache(home: &Path) -> Cache {
        // SAFETY: geteuid has no pointer preconditions.
        Cache::open(home, unsafe { libc::geteuid() }, true)
            .unwrap()
            .unwrap()
    }

    fn snapshot() -> Snapshot {
        let catalog = Catalog::parse(&fixture()).unwrap();
        Snapshot {
            version: 1,
            origin: ORIGIN.into(),
            type_filter: "chat".into(),
            normalizer: "substrate-models-dev-v1".into(),
            upstream_schema: "models.dev/catalog-v1".into(),
            fetched_utc: Utc::now(),
            http_status: 200,
            decoded_bytes: fixture().len(),
            source_sha256: digest(&fixture()),
            normalized_sha256: digest(&serde_json::to_vec(&catalog).unwrap()),
            etag: None,
            last_modified: None,
            catalog,
        }
    }

    #[test]
    fn model_catalog_status_missing_cache_is_nonmutating() {
        let home = home();
        // SAFETY: geteuid has no pointer preconditions.
        assert!(Cache::open(home.path(), unsafe { libc::geteuid() }, false)
            .unwrap()
            .is_none());
        assert_eq!(std::fs::read_dir(home.path()).unwrap().count(), 0);
    }

    #[test]
    fn model_catalog_status_age_stale_and_clock_uncertainty() {
        let home = home();
        let cache = cache(home.path());
        let mut snapshot = snapshot();
        snapshot.fetched_utc = Utc::now() - chrono::Duration::hours(25);
        let bytes = serde_json::to_vec(&snapshot).unwrap();
        cache.publish(SNAPSHOT, &bytes, None, BODY_LIMIT).unwrap();
        let status = cache.status(Utc::now()).unwrap();
        assert_eq!(status["stale"], true);
        assert!(status["age_seconds"].as_i64().unwrap() >= 25 * 3600);
        let status = cache
            .status(snapshot.fetched_utc - chrono::Duration::seconds(1))
            .unwrap();
        assert_eq!(status["clock_uncertain"], true);
        assert_eq!(status["age_seconds"], Value::Null);
        assert_eq!(
            std::fs::read(home.path().join("model-catalog").join(SNAPSHOT)).unwrap(),
            bytes
        );
    }

    #[test]
    fn model_catalog_interrupted_stage_recovery_keeps_last_valid() {
        let home = home();
        let cache = cache(home.path());
        let bytes = serde_json::to_vec(&snapshot()).unwrap();
        cache.publish(SNAPSHOT, &bytes, None, BODY_LIMIT).unwrap();
        let mut stage = cache.directory.create_file(STAGE).unwrap();
        stage.write_all(b"interrupted").unwrap();
        stage.sync().unwrap();
        assert_eq!(cache.status(Utc::now()).unwrap()["available"], true);
        assert!(home.path().join("model-catalog").join(STAGE).exists());
        cache.recover_stage().unwrap();
        assert!(!home.path().join("model-catalog").join(STAGE).exists());
        assert_eq!(cache.read(SNAPSHOT, BODY_LIMIT).unwrap().unwrap().0, bytes);
    }

    #[test]
    fn model_catalog_refuses_permissions_symlink_hardlink_fifo_and_oversize() {
        use std::os::unix::fs::{symlink, PermissionsExt};
        let home = home();
        let cache = cache(home.path());
        let path = home.path().join("model-catalog").join(SNAPSHOT);
        let outside = home.path().join("outside");
        std::fs::write(&outside, b"untouched").unwrap();
        symlink(&outside, &path).unwrap();
        assert!(cache.read(SNAPSHOT, BODY_LIMIT).is_err());
        std::fs::remove_file(&path).unwrap();
        std::fs::hard_link(&outside, &path).unwrap();
        assert!(cache.read(SNAPSHOT, BODY_LIMIT).is_err());
        std::fs::remove_file(&path).unwrap();
        std::fs::write(&path, b"x").unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
        assert!(cache.read(SNAPSHOT, BODY_LIMIT).is_err());
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
        File::options()
            .write(true)
            .open(&path)
            .unwrap()
            .set_len(BODY_LIMIT as u64 + 1)
            .unwrap();
        assert!(cache.read(SNAPSHOT, BODY_LIMIT).is_err());
        std::fs::remove_file(&path).unwrap();
        let name = std::ffi::CString::new(path.as_os_str().as_encoded_bytes()).unwrap();
        // SAFETY: name is a valid live C string and mode is a valid permission mask.
        assert_eq!(unsafe { libc::mkfifo(name.as_ptr(), 0o600) }, 0);
        assert!(cache.read(SNAPSHOT, BODY_LIMIT).is_err());
        assert_eq!(std::fs::read(outside).unwrap(), b"untouched");
    }

    #[test]
    fn model_catalog_refuses_home_and_directory_replacement_and_bad_preimage() {
        use std::os::unix::fs::PermissionsExt;
        let home = home();
        let cache = cache(home.path());
        let bytes = serde_json::to_vec(&snapshot()).unwrap();
        cache.publish(SNAPSHOT, &bytes, None, BODY_LIMIT).unwrap();
        assert!(cache.publish(SNAPSHOT, &bytes, None, BODY_LIMIT).is_err());
        std::fs::rename(
            home.path().join("model-catalog"),
            home.path().join("old-cache"),
        )
        .unwrap();
        std::fs::create_dir(home.path().join("model-catalog")).unwrap();
        std::fs::set_permissions(
            home.path().join("model-catalog"),
            std::fs::Permissions::from_mode(0o700),
        )
        .unwrap();
        assert!(cache.revalidate().is_err());
        assert!(cache.publish(SNAPSHOT, &bytes, None, BODY_LIMIT).is_err());
        assert!(!home.path().join("model-catalog").join(SNAPSHOT).exists());
    }

    #[test]
    fn model_catalog_refuses_accepted_home_replacement_and_acl() {
        use std::os::unix::fs::PermissionsExt;
        let parent = home();
        let path = parent.path().join("home");
        std::fs::create_dir(&path).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700)).unwrap();
        let cache = cache(&path);
        let bytes = serde_json::to_vec(&snapshot()).unwrap();
        cache.publish(SNAPSHOT, &bytes, None, BODY_LIMIT).unwrap();
        let file = File::open(path.join("model-catalog").join(SNAPSHOT)).unwrap();
        let mut acl = 2u32.to_le_bytes().to_vec();
        for (tag, permissions, id) in [
            (1u16, 6u16, u32::MAX),
            (2, 4, unsafe { libc::geteuid() } + 1),
            (4, 0, u32::MAX),
            (16, 0, u32::MAX),
            (32, 0, u32::MAX),
        ] {
            acl.extend_from_slice(&tag.to_le_bytes());
            acl.extend_from_slice(&permissions.to_le_bytes());
            acl.extend_from_slice(&id.to_le_bytes());
        }
        // SAFETY: descriptor, name and byte buffer remain live for the xattr call.
        assert_eq!(
            unsafe {
                libc::fsetxattr(
                    file.as_raw_fd(),
                    c"system.posix_acl_access".as_ptr(),
                    acl.as_ptr().cast(),
                    acl.len(),
                    0,
                )
            },
            0
        );
        assert!(cache.read(SNAPSHOT, BODY_LIMIT).is_err());
        // SAFETY: descriptor and name remain live for xattr removal.
        assert_eq!(
            unsafe { libc::fremovexattr(file.as_raw_fd(), c"system.posix_acl_access".as_ptr()) },
            0
        );
        std::fs::rename(&path, parent.path().join("old-home")).unwrap();
        std::fs::create_dir(&path).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700)).unwrap();
        assert!(cache.revalidate().is_err());
        assert!(cache.publish(SNAPSHOT, &bytes, None, BODY_LIMIT).is_err());
        assert_eq!(
            std::fs::read(parent.path().join("old-home/model-catalog").join(SNAPSHOT)).unwrap(),
            bytes
        );
        assert_eq!(std::fs::read_dir(path).unwrap().count(), 0);
    }

    #[test]
    fn model_catalog_unknown_storage_and_oversize_stage_refuse_without_deletion() {
        let home = home();
        let cache = cache(home.path());
        let mut foreign = cache.directory.create_file("foreign").unwrap();
        foreign.write_all(b"keep").unwrap();
        assert!(cache.inventory().is_err());
        assert_eq!(
            std::fs::read(home.path().join("model-catalog/foreign")).unwrap(),
            b"keep"
        );
        assert!(cache
            .publish(SNAPSHOT, &vec![b'x'; BODY_LIMIT + 1], None, BODY_LIMIT)
            .is_err());
        assert!(!home.path().join("model-catalog").join(STAGE).exists());
    }

    #[test]
    fn model_catalog_single_lock_refuses_concurrent_refresh_before_network() {
        let home = home();
        let cache = cache(home.path());
        // SAFETY: descriptor remains live until test completion.
        assert_eq!(
            unsafe {
                libc::flock(
                    cache.directory.borrow_fd().as_raw_fd(),
                    libc::LOCK_EX | libc::LOCK_NB,
                )
            },
            0
        );
        let second = super::tests::cache(home.path());
        let error = second
            .refresh("http://127.0.0.1:1/never", Duration::from_millis(30))
            .unwrap_err();
        assert!(error.to_string().contains("in flight"));
        assert!(!home.path().join("model-catalog").join(FAILURE).exists());
    }

    #[test]
    fn model_catalog_preserves_actual_baseline_config_and_state_store_producers() {
        use crate::execution::agent_runtime::{
            host_session_authority::facade::HostSessionAuthority,
            mapping::AgentRuntimeBackendKind,
            orchestration_session::{HostAttachContract, OrchestrationSessionRecord},
            session::AgentRuntimeParticipantRecord,
            state_store::AgentRuntimeStateStore,
            validator::RuntimeSelectionDescriptor,
        };
        use crate::execution::config_model::{
            self, AgentExecutionScope, CliConfigOverrides, SubstrateConfig,
        };
        use std::io::Write;
        let home = home();
        let config = SubstrateConfig::default();
        std::fs::write(
            home.path().join("config.yaml"),
            serde_yaml::to_string(&config).unwrap(),
        )
        .unwrap();
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(
            home.path().join("config.yaml"),
            std::fs::Permissions::from_mode(0o600),
        )
        .unwrap();
        let authority = HostSessionAuthority::open(home.path()).unwrap();
        let bootstrap = authority.bootstrap_home();
        let actual_config = config_model::resolve_effective_config_for_bootstrap_home(
            home.path(),
            &CliConfigOverrides::default(),
            &bootstrap,
        )
        .unwrap();
        let store = AgentRuntimeStateStore::for_bootstrap_home(&bootstrap).unwrap();
        let descriptor = RuntimeSelectionDescriptor {
            agent_id: "catalog_retention".into(),
            backend_id: "cli:catalog_retention".into(),
            backend_kind: AgentRuntimeBackendKind::Codex,
            protocol: "substrate.agent.session".into(),
            execution_scope: AgentExecutionScope::Host,
            binary_path: PathBuf::from("/never-launched/codex"),
        };
        let participant = AgentRuntimeParticipantRecord::new_orchestrator_participant(
            &descriptor,
            "catalog_session".into(),
            "catalog_participant".into(),
            "catalog_lease".into(),
        )
        .unwrap();
        let session = OrchestrationSessionRecord::new(
            "catalog_session".into(),
            "catalog_trace".into(),
            home.path().display().to_string(),
            &participant,
            HostAttachContract::from_manifest_for_test(&participant),
        );
        store.persist_participant(&participant).unwrap();
        store.persist_orchestration_session(&session).unwrap();
        fn retained(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
            let mut files = BTreeMap::new();
            for entry in std::fs::read_dir(root).unwrap() {
                let entry = entry.unwrap();
                if entry.file_name() == "model-catalog" {
                    continue;
                }
                if entry.file_type().unwrap().is_dir() {
                    files.extend(retained(&entry.path()));
                } else {
                    files.insert(entry.path(), std::fs::read(entry.path()).unwrap());
                }
            }
            files
        }
        let before = retained(home.path());
        assert!(
            before.len() >= 4,
            "actual state-store producer wrote no meaningful records"
        );
        let cache = cache(home.path());
        for body in [
            fixture(),
            br#"{"models":{},"providers":{}}"#.to_vec(),
            b"{bad".to_vec(),
        ] {
            let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
            let origin = format!("http://{}/catalog", listener.local_addr().unwrap());
            let valid = body != b"{bad";
            let server = std::thread::spawn(move || {
                let (mut stream, _) = listener.accept().unwrap();
                let mut request = [0u8; 4096];
                let bytes_read = stream.read(&mut request).unwrap();
                assert!(bytes_read > 0, "fixture server received no request bytes");
                write!(
                    stream,
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                )
                .unwrap();
                stream.write_all(&body).unwrap();
            });
            assert_eq!(
                cache.refresh(&origin, Duration::from_secs(3)).is_ok(),
                valid
            );
            server.join().unwrap();
            assert_eq!(retained(home.path()), before);
            assert_eq!(
                serde_json::to_value(
                    config_model::resolve_effective_config_for_bootstrap_home(
                        home.path(),
                        &CliConfigOverrides::default(),
                        &bootstrap
                    )
                    .unwrap()
                )
                .unwrap(),
                serde_json::to_value(&actual_config).unwrap()
            );
            assert_eq!(
                store
                    .load_participant("catalog_participant")
                    .unwrap()
                    .unwrap(),
                participant
            );
            assert_eq!(
                store
                    .load_session("catalog_session")
                    .unwrap()
                    .unwrap()
                    .session,
                session
            );
        }
        // This proves real baseline parsers/record producers, not Start/Resume or
        // V3 selection. Baseline RuntimeSelectionDescriptor has no model tuple.
    }
}
