#![cfg(target_os = "linux")]

use std::fs::File;
use std::io::{Read, Write};
use std::mem::zeroed;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd, RawFd};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use config_projection::{E3ElfExecutionModelV1, LinuxArtifactSourceV1};

const CANARY: u64 = 0x5a17_c0de_32ef_9001;
const LINUX_CAPABILITY_VERSION_3: u32 = 0x2008_0522;
const EXEC_TARGET_ENV: &str = "SUBSTRATE_E3D_EXEC_TARGET";
const TRACE_EXEC_TARGET_ENV: &str = "SUBSTRATE_E3D_TRACE_EXEC_TARGET";

#[repr(C)]
struct UserCapHeader {
    version: u32,
    pid: i32,
}

#[repr(C)]
struct UserCapData {
    effective: u32,
    permitted: u32,
    inheritable: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct TargetIdentity {
    address: usize,
    descriptor: RawFd,
    reserved: u32,
}

struct TargetProcess {
    pid: libc::pid_t,
    release: OwnedFd,
    identity: TargetIdentity,
}

impl Drop for TargetProcess {
    fn drop(&mut self) {
        let byte = [0x01_u8];
        unsafe {
            libc::write(self.release.as_raw_fd(), byte.as_ptr().cast(), 1);
        }
        let mut status = 0;
        assert_eq!(unsafe { libc::waitpid(self.pid, &mut status, 0) }, self.pid);
        assert!(libc::WIFEXITED(status));
        assert_eq!(libc::WEXITSTATUS(status), 0);
    }
}

fn pipe() -> (OwnedFd, OwnedFd) {
    let mut descriptors = [-1; 2];
    assert_eq!(
        unsafe { libc::pipe2(descriptors.as_mut_ptr(), libc::O_CLOEXEC) },
        0
    );
    unsafe {
        (
            OwnedFd::from_raw_fd(descriptors[0]),
            OwnedFd::from_raw_fd(descriptors[1]),
        )
    }
}

fn write_all(fd: RawFd, mut bytes: &[u8]) {
    while !bytes.is_empty() {
        let count = unsafe { libc::write(fd, bytes.as_ptr().cast(), bytes.len()) };
        assert!(
            count > 0,
            "write target handshake: {}",
            std::io::Error::last_os_error()
        );
        bytes = &bytes[count as usize..];
    }
}

fn read_all(fd: RawFd, mut bytes: &mut [u8]) {
    while !bytes.is_empty() {
        let count = unsafe { libc::read(fd, bytes.as_mut_ptr().cast(), bytes.len()) };
        assert!(
            count > 0,
            "read target handshake: {}",
            std::io::Error::last_os_error()
        );
        bytes = &mut bytes[count as usize..];
    }
}

fn drop_to_exact_nonroot_identity(target_uid: u32, target_gid: u32) {
    let last_cap: u32 = std::fs::read_to_string("/proc/sys/kernel/cap_last_cap")
        .unwrap()
        .trim()
        .parse()
        .unwrap();
    for capability in 0..=last_cap {
        assert_eq!(unsafe { libc::prctl(libc::PR_CAPBSET_DROP, capability) }, 0);
    }
    assert_eq!(unsafe { libc::setgroups(0, std::ptr::null()) }, 0);
    assert_eq!(
        unsafe { libc::setresgid(target_gid, target_gid, target_gid) },
        0
    );
    assert_eq!(
        unsafe { libc::setresuid(target_uid, target_uid, target_uid) },
        0
    );
    let header = UserCapHeader {
        version: LINUX_CAPABILITY_VERSION_3,
        pid: 0,
    };
    let data = [
        UserCapData {
            effective: 0,
            permitted: 0,
            inheritable: 0,
        },
        UserCapData {
            effective: 0,
            permitted: 0,
            inheritable: 0,
        },
    ];
    assert_eq!(
        unsafe {
            libc::syscall(
                libc::SYS_capset,
                &header as *const UserCapHeader,
                data.as_ptr(),
            )
        },
        0
    );
    let status = std::fs::read_to_string("/proc/self/status").unwrap();
    for field in ["CapInh:", "CapPrm:", "CapEff:", "CapBnd:", "CapAmb:"] {
        assert!(
            status
                .lines()
                .find(|line| line.starts_with(field))
                .is_some_and(|line| line.ends_with("0000000000000000")),
            "same-UID denial process must have zero {field}"
        );
    }
}

fn run_exec_target() {
    let target_uid: u32 = std::env::var("SUBSTRATE_E3D_EXEC_TARGET_UID")
        .unwrap()
        .parse()
        .unwrap();
    let target_gid: u32 = std::env::var("SUBSTRATE_E3D_EXEC_TARGET_GID")
        .unwrap()
        .parse()
        .unwrap();
    let authorized_tracer_pid = std::env::var("SUBSTRATE_E3D_EXEC_TARGET_TRACER_PID")
        .unwrap()
        .parse::<u32>()
        .unwrap();
    let identity_writer = std::env::var("SUBSTRATE_E3D_EXEC_TARGET_IDENTITY_FD")
        .unwrap()
        .parse::<RawFd>()
        .unwrap();
    let release_reader = std::env::var("SUBSTRATE_E3D_EXEC_TARGET_RELEASE_FD")
        .unwrap()
        .parse::<RawFd>()
        .unwrap();
    assert_eq!(unsafe { libc::getuid() }, target_uid);
    assert_eq!(unsafe { libc::geteuid() }, target_uid);
    assert_eq!(unsafe { libc::getgid() }, target_gid);
    assert_eq!(unsafe { libc::getegid() }, target_gid);
    let status = std::fs::read_to_string("/proc/self/status").unwrap();
    for field in ["CapInh:", "CapPrm:", "CapEff:", "CapBnd:", "CapAmb:"] {
        assert!(
            status
                .lines()
                .find(|line| line.starts_with(field))
                .is_some_and(|line| line.ends_with("0000000000000000")),
            "post-exec same-UID denial target must have zero {field}"
        );
    }
    if let Some(path) = std::env::var_os("SUBSTRATE_E3D_EXEC_TARGET_USER_FILE") {
        let mut file = std::fs::OpenOptions::new()
            .read(true)
            .append(true)
            .open(path)
            .unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        assert_eq!(contents, "owned-by-target\n");
        file.write_all(b"written-in-userns\n").unwrap();
    }
    assert_eq!(
        unsafe {
            libc::prctl(
                libc::PR_SET_PTRACER,
                authorized_tracer_pid as libc::c_ulong,
                0,
                0,
                0,
            )
        },
        0
    );
    assert_eq!(unsafe { libc::prctl(libc::PR_SET_DUMPABLE, 1, 0, 0, 0) }, 0);
    assert_eq!(unsafe { libc::prctl(libc::PR_GET_DUMPABLE, 0, 0, 0, 0) }, 1);
    let canary = CANARY;
    let held = File::open("/dev/null").unwrap();
    let identity = TargetIdentity {
        address: (&canary as *const u64) as usize,
        descriptor: held.as_raw_fd(),
        reserved: 0,
    };
    let identity_bytes = unsafe {
        std::slice::from_raw_parts(
            (&identity as *const TargetIdentity).cast::<u8>(),
            std::mem::size_of::<TargetIdentity>(),
        )
    };
    write_all(identity_writer, identity_bytes);
    unsafe { libc::close(identity_writer) };
    let mut release = 0u8;
    assert_eq!(
        unsafe { libc::read(release_reader, (&mut release as *mut u8).cast(), 1) },
        1
    );
    assert_eq!(release, 0x01);
    std::hint::black_box(canary);
    std::hint::black_box(held);
}

fn clear_close_on_exec(fd: RawFd) {
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFD) };
    assert!(flags >= 0);
    assert_eq!(
        unsafe { libc::fcntl(fd, libc::F_SETFD, flags & !libc::FD_CLOEXEC) },
        0
    );
}

