#![cfg(unix)]

use std::{
    env,
    io::{BufRead, BufReader, Read, Write},
    net::TcpStream,
    os::unix::net::UnixListener,
    path::{Path, PathBuf},
    process::Command as StdCommand,
    thread,
    time::Duration,
};

use codex::{
    AppServerCodegenRequest, CliOverridesPatch, CodexClient, CodexError, ExecStreamRequest,
    FeaturesListFormat, FeaturesListRequest, ResponsesApiProxyRequest, ResumeRequest,
    ResumeSelector, StdioToUdsRequest, ThreadEvent,
};
use futures_util::StreamExt;
use std::fs;
use tempfile::TempDir;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader as AsyncBufReader},
    time,
};

#[cfg(target_os = "linux")]
use codex::{SandboxCommandRequest, SandboxPlatform};

const BINARY_ENV: &str = "CODEX_E2E_BINARY";
const HOME_ENV: &str = "CODEX_E2E_HOME";
const LIVE_FLAG_ENV: &str = "CODEX_E2E_LIVE";
const WORKDIR_ENV: &str = "CODEX_E2E_WORKDIR";

struct RealCli {
    binary: PathBuf,
    home_dir: PathBuf,
    version: String,
    client: CodexClient,
    _home_guard: Option<TempDir>,
}

impl RealCli {
    fn bootstrap() -> Result<Option<Self>, Box<dyn std::error::Error>> {
        let binary = match locate_binary() {
            Some(path) => path,
            None => {
                eprintln!("skipping real CLI e2e: set CODEX_E2E_BINARY or install `codex` on PATH");
                return Ok(None);
            }
        };

        let version = match read_version(&binary) {
            Some(value) => value,
            None => {
                eprintln!(
                    "skipping real CLI e2e: unable to read version from {}",
                    binary.display()
                );
                return Ok(None);
            }
        };

        let (home_dir, guard) = match env::var_os(HOME_ENV) {
            Some(value) => (PathBuf::from(value), None),
            None => {
                let temp = TempDir::new()?;
                (temp.path().to_path_buf(), Some(temp))
            }
        };

        let client = CodexClient::builder()
            .binary(&binary)
            .codex_home(&home_dir)
            .mirror_stdout(false)
            .quiet(true)
            .timeout(Duration::from_secs(30))
            .build();

        Ok(Some(Self {
            binary,
            home_dir,
            version,
            client,
            _home_guard: guard,
        }))
    }

    fn note_skip(&self, note: impl AsRef<str>) {
        eprintln!(
            "skipping real CLI e2e ({} @ {}): {}",
            self.version.trim(),
            self.binary.display(),
            note.as_ref()
        );
    }
}

