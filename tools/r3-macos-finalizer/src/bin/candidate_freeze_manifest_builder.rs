use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
use std::path::Path;

use anyhow::{bail, Context, Result};
use substrate_common::macos_retirement_v2::{canonical_bytes_v2, parse_canonical_v2};
use substrate_r3_macos_finalizer::experiment::freeze_manifest::{
    build_candidate_freeze_manifest_v2, build_candidate_freeze_supporting_manifests_v2,
    candidate_freeze_coordinator_provenance_input_v2, candidate_freeze_global_provenance_input_v2,
    CandidateFreezeCoordinatorProvenanceInputV2, CandidateFreezeGlobalProvenanceInputV2,
    CandidateFreezeManifestInputV2, CandidateFreezeManifestV2,
    CANDIDATE_FREEZE_COORDINATOR_BUILD_INPUTS_PATH_V2,
    CANDIDATE_FREEZE_COORDINATOR_PROVENANCE_INPUT_PATH_V2, CANDIDATE_FREEZE_EXTERNAL_FILE_GID_V2,
    CANDIDATE_FREEZE_EXTERNAL_FILE_MODE_V2, CANDIDATE_FREEZE_EXTERNAL_FILE_UID_V2,
    CANDIDATE_FREEZE_GLOBAL_BUILD_INPUTS_PATH_V2, CANDIDATE_FREEZE_GLOBAL_PROVENANCE_INPUT_PATH_V2,
    CANDIDATE_FREEZE_MANIFEST_INPUT_PATH_V2, CANDIDATE_FREEZE_MANIFEST_PATH_V2,
};
use substrate_r3_macos_finalizer::experiment::freeze_provenance::CANDIDATE_FREEZE_SOURCE_HASHES_PATH_V2;

const FILE_MODE: u32 = 0o100400;
const MAX_INPUT_BYTES: usize = 16 * 1024 * 1024;

fn main() -> Result<()> {
    if std::env::args_os().count() != 1 {
        bail!("candidate freeze manifest builder accepts no arguments")
    }
    // SAFETY: geteuid/getegid have no preconditions and do not dereference caller memory.
    if unsafe { libc::geteuid() } != CANDIDATE_FREEZE_EXTERNAL_FILE_UID_V2
        || unsafe { libc::getegid() } != CANDIDATE_FREEZE_EXTERNAL_FILE_GID_V2
    {
        bail!("candidate freeze manifest builder requires exact UID501:staff")
    }
    let input_bytes = read_exact_user_file(Path::new(CANDIDATE_FREEZE_MANIFEST_INPUT_PATH_V2))?;
    let input: CandidateFreezeManifestInputV2 =
        serde_json::from_slice(&input_bytes).context("parse typed candidate manifest input")?;
    let coordinator_provenance_bytes = read_exact_user_file(Path::new(
        CANDIDATE_FREEZE_COORDINATOR_PROVENANCE_INPUT_PATH_V2,
    ))?;
    let coordinator_provenance: CandidateFreezeCoordinatorProvenanceInputV2 =
        serde_json::from_slice(&coordinator_provenance_bytes)
            .context("parse typed coordinator provenance input")?;
    let global_provenance_bytes =
        read_exact_user_file(Path::new(CANDIDATE_FREEZE_GLOBAL_PROVENANCE_INPUT_PATH_V2))?;
    let global_provenance: CandidateFreezeGlobalProvenanceInputV2 =
        serde_json::from_slice(&global_provenance_bytes)
            .context("parse typed global provenance input")?;
    if coordinator_provenance != candidate_freeze_coordinator_provenance_input_v2(&input)
        || global_provenance != candidate_freeze_global_provenance_input_v2(&input)?
    {
        bail!("final candidate input differs from the two prebuild provenance stages")
    }
    let supporting = build_candidate_freeze_supporting_manifests_v2(&input)?;
    for (path, expected) in [
        (
            CANDIDATE_FREEZE_SOURCE_HASHES_PATH_V2,
            canonical_bytes_v2(&supporting.source_hashes)?,
        ),
        (
            CANDIDATE_FREEZE_COORDINATOR_BUILD_INPUTS_PATH_V2,
            canonical_bytes_v2(&supporting.coordinator_build_inputs)?,
        ),
        (
            CANDIDATE_FREEZE_GLOBAL_BUILD_INPUTS_PATH_V2,
            canonical_bytes_v2(&supporting.global_build_inputs)?,
        ),
    ] {
        if read_exact_user_file(Path::new(path))? != expected {
            bail!("prebuild provenance manifest changed before final candidate freeze")
        }
    }
    let manifest = build_candidate_freeze_manifest_v2(input)?;
    let manifest_bytes = canonical_bytes_v2(&manifest)?;
    persist_exact_user_file(
        Path::new(CANDIDATE_FREEZE_MANIFEST_PATH_V2),
        &manifest_bytes,
    )?;
    let reopened = read_exact_user_file(Path::new(CANDIDATE_FREEZE_MANIFEST_PATH_V2))?;
    if reopened != manifest_bytes {
        bail!("candidate freeze manifest changed after durable reopen")
    }
    let parsed: CandidateFreezeManifestV2 = parse_canonical_v2(&reopened)?;
    parsed.validate()?;
    Ok(())
}

fn read_exact_user_file(path: &Path) -> Result<Vec<u8>> {
    let mut file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(path)
        .with_context(|| format!("open fixed candidate freeze file {}", path.display()))?;
    let before = file.metadata().context("fstat candidate freeze file")?;
    if !before.file_type().is_file()
        || before.uid() != CANDIDATE_FREEZE_EXTERNAL_FILE_UID_V2
        || before.gid() != CANDIDATE_FREEZE_EXTERNAL_FILE_GID_V2
        || before.mode() != FILE_MODE
        || before.nlink() != 1
    {
        bail!("candidate freeze file is not one 0400 no-follow user-owned regular file")
    }
    let mut bytes = Vec::new();
    (&mut file)
        .take(u64::try_from(MAX_INPUT_BYTES + 1).expect("bounded limit fits u64"))
        .read_to_end(&mut bytes)
        .context("read candidate freeze file")?;
    if bytes.is_empty() || bytes.len() > MAX_INPUT_BYTES {
        bail!("candidate freeze file is empty or exceeds its fixed bound")
    }
    let after = file.metadata().context("refstat candidate freeze file")?;
    if before.dev() != after.dev()
        || before.ino() != after.ino()
        || before.size() != after.size()
        || before.mtime() != after.mtime()
        || before.mtime_nsec() != after.mtime_nsec()
    {
        bail!("candidate freeze file changed during read")
    }
    Ok(bytes)
}

fn persist_exact_user_file(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path.parent().context("candidate manifest lacks parent")?;
    let mut options = OpenOptions::new();
    options
        .write(true)
        .create_new(true)
        .mode(CANDIDATE_FREEZE_EXTERNAL_FILE_MODE_V2)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC);
    match options.open(path) {
        Ok(mut file) => {
            file.write_all(bytes).context("write candidate manifest")?;
            file.sync_all().context("fsync candidate manifest")?;
            drop(file);
            sync_directory(parent)?;
        }
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            if read_exact_user_file(path)? != bytes {
                bail!("existing candidate manifest has alternate immutable bytes")
            }
        }
        Err(error) => return Err(error).context("create candidate manifest"),
    }
    sync_directory(parent)
}

fn sync_directory(path: &Path) -> Result<()> {
    let directory = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(path)
        .with_context(|| format!("open candidate freeze directory {}", path.display()))?;
    directory
        .sync_all()
        .context("fsync candidate freeze directory")
}