fn exec_target_process(
    identity_writer: RawFd,
    release_reader: RawFd,
    authorized_tracer_pid: u32,
    target_uid: u32,
    target_gid: u32,
    user_file: Option<&Path>,
) -> ! {
    clear_close_on_exec(identity_writer);
    clear_close_on_exec(release_reader);
    let executable = std::env::current_exe().unwrap();
    let executable = std::ffi::CString::new(executable.as_os_str().as_bytes()).unwrap();
    let arguments = [
        executable.clone(),
        std::ffi::CString::new("--exact").unwrap(),
        std::ffi::CString::new(
            "e3d_same_uid_namespace_denial_has_conclusive_controls_and_leaves_yama_unchanged",
        )
        .unwrap(),
        std::ffi::CString::new("--include-ignored").unwrap(),
        std::ffi::CString::new("--nocapture").unwrap(),
    ];
    let mut environment = vec![
        std::ffi::CString::new(format!("{EXEC_TARGET_ENV}=1")).unwrap(),
        std::ffi::CString::new(format!(
            "SUBSTRATE_E3D_EXEC_TARGET_IDENTITY_FD={identity_writer}"
        ))
        .unwrap(),
        std::ffi::CString::new(format!(
            "SUBSTRATE_E3D_EXEC_TARGET_RELEASE_FD={release_reader}"
        ))
        .unwrap(),
        std::ffi::CString::new(format!(
            "SUBSTRATE_E3D_EXEC_TARGET_TRACER_PID={authorized_tracer_pid}"
        ))
        .unwrap(),
        std::ffi::CString::new(format!("SUBSTRATE_E3D_EXEC_TARGET_UID={target_uid}")).unwrap(),
        std::ffi::CString::new(format!("SUBSTRATE_E3D_EXEC_TARGET_GID={target_gid}")).unwrap(),
        std::ffi::CString::new("RUST_TEST_THREADS=1").unwrap(),
    ];
    if let Some(path) = user_file {
        let mut value = b"SUBSTRATE_E3D_EXEC_TARGET_USER_FILE=".to_vec();
        value.extend_from_slice(path.as_os_str().as_bytes());
        environment.push(std::ffi::CString::new(value).unwrap());
    }
    let mut argument_pointers = arguments
        .iter()
        .map(|value| value.as_ptr())
        .collect::<Vec<_>>();
    argument_pointers.push(std::ptr::null());
    let mut environment_pointers = environment
        .iter()
        .map(|value| value.as_ptr())
        .collect::<Vec<_>>();
    environment_pointers.push(std::ptr::null());
    drop_to_exact_nonroot_identity(target_uid, target_gid);
    unsafe {
        libc::execve(
            executable.as_ptr(),
            argument_pointers.as_ptr(),
            environment_pointers.as_ptr(),
        );
        libc::_exit(114);
    }
}