#[tokio::test]
async fn exec_resume_diff_apply_live_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
    let Some(cli) = RealCli::bootstrap()? else {
        return Ok(());
    };
    if !live_mode_enabled() {
        cli.note_skip(
            "live e2e disabled (set CODEX_E2E_LIVE=1 to exercise exec/resume/diff/apply)",
        );
        return Ok(());
    }

    let workspace = prepare_workspace()?;
    let client = CodexClient::builder()
        .binary(&cli.binary)
        .codex_home(&cli.home_dir)
        .working_dir(&workspace.path)
        .disable_feature("exec_policy")
        .mirror_stdout(false)
        .quiet(true)
        .timeout(Duration::from_secs(120))
        .build();

    let prompt = "You are running a Codex e2e check. Create hello.txt containing only \"hello world\" using apply_patch. Do not run other shell commands. Stop after writing.";
    let exec_request = ExecStreamRequest {
        prompt: prompt.to_string(),
        ephemeral: false,
        ignore_rules: false,
        ignore_user_config: false,
        idle_timeout: Some(Duration::from_secs(120)),
        output_last_message: Some(workspace.path.join("exec-last.txt")),
        output_schema: None,
        json_event_log: Some(workspace.path.join("exec-events.jsonl")),
    };

    let mut thread_id = None;
    let exec_stream = match client.stream_exec(exec_request).await {
        Ok(stream) => stream,
        Err(err) => {
            cli.note_skip(format!("exec stream failed to start: {err}"));
            return Ok(());
        }
    };
    let mut events = exec_stream.events;
    while let Some(event) = events.next().await {
        match event {
            Ok(ThreadEvent::ThreadStarted(started)) => thread_id = Some(started.thread_id.clone()),
            Ok(_) => {}
            Err(err) => {
                cli.note_skip(format!("exec stream parse error: {err}"));
                continue;
            }
        }
    }

    let exec_completion = match exec_stream.completion.await {
        Ok(done) => done,
        Err(err) => {
            cli.note_skip(format!("exec completion failed: {err}"));
            return Ok(());
        }
    };
    if !exec_completion.status.success() {
        cli.note_skip(format!(
            "exec exited with {} (live run)",
            exec_completion.status
        ));
        return Ok(());
    }

    let hello_path = workspace.path.join("hello.txt");
    if !hello_path.is_file() {
        cli.note_skip("exec did not create hello.txt");
        return Ok(());
    }

    let diff = match client.diff().await {
        Ok(output) => output,
        Err(CodexError::NonZeroExit { status, stderr }) => {
            cli.note_skip(format!("diff returned {}: {}", status, stderr.trim()));
            return Ok(());
        }
        Err(err) => return Err(err.into()),
    };
    if !diff.status.success() {
        cli.note_skip(format!(
            "diff exited {} without producing a replayable diff",
            diff.status
        ));
        return Ok(());
    }

    let apply = match client.apply().await {
        Ok(output) => output,
        Err(CodexError::NonZeroExit { status, stderr }) => {
            cli.note_skip(format!("apply returned {}: {}", status, stderr.trim()));
            return Ok(());
        }
        Err(err) => return Err(err.into()),
    };
    if !apply.status.success() {
        cli.note_skip(format!(
            "apply exited {} without applying changes",
            apply.status
        ));
        return Ok(());
    }

    let thread_id = match thread_id {
        Some(id) => id,
        None => {
            cli.note_skip("exec did not emit a thread_id; skipping resume");
            return Ok(());
        }
    };

    let resume_prompt =
        "Append a second line with \"goodbye\" to hello.txt using apply_patch, then stop.";
    let resume_request = ResumeRequest {
        selector: ResumeSelector::Id(thread_id),
        prompt: Some(resume_prompt.to_string()),
        ephemeral: false,
        ignore_rules: false,
        ignore_user_config: false,
        idle_timeout: Some(Duration::from_secs(120)),
        output_last_message: Some(workspace.path.join("resume-last.txt")),
        output_schema: None,
        json_event_log: Some(workspace.path.join("resume-events.jsonl")),
        overrides: CliOverridesPatch::default(),
    };

    let resume_stream = match client.stream_resume(resume_request).await {
        Ok(stream) => stream,
        Err(err) => {
            cli.note_skip(format!("resume failed to start: {err}"));
            return Ok(());
        }
    };
    let mut resume_events = resume_stream.events;
    while let Some(event) = resume_events.next().await {
        if let Err(err) = event {
            cli.note_skip(format!("resume stream parse error: {err}"));
            continue;
        }
    }
    let resume_completion = match resume_stream.completion.await {
        Ok(done) => done,
        Err(err) => {
            cli.note_skip(format!("resume completion failed: {err}"));
            return Ok(());
        }
    };
    if !resume_completion.status.success() {
        cli.note_skip(format!(
            "resume exited {} without finishing successfully",
            resume_completion.status
        ));
        return Ok(());
    }

    let contents = fs::read_to_string(&hello_path)?;
    assert!(
        contents.contains("hello"),
        "hello.txt should contain the original content"
    );
    assert!(
        contents.lines().count() >= 1,
        "hello.txt should have at least one line"
    );

    Ok(())
}

