# Hyper/HTTP & WebSocket over Windows Named Pipes (with WSL forwarder): What production projects and Microsoft docs recommend

## TL;DR (actionable takeaways)

* **Use Tokio’s canonical accept loop** for named pipes: always create the *next* server instance **before** handing the current one to a task, and **explicitly call `disconnect()`** after each session. This avoids “pipe busy” races and ensures a listener is always present. ([Docs.rs][1])
* **Health‑check with a real client** (`WaitNamedPipe` or open/close), not `Test-Path`. `WaitNamedPipe` only succeeds when a server has a pending `ConnectNamedPipe`. ([Microsoft Learn][2])
* **Bridge strategy**: Named Pipe ⇄ Hyper (HTTP/1 + WS) on the host; downstream from the forwarder use **loopback TCP inside WSL** when possible (the Windows host can reach services in WSL on `localhost`), or fall back to a **WSL‑spawned `socat`** to reach a Unix socket. ([Microsoft Learn][3])
* **For WebSockets**, perform a normal Hyper upgrade and then `copy_bidirectional` between the upgraded stream and the downstream connection; `hyper‑tungstenite` smooths the handshake. ([Docs.rs][4])
* **Security & reliability**: set `first_pipe_instance`, `reject_remote_clients`, and a tight ACL; keep pipes **byte‑stream** mode; and use overlapped I/O (Tokio does this for you) with proper teardown (`FlushFileBuffers` → `DisconnectNamedPipe`). ([Docs.rs][1])

---

## 1) Canonical async accept pattern for Windows named pipes

**What Microsoft and Tokio show:**

* Microsoft’s overlapped server examples create multiple instances and **reuse** them: when a client finishes, the server **disconnects** and **re-issues** an accept on the same handle. ([Microsoft Learn][5])
* Tokio’s `NamedPipeServer` doc includes the *exact* pattern you want on Rust 1.79+ / Tokio 1.47+:

  > Create the first instance with `first_pipe_instance(true)`, `await server.connect()`, **clone off the connected instance**, **immediately create a new server** to keep a listener available, spawn a task to handle the connected client, loop. It also provides `disconnect()` for teardown. ([Docs.rs][1])

**Minimal loop (pattern):**

```rust
use tokio::net::windows::named_pipe::ServerOptions;

let pipe_name = r"\\.\pipe\substrate-agent";
let mut server = ServerOptions::new()
    .first_pipe_instance(true)
    .reject_remote_clients(true)
    .create(pipe_name)?; // keeps the listener visible to clients

loop {
    server.connect().await?;             // await client
    let connected = server;              // move connected instance out
    server = ServerOptions::new()
        .create(pipe_name)?;            // <-- precreate next instance

    tokio::spawn(async move {
        // ... serve HTTP/WS here over `connected` ...
        // When done:
        let _ = connected.disconnect(); // <-- important!
    });
}
```

Why this matters for you:

* It avoids the race where the pipe disappears between sessions (clients see `NotFound` / `ERROR_PIPE_BUSY`).
* It guarantees **at least one instance is always listening**, which is required for client calls like `WaitNamedPipe` to succeed. ([Docs.rs][1])

**Teardown details Microsoft calls out:**

* After sending final bytes, call **`FlushFileBuffers`**, then **`DisconnectNamedPipe`** before reusing the instance. Dropping the handle alone doesn’t guarantee timely disconnect. ([Microsoft Learn][6])

---

## 2) Running Hyper over a named pipe (HTTP + WebSocket)

**What works well in production ecosystems:**

* Hyper can serve *any* stream that implements `AsyncRead + AsyncWrite + Unpin`. Tokio’s `NamedPipeServer` implements these traits, so you can use `hyper::server::conn::http1::Builder::serve_connection` (or `hyper-util`’s `auto::Builder` if you ever need H2 as well). ([Docs.rs][7])

**HTTP/1 service on each accepted pipe:**

```rust
use hyper::{service::service_fn, Request, Response, Body};
use hyper::server::conn::http1;
use hyper_util::rt::TokioIo;

async fn serve_one(pipe: tokio::net::windows::named_pipe::NamedPipeServer) -> hyper::Result<()> {
    let io = TokioIo::new(pipe); // adapts to hyper IO
    let svc = service_fn(|_req: Request<Body>| async {
        Ok::<_, hyper::Error>(Response::new(Body::from("ok")))
    });
    http1::Builder::new().serve_connection(io, svc).await
}
```

**WebSocket upgrade pattern**:

