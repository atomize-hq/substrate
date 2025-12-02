use anyhow::{Context, Result};
use std::env;

#[cfg(unix)]
use std::os::unix::io::{FromRawFd, RawFd};
#[cfg(unix)]
use tokio::net::{TcpListener, UnixListener};

#[cfg(unix)]
use tracing::warn;

#[cfg(unix)]
const LISTEN_FDS_ENV: &str = "LISTEN_FDS";
#[cfg(unix)]
const LISTEN_PID_ENV: &str = "LISTEN_PID";
#[cfg(unix)]
const LISTEN_FDNAMES_ENV: &str = "LISTEN_FDNAMES";
#[cfg(unix)]
const LISTEN_FD_START_ENV: &str = "LISTEN_FD_START";
#[cfg(unix)]
const DEFAULT_FD_START: RawFd = 3;

#[cfg(unix)]
#[derive(Debug)]
pub(crate) struct SocketActivation {
    pub(crate) total_fds: usize,
    pub(crate) unix_listeners: Vec<InheritedUnixListener>,
    pub(crate) tcp_listeners: Vec<InheritedTcpListener>,
}

#[cfg(unix)]
#[derive(Debug)]
pub(crate) struct InheritedUnixListener {
    pub(crate) fd: RawFd,
    pub(crate) name: Option<String>,
    pub(crate) listener: UnixListener,
}

#[cfg(unix)]
#[derive(Debug)]
pub(crate) struct InheritedTcpListener {
    pub(crate) fd: RawFd,
    pub(crate) name: Option<String>,
    pub(crate) listener: TcpListener,
}

#[cfg(unix)]
pub(crate) fn collect_socket_activation() -> Result<Option<SocketActivation>> {
    let listen_fds = match env::var(LISTEN_FDS_ENV) {
        Ok(value) => value
            .trim()
            .parse::<i32>()
            .with_context(|| format!("Failed to parse {LISTEN_FDS_ENV}={value}"))?,
        Err(env::VarError::NotPresent) => return Ok(None),
        Err(env::VarError::NotUnicode(_)) => {
            anyhow::bail!("{LISTEN_FDS_ENV} contains non-Unicode data");
        }
    };

    if listen_fds <= 0 {
        cleanup_env();
        return Ok(None);
    }

    let pid_value = match env::var(LISTEN_PID_ENV) {
        Ok(value) => match value.trim().parse::<u32>() {
            Ok(pid) => Some(pid),
            Err(err) => {
                warn!(
                    error = %err,
                    raw = value,
                    "LISTEN_PID was set but could not be parsed; ignoring socket activation data"
                );
                None
            }
        },
        Err(env::VarError::NotPresent) => None,
        Err(env::VarError::NotUnicode(_)) => {
            anyhow::bail!("{LISTEN_PID_ENV} contains non-Unicode data");
        }
    };

    let current_pid = std::process::id();
    if pid_value != Some(current_pid) {
        cleanup_env();
        return Ok(None);
    }

    let fd_start = match env::var(LISTEN_FD_START_ENV) {
        Ok(value) => {
            let parsed = value
                .trim()
                .parse::<i32>()
                .with_context(|| format!("Failed to parse {LISTEN_FD_START_ENV}={value}"))?;
            parsed
        }
        Err(env::VarError::NotPresent) => DEFAULT_FD_START,
        Err(env::VarError::NotUnicode(_)) => {
            anyhow::bail!("{LISTEN_FD_START_ENV} contains non-Unicode data");
        }
    };

    if fd_start < 0 {
        anyhow::bail!("{LISTEN_FD_START_ENV} must be non-negative");
    }

    if listen_fds as usize > (i32::MAX as usize) {
        anyhow::bail!("{LISTEN_FDS_ENV} is too large");
    }

    let listen_fds = listen_fds as usize;
    let fd_start = fd_start as RawFd;
    if (fd_start as i64) + (listen_fds as i64) > i32::MAX as i64 {
        anyhow::bail!("Inherited descriptor range exceeds platform limits");
    }

    let names = env::var(LISTEN_FDNAMES_ENV)
        .ok()
        .map(parse_fd_names)
        .unwrap_or_default();

    let mut unix_listeners = Vec::new();
    let mut tcp_listeners = Vec::new();
    for offset in 0..listen_fds {
        let fd = fd_start + offset as RawFd;
        let name = names.get(offset).cloned().unwrap_or(None);
        match convert_fd(fd, name.clone())? {
            Some(ClassifiedListener::Unix(listener)) => {
                unix_listeners.push(InheritedUnixListener { fd, name, listener });
            }
            Some(ClassifiedListener::Tcp(listener)) => {
                tcp_listeners.push(InheritedTcpListener { fd, name, listener });
            }
            None => {}
        }
    }

    cleanup_env();

    if unix_listeners.is_empty() && tcp_listeners.is_empty() {
        return Ok(None);
    }

    Ok(Some(SocketActivation {
        total_fds: listen_fds,
        unix_listeners,
        tcp_listeners,
    }))
}

#[cfg(unix)]
fn cleanup_env() {
    env::remove_var(LISTEN_FDS_ENV);
    env::remove_var(LISTEN_PID_ENV);
    env::remove_var(LISTEN_FDNAMES_ENV);
    env::remove_var(LISTEN_FD_START_ENV);
}

#[cfg(unix)]
enum ClassifiedListener {
    Unix(UnixListener),
    Tcp(TcpListener),
}

