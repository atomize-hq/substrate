use anyhow::{Context, Result};
use substrate_r3_macos_signer_acl::supervisor::{
    run_disposable_publisher_supervisor, verify_closed_publisher_process_surface,
};
use substrate_r3_macos_signer_acl::NonInteractiveSecurity;

fn main() -> Result<()> {
    // Filesystem/process checks deliberately precede the first Security call.
    verify_closed_publisher_process_surface()?;
    let mut security = NonInteractiveSecurity::establish_first()?;
    // The sealed root runner observes from exec through this post-denial rendezvous, validates the
    // stopped process/code identity, and resumes it once the canonical no-UI report is durable.
    if unsafe { libc::raise(libc::SIGSTOP) } != 0 {
        return Err(std::io::Error::last_os_error())
            .context("enter exact publisher post-interaction-denial SIGSTOP rendezvous");
    }
    run_disposable_publisher_supervisor(&mut security)
}
