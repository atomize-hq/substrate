use anyhow::{bail, Context, Result};
use std::sync::OnceLock;
use world_service::{internal_exec, run_world_service};

#[cfg(target_os = "linux")]
const E3_TRANSITION_CAPS: [u32; 3] = [6, 7, 8];
#[cfg(target_os = "linux")]
const LINUX_CAPABILITY_VERSION_3: u32 = 0x2008_0522;
#[cfg(target_os = "linux")]
const SECURE_NOROOT_LOCKED: i32 = 0x03;

#[cfg(target_os = "linux")]
#[repr(C)]
#[derive(Clone, Copy)]
struct CapabilityHeader {
    version: u32,
    pid: i32,
}

#[cfg(target_os = "linux")]
#[repr(C)]
#[derive(Clone, Copy, Default, Eq, PartialEq)]
struct CapabilityData {
    effective: u32,
    permitted: u32,
    inheritable: u32,
}

#[cfg(target_os = "linux")]
#[derive(Clone, Debug, Eq, PartialEq)]
struct E3ParkedCapabilityStateV1 {
    inheritable: u64,
    permitted: u64,
    effective: u64,
    bounding: u64,
    ambient: u64,
    securebits: i32,
}

#[cfg(target_os = "linux")]
static E3_PARKED_CAPABILITY_STATE: OnceLock<E3ParkedCapabilityStateV1> = OnceLock::new();

fn main() -> Result<()> {
    if std::env::args()
        .nth(1)
        .is_some_and(|arg| arg == internal_exec::LANDLOCK_EXEC_ARG)
    {
        return internal_exec::run_landlock_exec();
    }

    park_e3_child_transition_capabilities_before_runtime()?;
    let runtime = build_primary_runtime_after_e3_capability_parking()?;
    runtime.block_on(run_world_service())
}

#[cfg(target_os = "linux")]
fn park_e3_child_transition_capabilities_before_runtime() -> Result<()> {
    let original = read_capability_state()?;
    let transition_mask = transition_capability_mask();
    if original.inheritable & transition_mask != transition_mask
        || original.permitted & transition_mask != transition_mask
        || original.effective & transition_mask != transition_mask
        || original.bounding & transition_mask != transition_mask
        || original.ambient & transition_mask != transition_mask
        || original.securebits != SECURE_NOROOT_LOCKED
    {
        bail!("E3-D transition capabilities or securebits do not match the installed unit");
    }

    for capability in E3_TRANSITION_CAPS {
        if unsafe {
            libc::prctl(
                libc::PR_CAP_AMBIENT,
                libc::PR_CAP_AMBIENT_LOWER,
                capability,
                0,
                0,
            )
        } != 0
        {
            return Err(std::io::Error::last_os_error())
                .context("lower E3-D transition capability from ambient set");
        }
    }
    let mut data = read_capability_data()?;
    set_effective_mask(&mut data, original.effective & !transition_mask);
    write_capability_data(&data)?;

    let expected = E3ParkedCapabilityStateV1 {
        inheritable: original.inheritable,
        permitted: original.permitted,
        effective: original.effective & !transition_mask,
        bounding: original.bounding,
        ambient: original.ambient & !transition_mask,
        securebits: original.securebits,
    };
    let observed = read_capability_state()?;
    if observed != expected {
        bail!("E3-D transition capability parking readback mismatch");
    }
    E3_PARKED_CAPABILITY_STATE
        .set(expected)
        .map_err(|_| anyhow::anyhow!("E3-D transition capabilities were parked more than once"))
}

#[cfg(not(target_os = "linux"))]
fn park_e3_child_transition_capabilities_before_runtime() -> Result<()> {
    Ok(())
}

#[cfg(target_os = "linux")]
fn verify_e3_transition_capabilities_parked() -> Result<()> {
    let expected = E3_PARKED_CAPABILITY_STATE
        .get()
        .context("E3-D transition capability state was not parked")?;
    let observed = read_capability_state()?;
    if &observed != expected {
        bail!("E3-D runtime thread inherited an invalid capability state");
    }
    Ok(())
}

#[cfg(not(target_os = "linux"))]
fn verify_e3_transition_capabilities_parked() -> Result<()> {
    Ok(())
}

fn build_primary_runtime_after_e3_capability_parking() -> Result<tokio::runtime::Runtime> {
    verify_e3_transition_capabilities_parked()?;
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .on_thread_start(|| {
            if let Err(error) = verify_e3_transition_capabilities_parked() {
                eprintln!("fatal E3-D worker capability mismatch: {error:#}");
                std::process::abort();
            }
        })
        .build()
        .context("build primary world-service runtime after E3-D capability parking")
}

#[cfg(target_os = "linux")]
fn transition_capability_mask() -> u64 {
    E3_TRANSITION_CAPS
        .into_iter()
        .fold(0u64, |mask, capability| mask | (1u64 << capability))
}

