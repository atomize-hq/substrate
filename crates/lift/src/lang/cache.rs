use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use crate::kernel::{FileId, Fingerprint};
use crate::lang::{
    AdapterDescriptor, AdapterName, FailedParse, LanguageId, ParseInput, ParsedUnit,
};

pub(crate) const PLATFORM_CACHE_VERSION: &str = "phase_b_v1";

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct ParseCacheKey {
    pub file_id: FileId,
    pub blob_fingerprint: Fingerprint,
    pub language: LanguageId,
    pub adapter: AdapterName,
    pub adapter_version: String,
    pub platform_cache_version: String,
}

impl ParseCacheKey {
    pub(crate) fn for_input(input: &ParseInput<'_>, descriptor: &AdapterDescriptor) -> Self {
        Self::with_platform_cache_version(input, descriptor, PLATFORM_CACHE_VERSION)
    }

    pub(crate) fn with_platform_cache_version(
        input: &ParseInput<'_>,
        descriptor: &AdapterDescriptor,
        platform_cache_version: &str,
    ) -> Self {
        Self {
            file_id: input.file_id.clone(),
            blob_fingerprint: input.blob_fingerprint.clone(),
            language: descriptor.language,
            adapter: descriptor.name.clone(),
            adapter_version: descriptor.version.clone(),
            platform_cache_version: platform_cache_version.to_owned(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CachedParseOutcome {
    Parsed(ParsedUnit),
    Failed(FailedParse),
}

pub(crate) enum ParseCacheLookup {
    Disabled,
    Hit(Box<CachedParseOutcome>),
    Miss,
}

pub(crate) trait ParseCache: Send + Sync {
    fn get(&self, key: &ParseCacheKey) -> ParseCacheLookup;
    fn put(&self, key: ParseCacheKey, value: CachedParseOutcome);
}

#[derive(Default)]
pub(crate) struct NoopParseCache;

impl ParseCache for NoopParseCache {
    fn get(&self, _key: &ParseCacheKey) -> ParseCacheLookup {
        ParseCacheLookup::Disabled
    }

    fn put(&self, _key: ParseCacheKey, _value: CachedParseOutcome) {}
}

#[derive(Clone, Default)]
pub(crate) struct InMemoryParseCache {
    entries: Arc<Mutex<BTreeMap<ParseCacheKey, CachedParseOutcome>>>,
}

impl InMemoryParseCache {
    pub(crate) fn len(&self) -> usize {
        self.entries
            .lock()
            .expect("in-memory parse cache lock poisoned")
            .len()
    }
}

impl ParseCache for InMemoryParseCache {
    fn get(&self, key: &ParseCacheKey) -> ParseCacheLookup {
        let entries = self
            .entries
            .lock()
            .expect("in-memory parse cache lock poisoned");
        match entries.get(key).cloned() {
            Some(value) => ParseCacheLookup::Hit(Box::new(value)),
            None => ParseCacheLookup::Miss,
        }
    }

    fn put(&self, key: ParseCacheKey, value: CachedParseOutcome) {
        self.entries
            .lock()
            .expect("in-memory parse cache lock poisoned")
            .insert(key, value);
    }
}
