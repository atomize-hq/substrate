use super::{
    read_last_message, unique_temp_path, ExecCompletion, ExecStream, ExecStreamControl,
    ExecStreamError, ExecStreamRequest, ExecTerminationHandle, ResumeRequest, ResumeSelector,
};
use crate::{
    builder::{apply_cli_overrides, resolve_cli_overrides},
    capabilities::{guard_is_supported, log_guard_skip},
    jsonl,
    process_custody::{drive_child, OwnedCompletion},
    CliOverridesPatch, CodexClient, CodexError, ExecProcessCustody,
};
use std::collections::BTreeMap;
use tokio::{
    process::Command,
    sync::{mpsc, oneshot},
};

pub(super) async fn stream_exec_with_overrides(
    client: &CodexClient,
    request: ExecStreamRequest,
    overrides: CliOverridesPatch,
) -> Result<ExecStream, ExecStreamError> {
    stream_exec_with_overrides_and_env_overrides(client, request, overrides, &[]).await
}

pub(super) async fn stream_exec_with_overrides_and_env_overrides(
    client: &CodexClient,
    request: ExecStreamRequest,
    overrides: CliOverridesPatch,
    env_overrides: &[(String, String)],
) -> Result<ExecStream, ExecStreamError> {
    let control = stream_exec_with_overrides_and_env_overrides_control(
        client,
        request,
        overrides,
        env_overrides,
    )
    .await?;

    Ok(ExecStream {
        events: control.events,
        completion: control.completion,
    })
}

pub(super) async fn stream_exec_with_overrides_and_env_overrides_control(
    client: &CodexClient,
    request: ExecStreamRequest,
    overrides: CliOverridesPatch,
    env_overrides: &[(String, String)],
) -> Result<ExecStreamControl, ExecStreamError> {
    let custody = ExecProcessCustody::new();
    stream_exec_with_custody(client, request, overrides, env_overrides, &custody).await
}

pub(super) async fn stream_exec_with_custody(
    client: &CodexClient,
    request: ExecStreamRequest,
    overrides: CliOverridesPatch,
    env_overrides: &[(String, String)],
    custody: &ExecProcessCustody,
) -> Result<ExecStreamControl, ExecStreamError> {
    custody.ensure_open()?;
    if request.prompt.trim().is_empty() {
        return Err(CodexError::EmptyPrompt.into());
    }

    let ExecStreamRequest {
        prompt,
        ephemeral,
        ignore_rules,
        ignore_user_config,
        idle_timeout,
        output_last_message,
        output_schema,
        json_event_log,
    } = request;

    let dir_ctx = client.directory_context()?;
    let dir_path = dir_ctx.path().to_path_buf();
    let last_message_path =
        output_last_message.unwrap_or_else(|| unique_temp_path("codex_last_message_", "txt"));
    let needs_capabilities = output_schema.is_some() || !client.add_dirs.is_empty();
    let capabilities = if needs_capabilities {
        if env_overrides.is_empty() {
            Some(
                client
                    .probe_capabilities_for_current_dir_with_custody(&dir_path, Some(custody))
                    .await,
            )
        } else {
            let env_overrides_map: BTreeMap<String, String> =
                env_overrides.iter().cloned().collect();
            Some(
                client
                    .probe_capabilities_with_env_overrides_for_current_dir_with_custody(
                        &env_overrides_map,
                        &dir_path,
                        Some(custody),
                    )
                    .await,
            )
        }
    } else {
        None
    };
    let resolved_overrides =
        resolve_cli_overrides(&client.cli_overrides, &overrides, client.model.as_deref());

    let mut command = Command::new(client.command_env.binary_path());
    command
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .stdin(std::process::Stdio::piped())
        .kill_on_drop(true)
        .current_dir(&dir_path);

    apply_cli_overrides(&mut command, &resolved_overrides, true);
    command
        .arg("exec")
        .arg("--color")
        .arg(client.color_mode.as_str())
        .arg("--skip-git-repo-check")
        .arg("--json");

    if ephemeral {
        command.arg("--ephemeral");
    }
    if ignore_rules {
        command.arg("--ignore-rules");
    }
    if ignore_user_config {
        command.arg("--ignore-user-config");
    }

    if let Some(model) = &client.model {
        command.arg("--model").arg(model);
    }

    if let Some(capabilities) = &capabilities {
        if !client.add_dirs.is_empty() {
            let guard = capabilities.guard_add_dir();
            if guard_is_supported(&guard) {
                for dir in &client.add_dirs {
                    command.arg("--add-dir").arg(dir);
                }
            } else {
                log_guard_skip(&guard);
            }
        }
    }

    for image in &client.images {
        command.arg("--image").arg(image);
    }

    command.arg("--output-last-message").arg(&last_message_path);

    if let Some(schema_path) = &output_schema {
        if let Some(capabilities) = &capabilities {
            let guard = capabilities.guard_output_schema();
            if guard_is_supported(&guard) {
                command.arg("--output-schema").arg(schema_path);
            } else {
                log_guard_skip(&guard);
            }
        } else {
            command.arg("--output-schema").arg(schema_path);
        }
    }

    client.command_env.apply(&mut command)?;
    for (key, value) in env_overrides {
        command.env(key, value);
    }

    let mut input = prompt.into_bytes();
    input.push(b'\n');
    launch_stream(
        client,
        command,
        dir_ctx,
        Some(input),
        idle_timeout,
        json_event_log,
        last_message_path,
        output_schema,
        custody,
    )
    .await
}

