
use anyhow::{Result,bail};
use std::os::fd::{AsRawFd,FromRawFd,OwnedFd,RawFd};
use std::mem::zeroed;
use config_projection::ConfigProjectionCodecV1;
thread_local! { static CHECK: std::cell::RefCell<Option<std::sync::mpsc::SyncSender<()>>> = const { std::cell::RefCell::new(None) }; }
struct Security { security: Option<std::collections::BTreeMap<String,String>> }
struct Launch;
impl Launch {     fn e3_remaining_ms(deadline: &libc::timespec) -> Result<i32> {
        let mut now = libc::timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };
        anyhow::ensure!(
            unsafe { libc::clock_gettime(libc::CLOCK_BOOTTIME, &mut now) } == 0,
            "read E3 activation deadline"
        );
        let ns = (i128::from(deadline.tv_sec) - i128::from(now.tv_sec)) * 1_000_000_000
            + i128::from(deadline.tv_nsec)
            - i128::from(now.tv_nsec);
        anyhow::ensure!(ns > 0, "E3 activation deadline elapsed");
        Ok(((ns + 999_999) / 1_000_000).min(i128::from(i32::MAX)) as i32)
    }

    fn e3_read_bounded(fd: i32, deadline: &libc::timespec) -> Result<Vec<u8>> { CHECK.with(|c| c.borrow_mut().take().unwrap().send(()).unwrap());
        let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
        anyhow::ensure!(
            flags >= 0 && unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) } == 0,
            "bound E3 protocol read"
        );
        let mut bytes = Vec::with_capacity(65_536);
        loop {
            let mut poll = libc::pollfd {
                fd,
                events: libc::POLLIN,
                revents: 0,
            };
            let rc = unsafe { libc::poll(&mut poll, 1, Self::e3_remaining_ms(deadline)?) };
            if rc < 0 && std::io::Error::last_os_error().raw_os_error() == Some(libc::EINTR) {
                continue;
            }
            anyhow::ensure!(
                rc > 0 && poll.revents & (libc::POLLNVAL | libc::POLLERR) == 0,
                "E3 pipe read deadline or failure"
            );
            let mut buffer = [0u8; 4096];
            let count = unsafe { libc::read(fd, buffer.as_mut_ptr().cast(), buffer.len()) };
            if count == 0 {
                return Ok(bytes);
            }
            if count < 0 {
                let error = std::io::Error::last_os_error();
                if matches!(error.raw_os_error(), Some(libc::EINTR | libc::EAGAIN)) {
                    continue;
                }
                return Err(error.into());
            }
            anyhow::ensure!(
                count as usize <= 65_536 - bytes.len(),
                "E3 pipe exceeded bounded protocol"
            );
            bytes.extend_from_slice(&buffer[..count as usize]);
        }
    }

}
fn receive_namespace_setup_byte(fd: RawFd, expected: u8) -> Result<()> {
    let mut byte = 0u8;
    let mut control = [0u8; 64];
    let mut iovec = libc::iovec {
        iov_base: (&mut byte as *mut u8).cast(),
        iov_len: 1,
    };
    let mut message: libc::msghdr = unsafe { zeroed() };
    message.msg_iov = &mut iovec;
    message.msg_iovlen = 1;
    message.msg_control = control.as_mut_ptr().cast();
    message.msg_controllen = control.len() as _;
    let received = unsafe { libc::recvmsg(fd, &mut message, libc::MSG_CMSG_CLOEXEC) };
    if received != 1
        || byte != expected
        || message.msg_controllen != 0
        || message.msg_flags & (libc::MSG_TRUNC | libc::MSG_CTRUNC) != 0
    {
        bail!("missing or malformed wrapper release byte");
    }
    let trailing = unsafe {
        libc::recv(
            fd,
            (&mut byte as *mut u8).cast(),
            1,
            libc::MSG_DONTWAIT | libc::MSG_PEEK,
        )
    };
    if trailing >= 0 || std::io::Error::last_os_error().kind() != std::io::ErrorKind::WouldBlock {
        bail!("duplicate, trailing, or closed wrapper release channel");
    }
    Ok(())
}

fn spawn_descriptor_pinned(userns_parent: OwnedFd, setup_reader: OwnedFd, deadline: libc::timespec) -> Result<()> { let mut owned = Security { security: None };        let bytes = Launch::e3_read_bounded(setup_reader.as_raw_fd(), &deadline)?;
        drop(setup_reader);
        owned.security = Some(ConfigProjectionCodecV1::decode_canonical_json(&bytes)?);
        // Setup-ready proves the child has passed the mapped-release trailing-data check.
        drop(userns_parent);
Ok(()) }
fn spawn_readiness_probe(userns_parent: OwnedFd, setup_reader: OwnedFd, deadline: libc::timespec) -> Result<()> { let mut probe = Security { security: None };        let bytes = Launch::e3_read_bounded(setup_reader.as_raw_fd(), &deadline)?;
        drop(setup_reader);
        probe.security = Some(ConfigProjectionCodecV1::decode_canonical_json(&bytes)?);
        // Setup-ready proves the child has passed the mapped-release trailing-data check.
        drop(userns_parent);
Ok(()) }
#[test]
fn both_production_parent_tails_with_actual_receiver() {
 let mut failures=Vec::new();
 for (name,parent_fn) in [("gateway",spawn_descriptor_pinned as fn(OwnedFd,OwnedFd,libc::timespec)->Result<()>),("probe",spawn_readiness_probe)] {
  let mut sockets=[-1;2]; let mut pipes=[-1;2];
  assert_eq!(unsafe { libc::socketpair(libc::AF_UNIX,libc::SOCK_SEQPACKET|libc::SOCK_CLOEXEC,0,sockets.as_mut_ptr()) },0);
  assert_eq!(unsafe { libc::pipe2(pipes.as_mut_ptr(),libc::O_CLOEXEC) },0);
  let parent=unsafe {OwnedFd::from_raw_fd(sockets[0])};let child=unsafe {OwnedFd::from_raw_fd(sockets[1])};
  let reader=unsafe {OwnedFd::from_raw_fd(pipes[0])};let writer=unsafe {OwnedFd::from_raw_fd(pipes[1])};
  let (tx,rx)=std::sync::mpsc::sync_channel(0);CHECK.with(|c| *c.borrow_mut()=Some(tx));
  let child_thread=std::thread::spawn(move || {
    rx.recv().unwrap();
    let result=receive_namespace_setup_byte(child.as_raw_fd(),2);
    if result.is_ok() { assert_eq!(unsafe {libc::write(writer.as_raw_fd(),b"{}".as_ptr().cast(),2)},2); }
    drop(writer);(child,result)
  });
  assert_eq!(unsafe {libc::send(parent.as_raw_fd(),[2u8].as_ptr().cast(),1,libc::MSG_NOSIGNAL)},1);
  let mut deadline=libc::timespec {tv_sec:0,tv_nsec:0};assert_eq!(unsafe {libc::clock_gettime(libc::CLOCK_BOOTTIME,&mut deadline)},0);deadline.tv_sec+=2;
  let result=parent_fn(parent,reader,deadline);
  let (child,received)=child_thread.join().unwrap();
  eprintln!("{name}: parent={result:?}; receiver={received:?}");
  let mut byte=0u8;assert_eq!(unsafe {libc::recv(child.as_raw_fd(),(&mut byte as *mut u8).cast(),1,libc::MSG_DONTWAIT)},0);
  if result.is_err() || received.is_err() { failures.push(name); }
 }
 assert!(failures.is_empty(),"failed startup paths: {failures:?}");
}
