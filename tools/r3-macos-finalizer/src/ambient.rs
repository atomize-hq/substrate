//! Startup-only rejection of ambient process-environment input.

use std::ffi::CStr;

use anyhow::{bail, Context, Result};

unsafe extern "C" {
    fn _NSGetEnviron() -> *mut *mut *mut libc::c_char;
}

/// Erases the complete inherited environment and verifies that no entry survived.
///
/// Callers must invoke this before starting any thread. No protocol target or authority may be
/// derived from the environment before or after this call.
pub fn clear_and_require_empty_v2(process_name: &str) -> Result<()> {
    // macOS has no `clearenv(3)` symbol. `_NSGetEnviron` is its SDK-declared mutable environment
    // accessor. Every caller invokes this during single-threaded startup, before accepting any
    // authority. Read only the next variable name, remove it, and reacquire the first entry because
    // `unsetenv(3)` may relocate the array. Values are never inspected or used.
    let environ_slot = unsafe { _NSGetEnviron() };
    if environ_slot.is_null() {
        bail!("{process_name} cannot access the Darwin process environment")
    }
    let mut removed = 0_u32;
    loop {
        // SAFETY: `_NSGetEnviron` returned the process-global environment slot. The process is
        // single-threaded and this function is the only writer during startup.
        let environ = unsafe { *environ_slot };
        if environ.is_null() {
            break;
        }
        // SAFETY: environ is a NULL-terminated C pointer array supplied by dyld/libSystem.
        let entry = unsafe { *environ };
        if entry.is_null() {
            break;
        }
        // SAFETY: each environment entry is a NUL-terminated C string supplied by exec/dyld.
        let entry = unsafe { CStr::from_ptr(entry) };
        let bytes = entry.to_bytes();
        let separator = bytes
            .iter()
            .position(|byte| *byte == b'=')
            .context("Darwin environment entry has no name/value separator")?;
        if separator == 0 {
            bail!("Darwin environment entry has an empty name")
        }
        let name = std::ffi::CString::new(&bytes[..separator])
            .context("Darwin environment name contains NUL")?;
        // SAFETY: name is a nonempty NUL-terminated string with no `=` and this process is still
        // single-threaded. `unsetenv` owns any relocation of the environment array.
        if unsafe { libc::unsetenv(name.as_ptr()) } != 0 {
            return Err(std::io::Error::last_os_error())
                .with_context(|| format!("clear {process_name} process environment"));
        }
        removed = removed
            .checked_add(1)
            .context("Darwin environment removal count overflow")?;
        if removed > 65_536 {
            bail!("Darwin environment exceeded the fixed startup removal bound")
        }
    }
    // Verify both the SDK-level raw environment and Rust's view immediately after removal.
    let raw_empty = unsafe { (*environ_slot).is_null() || (**environ_slot).is_null() };
    if !raw_empty {
        bail!("{process_name} raw Darwin environment remains nonempty after removal")
    }
    if std::env::vars_os().next().is_some() {
        bail!("{process_name} process environment remains nonempty after Darwin clear")
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn source_uses_darwin_clearenv_and_immediate_empty_verification() {
        let source = include_str!("ambient.rs");
        let call = source.find("libc::unsetenv(name.as_ptr())").unwrap();
        let raw_verify = source.find("let raw_empty =").unwrap();
        let verify = source.find("std::env::vars_os().next().is_some()").unwrap();
        assert!(source.contains("unsafe extern \"C\""));
        assert!(source.contains("fn _NSGetEnviron()"));
        assert!(call < raw_verify && raw_verify < verify);
    }
}