#[cfg(unix)]
fn convert_fd(fd: RawFd, name: Option<String>) -> Result<Option<ClassifiedListener>> {
    set_cloexec(fd);

    let family = unsafe { get_socket_family(fd)? };
    let sock_type = unsafe { get_socket_type(fd)? };

    if sock_type != libc::SOCK_STREAM {
        warn!(
            fd,
            family, sock_type, "Skipping inherited descriptor because it is not a stream socket"
        );
        close_inherited_fd(fd);
        return Ok(None);
    }

    match family {
        libc::AF_UNIX => {
            let std_listener = unsafe { std::os::unix::net::UnixListener::from_raw_fd(fd) };
            std_listener
                .set_nonblocking(true)
                .context("Failed to configure inherited Unix listener")?;
            let listener = UnixListener::from_std(std_listener)?;
            Ok(Some(ClassifiedListener::Unix(listener)))
        }
        libc::AF_INET | libc::AF_INET6 => {
            let std_listener = unsafe { std::net::TcpListener::from_raw_fd(fd) };
            std_listener
                .set_nonblocking(true)
                .context("Failed to configure inherited TCP listener")?;
            let listener = TcpListener::from_std(std_listener)?;
            Ok(Some(ClassifiedListener::Tcp(listener)))
        }
        family => {
            warn!(
                fd,
                family,
                inherited_name = name.as_deref().unwrap_or(""),
                "Skipping inherited descriptor because it uses an unsupported address family"
            );
            close_inherited_fd(fd);
            Ok(None)
        }
    }
}

#[cfg(unix)]
unsafe fn get_socket_family(fd: RawFd) -> Result<libc::c_int> {
    let mut addr: libc::sockaddr_storage = std::mem::zeroed();
    let mut len = std::mem::size_of::<libc::sockaddr_storage>() as libc::socklen_t;
    if libc::getsockname(fd, &mut addr as *mut _ as *mut libc::sockaddr, &mut len) != 0 {
        return Err(std::io::Error::last_os_error())
            .context("getsockname failed for inherited descriptor");
    }
    Ok(addr.ss_family as libc::c_int)
}

#[cfg(unix)]
unsafe fn get_socket_type(fd: RawFd) -> Result<libc::c_int> {
    let mut sock_type: libc::c_int = 0;
    let mut len = std::mem::size_of::<libc::c_int>() as libc::socklen_t;
    if libc::getsockopt(
        fd,
        libc::SOL_SOCKET,
        libc::SO_TYPE,
        &mut sock_type as *mut _ as *mut libc::c_void,
        &mut len,
    ) != 0
    {
        return Err(std::io::Error::last_os_error())
            .context("getsockopt(SO_TYPE) failed for inherited descriptor");
    }
    Ok(sock_type)
}

#[cfg(unix)]
fn set_cloexec(fd: RawFd) {
    unsafe {
        let flags = libc::fcntl(fd, libc::F_GETFD);
        if flags >= 0 {
            let _ = libc::fcntl(fd, libc::F_SETFD, flags | libc::FD_CLOEXEC);
        }
    }
}

#[cfg(unix)]
fn close_inherited_fd(fd: RawFd) {
    unsafe {
        let _ = libc::close(fd);
    }
}

#[cfg(unix)]
fn parse_fd_names(value: String) -> Vec<Option<String>> {
    if value.is_empty() {
        return Vec::new();
    }
    let mut names = Vec::new();
    let mut current = String::new();
    let mut chars = value.chars();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            if let Some(next) = chars.next() {
                current.push(next);
            }
        } else if ch == ':' {
            names.push(if current.is_empty() {
                None
            } else {
                Some(current.clone())
            });
            current.clear();
        } else {
            current.push(ch);
        }
    }
    names.push(if current.is_empty() {
        None
    } else {
        Some(current)
    });
    names
}

#[cfg(test)]
mod tests {
    use super::parse_fd_names;

    #[test]
    fn parse_fd_names_handles_escapes() {
        let parsed = parse_fd_names("uds\\:sock:tcp\\:9000".to_string());
        assert_eq!(
            parsed,
            vec![Some("uds:sock".to_string()), Some("tcp:9000".to_string())]
        );
    }

    #[test]
    fn parse_fd_names_emits_none_for_empty_segments() {
        let parsed = parse_fd_names("one::two:".to_string());
        assert_eq!(
            parsed,
            vec![Some("one".to_string()), None, Some("two".to_string()), None]
        );
    }
}

#[cfg(unix)]
pub mod test_support {
    use super::{collect_socket_activation, SocketActivation};
    use anyhow::Result;
    use std::os::unix::io::RawFd;

    #[derive(Debug, Clone)]
    pub struct SocketActivationSummary {
        pub total_fds: usize,
        pub unix_listeners: Vec<ListenerMeta>,
        pub tcp_listeners: Vec<ListenerMeta>,
    }

    #[derive(Debug, Clone)]
    pub struct ListenerMeta {
        pub fd: RawFd,
        pub name: Option<String>,
    }

    impl From<SocketActivation> for SocketActivationSummary {
        fn from(activation: SocketActivation) -> Self {
            Self {
                total_fds: activation.total_fds,
                unix_listeners: activation
                    .unix_listeners
                    .into_iter()
                    .map(|listener| ListenerMeta {
                        fd: listener.fd,
                        name: listener.name,
                    })
                    .collect(),
                tcp_listeners: activation
                    .tcp_listeners
                    .into_iter()
                    .map(|listener| ListenerMeta {
                        fd: listener.fd,
                        name: listener.name,
                    })
                    .collect(),
            }
        }
    }

    pub fn collect_summary() -> Result<Option<SocketActivationSummary>> {
        collect_socket_activation().map(|activation| activation.map(SocketActivationSummary::from))
    }
}
