use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::lang::{
    AdapterCapabilities, AdapterDescriptor, AdapterName, LangError, LangResult, LanguageAdapter,
    LanguageId, QueryEngineKind,
};

pub(crate) const BUILT_IN_LANGUAGE_ORDER: &[LanguageId] = &[
    LanguageId::Json,
    LanguageId::Toml,
    LanguageId::Yaml,
    LanguageId::Rust,
    LanguageId::Python,
    LanguageId::Javascript,
    LanguageId::Typescript,
];

#[derive(Default)]
pub(crate) struct LanguageRegistryBuilder {
    adapters: BTreeMap<AdapterName, Arc<dyn LanguageAdapter>>,
    languages: BTreeMap<LanguageId, AdapterName>,
}

pub(crate) struct LanguageRegistry {
    adapters: BTreeMap<AdapterName, Arc<dyn LanguageAdapter>>,
    languages: BTreeMap<LanguageId, AdapterName>,
}

pub(crate) fn built_in_registry() -> LangResult<LanguageRegistry> {
    bootstrap_built_in_registry(LanguageRegistryBuilder::new())?.build()
}

impl LanguageRegistryBuilder {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn register<A: LanguageAdapter + 'static>(mut self, adapter: A) -> LangResult<Self> {
        let descriptor = adapter.descriptor();
        let name = descriptor.name.clone();
        let language = descriptor.language;

        if self.adapters.contains_key(&name) {
            return Err(LangError::DuplicateAdapterName {
                name: name.as_str().to_owned(),
            });
        }

        if let Some(existing) = self.languages.get(&language) {
            return Err(LangError::DuplicateLanguageAdapter {
                language,
                existing: existing.as_str().to_owned(),
                duplicate: name.as_str().to_owned(),
            });
        }

        self.languages.insert(language, name.clone());
        self.adapters.insert(name, Arc::new(adapter));
        Ok(self)
    }

    pub(crate) fn build(self) -> LangResult<LanguageRegistry> {
        Ok(LanguageRegistry {
            adapters: self.adapters,
            languages: self.languages,
        })
    }
}

impl LanguageRegistry {
    pub(crate) fn descriptors(&self) -> Vec<AdapterDescriptor> {
        self.adapters
            .values()
            .map(|adapter| adapter.descriptor())
            .collect()
    }

    pub(crate) fn adapter_for_language(
        &self,
        language: LanguageId,
    ) -> Option<&Arc<dyn LanguageAdapter>> {
        let name = self.languages.get(&language)?;
        self.adapters.get(name)
    }

    pub(crate) fn capabilities_for_language(
        &self,
        language: LanguageId,
    ) -> Option<AdapterCapabilities> {
        self.adapter_for_language(language)
            .map(|adapter| adapter.capabilities())
    }

    pub(crate) fn supported_query_engines_for_language(
        &self,
        language: LanguageId,
    ) -> BTreeSet<QueryEngineKind> {
        self.capabilities_for_language(language)
            .map(|capabilities| capabilities.query_engines)
            .unwrap_or_default()
    }
}

fn bootstrap_built_in_registry(
    builder: LanguageRegistryBuilder,
) -> LangResult<LanguageRegistryBuilder> {
    Ok(builder)
}