* Use `hyper-tungstenite` to validate headers and build the 101 response, then await the upgraded I/O and run a WS echo/forward loop. This works over named pipes the same way it works over TCP. ([Docs.rs][4])

---

## 3) Forwarding into WSL: Unix socket vs. TCP (what projects actually do)

There are two robust patterns seen “in the wild”:

### A) **Prefer loopback TCP inside WSL** (simplest and fast)

* If your `world-agent` exposes `127.0.0.1:<port>` **inside the distro**, the Windows host can reach it on **`localhost:<port>`** (WSL sets up the port proxy; mirrored networking improves this further). That makes the forwarder a straight **NamedPipe ⇄ TCP** bridge, with no per-session `wsl.exe` overhead. ([Microsoft Learn][3])

**Why this is recommended:** It minimizes moving parts, avoids AF_UNIX interop limitations across the VM boundary, and stays compatible with Hyper’s HTTP/WS semantics. (Microsoft has long documented that AF_UNIX interop is limited with WSL2; crossing the boundary reliably still needs a relay.) ([GitHub][8])

### B) **If you must target a Unix socket** inside WSL

* Adopt the **Docker‑style relay** used by multiple production tools: spawn a helper in WSL that connects to the Unix socket and relays bytes over stdio, while the Windows side forwards the named pipe to that process.
* The reference implementation is **`npiperelay.exe` + `socat`**; it’s widely used to bridge `\\.\pipe\docker_engine` and WSL’s `/var/run/docker.sock`, and for SSH‑agent forwarding. Your forwarder can internalize the same topology without shipping external binaries. ([GitHub][9])

**Topology (B)**

```
Hyper over Named Pipe   <=>  Forwarder
                                 \_ spawn "wsl.exe -d <distro> -- socat - UNIX-CONNECT:/run/substrate.sock"
                                     \_ child's stdin/stdout <=> agent's UDS
```

**Why not AF_UNIX directly from Windows to WSL?** AF_UNIX on Windows doesn’t bridge into WSL2’s separate Linux kernel namespace; Microsoft issues track this limitation. Hence the need for a relay or the simpler TCP listener. ([GitHub][10])

---

## 4) Concrete library choices

* **Tokio named pipes** (`tokio::net::windows::named_pipe`) – provides `ServerOptions`, `NamedPipeServer::connect`, and `disconnect`, with the **documented accept pattern** (pre‑create next instance). ([Docs.rs][1])
* **Hyper 1.x** – use `http1::Builder::serve_connection` (or `hyper‑util`’s auto builder if you later add HTTP/2). ([Docs.rs][7])
* **WebSocket** – `hyper‑tungstenite` for the handshake and `WebSocketStream` management. ([Docs.rs][4])
* **For inspiration / parity** – Docker on Windows exposes an HTTP API over `\\.\pipe\docker_engine` and uses **overlapped named pipes** in **`go‑winio`**; its code shows mature patterns for accept loops and timeouts. (You’re not in Go, but the design translates.) ([Go Packages][11])

---

## 5) Pitfalls and how to avoid them

1. **“All pipe instances are busy” (231)**

   * Cause: No server instance is listening when a client attempts to connect.
   * Fix: **Always pre‑create the next instance** *before* handing a connected one to a task; maintain at least one pending `connect()` at all times. On teardown, **call `disconnect()`**. ([Docs.rs][1])

2. **Server never “appears” to clients**

   * PowerShell `Test-Path \\.\pipe\name` is not a reliable readiness probe. Instead, use **`WaitNamedPipe`** from the client side or just attempt `CreateFile`/open with a short timeout and backoff. ([Microsoft Learn][2])

3. **Wrong pipe mode**

   * Use **byte mode** (`PIPE_TYPE_BYTE`, `PIPE_READMODE_BYTE`, `PIPE_WAIT`), which matches HTTP’s stream semantics and avoids message‑framing surprises. ([Microsoft Learn][12])

4. **Forgot to flush before disconnect**

   * Call `FlushFileBuffers` to ensure the client has read the last bytes, then `DisconnectNamedPipe`. Dropping the handle early can truncate responses or strand half‑closed connections. ([Microsoft Learn][6])

5. **Security descriptors / multi‑user hosts**

   * Harden the listener with `FILE_FLAG_FIRST_PIPE_INSTANCE` and `PIPE_REJECT_REMOTE_CLIENTS`; apply an ACL that grants `SY`, `BA`, and **interactive users** (or your service SID) only. ([Docs.rs][13])

