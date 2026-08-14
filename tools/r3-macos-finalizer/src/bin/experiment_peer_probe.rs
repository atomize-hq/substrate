#[cfg(not(target_os = "macos"))]
fn main() {
    eprintln!("substrate R3 macOS peer-code probe is available only on macOS");
    std::process::exit(78);
}

#[cfg(target_os = "macos")]
fn main() -> anyhow::Result<()> {
    macos::run()
}

#[cfg(target_os = "macos")]
mod macos {
    use std::fs::OpenOptions;
    use std::io::{Read, Write};
    use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
    use std::path::Path;

    use anyhow::{bail, Context, Result};
    use serde::{de::DeserializeOwned, Serialize};
    use substrate_common::macos_retirement_v2::{
        canonical_bytes_v2, parse_canonical_v2, FinalizationRequestV2,
        MAC_R3_FINALIZER_REQUEST_PATH_V2,
    };
    use substrate_r3_macos_finalizer::experiment::peer_probe::{
        exchange_one_peer_probe_v2, require_no_peer_probe_ambient_input,
        stop_before_peer_probe_exchange_v2,
    };
    use substrate_r3_macos_finalizer::experiment::process::attest_fixed_peer_process_v2;
    use substrate_r3_macos_finalizer::experiment::publisher_protocol::CandidateIdentityPacketV2;
    use substrate_r3_macos_finalizer::experiment::{
        controls::PeerControlIdentityPacketV2, CANDIDATE_IDENTITY_PACKET_PATH_V2,
        DISPOSABLE_HARNESS_ACCOUNT_V2, DISPOSABLE_HARNESS_GID_V2, DISPOSABLE_HARNESS_UID_V2,
        PEER_CODE_PROBE_PATH_V2, PEER_CONTROL_IDENTITY_PACKET_PATH_V2,
    };
    use substrate_r3_macos_finalizer::fixed_inbox::read_fixed_coordinator_inbox_v2;
    use substrate_r3_macos_finalizer::frame::write_one_frame;

    pub fn run() -> Result<()> {
        require_no_peer_probe_ambient_input(PEER_CODE_PROBE_PATH_V2)?;
        let candidate: CandidateIdentityPacketV2 =
            read_installed_packet(Path::new(CANDIDATE_IDENTITY_PACKET_PATH_V2))?;
        let peer_identities: PeerControlIdentityPacketV2 =
            read_installed_packet(Path::new(PEER_CONTROL_IDENTITY_PACKET_PATH_V2))?;
        peer_identities.validate(&candidate)?;
        attest_fixed_peer_process_v2(
            i32::try_from(std::process::id()).context("peer-code probe PID exceeds i32")?,
            &peer_identities.alternate_code_identity,
            DISPOSABLE_HARNESS_UID_V2,
            DISPOSABLE_HARNESS_GID_V2,
            DISPOSABLE_HARNESS_ACCOUNT_V2,
        )?;
        clear_process_environment()?;
        let request_bytes = read_request()?;
        let request: FinalizationRequestV2 = parse_canonical_v2(&request_bytes)?;
        stop_before_peer_probe_exchange_v2()?;
        let result = exchange_one_peer_probe_v2(&request)?;
        write_one_frame(&mut std::io::stdout(), &canonical_bytes_v2(&result)?)?;
        std::io::stdout()
            .flush()
            .context("flush peer-code probe receipt")
    }

    fn read_request() -> Result<Vec<u8>> {
        read_fixed_coordinator_inbox_v2(Path::new(MAC_R3_FINALIZER_REQUEST_PATH_V2))
    }

    fn clear_process_environment() -> Result<()> {
        substrate_r3_macos_finalizer::ambient::clear_and_require_empty_v2("peer-code probe")
    }

    fn read_installed_packet<T: DeserializeOwned + Serialize>(path: &Path) -> Result<T> {
        let mut current = std::path::PathBuf::from("/");
        for component in path.components().skip(1) {
            current.push(component.as_os_str());
            let metadata = std::fs::symlink_metadata(&current)
                .with_context(|| format!("inspect installed probe packet {}", current.display()))?;
            if metadata.file_type().is_symlink()
                || metadata.uid() != 0
                || metadata.mode() & 0o022 != 0
            {
                bail!("installed probe packet path is not root-owned immutable no-follow state")
            }
            if current == path {
                if !metadata.file_type().is_file()
                    || metadata.mode() & 0o7777 != 0o444
                    || metadata.nlink() != 1
                {
                    bail!("installed probe packet is not one root-owned 0444 regular file")
                }
            } else if !metadata.is_dir() {
                bail!("installed probe packet parent is not a directory")
            }
        }
        let mut file = OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
            .open(path)
            .with_context(|| format!("open installed probe packet {}", path.display()))?;
        let before = file.metadata().context("fstat installed probe packet")?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)
            .context("read installed probe packet")?;
        let after = file.metadata().context("refstat installed probe packet")?;
        if before.dev() != after.dev()
            || before.ino() != after.ino()
            || before.size() != after.size()
            || before.mtime() != after.mtime()
            || before.mtime_nsec() != after.mtime_nsec()
        {
            bail!("installed probe packet changed while reading")
        }
        parse_canonical_v2(&bytes)
    }
}