fn run_trace_exec_target() {
    let target_uid: u32 = std::env::var("SUBSTRATE_E3D_TRACE_TARGET_UID")
        .unwrap()
        .parse()
        .unwrap();
    let target_gid: u32 = std::env::var("SUBSTRATE_E3D_TRACE_TARGET_GID")
        .unwrap()
        .parse()
        .unwrap();
    let tracer_pid: u32 = std::env::var("SUBSTRATE_E3D_TRACE_PARENT_PID")
        .unwrap()
        .parse()
        .unwrap();
    assert_eq!(unsafe { libc::getuid() }, target_uid);
    assert_eq!(unsafe { libc::geteuid() }, target_uid);
    assert_eq!(unsafe { libc::getgid() }, target_gid);
    assert_eq!(unsafe { libc::getegid() }, target_gid);
    assert_eq!(unsafe { libc::prctl(libc::PR_GET_DUMPABLE) }, 1);
    let status = std::fs::read_to_string("/proc/self/status").unwrap();
    let observed_tracer = status
        .lines()
        .find_map(|line| line.strip_prefix("TracerPid:"))
        .unwrap()
        .trim()
        .parse::<u32>()
        .unwrap();
    assert_eq!(observed_tracer, tracer_pid);
    assert!(status
        .lines()
        .find_map(|line| line.strip_prefix("NoNewPrivs:"))
        .is_some_and(|value| value.trim() == "1"));
    for field in ["CapInh:", "CapPrm:", "CapEff:", "CapBnd:", "CapAmb:"] {
        assert!(status
            .lines()
            .find(|line| line.starts_with(field))
            .is_some_and(|line| line.ends_with("0000000000000000")));
    }
}

fn exec_trace_target(
    target_uid: u32,
    target_gid: u32,
    tracer_tid: u32,
    setup_writer: RawFd,
    final_reader: RawFd,
) -> ! {
    let executable = std::env::current_exe().unwrap();
    let executable = std::ffi::CString::new(executable.as_os_str().as_bytes()).unwrap();
    let arguments = [
        executable.clone(),
        std::ffi::CString::new("--exact").unwrap(),
        std::ffi::CString::new("e3d_trusted_trace_relationship_survives_actual_exec_ordering")
            .unwrap(),
        std::ffi::CString::new("--include-ignored").unwrap(),
        std::ffi::CString::new("--nocapture").unwrap(),
    ];
    let environment = [
        std::ffi::CString::new(format!("{TRACE_EXEC_TARGET_ENV}=1")).unwrap(),
        std::ffi::CString::new(format!("SUBSTRATE_E3D_TRACE_TARGET_UID={target_uid}")).unwrap(),
        std::ffi::CString::new(format!("SUBSTRATE_E3D_TRACE_TARGET_GID={target_gid}")).unwrap(),
        std::ffi::CString::new(format!("SUBSTRATE_E3D_TRACE_PARENT_PID={tracer_tid}")).unwrap(),
        std::ffi::CString::new("RUST_TEST_THREADS=1").unwrap(),
    ];
    let mut argument_pointers = arguments
        .iter()
        .map(|value| value.as_ptr())
        .collect::<Vec<_>>();
    argument_pointers.push(std::ptr::null());
    let mut environment_pointers = environment
        .iter()
        .map(|value| value.as_ptr())
        .collect::<Vec<_>>();
    environment_pointers.push(std::ptr::null());
    drop_to_exact_nonroot_identity(target_uid, target_gid);
    assert_eq!(
        unsafe { libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) },
        0
    );
    assert_eq!(
        unsafe { libc::prctl(libc::PR_SET_PTRACER, tracer_tid as libc::c_ulong, 0, 0, 0) },
        0
    );
    assert_eq!(unsafe { libc::ptrace(libc::PTRACE_TRACEME, 0, 0, 0) }, 0);
    assert_eq!(unsafe { libc::prctl(libc::PR_SET_DUMPABLE, 0, 0, 0, 0) }, 0);
    assert_eq!(unsafe { libc::prctl(libc::PR_GET_DUMPABLE) }, 0);
    write_all(setup_writer, &[0x01]);
    let mut final_release = [0_u8; 1];
    read_all(final_reader, &mut final_release);
    assert_eq!(final_release, [0x02]);
    assert_eq!(unsafe { libc::prctl(libc::PR_SET_DUMPABLE, 1, 0, 0, 0) }, 0);
    unsafe {
        libc::execve(
            executable.as_ptr(),
            argument_pointers.as_ptr(),
            environment_pointers.as_ptr(),
        );
        libc::_exit(115);
    }
}

