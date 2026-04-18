use std::collections::BTreeMap;
use std::sync::Arc;

use crate::lang::{
    AdapterDescriptor, AdapterName, LangError, LangResult, LanguageAdapter, LanguageId,
};

#[derive(Default)]
pub(crate) struct LanguageRegistryBuilder {
    adapters: BTreeMap<AdapterName, Arc<dyn LanguageAdapter>>,
    languages: BTreeMap<LanguageId, AdapterName>,
}

pub(crate) struct LanguageRegistry {
    adapters: BTreeMap<AdapterName, Arc<dyn LanguageAdapter>>,
    languages: BTreeMap<LanguageId, AdapterName>,
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
}
