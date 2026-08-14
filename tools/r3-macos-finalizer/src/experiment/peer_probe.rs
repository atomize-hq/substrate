//! One fixed, well-framed negative peer request with no Security.framework calls.

use std::os::unix::net::UnixStream;
use std::time::Duration;

use anyhow::{bail, Context, Result};
use substrate_common::macos_retirement_v2::{
    document_sha256_v2, parse_canonical_v2, FinalizationRequestV2, FinalizerResponseV2,
    MAC_R3_FINALIZER_ENDPOINT_V2,
};

use crate::frame::{read_one_frame_to_eof, write_one_frame};

use super::controls::{
    PeerProbeExchangeResultV2, PeerProbeTerminationV2, PEER_CONTROL_RECEIPT_OWNER_V2,
};
use super::{EXPERIMENT_ID_V2, EXPERIMENT_VERSION_V2};

const PEER_PROBE_TIMEOUT: Duration = Duration::from_secs(30);

pub fn exchange_one_peer_probe_v2(
    request: &FinalizationRequestV2,
) -> Result<PeerProbeExchangeResultV2> {
    substrate_common::macos_retirement_v2::validate_finalization_request_v2(request, None)?;
    let request_bytes = substrate_common::macos_retirement_v2::canonical_bytes_v2(request)?;
    let mut stream = UnixStream::connect(MAC_R3_FINALIZER_ENDPOINT_V2)
        .context("connect fixed finalizer endpoint for peer-substitution control")?;
    stream
        .set_read_timeout(Some(PEER_PROBE_TIMEOUT))
        .context("set peer-probe response timeout")?;
    stream
        .set_write_timeout(Some(PEER_PROBE_TIMEOUT))
        .context("set peer-probe request timeout")?;
    write_one_frame(&mut stream, &request_bytes)?;
    stream
        .shutdown(std::net::Shutdown::Write)
        .context("send peer-probe request EOF")?;
    let (termination, response) = match read_one_frame_to_eof(&mut stream) {
        Ok(bytes) => (
            PeerProbeTerminationV2::CanonicalSafePreAcceptanceStop,
            Some(parse_canonical_v2::<FinalizerResponseV2>(&bytes)?),
        ),
        Err(error) if is_exact_connection_close(&error) => {
            (PeerProbeTerminationV2::ConnectionClosedBeforeResponse, None)
        }
        Err(error) => {
            return Err(error)
                .context("peer-probe response was neither canonical nor a connection close")
        }
    };
    Ok(PeerProbeExchangeResultV2 {
        schema_owner: PEER_CONTROL_RECEIPT_OWNER_V2.to_string(),
        schema_version: EXPERIMENT_VERSION_V2,
        experiment_id: EXPERIMENT_ID_V2.to_string(),
        request_sha256: document_sha256_v2(request)?,
        request_eof_sent: true,
        termination,
        response,
        response_eof_observed: true,
    })
}

/// Fixed process-level rendezvous used only so the harness can attest the already-exec'd probe
/// before it connects. The harness resumes it with one exact SIGCONT; there is no caller data.
pub fn stop_before_peer_probe_exchange_v2() -> Result<()> {
    // SAFETY: raising SIGSTOP affects only this fixed probe process.
    if unsafe { libc::raise(libc::SIGSTOP) } != 0 {
        return Err(std::io::Error::last_os_error())
            .context("stop fixed peer probe for attestation");
    }
    Ok(())
}

fn is_exact_connection_close(error: &anyhow::Error) -> bool {
    error
        .chain()
        .filter_map(|cause| cause.downcast_ref::<std::io::Error>())
        .any(|cause| {
            matches!(
                cause.kind(),
                std::io::ErrorKind::UnexpectedEof
                    | std::io::ErrorKind::ConnectionReset
                    | std::io::ErrorKind::BrokenPipe
            )
        })
}

pub fn require_no_peer_probe_ambient_input(expected_path: &str) -> Result<()> {
    if std::env::args_os().count() != 1 {
        bail!("peer probe accepts no argv")
    }
    if std::env::current_dir().context("read peer-probe cwd")? != std::path::Path::new("/") {
        bail!("peer-probe cwd is not the fixed root directory")
    }
    use std::os::unix::fs::MetadataExt as _;
    let stdin = std::fs::metadata("/dev/fd/0").context("inspect peer-probe stdin")?;
    let dev_null = std::fs::metadata("/dev/null").context("inspect fixed /dev/null")?;
    if stdin.file_type() != dev_null.file_type()
        || stdin.dev() != dev_null.dev()
        || stdin.ino() != dev_null.ino()
        || stdin.rdev() != dev_null.rdev()
    {
        bail!("peer probe stdin is not the fixed /dev/null EOF source")
    }
    if std::env::vars_os().any(|(key, _)| key.to_string_lossy().starts_with("SUBSTRATE_")) {
        bail!("peer probe rejects ambient SUBSTRATE_* input")
    }
    if std::fs::canonicalize(std::env::current_exe()?)? != std::path::Path::new(expected_path) {
        bail!("peer probe is not running from its one compiled physical path")
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn peer_probe_surface_has_one_request_and_no_selector() {
        let source = include_str!("peer_probe.rs");
        let production = source.split("#[cfg(test)]").next().unwrap();
        assert!(production.contains("args_os().count() != 1"));
        assert!(production.contains("write_one_frame"));
        assert!(production.contains("Shutdown::Write"));
        assert!(production.contains("libc::SIGSTOP"));
        assert!(!production.contains("--action"));
        assert!(!production.contains("--path"));
        assert!(!production.contains("--target"));
        assert!(!production.contains("SecurityUiDenied"));
        assert!(production.contains("/dev/fd/0"));
        let binary = include_str!("../bin/experiment_peer_probe.rs");
        assert!(binary.contains("clear_process_environment()?"));
        let ambient = include_str!("../ambient.rs");
        assert!(ambient.contains("libc::unsetenv(name.as_ptr())"));
        assert!(ambient.contains("vars_os().next().is_some()"));
    }
}