fn spawn_target(
    protected_user_namespace: bool,
    authorized_tracer_pid: u32,
    target_uid: u32,
    target_gid: u32,
    user_file: Option<&Path>,
) -> TargetProcess {
    let (identity_reader, identity_writer) = pipe();
    let (release_reader, release_writer) = pipe();
    let (namespace_ready_reader, namespace_ready_writer) = pipe();
    let (mapped_reader, mapped_writer) = pipe();
    let pid = unsafe { libc::fork() };
    assert!(pid >= 0);
    if pid == 0 {
        drop(identity_reader);
        drop(release_writer);
        drop(namespace_ready_reader);
        drop(mapped_writer);
        if protected_user_namespace {
            if unsafe { libc::unshare(libc::CLONE_NEWUSER) } != 0 {
                unsafe { libc::_exit(110) };
            }
            write_all(namespace_ready_writer.as_raw_fd(), &[0x01]);
            let mut mapped = [0_u8; 1];
            read_all(mapped_reader.as_raw_fd(), &mut mapped);
            if mapped != [0x02] {
                unsafe { libc::_exit(111) };
            }
        }
        drop(namespace_ready_writer);
        drop(mapped_reader);
        exec_target_process(
            identity_writer.as_raw_fd(),
            release_reader.as_raw_fd(),
            authorized_tracer_pid,
            target_uid,
            target_gid,
            user_file,
        );
    }
    drop(identity_writer);
    drop(release_reader);
    drop(namespace_ready_writer);
    drop(mapped_reader);
    if protected_user_namespace {
        let mut ready = [0_u8; 1];
        read_all(namespace_ready_reader.as_raw_fd(), &mut ready);
        assert_eq!(ready, [0x01]);
        std::fs::write(
            format!("/proc/{pid}/uid_map"),
            format!("{target_uid} {target_uid} 1\n"),
        )
        .unwrap();
        std::fs::write(
            format!("/proc/{pid}/gid_map"),
            format!("{target_gid} {target_gid} 1\n"),
        )
        .unwrap();
        write_all(mapped_writer.as_raw_fd(), &[0x02]);
    }
    drop(namespace_ready_reader);
    drop(mapped_writer);
    let mut identity = TargetIdentity {
        address: 0,
        descriptor: -1,
        reserved: 0,
    };
    let identity_bytes = unsafe {
        std::slice::from_raw_parts_mut(
            (&mut identity as *mut TargetIdentity).cast::<u8>(),
            std::mem::size_of::<TargetIdentity>(),
        )
    };
    read_all(identity_reader.as_raw_fd(), identity_bytes);
    assert_ne!(identity.address, 0);
    assert!(identity.descriptor >= 3);
    TargetProcess {
        pid,
        release: release_writer,
        identity,
    }
}

fn process_vm_read_canary(pid: libc::pid_t, identity: TargetIdentity) -> Result<u64, i32> {
    let mut value = 0u64;
    let local = libc::iovec {
        iov_base: (&mut value as *mut u64).cast(),
        iov_len: std::mem::size_of::<u64>(),
    };
    let remote = libc::iovec {
        iov_base: identity.address as *mut libc::c_void,
        iov_len: std::mem::size_of::<u64>(),
    };
    let count = unsafe { libc::process_vm_readv(pid, &local, 1, &remote, 1, 0) };
    if count == std::mem::size_of::<u64>() as isize {
        Ok(value)
    } else {
        Err(std::io::Error::last_os_error().raw_os_error().unwrap_or(0))
    }
}

fn process_vm_write_canary(pid: libc::pid_t, identity: TargetIdentity) -> Result<(), i32> {
    let replacement = CANARY ^ u64::MAX;
    let local = libc::iovec {
        iov_base: (&replacement as *const u64).cast_mut().cast(),
        iov_len: std::mem::size_of::<u64>(),
    };
    let remote = libc::iovec {
        iov_base: identity.address as *mut libc::c_void,
        iov_len: std::mem::size_of::<u64>(),
    };
    let count = unsafe { libc::process_vm_writev(pid, &local, 1, &remote, 1, 0) };
    if count == std::mem::size_of::<u64>() as isize {
        Ok(())
    } else {
        Err(std::io::Error::last_os_error().raw_os_error().unwrap_or(0))
    }
}

fn proc_mem_read_canary(pid: libc::pid_t, identity: TargetIdentity) -> Result<u64, i32> {
    let path = format!("/proc/{pid}/mem");
    let path = std::ffi::CString::new(path).unwrap();
    let fd = unsafe { libc::open(path.as_ptr(), libc::O_RDONLY | libc::O_CLOEXEC) };
    if fd < 0 {
        return Err(std::io::Error::last_os_error().raw_os_error().unwrap_or(0));
    }
    let fd = unsafe { OwnedFd::from_raw_fd(fd) };
    let mut value = 0u64;
    let count = unsafe {
        libc::pread(
            fd.as_raw_fd(),
            (&mut value as *mut u64).cast(),
            std::mem::size_of::<u64>(),
            identity.address as libc::off_t,
        )
    };
    if count == std::mem::size_of::<u64>() as isize {
        Ok(value)
    } else {
        Err(std::io::Error::last_os_error().raw_os_error().unwrap_or(0))
    }
}

