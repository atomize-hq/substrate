use super::{DirectoryEntry, EntryKind, StoreError, TrustedDirectory};

pub(super) struct LegacyObservation {
    sessions: LegacyCollectionObservation,
    participants: LegacyCollectionObservation,
    pub(super) has_artifact: bool,
}

impl LegacyObservation {
    pub(super) fn capture(root: &TrustedDirectory) -> Result<Self, StoreError> {
        let sessions =
            LegacyCollectionObservation::capture(root, &["run", "agent-hub", "sessions"], true)?;
        let participants = LegacyCollectionObservation::capture(
            root,
            &["run", "agent-hub", "participants"],
            false,
        )?;
        Ok(Self {
            has_artifact: sessions.has_artifact || participants.has_artifact,
            sessions,
            participants,
        })
    }

    pub(super) fn revalidate(&self, root: &TrustedDirectory) -> Result<(), StoreError> {
        if self.has_artifact {
            return Err(StoreError("pre-A1 authority state was observed"));
        }
        self.sessions.revalidate(root)?;
        self.participants.revalidate(root)
    }
}

struct LegacyCollectionObservation {
    components: Vec<ObservedLegacyDirectory>,
    missing_suffix: Vec<String>,
    recursive: bool,
    has_artifact: bool,
}

struct ObservedLegacyDirectory {
    entry: DirectoryEntry,
    directory: TrustedDirectory,
}

impl LegacyCollectionObservation {
    fn capture(
        root: &TrustedDirectory,
        names: &[&str],
        recursive: bool,
    ) -> Result<Self, StoreError> {
        let mut components = Vec::new();
        let mut missing_suffix = Vec::new();
        for (index, name) in names.iter().enumerate() {
            let parent = components
                .last()
                .map(|value: &ObservedLegacyDirectory| &value.directory)
                .unwrap_or(root);
            let entry = parent
                .entries()
                .map_err(|_| StoreError("enumerate legacy authority path"))?
                .into_iter()
                .find(|entry| entry.name == *name);
            let Some(entry) = entry else {
                missing_suffix.extend(names[index..].iter().map(|value| (*value).to_string()));
                break;
            };
            let directory = parent
                .open_controlled_directory_entry(&entry)
                .map_err(|_| StoreError("open legacy authority path"))?;
            components.push(ObservedLegacyDirectory { entry, directory });
        }
        let has_artifact = if missing_suffix.is_empty() {
            let collection = &components
                .last()
                .ok_or(StoreError("legacy collection observation is empty"))?
                .directory;
            if recursive {
                inspect_legacy_tree(collection)?
            } else {
                inspect_legacy_level(collection)?
            }
        } else {
            false
        };
        Ok(Self {
            components,
            missing_suffix,
            recursive,
            has_artifact,
        })
    }

    fn revalidate(&self, root: &TrustedDirectory) -> Result<(), StoreError> {
        let mut parent = root;
        for component in &self.components {
            parent
                .revalidate_entry(&component.entry)
                .map_err(|_| StoreError("legacy authority directory identity changed"))?;
            let reopened = parent
                .open_controlled_directory_entry(&component.entry)
                .map_err(|_| StoreError("reopen observed legacy authority directory"))?;
            drop(reopened);
            parent = &component.directory;
        }
        if let Some(first_missing) = self.missing_suffix.first() {
            if parent
                .entry_kind(first_missing)
                .map_err(|_| StoreError("revalidate missing legacy authority suffix"))?
                .is_some()
            {
                return Err(StoreError("missing legacy authority suffix appeared"));
            }
            return Ok(());
        }
        let has_artifact = if self.recursive {
            inspect_legacy_tree(parent)?
        } else {
            inspect_legacy_level(parent)?
        };
        if has_artifact {
            Err(StoreError("pre-A1 authority state appeared"))
        } else {
            Ok(())
        }
    }
}

fn inspect_legacy_tree(directory: &TrustedDirectory) -> Result<bool, StoreError> {
    let entries = directory
        .entries()
        .map_err(|_| StoreError("enumerate legacy session state"))?;
    let mut found = false;
    for entry in entries {
        found = true;
        match entry.kind {
            EntryKind::Directory => {
                let child = directory
                    .open_controlled_directory_entry(&entry)
                    .map_err(|_| StoreError("open legacy session directory"))?;
                inspect_legacy_tree(&child)?;
            }
            EntryKind::RegularFile => directory
                .revalidate_entry(&entry)
                .map_err(|_| StoreError("validate legacy session record"))?,
            EntryKind::Symlink | EntryKind::Other => {
                return Err(StoreError("legacy session state is unsafe"));
            }
        }
    }
    Ok(found)
}

fn inspect_legacy_level(directory: &TrustedDirectory) -> Result<bool, StoreError> {
    let entries = directory
        .entries()
        .map_err(|_| StoreError("enumerate legacy participant state"))?;
    for entry in &entries {
        match entry.kind {
            EntryKind::Directory => {
                directory
                    .open_controlled_directory_entry(entry)
                    .map_err(|_| StoreError("validate legacy participant directory"))?;
            }
            EntryKind::RegularFile => directory
                .revalidate_entry(entry)
                .map_err(|_| StoreError("validate legacy participant record"))?,
            EntryKind::Symlink | EntryKind::Other => {
                return Err(StoreError("legacy participant state is unsafe"));
            }
        }
    }
    Ok(!entries.is_empty())
}
