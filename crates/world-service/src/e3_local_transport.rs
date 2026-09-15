//! Connection authentication for the inherited Linux E3 listener.
use std::os::fd::{AsRawFd, FromRawFd};
use std::os::unix::fs::MetadataExt;
use std::sync::Arc;

use config_projection::{ConfigProjectionFailureV1, ConfiguredAcceptedHomeAuthorityV1};
use tokio::net::UnixStream;

use crate::socket_activation::InheritedUnixListener;

pub(crate) struct E3AuthenticatedLinuxUdsListenerV1 {
    pub(crate) descriptor_device_id: u64,
    pub(crate) descriptor_inode: u64,
    filesystem_device_id: u64,
    filesystem_inode: u64,
    pub(crate) expected_peer_uid: u64,
    pub(crate) kernel_boot_id: String,
    configured: Arc<ConfiguredAcceptedHomeAuthorityV1>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    /// Run only in a separate root test process: its private mount namespace masks /run.
    #[test]
    #[ignore = "requires explicit isolated E3-E root acceptance"]
    fn e3_e_inherited_uds_admission_and_peer_negatives() {
        assert_eq!(std::env::var("SUBSTRATE_E3E_UDS_TEST").as_deref(), Ok("1"));
        assert_eq!(unsafe { libc::geteuid() }, 0);
        let configured = Arc::new(
            ConfiguredAcceptedHomeAuthorityV1::from_installed_bootstrap_authority().unwrap(),
        );
        let uid = u32::try_from(configured.intended_uid()).unwrap();
        assert_ne!(
            uid, 0,
            "the authenticated-peer negative needs a non-root installed principal"
        );
        let account = unsafe { libc::getpwuid(uid) };
        assert!(!account.is_null());
        let gid = unsafe { (*account).pw_gid };
        // Only this test thread changes namespace/GID. No installed socket or host mount is changed.
        assert_eq!(unsafe { libc::unshare(libc::CLONE_NEWNS) }, 0);
        assert_eq!(
            unsafe {
                libc::mount(
                    std::ptr::null(),
                    c"/".as_ptr(),
                    std::ptr::null(),
                    libc::MS_REC | libc::MS_PRIVATE,
                    std::ptr::null(),
                )
            },
            0
        );
        assert_eq!(
            unsafe {
                libc::mount(
                    c"tmpfs".as_ptr(),
                    c"/run".as_ptr(),
                    c"tmpfs".as_ptr(),
                    libc::MS_NOSUID | libc::MS_NODEV,
                    c"mode=0755,size=1m".as_ptr().cast(),
                )
            },
            0
        );
        assert_eq!(unsafe { libc::setegid(gid) }, 0);
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        runtime.block_on(async {
            let socket = tokio::net::UnixListener::bind("/run/substrate.sock").unwrap();
            std::fs::set_permissions(
                "/run/substrate.sock",
                std::fs::Permissions::from_mode(0o660),
            )
            .unwrap();
            let mut inherited = InheritedUnixListener {
                fd: socket.as_raw_fd(),
                name: None,
                listener: socket,
            };
            for count in [0, 2] {
                assert!(matches!(
                    E3AuthenticatedLinuxUdsListenerV1::from_inherited(
                        &inherited,
                        count,
                        Arc::clone(&configured)
                    ),
                    Err(ConfigProjectionFailureV1::UnsupportedConfiguration)
                ));
            }
            let fd = inherited.fd;
            inherited.fd += 1;
            assert!(E3AuthenticatedLinuxUdsListenerV1::from_inherited(
                &inherited,
                1,
                Arc::clone(&configured)
            )
            .is_err());
            inherited.fd = fd;
            for mode in [0o600, 0o666, 0o1660] {
                std::fs::set_permissions(
                    "/run/substrate.sock",
                    std::fs::Permissions::from_mode(mode),
                )
                .unwrap();
                assert!(E3AuthenticatedLinuxUdsListenerV1::from_inherited(
                    &inherited,
                    1,
                    Arc::clone(&configured)
                )
                .is_err());
            }
            std::fs::set_permissions(
                "/run/substrate.sock",
                std::fs::Permissions::from_mode(0o660),
            )
            .unwrap();
            let authentication = E3AuthenticatedLinuxUdsListenerV1::from_inherited(
                &inherited,
                1,
                Arc::clone(&configured),
            )
            .unwrap();
            assert_ne!(authentication.descriptor_inode, 0);
            assert_ne!(authentication.filesystem_inode, 0);
            let mut root_peer = UnixStream::connect("/run/substrate.sock").await.unwrap();
            root_peer.write_all(b"unread-root-canary").await.unwrap();
            let (mut rejected, _) = inherited.listener.accept().await.unwrap();
            assert!(matches!(
                authentication.accept_peer(&inherited, &rejected),
                Err(ConfigProjectionFailureV1::WrongBinding)
            ));
            let mut canary = [0; 18];
            rejected.read_exact(&mut canary).await.unwrap();
            assert_eq!(&canary, b"unread-root-canary");
            drop(root_peer);
            drop(rejected);

            let mut address: libc::sockaddr_un = unsafe { std::mem::zeroed() };
            address.sun_family = libc::AF_UNIX as _;
            for (out, byte) in address.sun_path.iter_mut().zip(b"/run/substrate.sock\0") {
                *out = *byte as _;
            }
            let pid = unsafe { libc::fork() };
            assert!(pid >= 0);
            if pid == 0 {
                // Syscall-only child; no Rust allocation or inherited runtime use after fork.
                unsafe {
                    if libc::setgroups(0, std::ptr::null()) != 0
                        || libc::setgid(gid) != 0
                        || libc::setuid(uid) != 0
                    {
                        libc::_exit(11);
                    }
                    let client =
                        libc::socket(libc::AF_UNIX, libc::SOCK_STREAM | libc::SOCK_CLOEXEC, 0);
                    if client < 0
                        || libc::connect(
                            client,
                            (&address as *const libc::sockaddr_un).cast(),
                            std::mem::size_of_val(&address) as _,
                        ) != 0
                    {
                        libc::_exit(12);
                    }
                    if libc::write(client, b"x".as_ptr().cast(), 1) != 1 {
                        libc::_exit(13);
                    }
                    let mut poll = libc::pollfd {
                        fd: client,
                        events: libc::POLLIN,
                        revents: 0,
                    };
                    let _ = libc::poll(&mut poll, 1, 3000);
                    libc::close(client);
                    libc::_exit(0);
                }
            }
            let (mut accepted, _) = tokio::time::timeout(
                std::time::Duration::from_secs(2),
                inherited.listener.accept(),
            )
            .await
            .unwrap()
            .unwrap();
            let peer = authentication.accept_peer(&inherited, &accepted).unwrap();
            assert_eq!(peer.peer_uid, u64::from(uid));
            assert_eq!(peer.peer_gid, u64::from(gid));
            assert_eq!(peer.peer_pid, pid as u32);
            assert_ne!(peer.peer_pid_start_time_ticks, 0);
            assert_eq!(accepted.read_u8().await.unwrap(), b'x');
            std::fs::rename("/run/substrate.sock", "/run/original.sock").unwrap();
            let _replacement = tokio::net::UnixListener::bind("/run/substrate.sock").unwrap();
            std::fs::set_permissions(
                "/run/substrate.sock",
                std::fs::Permissions::from_mode(0o660),
            )
            .unwrap();
            assert!(matches!(
                authentication.accept_peer(&inherited, &accepted),
                Err(ConfigProjectionFailureV1::WrongBinding)
            ));
            drop(accepted);
            let mut status = 0;
            assert_eq!(unsafe { libc::waitpid(pid, &mut status, 0) }, pid);
            assert!(libc::WIFEXITED(status));
            assert_eq!(libc::WEXITSTATUS(status), 0);
            let alternate = tokio::net::UnixListener::bind("/run/alternate.sock").unwrap();
            let alternate = InheritedUnixListener {
                fd: alternate.as_raw_fd(),
                name: None,
                listener: alternate,
            };
            assert!(
                E3AuthenticatedLinuxUdsListenerV1::from_inherited(&alternate, 1, configured)
                    .is_err()
            );
        });
    }
}