fn pidfd_getfd(pid: libc::pid_t, identity: TargetIdentity) -> Result<OwnedFd, (i32, i32)> {
    let pidfd = unsafe { libc::syscall(libc::SYS_pidfd_open, pid, 0) } as RawFd;
    if pidfd < 0 {
        return Err((
            std::io::Error::last_os_error().raw_os_error().unwrap_or(0),
            0,
        ));
    }
    let pidfd = unsafe { OwnedFd::from_raw_fd(pidfd) };
    let duplicated = unsafe {
        libc::syscall(
            libc::SYS_pidfd_getfd,
            pidfd.as_raw_fd(),
            identity.descriptor,
            0,
        )
    } as RawFd;
    if duplicated < 0 {
        Err((
            0,
            std::io::Error::last_os_error().raw_os_error().unwrap_or(0),
        ))
    } else {
        Ok(unsafe { OwnedFd::from_raw_fd(duplicated) })
    }
}

fn ptrace_attach(pid: libc::pid_t) -> Result<(), i32> {
    if unsafe { libc::ptrace(libc::PTRACE_ATTACH, pid, 0, 0) } != 0 {
        return Err(std::io::Error::last_os_error().raw_os_error().unwrap_or(0));
    }
    let mut status = 0;
    assert_eq!(unsafe { libc::waitpid(pid, &mut status, 0) }, pid);
    assert!(libc::WIFSTOPPED(status));
    assert_eq!(unsafe { libc::ptrace(libc::PTRACE_DETACH, pid, 0, 0) }, 0);
    Ok(())
}

fn assert_not_descendant_of(mut pid: libc::pid_t, ancestor: u32) {
    for _ in 0..64 {
        assert_ne!(
            pid as u32, ancestor,
            "inspector remained under trusted tracer"
        );
        if pid <= 1 {
            return;
        }
        let stat = std::fs::read_to_string(format!("/proc/{pid}/stat")).unwrap();
        let suffix = stat.rsplit_once(") ").unwrap().1;
        pid = suffix
            .split_ascii_whitespace()
            .nth(1)
            .unwrap()
            .parse()
            .unwrap();
    }
    panic!("inspector ancestry exceeded the bounded validation depth");
}