#[cfg(target_os = "linux")]
fn read_capability_state() -> Result<E3ParkedCapabilityStateV1> {
    let data = read_capability_data()?;
    let cap_last = std::fs::read_to_string("/proc/sys/kernel/cap_last_cap")
        .context("read cap_last_cap")?
        .trim()
        .parse::<u32>()
        .context("parse cap_last_cap")?;
    if cap_last > 63 {
        bail!("E3-D does not support capability numbers above 63");
    }
    let mut bounding = 0u64;
    let mut ambient = 0u64;
    for capability in 0..=cap_last {
        let bounded = unsafe { libc::prctl(libc::PR_CAPBSET_READ, capability, 0, 0, 0) };
        if bounded < 0 {
            return Err(std::io::Error::last_os_error()).context("read capability bounding set");
        }
        if bounded == 1 {
            bounding |= 1u64 << capability;
        }
        let ambient_member = unsafe {
            libc::prctl(
                libc::PR_CAP_AMBIENT,
                libc::PR_CAP_AMBIENT_IS_SET,
                capability,
                0,
                0,
            )
        };
        if ambient_member < 0 {
            return Err(std::io::Error::last_os_error()).context("read capability ambient set");
        }
        if ambient_member == 1 {
            ambient |= 1u64 << capability;
        }
    }
    let securebits = unsafe { libc::prctl(libc::PR_GET_SECUREBITS, 0, 0, 0, 0) };
    if securebits < 0 {
        return Err(std::io::Error::last_os_error()).context("read service securebits");
    }
    Ok(E3ParkedCapabilityStateV1 {
        inheritable: u64::from(data[0].inheritable) | (u64::from(data[1].inheritable) << 32),
        permitted: u64::from(data[0].permitted) | (u64::from(data[1].permitted) << 32),
        effective: u64::from(data[0].effective) | (u64::from(data[1].effective) << 32),
        bounding,
        ambient,
        securebits,
    })
}

#[cfg(target_os = "linux")]
fn read_capability_data() -> Result<[CapabilityData; 2]> {
    let mut header = CapabilityHeader {
        version: LINUX_CAPABILITY_VERSION_3,
        pid: 0,
    };
    let mut data = [CapabilityData::default(); 2];
    if unsafe { libc::syscall(libc::SYS_capget, &mut header, data.as_mut_ptr()) } != 0 {
        return Err(std::io::Error::last_os_error()).context("capget primary service thread");
    }
    Ok(data)
}

#[cfg(target_os = "linux")]
fn write_capability_data(data: &[CapabilityData; 2]) -> Result<()> {
    let mut header = CapabilityHeader {
        version: LINUX_CAPABILITY_VERSION_3,
        pid: 0,
    };
    if unsafe { libc::syscall(libc::SYS_capset, &mut header, data.as_ptr()) } != 0 {
        return Err(std::io::Error::last_os_error()).context("capset primary service thread");
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn set_effective_mask(data: &mut [CapabilityData; 2], mask: u64) {
    data[0].effective = mask as u32;
    data[1].effective = (mask >> 32) as u32;
}

#[cfg(test)]
mod tests {
    #[cfg(target_os = "linux")]
    #[test]
    fn transition_capability_mask_is_exact() {
        assert_eq!(
            super::transition_capability_mask(),
            (1 << 6) | (1 << 7) | (1 << 8)
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    #[ignore = "requires the explicit privileged E3-D acceptance environment"]
    fn privileged_parking_is_runtime_read_back_before_and_after_thread_creation() {
        assert_eq!(
            std::env::var_os("SUBSTRATE_E3D_PRIVILEGED_TEST").as_deref(),
            Some(std::ffi::OsStr::new("1")),
            "privileged E3-D acceptance must be explicitly enabled"
        );
        assert_eq!(unsafe { libc::geteuid() }, 0);
        let pid = unsafe { libc::fork() };
        assert!(pid >= 0);
        if pid == 0 {
            assert_eq!(std::fs::read_dir("/proc/self/task").unwrap().count(), 1);
            let mut capabilities = super::read_capability_data().unwrap();
            let transition = super::transition_capability_mask();
            capabilities[0].inheritable |= transition as u32;
            capabilities[1].inheritable |= (transition >> 32) as u32;
            capabilities[0].permitted |= transition as u32;
            capabilities[1].permitted |= (transition >> 32) as u32;
            capabilities[0].effective |= transition as u32;
            capabilities[1].effective |= (transition >> 32) as u32;
            super::write_capability_data(&capabilities).unwrap();
            for capability in super::E3_TRANSITION_CAPS {
                assert_eq!(
                    unsafe {
                        libc::prctl(
                            libc::PR_CAP_AMBIENT,
                            libc::PR_CAP_AMBIENT_RAISE,
                            capability,
                            0,
                            0,
                        )
                    },
                    0
                );
            }
            assert_eq!(
                unsafe { libc::prctl(libc::PR_SET_SECUREBITS, super::SECURE_NOROOT_LOCKED) },
                0
            );
            super::park_e3_child_transition_capabilities_before_runtime().unwrap();
            super::verify_e3_transition_capabilities_parked().unwrap();
            std::thread::spawn(|| super::verify_e3_transition_capabilities_parked().unwrap())
                .join()
                .unwrap();
            unsafe { libc::_exit(0) };
        }
        let mut status = 0;
        assert_eq!(unsafe { libc::waitpid(pid, &mut status, 0) }, pid);
        assert!(libc::WIFEXITED(status));
        assert_eq!(libc::WEXITSTATUS(status), 0);
    }
}