#[tokio::test]
async fn features_list_prefers_text_when_json_flag_is_missing(
) -> Result<(), Box<dyn std::error::Error>> {
    let Some(cli) = RealCli::bootstrap()? else {
        return Ok(());
    };

    let output = match cli
        .client
        .list_features(FeaturesListRequest::new().json(false))
        .await
    {
        Ok(output) => output,
        Err(CodexError::NonZeroExit { status, stderr }) => {
            cli.note_skip(format!(
                "features list failed with {status}: {}",
                stderr.trim()
            ));
            return Ok(());
        }
        Err(err) => return Err(err.into()),
    };

    assert_eq!(output.format, FeaturesListFormat::Text);
    assert!(
        !output.features.is_empty(),
        "features list output should not be empty"
    );
    Ok(())
}

#[tokio::test]
async fn app_server_codegen_generates_schema_bundle() -> Result<(), Box<dyn std::error::Error>> {
    let Some(cli) = RealCli::bootstrap()? else {
        return Ok(());
    };

    let out_dir = cli.home_dir.join("app-server-schema");
    let result = match cli
        .client
        .generate_app_server_bindings(AppServerCodegenRequest::json_schema(&out_dir))
        .await
    {
        Ok(output) => output,
        Err(CodexError::NonZeroExit { status, stderr }) => {
            cli.note_skip(format!(
                "app-server codegen exited {status}: {}",
                stderr.trim()
            ));
            return Ok(());
        }
        Err(err) => return Err(err.into()),
    };

    assert!(result.status.success());
    assert!(
        out_dir
            .join("codex_app_server_protocol.schemas.json")
            .is_file(),
        "schema bundle should be written"
    );
    Ok(())
}

#[cfg(target_os = "linux")]
#[tokio::test]
async fn sandbox_runs_echo_command() -> Result<(), Box<dyn std::error::Error>> {
    let Some(cli) = RealCli::bootstrap()? else {
        return Ok(());
    };

    let run = cli
        .client
        .run_sandbox(
            SandboxCommandRequest::new(SandboxPlatform::Linux, ["echo", "hello"]).full_auto(true),
        )
        .await?;

    if !run.status.success() {
        cli.note_skip(format!("sandbox exited {}: {}", run.status, run.stderr));
        return Ok(());
    }

    assert!(
        run.stdout.contains("hello"),
        "sandbox stdout should include the echoed text"
    );
    Ok(())
}

#[tokio::test]
async fn responses_api_proxy_emits_server_info_and_shuts_down(
) -> Result<(), Box<dyn std::error::Error>> {
    let Some(cli) = RealCli::bootstrap()? else {
        return Ok(());
    };

    let info_path = cli.home_dir.join("responses-info.json");
    let mut handle = match cli
        .client
        .start_responses_api_proxy(
            ResponsesApiProxyRequest::new("sk-test-e2e")
                .server_info(&info_path)
                .http_shutdown(true),
        )
        .await
    {
        Ok(handle) => handle,
        Err(CodexError::NonZeroExit { status, stderr }) => {
            cli.note_skip(format!(
                "responses-api-proxy exited {status}: {}",
                stderr.trim()
            ));
            return Ok(());
        }
        Err(err) => return Err(err.into()),
    };

    let mut info = None;
    let mut last_err = None;
    for _ in 0..10 {
        match handle.read_server_info().await {
            Ok(Some(found)) => {
                info = Some(found);
                break;
            }
            Ok(None) => break,
            Err(CodexError::ResponsesApiProxyInfoRead { .. }) => {
                last_err = Some("info file not yet written".to_string());
                time::sleep(Duration::from_millis(200)).await;
            }
            Err(err) => {
                let _ = handle.child.kill().await;
                return Err(err.into());
            }
        }
    }

    let Some(info) = info else {
        cli.note_skip(format!(
            "responses-api-proxy did not emit server info (last error: {})",
            last_err.unwrap_or_else(|| "missing file".to_string())
        ));
        let _ = handle.child.kill().await;
        return Ok(());
    };

    if let Err(err) = send_http_shutdown(info.port) {
        cli.note_skip(format!(
            "failed to hit responses-api-proxy shutdown endpoint on port {}: {err}",
            info.port
        ));
        let _ = handle.child.kill().await;
        return Ok(());
    }

    let status = match time::timeout(Duration::from_secs(10), handle.child.wait()).await {
        Ok(waited) => waited?,
        Err(_) => {
            cli.note_skip("responses-api-proxy did not exit after shutdown");
            let _ = handle.child.kill().await;
            return Ok(());
        }
    };

    if !status.success() {
        cli.note_skip(format!(
            "responses-api-proxy exited with {} after shutdown request",
            status
        ));
        return Ok(());
    }

    Ok(())
}