6. **Blocking accept loop**

   * Use overlapped I/O (Tokio does) and never run `connect()` on the same instance you intend to reuse without pre‑provisioning another. Microsoft’s overlapped samples and Tokio’s docs are aligned here. ([Microsoft Learn][5])

7. **AF_UNIX across WSL boundary**

   * Direct AF_UNIX interop between Windows and WSL2 is **not** generally supported; rely on TCP (preferred) or a relay (`socat`). ([GitHub][8])

---

## 6) Reference patterns you can copy

### Pattern A — Named Pipe ⇄ Hyper (HTTP/1 + WS) ⇄ **TCP to WSL**

* **Server (host)**: Tokio named pipe accept loop (above).
* **Per connection**:

  1. Run Hyper `serve_connection` on the pipe.
  2. For WebSocket requests, use `hyper‑tungstenite` to upgrade and then `copy_bidirectional` to a `TcpStream` connected to `localhost:<port>`.
  3. For HTTP requests, use a standard Hyper client (or a raw `TcpStream`) to **proxy** to `localhost:<port>`.

**Why it’s solid:** Simplest datapath; zero `wsl.exe` processes per session; leverages WSL’s documented localhost access from Windows to WSL. ([Microsoft Learn][3])

### Pattern B — Named Pipe ⇄ Hyper ⇄ **WSL helper → Unix socket**

* **Server (host)**: Same as Pattern A.
* **Per connection**:

  1. On first byte of a request or on upgrade, spawn `wsl.exe -d <distro> -- socat - UNIX-CONNECT:/run/substrate.sock`.
  2. Wire the pipe/Hyper side to the child’s stdio; stream until EOF; kill child on close.

This mirrors **`npiperelay`**’s production‑tested design for Docker and SSH agent forwarding. ([GitHub][9])

---

## 7) Health checks & diagnostics that match the above

* **Readiness probe:** Attempt `WaitNamedPipe("\\\\.\\pipe\\substrate-agent", 1000)` or open and immediately close a client stream (PowerShell: `[System.IO.Pipes.NamedPipeClientStream]`). This guarantees the server has a pending `connect()`. ([Microsoft Learn][2])
* **WSL reachability:** If using TCP, `Test-Connection -TcpPort <port> localhost` confirms the port is forwarded to WSL. ([Microsoft Learn][14])
* **ACL verification & single‑owner guard:** Create first instance with `first_pipe_instance(true)` and reject remote clients; validate ACLs with `icacls`/`accesschk` (policy‑dependent). ([Docs.rs][13])

---

## 8) How this maps to your current failure picture

* Your log of a single “instance ready” followed by repeated `ERROR_PIPE_BUSY` is exactly what Tokio’s docs warn about when a server **drops its only instance** (or never pre‑creates a successor). Implement the **precreate‑next‑instance** loop and **call `disconnect()`** after each session. ([Docs.rs][1])
* Replace `Test‑Path \\.\pipe\substrate-agent` in warm with an **active probe** (`WaitNamedPipe` or a client open/close). This will also surface ACL problems immediately if present. ([Microsoft Learn][2])
* If you want the fastest path to green, **enable the agent’s TCP loopback listener in WSL** and dial `localhost:<port>` from the forwarder (Pattern A). Move the UDS target to a follow‑up once everything passes smoke. ([Microsoft Learn][3])

---

## 9) Production projects that validate these choices

* **Docker Desktop (Windows)** – HTTP over `\\.\pipe\docker_engine`, using overlapped I/O and robust accept/client semantics (`go‑winio`). It demonstrates large‑scale, real‑world stability for HTTP over named pipes. ([Go Packages][11])
* **WSL SSH agent forwarders** (e.g., `wsl‑ssh‑pageant`) – rely on **`npiperelay + socat`** design to bridge named pipes and Unix sockets across the WSL2 boundary. Same pattern applies to your forwarder when UDS is mandatory. ([GitHub][9])

---

## 10) A compact checklist you can apply now

1. **Accept loop**

   * [ ] `first_pipe_instance(true)` and `reject_remote_clients(true)`. ([Docs.rs][1])
   * [ ] Pre‑create next instance **before** spawning a task for the current one. ([Docs.rs][1])
   * [ ] On session end: `FlushFileBuffers` → `disconnect()` → drop handle. ([Microsoft Learn][6])

2. **Health checks**

   * [ ] Replace `Test-Path` with `WaitNamedPipe` or client open/close probe. ([Microsoft Learn][2])