#[repr(C)]
#[derive(Clone, Copy)]
struct ProbeRequest {
    pid: libc::pid_t,
    reserved: u32,
    identity: TargetIdentity,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct ProbeOutcome {
    value: u64,
    errno: i32,
    auxiliary_errno: i32,
    succeeded: u8,
    reserved: [u8; 7],
}

fn inspect_with_same_uid_sibling(
    protected: bool,
    operation: u8,
    target_uid: u32,
    target_gid: u32,
) -> ProbeOutcome {
    let (request_reader, request_writer) = pipe();
    let (result_reader, result_writer) = pipe();
    let (pid_reader, pid_writer) = pipe();
    let intermediate_pid = unsafe { libc::fork() };
    assert!(intermediate_pid >= 0);
    if intermediate_pid == 0 {
        drop(pid_reader);
        drop(request_writer);
        drop(result_reader);
        let inspector_pid = unsafe { libc::fork() };
        if inspector_pid < 0 {
            unsafe { libc::_exit(121) };
        }
        if inspector_pid > 0 {
            drop(request_reader);
            drop(result_writer);
            write_all(pid_writer.as_raw_fd(), &inspector_pid.to_ne_bytes());
            unsafe { libc::_exit(0) };
        }
        drop(pid_writer);
        drop_to_exact_nonroot_identity(target_uid, target_gid);
        let mut request = ProbeRequest {
            pid: 0,
            reserved: 0,
            identity: TargetIdentity {
                address: 0,
                descriptor: -1,
                reserved: 0,
            },
        };
        let request_bytes = unsafe {
            std::slice::from_raw_parts_mut(
                (&mut request as *mut ProbeRequest).cast::<u8>(),
                std::mem::size_of::<ProbeRequest>(),
            )
        };
        read_all(request_reader.as_raw_fd(), request_bytes);
        let mut outcome = ProbeOutcome {
            value: 0,
            errno: 0,
            auxiliary_errno: 0,
            succeeded: 0,
            reserved: [0; 7],
        };
        match operation {
            0 => match ptrace_attach(request.pid) {
                Ok(()) => outcome.succeeded = 1,
                Err(errno) => outcome.errno = errno,
            },
            1 => match process_vm_read_canary(request.pid, request.identity) {
                Ok(value) => {
                    outcome.succeeded = 1;
                    outcome.value = value;
                }
                Err(errno) => outcome.errno = errno,
            },
            2 => match proc_mem_read_canary(request.pid, request.identity) {
                Ok(value) => {
                    outcome.succeeded = 1;
                    outcome.value = value;
                }
                Err(errno) => outcome.errno = errno,
            },
            3 => match pidfd_getfd(request.pid, request.identity) {
                Ok(duplicate) => {
                    let mut stat: libc::stat = unsafe { zeroed() };
                    if unsafe { libc::fstat(duplicate.as_raw_fd(), &mut stat) } == 0
                        && stat.st_mode & libc::S_IFMT == libc::S_IFCHR
                    {
                        outcome.succeeded = 1;
                    } else {
                        outcome.errno = std::io::Error::last_os_error().raw_os_error().unwrap_or(0);
                    }
                }
                Err((open_errno, getfd_errno)) => {
                    outcome.errno = getfd_errno;
                    outcome.auxiliary_errno = open_errno;
                }
            },
            4 => match process_vm_write_canary(request.pid, request.identity) {
                Ok(()) => outcome.succeeded = 1,
                Err(errno) => outcome.errno = errno,
            },
            _ => unsafe { libc::_exit(120) },
        }
        let outcome_bytes = unsafe {
            std::slice::from_raw_parts(
                (&outcome as *const ProbeOutcome).cast::<u8>(),
                std::mem::size_of::<ProbeOutcome>(),
            )
        };
        write_all(result_writer.as_raw_fd(), outcome_bytes);
        unsafe { libc::_exit(0) };
    }
    drop(pid_writer);
    drop(request_reader);
    drop(result_writer);
    let mut inspector_pid_bytes = [0_u8; std::mem::size_of::<libc::pid_t>()];
    read_all(pid_reader.as_raw_fd(), &mut inspector_pid_bytes);
    let inspector_pid = libc::pid_t::from_ne_bytes(inspector_pid_bytes);
    assert!(inspector_pid > 0);
    let mut intermediate_status = 0;
    assert_eq!(
        unsafe { libc::waitpid(intermediate_pid, &mut intermediate_status, 0) },
        intermediate_pid
    );
    assert!(libc::WIFEXITED(intermediate_status));
    assert_eq!(libc::WEXITSTATUS(intermediate_status), 0);

    let inspector_pidfd = unsafe { libc::syscall(libc::SYS_pidfd_open, inspector_pid, 0) } as RawFd;
    assert!(
        inspector_pidfd >= 0,
        "open detached inspector pidfd: {}",
        std::io::Error::last_os_error()
    );
    let inspector_pidfd = unsafe { OwnedFd::from_raw_fd(inspector_pidfd) };
    let trusted_service_pid = std::process::id();
    assert_not_descendant_of(inspector_pid, trusted_service_pid);
    let target = spawn_target(
        protected,
        if protected {
            trusted_service_pid
        } else {
            inspector_pid as u32
        },
        target_uid,
        target_gid,
        None,
    );
    let request = ProbeRequest {
        pid: target.pid,
        reserved: 0,
        identity: target.identity,
    };
    let request_bytes = unsafe {
        std::slice::from_raw_parts(
            (&request as *const ProbeRequest).cast::<u8>(),
            std::mem::size_of::<ProbeRequest>(),
        )
    };
    write_all(request_writer.as_raw_fd(), request_bytes);
    drop(request_writer);
    let mut outcome = ProbeOutcome {
        value: 0,
        errno: 0,
        auxiliary_errno: 0,
        succeeded: 0,
        reserved: [0; 7],
    };
    let outcome_bytes = unsafe {
        std::slice::from_raw_parts_mut(
            (&mut outcome as *mut ProbeOutcome).cast::<u8>(),
            std::mem::size_of::<ProbeOutcome>(),
        )
    };
    read_all(result_reader.as_raw_fd(), outcome_bytes);
    let mut inspector_poll = libc::pollfd {
        fd: inspector_pidfd.as_raw_fd(),
        events: libc::POLLIN,
        revents: 0,
    };
    assert_eq!(unsafe { libc::poll(&mut inspector_poll, 1, 1_000) }, 1);
    assert_ne!(inspector_poll.revents & libc::POLLIN, 0);
    drop(target);
    outcome
}

fn assert_positive_control(target_uid: u32, target_gid: u32) {
    for operation in 0..=4 {
        let outcome = inspect_with_same_uid_sibling(false, operation, target_uid, target_gid);
        assert_eq!(
            outcome.succeeded, 1,
            "positive control {operation}: {outcome:?}"
        );
        if operation == 1 || operation == 2 {
            assert_eq!(outcome.value, CANARY);
        }
    }
}

#[test]
#[ignore = "requires the explicit privileged E3-D acceptance environment"]
fn e3d_same_uid_namespace_denial_has_conclusive_controls_and_leaves_yama_unchanged() {
    if std::env::var_os(EXEC_TARGET_ENV).as_deref() == Some(std::ffi::OsStr::new("1")) {
        run_exec_target();
        return;
    }
    assert_eq!(
        std::env::var_os("SUBSTRATE_E3D_PRIVILEGED_TEST").as_deref(),
        Some(std::ffi::OsStr::new("1")),
        "privileged E3-D acceptance must be explicitly enabled"
    );
    assert_eq!(unsafe { libc::geteuid() }, 0, "denial harness must be root");
    let target_uid: u32 = std::env::var("SUBSTRATE_E3D_TARGET_UID")
        .unwrap()
        .parse()
        .unwrap();
    let target_gid: u32 = std::env::var("SUBSTRATE_E3D_TARGET_GID")
        .unwrap()
        .parse()
        .unwrap();
    assert_ne!(target_uid, 0);
    assert_ne!(target_gid, 0);
    let yama_before = std::fs::read("/proc/sys/kernel/yama/ptrace_scope").unwrap();
    assert_positive_control(target_uid, target_gid);

    let user_file_directory = tempfile::tempdir().unwrap();
    let directory_path = user_file_directory.path();
    let directory_c = std::ffi::CString::new(directory_path.as_os_str().as_bytes()).unwrap();
    assert_eq!(
        unsafe { libc::chown(directory_c.as_ptr(), target_uid, target_gid) },
        0
    );
    std::fs::set_permissions(directory_path, std::fs::Permissions::from_mode(0o700)).unwrap();
    let user_file = directory_path.join("target-owned.txt");
    std::fs::write(&user_file, b"owned-by-target\n").unwrap();
    let user_file_c = std::ffi::CString::new(user_file.as_os_str().as_bytes()).unwrap();
    assert_eq!(
        unsafe { libc::chown(user_file_c.as_ptr(), target_uid, target_gid) },
        0
    );
    std::fs::set_permissions(&user_file, std::fs::Permissions::from_mode(0o600)).unwrap();

    let protected_epoch = spawn_target(
        true,
        std::process::id(),
        target_uid,
        target_gid,
        Some(&user_file),
    );
    assert_positive_control(target_uid, target_gid);
    drop(protected_epoch);
    use std::os::unix::fs::MetadataExt;
    let metadata = std::fs::metadata(&user_file).unwrap();
    assert_eq!(metadata.uid(), target_uid);
    assert_eq!(metadata.gid(), target_gid);
    assert_eq!(metadata.mode() & 0o777, 0o600);
    assert_eq!(
        std::fs::read(&user_file).unwrap(),
        b"owned-by-target\nwritten-in-userns\n"
    );
    for operation in 0..=4 {
        let outcome = inspect_with_same_uid_sibling(true, operation, target_uid, target_gid);
        assert_eq!(
            outcome.succeeded, 0,
            "protected probe {operation}: {outcome:?}"
        );
        match operation {
            0 | 1 | 4 => assert_eq!(outcome.errno, libc::EPERM),
            2 => assert!(outcome.errno == libc::EACCES || outcome.errno == libc::EPERM),
            3 => {
                assert_eq!(outcome.auxiliary_errno, 0, "pidfd_open must succeed");
                assert_eq!(outcome.errno, libc::EPERM);
            }
            _ => unreachable!(),
        }
    }
    assert_positive_control(target_uid, target_gid);
    assert_eq!(
        std::fs::read("/proc/sys/kernel/yama/ptrace_scope").unwrap(),
        yama_before
    );
}

#[test]
#[ignore = "requires the explicit privileged E3-D acceptance environment"]
fn e3d_trusted_trace_relationship_survives_actual_exec_ordering() {
    if std::env::var_os(TRACE_EXEC_TARGET_ENV).as_deref() == Some(std::ffi::OsStr::new("1")) {
        run_trace_exec_target();
        return;
    }
    assert_eq!(
        std::env::var_os("SUBSTRATE_E3D_PRIVILEGED_TEST").as_deref(),
        Some(std::ffi::OsStr::new("1")),
        "privileged E3-D acceptance must be explicitly enabled"
    );
    assert_eq!(unsafe { libc::geteuid() }, 0);
    let target_uid: u32 = std::env::var("SUBSTRATE_E3D_TARGET_UID")
        .unwrap()
        .parse()
        .unwrap();
    let target_gid: u32 = std::env::var("SUBSTRATE_E3D_TARGET_GID")
        .unwrap()
        .parse()
        .unwrap();
    let yama_before = std::fs::read("/proc/sys/kernel/yama/ptrace_scope").unwrap();
    let expected_cwd = std::env::current_dir().unwrap();
    let (namespace_ready_reader, namespace_ready_writer) = pipe();
    let (mapped_reader, mapped_writer) = pipe();
    let (setup_reader, setup_writer) = pipe();
    let (final_reader, final_writer) = pipe();
    let tracer_tid = unsafe { libc::syscall(libc::SYS_gettid) as u32 };
    let child_pid = unsafe { libc::fork() };
    assert!(child_pid >= 0);
    if child_pid == 0 {
        drop(namespace_ready_reader);
        drop(mapped_writer);
        drop(setup_reader);
        drop(final_writer);
        if unsafe { libc::unshare(libc::CLONE_NEWUSER) } != 0 {
            unsafe { libc::_exit(116) };
        }
        write_all(namespace_ready_writer.as_raw_fd(), &[0x01]);
        let mut mapped = [0_u8; 1];
        read_all(mapped_reader.as_raw_fd(), &mut mapped);
        if mapped != [0x02] {
            unsafe { libc::_exit(117) };
        }
        drop(namespace_ready_writer);
        drop(mapped_reader);
        exec_trace_target(
            target_uid,
            target_gid,
            tracer_tid,
            setup_writer.as_raw_fd(),
            final_reader.as_raw_fd(),
        );
    }
    drop(namespace_ready_writer);
    drop(mapped_reader);
    drop(setup_writer);
    drop(final_reader);
    let mut namespace_ready = [0_u8; 1];
    read_all(namespace_ready_reader.as_raw_fd(), &mut namespace_ready);
    assert_eq!(namespace_ready, [0x01]);
    std::fs::write(
        format!("/proc/{child_pid}/uid_map"),
        format!("{target_uid} {target_uid} 1\n"),
    )
    .unwrap();
    std::fs::write(
        format!("/proc/{child_pid}/gid_map"),
        format!("{target_gid} {target_gid} 1\n"),
    )
    .unwrap();
    write_all(mapped_writer.as_raw_fd(), &[0x02]);
    drop(mapped_writer);
    let mut setup = [0_u8; 1];
    read_all(setup_reader.as_raw_fd(), &mut setup);
    assert_eq!(setup, [0x01]);
    write_all(final_writer.as_raw_fd(), &[0x02]);
    drop(final_writer);

    let mut status = 0;
    assert_eq!(
        unsafe { libc::waitpid(child_pid, &mut status, 0) },
        child_pid
    );
    assert!(libc::WIFSTOPPED(status));
    assert_eq!(libc::WSTOPSIG(status), libc::SIGTRAP);
    let exec_status = std::fs::read_to_string(format!("/proc/{child_pid}/status")).unwrap();
    let observed_tracer = exec_status
        .lines()
        .find_map(|line| line.strip_prefix("TracerPid:"))
        .unwrap()
        .trim()
        .parse::<u32>()
        .unwrap();
    assert_eq!(observed_tracer, tracer_tid);
    assert_eq!(
        std::fs::read_link(format!("/proc/{child_pid}/cwd")).unwrap(),
        expected_cwd
    );
    let environment = std::fs::read(format!("/proc/{child_pid}/environ")).unwrap();
    assert!(environment
        .split(|byte| *byte == 0)
        .any(|entry| entry == format!("{TRACE_EXEC_TARGET_ENV}=1").as_bytes()));
    let command_line = std::fs::read(format!("/proc/{child_pid}/cmdline")).unwrap();
    assert!(command_line
        .windows(b"e3d_trusted_trace_relationship_survives_actual_exec_ordering".len())
        .any(|window| window == b"e3d_trusted_trace_relationship_survives_actual_exec_ordering"));
    assert_eq!(
        unsafe {
            libc::ptrace(
                libc::PTRACE_SETOPTIONS,
                child_pid,
                0,
                libc::PTRACE_O_TRACEEXIT,
            )
        },
        0
    );
    assert_eq!(
        unsafe { libc::ptrace(libc::PTRACE_CONT, child_pid, 0, 0) },
        0
    );
    assert_eq!(
        unsafe { libc::waitpid(child_pid, &mut status, 0) },
        child_pid
    );
    assert!(libc::WIFSTOPPED(status));
    assert_eq!(libc::WSTOPSIG(status), libc::SIGTRAP);
    assert_eq!(status >> 16, libc::PTRACE_EVENT_EXIT);
    assert_eq!(
        unsafe { libc::ptrace(libc::PTRACE_CONT, child_pid, 0, 0) },
        0
    );
    assert_eq!(
        unsafe { libc::waitpid(child_pid, &mut status, 0) },
        child_pid
    );
    assert!(libc::WIFEXITED(status));
    assert_eq!(libc::WEXITSTATUS(status), 0);
    assert_eq!(
        std::fs::read("/proc/sys/kernel/yama/ptrace_scope").unwrap(),
        yama_before
    );
}

#[test]
#[ignore = "requires explicitly supplied static E3-D artifact paths"]
fn e3d_built_static_artifacts_match_the_exact_elf_support_contract() {
    let wrapper = std::env::var_os("SUBSTRATE_E3D_STATIC_WORLD_ENTRY")
        .expect("explicit static wrapper proof input is required");
    let gateway = std::env::var_os("SUBSTRATE_E3D_STATIC_GATEWAY")
        .expect("static gateway path accompanies the wrapper proof input");
    let installed = tempfile::tempdir().unwrap();
    for (index, path) in [wrapper, gateway].into_iter().enumerate() {
        let installed_path = installed.path().join(format!("artifact-{index}"));
        std::fs::copy(&path, &installed_path).unwrap();
        let file = File::open(&installed_path).unwrap();
        assert!(matches!(
            LinuxArtifactSourceV1::validate_e3_static_elf_v1(&file)
                .unwrap_or_else(|error| panic!("{}: {error:?}", Path::new(&path).display())),
            E3ElfExecutionModelV1::StaticExec | E3ElfExecutionModelV1::StaticPie(_)
        ));
        let mut magic = [0u8; 4];
        File::open(installed_path)
            .unwrap()
            .read_exact(&mut magic)
            .unwrap();
        assert_eq!(&magic, b"\x7fELF");
    }
}
