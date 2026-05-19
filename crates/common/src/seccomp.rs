use anyhow::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeccompAction {
    Log,
    Errno(i32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SeccompRule<'a> {
    pub syscall: &'a str,
    pub action: SeccompAction,
}

impl<'a> SeccompRule<'a> {
    pub const fn log(syscall: &'a str) -> Self {
        Self {
            syscall,
            action: SeccompAction::Log,
        }
    }

    pub const fn errno(syscall: &'a str, errno: i32) -> Self {
        Self {
            syscall,
            action: SeccompAction::Errno(errno),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeccompInstallStatus {
    Installed,
    Unavailable(&'static str),
}

#[cfg(all(target_os = "linux", not(target_env = "musl")))]
mod imp {
    use super::{SeccompAction, SeccompInstallStatus, SeccompRule};
    use anyhow::{anyhow, Context, Result};
    use std::ffi::{c_char, c_int, c_uint, c_void, CStr, CString};
    use std::mem;

    const RTLD_NOW: c_int = 0x0002;
    const SCMP_ACT_ERRNO_MASK: u32 = 0x0005_0000;
    const SCMP_ACT_LOG: u32 = 0x7ffc_0000;
    const SCMP_ACT_ALLOW: u32 = 0x7fff_0000;
    const __NR_SCMP_ERROR: c_int = -1;
    const LIBSECCOMP_NAMES: [&str; 2] = ["libseccomp.so.2", "libseccomp.so"];

    unsafe extern "C" {
        fn dlopen(filename: *const c_char, flags: c_int) -> *mut c_void;
        fn dlclose(handle: *mut c_void) -> c_int;
        fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
        fn dlerror() -> *const c_char;
    }

    type SeccompInit = unsafe extern "C" fn(u32) -> *mut c_void;
    type SeccompRelease = unsafe extern "C" fn(*mut c_void);
    type SeccompLoad = unsafe extern "C" fn(*const c_void) -> c_int;
    type SeccompSyscallResolveName = unsafe extern "C" fn(*const c_char) -> c_int;
    type SeccompRuleAddArray =
        unsafe extern "C" fn(*mut c_void, u32, c_int, c_uint, *const c_void) -> c_int;

    pub(super) fn install_allow_filter(rules: &[SeccompRule<'_>]) -> Result<SeccompInstallStatus> {
        let Some(runtime) = Runtime::load()? else {
            return Ok(SeccompInstallStatus::Unavailable(
                "libseccomp shared library not found",
            ));
        };
        runtime.install_allow_filter(rules)?;
        Ok(SeccompInstallStatus::Installed)
    }

    struct Runtime {
        handle: *mut c_void,
        seccomp_init: SeccompInit,
        seccomp_release: SeccompRelease,
        seccomp_load: SeccompLoad,
        seccomp_syscall_resolve_name: SeccompSyscallResolveName,
        seccomp_rule_add_array: SeccompRuleAddArray,
    }

    impl Runtime {
        fn load() -> Result<Option<Self>> {
            for candidate in LIBSECCOMP_NAMES {
                if let Some(runtime) = Self::load_candidate(candidate)? {
                    return Ok(Some(runtime));
                }
            }
            Ok(None)
        }

        fn load_candidate(candidate: &str) -> Result<Option<Self>> {
            let name = CString::new(candidate).expect("libseccomp candidate name");
            let handle = unsafe {
                clear_dlerror();
                dlopen(name.as_ptr(), RTLD_NOW)
            };
            if handle.is_null() {
                return Ok(None);
            }

            let runtime = unsafe {
                Self {
                    handle,
                    seccomp_init: load_symbol(handle, b"seccomp_init\0")?,
                    seccomp_release: load_symbol(handle, b"seccomp_release\0")?,
                    seccomp_load: load_symbol(handle, b"seccomp_load\0")?,
                    seccomp_syscall_resolve_name: load_symbol(
                        handle,
                        b"seccomp_syscall_resolve_name\0",
                    )?,
                    seccomp_rule_add_array: load_symbol(handle, b"seccomp_rule_add_array\0")?,
                }
            };

            Ok(Some(runtime))
        }

        fn install_allow_filter(&self, rules: &[SeccompRule<'_>]) -> Result<()> {
            let ctx = unsafe { (self.seccomp_init)(SCMP_ACT_ALLOW) };
            if ctx.is_null() {
                return Err(anyhow!("seccomp init failed"));
            }
            let guard = FilterContextGuard {
                ctx,
                release: self.seccomp_release,
            };

            for rule in rules {
                let name = CString::new(rule.syscall)
                    .with_context(|| format!("invalid seccomp syscall name '{}'", rule.syscall))?;
                let syscall = unsafe { (self.seccomp_syscall_resolve_name)(name.as_ptr()) };
                if syscall == __NR_SCMP_ERROR {
                    continue;
                }

                cvt(
                    unsafe {
                        (self.seccomp_rule_add_array)(
                            guard.ctx,
                            action_to_raw(rule.action),
                            syscall,
                            0,
                            std::ptr::null(),
                        )
                    },
                    &format!("seccomp add_rule({})", rule.syscall),
                )?;
            }

            cvt(
                unsafe { (self.seccomp_load)(guard.ctx.cast_const()) },
                "seccomp load",
            )
        }
    }

    impl Drop for Runtime {
        fn drop(&mut self) {
            unsafe {
                let _ = dlclose(self.handle);
            }
        }
    }

    struct FilterContextGuard {
        ctx: *mut c_void,
        release: SeccompRelease,
    }

    impl Drop for FilterContextGuard {
        fn drop(&mut self) {
            unsafe {
                (self.release)(self.ctx);
            }
        }
    }

    unsafe fn load_symbol<T>(handle: *mut c_void, symbol: &[u8]) -> Result<T>
    where
        T: Copy,
    {
        clear_dlerror();
        let ptr = dlsym(handle, symbol.as_ptr().cast());
        if ptr.is_null() {
            return Err(anyhow!(dlerror_message()).context(format!(
                "failed to resolve {}",
                String::from_utf8_lossy(symbol).trim_end_matches('\0')
            )));
        }
        Ok(mem::transmute_copy(&ptr))
    }

    unsafe fn clear_dlerror() {
        let _ = dlerror();
    }

    unsafe fn dlerror_message() -> String {
        let ptr = dlerror();
        if ptr.is_null() {
            "dynamic loader error".to_string()
        } else {
            CStr::from_ptr(ptr).to_string_lossy().into_owned()
        }
    }

    fn action_to_raw(action: SeccompAction) -> u32 {
        match action {
            SeccompAction::Log => SCMP_ACT_LOG,
            SeccompAction::Errno(errno) => SCMP_ACT_ERRNO_MASK | ((errno as u32) & 0xffff),
        }
    }

    fn cvt(rc: c_int, operation: &str) -> Result<()> {
        if rc == 0 {
            return Ok(());
        }
        let errno = if rc < 0 { -rc } else { rc };
        let error = std::io::Error::from_raw_os_error(errno);
        Err(error).with_context(|| operation.to_string())
    }
}

#[cfg(all(target_os = "linux", not(target_env = "musl")))]
pub fn install_allow_filter(rules: &[SeccompRule<'_>]) -> Result<SeccompInstallStatus> {
    imp::install_allow_filter(rules)
}

#[cfg(not(all(target_os = "linux", not(target_env = "musl"))))]
pub fn install_allow_filter(_rules: &[SeccompRule<'_>]) -> Result<SeccompInstallStatus> {
    Ok(SeccompInstallStatus::Unavailable(
        "libseccomp unavailable on this target",
    ))
}
