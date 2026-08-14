use anyhow::{bail, Context, Result};
use std::fs::File;
use std::io::Read;
use std::os::fd::FromRawFd;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::{FileTypeExt, MetadataExt};
use std::path::Path;
use substrate_r3_macos_finalizer::experiment::controls::{
    NOBODY_PRINCIPAL_GID_V2, NOBODY_PRINCIPAL_UID_V2,
};
use substrate_r3_macos_finalizer::experiment::NOBODY_OWNER_PROBE_PATH_V2;
use substrate_r3_macos_signer_acl::{
    invoke_compiled_nobody_owner_authority_control, NobodyOwnerProbeCommandV2,
};

fn main() -> Result<()> {
    if std::env::args_os().count() != 1
        || std::env::current_dir()? != Path::new("/")
        || std::env::current_exe()? != Path::new(NOBODY_OWNER_PROBE_PATH_V2)
        || unsafe { libc::geteuid() } != NOBODY_PRINCIPAL_UID_V2
        || unsafe { libc::getegid() } != NOBODY_PRINCIPAL_GID_V2
    {
        bail!("sealed nobody owner probe process identity changed")
    }
    let stdin = std::fs::metadata("/dev/fd/0")?;
    let null = std::fs::metadata("/dev/null")?;
    if stdin.dev() != null.dev() || stdin.ino() != null.ino() || stdin.rdev() != null.rdev() {
        bail!("sealed nobody owner probe stdin is not /dev/null")
    }
    if std::env::vars_os().any(|(key, _)| key.as_bytes().starts_with(b"SUBSTRATE_")) {
        bail!("sealed nobody owner probe rejects SUBSTRATE input")
    }
    for key in std::env::vars_os().map(|(key, _)| key).collect::<Vec<_>>() {
        std::env::remove_var(key);
    }
    if std::env::vars_os().next().is_some() {
        bail!("sealed nobody owner probe environment is not empty")
    }
    let mut descriptor = unsafe { File::from_raw_fd(3) };
    let metadata = descriptor.metadata()?;
    if !metadata.file_type().is_socket() {
        bail!("sealed nobody owner probe FD3 is not its inherited command socket")
    }
    let mut bytes = Vec::new();
    descriptor
        .by_ref()
        .take(8_193)
        .read_to_end(&mut bytes)
        .context("read sealed nobody owner command")?;
    if bytes.is_empty() || bytes.len() > 8_192 {
        bail!("sealed nobody owner command is empty or oversized")
    }
    let command: NobodyOwnerProbeCommandV2 =
        substrate_common::macos_retirement_v2::parse_canonical_v2(&bytes)?;
    command.validate()?;
    // No Security.framework call may precede the root runner's post-exec process measurement.
    if unsafe { libc::raise(libc::SIGSTOP) } != 0 {
        return Err(std::io::Error::last_os_error())
            .context("enter nobody owner probe attestation rendezvous");
    }
    let receipt = invoke_compiled_nobody_owner_authority_control(&command)?;
    println!(
        "{}",
        String::from_utf8(substrate_common::macos_retirement_v2::canonical_bytes_v2(
            &receipt,
        )?)?
    );
    Ok(())
}
