use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
use std::path::Path;

use anyhow::{bail, Context, Result};
use substrate_common::macos_retirement_v2::canonical_bytes_v2;
use substrate_r3_macos_finalizer::experiment::freeze_manifest::{
    build_candidate_freeze_coordinator_supporting_manifests_v2,
    CandidateFreezeCoordinatorProvenanceInputV2, CANDIDATE_FREEZE_COORDINATOR_BUILD_INPUTS_PATH_V2,
    CANDIDATE_FREEZE_COORDINATOR_PROVENANCE_INPUT_PATH_V2, CANDIDATE_FREEZE_EXTERNAL_FILE_GID_V2,
    CANDIDATE_FREEZE_EXTERNAL_FILE_MODE_V2, CANDIDATE_FREEZE_EXTERNAL_FILE_UID_V2,
};
use substrate_r3_macos_finalizer::experiment::freeze_provenance::CANDIDATE_FREEZE_SOURCE_HASHES_PATH_V2;

const FILE_MODE: u32 = 0o100400;
const MAX_INPUT_BYTES: usize = 16 * 1024 * 1024;

fn main() -> Result<()> {
    if std::env::args_os().count() != 1 {
        bail!("candidate freeze provenance builder accepts no arguments")
    }
    // SAFETY: geteuid/getegid have no preconditions and do not dereference caller memory.
    if unsafe { libc::geteuid() } != CANDIDATE_FREEZE_EXTERNAL_FILE_UID_V2
        || unsafe { libc::getegid() } != CANDIDATE_FREEZE_EXTERNAL_FILE_GID_V2
    {
        bail!("candidate freeze provenance builder requires exact UID501:staff")
    }
    let input_bytes = read_exact_user_file(Path::new(
        CANDIDATE_FREEZE_COORDINATOR_PROVENANCE_INPUT_PATH_V2,
    ))?;
    let input: CandidateFreezeCoordinatorProvenanceInputV2 =
        serde_json::from_slice(&input_bytes).context("parse typed candidate provenance input")?;
    let supporting = build_candidate_freeze_coordinator_supporting_manifests_v2(&input)?;
    for (path, bytes) in [
        (
            CANDIDATE_FREEZE_SOURCE_HASHES_PATH_V2,
            canonical_bytes_v2(&supporting.source_hashes)?,
        ),
        (
            CANDIDATE_FREEZE_COORDINATOR_BUILD_INPUTS_PATH_V2,
            canonical_bytes_v2(&supporting.coordinator_build_inputs)?,
        ),
    ] {
        persist_exact_user_file(Path::new(path), &bytes)?;
        if read_exact_user_file(Path::new(path))? != bytes {
            bail!("candidate provenance output changed after durable reopen")
        }
    }
    Ok(())
}

fn read_exact_user_file(path: &Path) -> Result<Vec<u8>> {
    let mut file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(path)
        .with_context(|| format!("open fixed candidate provenance file {}", path.display()))?;
    let before = file.metadata().context("fstat candidate provenance file")?;
    if !before.file_type().is_file()
        || before.uid() != CANDIDATE_FREEZE_EXTERNAL_FILE_UID_V2
        || before.gid() != CANDIDATE_FREEZE_EXTERNAL_FILE_GID_V2
        || before.mode() != FILE_MODE
        || before.nlink() != 1
    {
        bail!("candidate provenance file is not one UID501:staff 0400 no-follow regular file")
    }
    let mut bytes = Vec::new();
    (&mut file)
        .take(u64::try_from(MAX_INPUT_BYTES + 1).expect("bounded limit fits u64"))
        .read_to_end(&mut bytes)
        .context("read candidate provenance file")?;
    if bytes.is_empty() || bytes.len() > MAX_INPUT_BYTES {
        bail!("candidate provenance file is empty or exceeds its fixed bound")
    }
    let after = file
        .metadata()
        .context("refstat candidate provenance file")?;
    if physical_identity(&before) != physical_identity(&after) {
        bail!("candidate provenance file changed during read")
    }
    Ok(bytes)
}

fn persist_exact_user_file(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path
        .parent()
        .context("candidate provenance output lacks parent")?;
    let mut options = OpenOptions::new();
    options
        .write(true)
        .create_new(true)
        .mode(CANDIDATE_FREEZE_EXTERNAL_FILE_MODE_V2)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC);
    match options.open(path) {
        Ok(mut file) => {
            file.write_all(bytes)
                .context("write candidate provenance output")?;
            file.sync_all()
                .context("fsync candidate provenance output")?;
            drop(file);
            sync_directory(parent)?;
        }
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            if read_exact_user_file(path)? != bytes {
                bail!("existing candidate provenance output has alternate immutable bytes")
            }
        }
        Err(error) => return Err(error).context("create candidate provenance output"),
    }
    sync_directory(parent)
}

fn sync_directory(path: &Path) -> Result<()> {
    let directory = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(path)
        .with_context(|| format!("open candidate provenance directory {}", path.display()))?;
    directory
        .sync_all()
        .context("fsync candidate provenance directory")
}

fn physical_identity(metadata: &std::fs::Metadata) -> (u64, u64, u64, i64, i64) {
    (
        metadata.dev(),
        metadata.ino(),
        metadata.size(),
        metadata.mtime(),
        metadata.mtime_nsec(),
    )
}