pub(super) async fn stream_resume(
    client: &CodexClient,
    request: ResumeRequest,
) -> Result<ExecStream, ExecStreamError> {
    let control = stream_resume_with_env_overrides_control(client, request, &[]).await?;

    Ok(ExecStream {
        events: control.events,
        completion: control.completion,
    })
}

pub(super) async fn stream_resume_with_env_overrides_control(
    client: &CodexClient,
    request: ResumeRequest,
    env_overrides: &[(String, String)],
) -> Result<ExecStreamControl, ExecStreamError> {
    let custody = ExecProcessCustody::new();
    stream_resume_with_custody(client, request, env_overrides, &custody).await
}

pub(super) async fn stream_resume_with_custody(
    client: &CodexClient,
    request: ResumeRequest,
    env_overrides: &[(String, String)],
    custody: &ExecProcessCustody,
) -> Result<ExecStreamControl, ExecStreamError> {
    custody.ensure_open()?;
    if let Some(prompt) = &request.prompt {
        if prompt.trim().is_empty() {
            return Err(CodexError::EmptyPrompt.into());
        }
    }

    let ResumeRequest {
        selector,
        prompt,
        ephemeral,
        ignore_rules,
        ignore_user_config,
        idle_timeout,
        output_last_message,
        output_schema,
        json_event_log,
        overrides,
    } = request;

    let dir_ctx = client.directory_context()?;
    let dir_path = dir_ctx.path().to_path_buf();
    let last_message_path =
        output_last_message.unwrap_or_else(|| unique_temp_path("codex_last_message_", "txt"));
    let needs_capabilities = output_schema.is_some() || !client.add_dirs.is_empty();
    let capabilities = if needs_capabilities {
        if env_overrides.is_empty() {
            Some(
                client
                    .probe_capabilities_for_current_dir_with_custody(&dir_path, Some(custody))
                    .await,
            )
        } else {
            let env_overrides_map: BTreeMap<String, String> =
                env_overrides.iter().cloned().collect();
            Some(
                client
                    .probe_capabilities_with_env_overrides_for_current_dir_with_custody(
                        &env_overrides_map,
                        &dir_path,
                        Some(custody),
                    )
                    .await,
            )
        }
    } else {
        None
    };
    let resolved_overrides =
        resolve_cli_overrides(&client.cli_overrides, &overrides, client.model.as_deref());

    let mut command = Command::new(client.command_env.binary_path());
    command
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .stdin(std::process::Stdio::piped())
        .kill_on_drop(true)
        .current_dir(&dir_path);

    apply_cli_overrides(&mut command, &resolved_overrides, true);
    command
        .arg("exec")
        .arg("--color")
        .arg(client.color_mode.as_str())
        .arg("--skip-git-repo-check")
        .arg("--json");

    if ephemeral {
        command.arg("--ephemeral");
    }
    if ignore_rules {
        command.arg("--ignore-rules");
    }
    if ignore_user_config {
        command.arg("--ignore-user-config");
    }

    if let Some(model) = &client.model {
        command.arg("--model").arg(model);
    }

    if let Some(capabilities) = &capabilities {
        if !client.add_dirs.is_empty() {
            let guard = capabilities.guard_add_dir();
            if guard_is_supported(&guard) {
                for dir in &client.add_dirs {
                    command.arg("--add-dir").arg(dir);
                }
            } else {
                log_guard_skip(&guard);
            }
        }
    }

    for image in &client.images {
        command.arg("--image").arg(image);
    }

    command.arg("--output-last-message").arg(&last_message_path);

    if let Some(schema_path) = &output_schema {
        if let Some(capabilities) = &capabilities {
            let guard = capabilities.guard_output_schema();
            if guard_is_supported(&guard) {
                command.arg("--output-schema").arg(schema_path);
            } else {
                log_guard_skip(&guard);
            }
        } else {
            command.arg("--output-schema").arg(schema_path);
        }
    }

    command.arg("resume");

    match selector {
        ResumeSelector::Id(id) => {
            command.arg(id);
        }
        ResumeSelector::Last => {
            command.arg("--last");
        }
        ResumeSelector::All => {
            command.arg("--all");
        }
    }

    if prompt.is_some() {
        // `codex exec resume` reads the follow-up prompt from stdin when `-` is supplied.
        command.arg("-");
    }

    client.command_env.apply(&mut command)?;
    for (key, value) in env_overrides {
        command.env(key, value);
    }

    let input = prompt.map(|prompt| {
        let mut bytes = prompt.into_bytes();
        bytes.push(b'\n');
        bytes
    });
    launch_stream(
        client,
        command,
        dir_ctx,
        input,
        idle_timeout,
        json_event_log,
        last_message_path,
        output_schema,
        custody,
    )
    .await
}