#[derive(Clone)]
pub(crate) struct E3AuthenticatedLinuxUdsPeerV1 {
    pub(crate) peer_pid: u32,
    pub(crate) peer_pid_start_time_ticks: u64,
    pub(crate) peer_uid: u64,
    pub(crate) peer_gid: u64,
    pub(crate) kernel_boot_id: String,
    pub(crate) listener_descriptor_device_id: u64,
    pub(crate) listener_descriptor_inode: u64,
}

impl E3AuthenticatedLinuxUdsListenerV1 {
    pub(crate) fn from_inherited(
        inherited: &InheritedUnixListener,
        inherited_uds_count: usize,
        configured: Arc<ConfiguredAcceptedHomeAuthorityV1>,
    ) -> Result<Self, ConfigProjectionFailureV1> {
        use ConfigProjectionFailureV1::UnsupportedConfiguration as Unsupported;
        if inherited_uds_count != 1 {
            return Err(Unsupported);
        }
        configured.revalidate()?;
        let fd = inherited.listener.as_raw_fd();
        if fd != inherited.fd {
            return Err(Unsupported);
        }
        for (option, expected) in [
            (libc::SO_DOMAIN, libc::AF_UNIX),
            (libc::SO_TYPE, libc::SOCK_STREAM),
            (libc::SO_ACCEPTCONN, 1),
        ] {
            let mut value: libc::c_int = 0;
            let mut length = std::mem::size_of_val(&value) as libc::socklen_t;
            // SAFETY: getsockopt writes within the initialized integer and its length.
            if unsafe {
                libc::getsockopt(
                    fd,
                    libc::SOL_SOCKET,
                    option,
                    (&mut value as *mut libc::c_int).cast(),
                    &mut length,
                )
            } != 0
                || length as usize != std::mem::size_of_val(&value)
                || value != expected
            {
                return Err(Unsupported);
            }
        }
        if inherited
            .listener
            .local_addr()
            .map_err(|_| Unsupported)?
            .as_pathname()
            != Some(std::path::Path::new("/run/substrate.sock"))
        {
            return Err(Unsupported);
        }
        let mut descriptor: libc::stat = unsafe { std::mem::zeroed() };
        if unsafe { libc::fstat(fd, &mut descriptor) } != 0
            || descriptor.st_mode & libc::S_IFMT != libc::S_IFSOCK
            || descriptor.st_dev == 0
            || descriptor.st_ino == 0
        {
            return Err(Unsupported);
        }
        // A pathname socket and its open socket descriptor inhabit different inode domains.
        let path = std::fs::symlink_metadata("/run/substrate.sock").map_err(|_| Unsupported)?;
        if path.mode() & libc::S_IFMT != libc::S_IFSOCK
            || path.dev() == 0
            || path.ino() == 0
            || path.uid() != 0
            || path.gid() != unsafe { libc::getegid() }
            || path.mode() & 0o7777 != 0o660
            || path.nlink() != 1
        {
            return Err(Unsupported);
        }
        let boot =
            std::fs::read_to_string("/proc/sys/kernel/random/boot_id").map_err(|_| Unsupported)?;
        let kernel_boot_id = boot.trim_end_matches('\n').to_owned();
        if uuid::Uuid::parse_str(&kernel_boot_id)
            .map_err(|_| Unsupported)?
            .to_string()
            != kernel_boot_id
        {
            return Err(Unsupported);
        }
        Ok(Self {
            descriptor_device_id: descriptor.st_dev,
            descriptor_inode: descriptor.st_ino,
            filesystem_device_id: path.dev(),
            filesystem_inode: path.ino(),
            expected_peer_uid: configured.intended_uid(),
            kernel_boot_id,
            configured,
        })
    }