3. **Downstream target**

   * [ ] Prefer **TCP in WSL** (bind agent to `127.0.0.1:<port>` and connect via `localhost:<port>` from Windows). ([Microsoft Learn][3])
   * [ ] If UDS is required, use a `wsl.exe ... socat` child (npiperelay pattern). ([GitHub][9])

4. **HTTP/WS handling**

   * [ ] Hyper `http1::Builder::serve_connection` per pipe. ([Docs.rs][7])
   * [ ] `hyper‑tungstenite` for WebSocket upgrade + `copy_bidirectional`. ([Docs.rs][4])

5. **Security**

   * [ ] Harden ACLs, keep pipe in byte mode. ([Microsoft Learn][12])

---

## 11) Useful docs & references (the “why” behind the above)

* **Tokio named pipe server pattern + `disconnect()`** – the exact accept loop you need and teardown method. ([Docs.rs][1])
* **Microsoft Learn – named pipes**: `ConnectNamedPipe`, `DisconnectNamedPipe`, types/modes, overlapped server examples, and sync vs. overlapped I/O discussion. ([Microsoft Learn][15])
* **Hyper 1.x server over arbitrary IO + `hyper-util` auto builder**. ([Docs.rs][7])
* **WebSocket upgrade with Hyper** (`hyper‑tungstenite`). ([Docs.rs][4])
* **WSL networking (Windows → WSL on localhost)** and mirrored mode improvements. ([Microsoft Learn][3])
* **npiperelay** reference design for Named Pipe ⇄ UDS via `socat`. ([GitHub][9])

---

### If you want, I can turn this into:

* a short **Rust diff** that drops into your forwarder’s `PipeListener` and **serve loop**, and
* a **PowerShell probe** that uses `WaitNamedPipe` so warm doesn’t race.

[1]: https://docs.rs/tokio/latest/tokio/net/windows/named_pipe/struct.NamedPipeServer.html "NamedPipeServer in tokio::net::windows::named_pipe - Rust"
[2]: https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-waitnamedpipea?utm_source=chatgpt.com "WaitNamedPipeA function (winbase.h) - Win32 apps"
[3]: https://learn.microsoft.com/en-us/windows/wsl/networking "Accessing network applications with WSL | Microsoft Learn"
[4]: https://docs.rs/hyper-tungstenite?utm_source=chatgpt.com "hyper_tungstenite - Rust"
[5]: https://learn.microsoft.com/en-us/windows/win32/ipc/named-pipe-server-using-overlapped-i-o?utm_source=chatgpt.com "Named Pipe Server Using Overlapped I/O - Win32 apps"
[6]: https://learn.microsoft.com/en-us/windows/win32/ipc/named-pipe-operations?utm_source=chatgpt.com "Named Pipe Operations - Win32 apps"
[7]: https://docs.rs/hyper/latest/hyper/server/conn/http1/struct.Builder.html?utm_source=chatgpt.com "Builder in hyper::server::conn::http1 - Rust"
[8]: https://github.com/microsoft/WSL/issues/8321?utm_source=chatgpt.com "Connection refused when using WSL/Windows Unix ..."
[9]: https://github.com/jstarks/npiperelay "GitHub - jstarks/npiperelay: npiperelay allows you to access Windows named pipes from WSL"
[10]: https://github.com/microsoft/WSL/issues/5961?utm_source=chatgpt.com "Windows/WSL AF_UNIX Interop doesn't work in WSL2 #5961"
[11]: https://pkg.go.dev/github.com/Microsoft/go-winio?utm_source=chatgpt.com "winio package - github.com/Microsoft ..."
[12]: https://learn.microsoft.com/en-us/windows/win32/ipc/named-pipe-type-read-and-wait-modes?utm_source=chatgpt.com "Named Pipe Type, Read, and Wait Modes - Win32 apps"
[13]: https://docs.rs/tokio/latest/tokio/net/windows/named_pipe/struct.ServerOptions.html?utm_source=chatgpt.com "ServerOptions in tokio::net::windows::named_pipe - Rust"
[14]: https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.management/test-connection?view=powershell-7.5&utm_source=chatgpt.com "Test-Connection (Microsoft.PowerShell.Management)"
[15]: https://learn.microsoft.com/en-us/windows/win32/api/namedpipeapi/nf-namedpipeapi-connectnamedpipe?utm_source=chatgpt.com "ConnectNamedPipe function (namedpipeapi.h) - Win32 apps"