#[allow(clippy::too_many_arguments)]
async fn launch_stream(
    client: &CodexClient,
    mut command: Command,
    dir_ctx: crate::client_core::DirectoryContext,
    input: Option<Vec<u8>>,
    idle_timeout: Option<std::time::Duration>,
    json_event_log: Option<std::path::PathBuf>,
    last_message_path: std::path::PathBuf,
    schema_path: Option<std::path::PathBuf>,
    custody: &ExecProcessCustody,
) -> Result<ExecStreamControl, ExecStreamError> {
    // All fallible asynchronous file preparation precedes child admission.
    let json_log = jsonl::prepare_json_log(
        json_event_log
            .or_else(|| client.json_event_log.clone())
            .filter(|path| !path.as_os_str().is_empty()),
    )
    .await?;
    let (child, owner) = custody
        .spawn_admitted(&mut command, client.command_env.binary_path())
        .await?;
    // Child still owns every pipe here; move it into the independent runner without any await.
    let (tx, rx) = mpsc::channel(32);
    let (ready_tx, ready_rx) = oneshot::channel();
    let (completion_tx, completion_rx) = oneshot::channel();
    let timeout = client.timeout;
    let mirror_stdout = client.mirror_stdout;
    let mirror_stderr = !client.quiet;
    tokio::spawn(async move {
        let _dir_ctx = dir_ctx;
        let result = drive_child(
            child,
            owner,
            input,
            true,
            (!timeout.is_zero()).then_some(timeout),
            true,
            mirror_stderr,
            ready_tx,
            move |stdout| async move {
                jsonl::forward_json_events(stdout, tx, mirror_stdout, json_log)
                    .await
                    .map_err(std::io::Error::other)
            },
        )
        .await;
        let completion = async {
            let status = result
                .status
                .map_err(|source| CodexError::Wait { source })?;
            let _ = result.stdout.map_err(CodexError::CaptureIo)?;
            let stderr = result
                .stderr
                .map_err(CodexError::CaptureIo)?
                .unwrap_or_default();
            if result.settlement_failed {
                return Err(CodexError::CaptureIo(std::io::Error::other(
                    "process settlement failed",
                ))
                .into());
            }
            if result.timed_out {
                return Err(CodexError::Timeout { timeout }.into());
            }
            if !status.success() {
                return Err(CodexError::NonZeroExit {
                    status,
                    stderr: String::from_utf8(stderr).unwrap_or_default(),
                }
                .into());
            }
            result.stdin.map_err(CodexError::StdinWrite)?;
            let last_message = read_last_message(&last_message_path).await;
            Ok(ExecCompletion {
                status,
                last_message_path: Some(last_message_path),
                last_message,
                schema_path,
            })
        }
        .await;
        let _ = completion_tx.send(completion);
    });
    let mut startup_drop = custody.cancel_on_drop();
    ready_rx
        .await
        .map_err(|_| CodexError::CaptureIo(std::io::Error::other("stream startup owner failed")))?
        .map_err(CodexError::StdinWrite)?;
    startup_drop.disarm();
    let completion = OwnedCompletion {
        receiver: completion_rx,
        cancel: custody.cancel_on_drop(),
    };
    Ok(ExecStreamControl {
        events: Box::pin(jsonl::EventChannelStream::new(rx, idle_timeout)),
        completion: Box::pin(async move {
            completion.await.map_err(|_| {
                CodexError::CaptureIo(std::io::Error::other("stream completion owner failed"))
            })?
        }),
        termination: ExecTerminationHandle::with_custody(custody.clone()),
    })
}