    /// Inspect credentials and process identity without reading any connection bytes.
    pub(crate) fn accept_peer(
        &self,
        inherited: &InheritedUnixListener,
        stream: &UnixStream,
    ) -> Result<E3AuthenticatedLinuxUdsPeerV1, ConfigProjectionFailureV1> {
        use ConfigProjectionFailureV1::WrongBinding;
        let current = Self::from_inherited(inherited, 1, Arc::clone(&self.configured))?;
        if current.descriptor_device_id != self.descriptor_device_id
            || current.descriptor_inode != self.descriptor_inode
            || current.filesystem_device_id != self.filesystem_device_id
            || current.filesystem_inode != self.filesystem_inode
            || current.expected_peer_uid != self.expected_peer_uid
            || current.kernel_boot_id != self.kernel_boot_id
        {
            return Err(WrongBinding);
        }
        if stream.local_addr().map_err(|_| WrongBinding)?.as_pathname()
            != Some(std::path::Path::new("/run/substrate.sock"))
        {
            return Err(WrongBinding);
        }
        let mut peer: libc::ucred = unsafe { std::mem::zeroed() };
        let mut length = std::mem::size_of_val(&peer) as libc::socklen_t;
        if unsafe {
            libc::getsockopt(
                stream.as_raw_fd(),
                libc::SOL_SOCKET,
                libc::SO_PEERCRED,
                (&mut peer as *mut libc::ucred).cast(),
                &mut length,
            )
        } != 0
            || length as usize != std::mem::size_of_val(&peer)
            || peer.pid <= 0
            || u64::from(peer.uid) != self.expected_peer_uid
        {
            return Err(WrongBinding);
        }
        let process_path =
            std::ffi::CString::new(format!("/proc/{}", peer.pid)).map_err(|_| WrongBinding)?;
        let process_fd = unsafe {
            libc::open(
                process_path.as_ptr(),
                libc::O_DIRECTORY | libc::O_RDONLY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
            )
        };
        if process_fd < 0 {
            return Err(WrongBinding);
        }
        let process = unsafe { std::os::fd::OwnedFd::from_raw_fd(process_fd) };
        let stat_fd = unsafe {
            libc::openat(
                process.as_raw_fd(),
                c"stat".as_ptr(),
                libc::O_RDONLY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
            )
        };
        if stat_fd < 0 {
            return Err(WrongBinding);
        }
        let stat = unsafe { std::fs::File::from_raw_fd(stat_fd) };
        use std::io::Read;
        let mut bytes = String::new();
        stat.take(8193)
            .read_to_string(&mut bytes)
            .map_err(|_| WrongBinding)?;
        if bytes.len() > 8192 || !bytes.starts_with(&format!("{} (", peer.pid)) {
            return Err(WrongBinding);
        }
        let fields = bytes.rsplit_once(") ").ok_or(WrongBinding)?.1;
        let start: u64 = fields
            .split_whitespace()
            .nth(19)
            .ok_or(WrongBinding)?
            .parse()
            .map_err(|_| WrongBinding)?;
        if start == 0 {
            return Err(WrongBinding);
        }
        self.configured.revalidate()?;
        Ok(E3AuthenticatedLinuxUdsPeerV1 {
            peer_pid: peer.pid as u32,
            peer_pid_start_time_ticks: start,
            peer_uid: u64::from(peer.uid),
            peer_gid: u64::from(peer.gid),
            kernel_boot_id: self.kernel_boot_id.clone(),
            listener_descriptor_device_id: self.descriptor_device_id,
            listener_descriptor_inode: self.descriptor_inode,
        })
    }
}