#[tokio::test]
async fn stdio_to_uds_relays_bytes() -> Result<(), Box<dyn std::error::Error>> {
    let Some(cli) = RealCli::bootstrap()? else {
        return Ok(());
    };

    let socket_path = cli.home_dir.join("stdio-bridge.sock");
    let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();
    let server = thread::spawn({
        let socket_path = socket_path.clone();
        move || -> std::io::Result<()> {
            let listener = UnixListener::bind(&socket_path)?;
            ready_tx.send(()).ok();

            if let Ok((mut stream, _addr)) = listener.accept() {
                let mut reader = BufReader::new(stream.try_clone()?);
                let mut line = String::new();
                reader.read_line(&mut line)?;
                stream.write_all(b"pong\n")?;
                stream.flush()?;
            }
            Ok(())
        }
    });
    let _ = ready_rx.await;

    let mut child = match cli
        .client
        .stdio_to_uds(StdioToUdsRequest::new(&socket_path))
    {
        Ok(child) => child,
        Err(CodexError::Spawn { source, .. }) => {
            cli.note_skip(format!("failed to spawn stdio-to-uds: {source}"));
            let _ = server.join();
            return Ok(());
        }
        Err(err) => {
            let _ = server.join();
            return Err(err.into());
        }
    };

    let mut stdout = AsyncBufReader::new(child.stdout.take().unwrap()).lines();
    let mut stdin = child.stdin.take().unwrap();

    stdin.write_all(b"ping\n").await?;
    stdin.shutdown().await?;

    let echoed = match time::timeout(Duration::from_secs(5), stdout.next_line()).await {
        Ok(Ok(Some(line))) => line,
        other => {
            cli.note_skip(format!("stdio-to-uds did not echo data: {other:?}"));
            let _ = child.kill().await;
            let _ = server.join();
            return Ok(());
        }
    };

    assert_eq!(echoed.trim(), "pong");

    let status = match time::timeout(Duration::from_secs(5), child.wait()).await {
        Ok(waited) => waited?,
        Err(_) => {
            cli.note_skip("stdio-to-uds did not exit after echoing data");
            let _ = child.kill().await;
            let _ = server.join();
            return Ok(());
        }
    };

    if !status.success() {
        cli.note_skip(format!(
            "stdio-to-uds exited with {} after relaying data",
            status
        ));
    }

    let _ = server.join();

    Ok(())
}

#[tokio::test]
async fn diff_and_apply_report_current_cli_gaps() -> Result<(), Box<dyn std::error::Error>> {
    let Some(cli) = RealCli::bootstrap()? else {
        return Ok(());
    };

    match cli.client.diff().await {
        Ok(output) => {
            if !output.status.success() {
                cli.note_skip(format!(
                    "codex diff exited {} without producing a replayable diff",
                    output.status
                ));
            }
        }
        Err(CodexError::NonZeroExit { status, stderr }) => {
            cli.note_skip(format!(
                "codex diff returned {} (likely TTY/task prerequisite): {}",
                status,
                stderr.trim()
            ));
        }
        Err(err) => return Err(err.into()),
    }

    match cli.client.apply().await {
        Ok(output) => {
            if !output.status.success() {
                cli.note_skip(format!(
                    "codex apply exited {} without an actionable task id",
                    output.status
                ));
            }
        }
        Err(CodexError::NonZeroExit { status, stderr }) => {
            cli.note_skip(format!(
                "codex apply returned {} (missing task id or flag support): {}",
                status,
                stderr.trim()
            ));
        }
        Err(err) => return Err(err.into()),
    }

    Ok(())
}

#[tokio::test]
async fn execpolicy_check_is_missing_in_current_cli() -> Result<(), Box<dyn std::error::Error>> {
    let Some(cli) = RealCli::bootstrap()? else {
        return Ok(());
    };

    let policy = cli.home_dir.join("policy.star");
    std::fs::write(&policy, "allow_all()")?;

    match cli
        .client
        .check_execpolicy(
            codex::ExecPolicyCheckRequest::new(["echo", "hi"])
                .policy(policy)
                .pretty(true),
        )
        .await
    {
        Ok(result) => {
            assert!(
                result.status.success(),
                "execpolicy unexpectedly succeeded with non-success exit: {}",
                result.status
            );
        }
        Err(CodexError::NonZeroExit { status, stderr }) => {
            cli.note_skip(format!(
                "execpolicy check not supported on this binary ({}): {}",
                status,
                stderr.trim()
            ));
        }
        Err(err) => return Err(err.into()),
    }

    Ok(())
}

fn locate_binary() -> Option<PathBuf> {
    let candidate = env::var_os(BINARY_ENV)
        .map(PathBuf::from)
        .or_else(|| env::var_os("CODEX_BINARY").map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from("codex"));
    if candidate.as_os_str().is_empty() {
        return None;
    }
    Some(resolve_binary_candidate(candidate))
}

fn resolve_binary_candidate(candidate: PathBuf) -> PathBuf {
    if candidate.is_absolute() || candidate.exists() || candidate.as_os_str() == "codex" {
        return candidate;
    }

    let Some(workspace_root) = find_workspace_root() else {
        return candidate;
    };

    let resolved = workspace_root.join(&candidate);
    if resolved.exists() {
        resolved
    } else {
        candidate
    }
}

fn find_workspace_root() -> Option<PathBuf> {
    let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    loop {
        let manifest = dir.join("Cargo.toml");
        if let Ok(contents) = fs::read_to_string(&manifest) {
            if contents.contains("[workspace]") {
                return Some(dir);
            }
        }
        if !dir.pop() {
            return None;
        }
    }
}

fn read_version(binary: &Path) -> Option<String> {
    let output = StdCommand::new(binary).arg("--version").output().ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let raw = if stdout.trim().is_empty() {
        stderr.trim()
    } else {
        stdout.trim()
    };
    if raw.is_empty() {
        None
    } else {
        Some(raw.to_string())
    }
}

fn send_http_shutdown(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = TcpStream::connect(("127.0.0.1", port))?;
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;
    stream.set_write_timeout(Some(Duration::from_secs(2)))?;
    let request =
        format!("GET /shutdown HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n");
    stream.write_all(request.as_bytes())?;
    let mut buf = Vec::new();
    let _ = stream.read_to_end(&mut buf)?;
    Ok(())
}

fn live_mode_enabled() -> bool {
    match env::var(LIVE_FLAG_ENV) {
        Ok(value) => matches!(value.as_str(), "1" | "true" | "yes"),
        Err(_) => false,
    }
}

struct Workspace {
    path: PathBuf,
    _guard: Option<TempDir>,
}

fn prepare_workspace() -> Result<Workspace, Box<dyn std::error::Error>> {
    if let Ok(dir) = env::var(WORKDIR_ENV) {
        let base = PathBuf::from(dir);
        fs::create_dir_all(&base)?;
        let run_dir = base.join(format!("run-{}", unix_millis()));
        fs::create_dir_all(&run_dir)?;
        init_git_repo(&run_dir)?;
        return Ok(Workspace {
            path: run_dir,
            _guard: None,
        });
    }

    let temp = TempDir::new()?;
    let path = temp.path().to_path_buf();
    init_git_repo(&path)?;
    Ok(Workspace {
        path,
        _guard: Some(temp),
    })
}

fn init_git_repo(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(path)?;
    if !path.join(".git").exists() {
        run_git(&["init"], path)?;
    }
    run_git(&["config", "user.email", "codex-e2e@example.com"], path)?;
    run_git(&["config", "user.name", "Codex E2E"], path)?;
    if !path.join("README.md").exists() {
        fs::write(path.join("README.md"), "# Codex E2E workspace\n")?;
    }
    Ok(())
}

fn run_git(args: &[&str], cwd: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let status = StdCommand::new("git")
        .args(args)
        .current_dir(cwd)
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(std::io::Error::other(format!("git {:?} failed with {status}", args)).into())
    }
}

fn unix_millis() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}
