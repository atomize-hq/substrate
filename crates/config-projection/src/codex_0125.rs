use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use serde_json::json;
use sha2::{Digest, Sha256};

use crate::{
    CodexAmbientConfigClosureV1, CodexLoaderInputAttestationV1, CodexLoaderInputDispositionV1,
    CodexLoaderSourceIdentityV1, CodexNativeInvocationV1, ConfigProjectionCodecV1,
    ConfigProjectionFailureV1, ConfigProjectionIdentityV1, EffectiveAgentConfigProjectionV1,
    EffectiveEnvironmentV1, LogicalFeatureV1, LogicalMcpTransportV1, ManagedGatewayProjectionV1,
    NamedValueV1, NativeAgentConfigProjectionV1, NativeProjectedDirectoryV1,
    NativeProjectedFileRoleV1, NativeProjectedFileV1, NativeProjectionRootV1,
    NativeRendererIdentityV1, WorkspaceOverlayPostureV1,
};

pub struct Codex0125ProjectionV1;

impl Codex0125ProjectionV1 {
    pub fn render(
        identity: &ConfigProjectionIdentityV1,
        effective: &EffectiveAgentConfigProjectionV1,
        managed_gateway: &ManagedGatewayProjectionV1,
        root: NativeProjectionRootV1,
        fence_id: &str,
        loader_inputs: Vec<CodexLoaderInputAttestationV1>,
    ) -> Result<NativeAgentConfigProjectionV1, ConfigProjectionFailureV1> {
        struct RenderConfigTomlInput<'a> {
            model: &'a str,
            mcp_servers: &'a [crate::EffectiveMcpServerV1],
            features: &'a [LogicalFeatureV1],
            codex_base_url: &'a str,
            root_guest_absolute_path: &'a str,
            workspace_physical_path: &'a str,
            orchestration_session_id: &'a str,
            retained_participant_id: &'a str,
            identity_hash: &'a str,
        }

        fn render_config_toml(
            input: RenderConfigTomlInput<'_>,
        ) -> Result<Vec<u8>, ConfigProjectionFailureV1> {
            fn toml_string(value: &str) -> String {
                let mut rendered = String::from("\"");
                for character in value.chars() {
                    match character {
                        '"' => rendered.push_str("\\\""),
                        '\\' => rendered.push_str("\\\\"),
                        '\u{0008}' => rendered.push_str("\\b"),
                        '\t' => rendered.push_str("\\t"),
                        '\n' => rendered.push_str("\\n"),
                        '\u{000c}' => rendered.push_str("\\f"),
                        '\r' => rendered.push_str("\\r"),
                        character if character <= '\u{001f}' || character == '\u{007f}' => {
                            rendered.push_str(&format!("\\u{:04x}", character as u32));
                        }
                        character => rendered.push(character),
                    }
                }
                rendered.push('"');
                rendered
            }
            fn ensure_sorted_unique<'a>(
                values: impl Iterator<Item = &'a str>,
            ) -> Result<(), ConfigProjectionFailureV1> {
                let mut previous: Option<&[u8]> = None;
                for value in values {
                    if value.is_empty() || previous.is_some_and(|item| value.as_bytes() <= item) {
                        return Err(ConfigProjectionFailureV1::Conflict);
                    }
                    previous = Some(value.as_bytes());
                }
                Ok(())
            }
            fn validate_named_values(
                values: &[NamedValueV1],
            ) -> Result<(), ConfigProjectionFailureV1> {
                ensure_sorted_unique(values.iter().map(|value| value.name.as_str()))?;
                for value in values {
                    let upper = value.name.to_ascii_uppercase();
                    if upper.contains("AUTHORIZATION")
                        || upper.contains("COOKIE")
                        || upper.ends_with("_TOKEN")
                        || upper.ends_with("_SECRET")
                        || upper.ends_with("_KEY")
                    {
                        return Err(ConfigProjectionFailureV1::Malformed);
                    }
                }
                Ok(())
            }
            fn toml_array(values: &[String]) -> String {
                format!(
                    "[{}]",
                    values
                        .iter()
                        .map(|value| toml_string(value))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            fn toml_table(values: &[NamedValueV1]) -> String {
                format!(
                    "{{ {} }}",
                    values
                        .iter()
                        .map(|value| format!(
                            "{} = {}",
                            toml_string(&value.name),
                            toml_string(&value.value)
                        ))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            fn known_feature(name: &str) -> bool {
                matches!(
                    name,
                    "apps"
                        | "apply_patch_freeform"
                        | "artifact"
                        | "browser_use"
                        | "code_mode"
                        | "codex_hooks"
                        | "computer_use"
                        | "enable_request_compression"
                        | "fast_mode"
                        | "guardian_approval"
                        | "image_generation"
                        | "js_repl"
                        | "memories"
                        | "multi_agent"
                        | "personality"
                        | "request_permissions_tool"
                        | "runtime_metrics"
                        | "shell_snapshot"
                        | "shell_tool"
                        | "tool_call_mcp_elicitation"
                        | "tool_search"
                        | "tool_suggest"
                        | "unified_exec"
                        | "undo"
                        | "workspace_dependencies"
                )
            }

            let RenderConfigTomlInput {
                model,
                mcp_servers,
                features,
                codex_base_url,
                root_guest_absolute_path,
                workspace_physical_path,
                orchestration_session_id,
                retained_participant_id,
                identity_hash,
            } = input;

            if codex_base_url.contains('#')
                || codex_base_url.contains('@')
                || !(codex_base_url.starts_with("http://127.0.0.1:")
                    || codex_base_url.starts_with("http://[::1]:"))
            {
                return Err(ConfigProjectionFailureV1::UnsupportedConfiguration);
            }
            ensure_sorted_unique(mcp_servers.iter().map(|server| server.server_id.as_str()))?;
            ensure_sorted_unique(features.iter().map(|feature| feature.name.as_str()))?;

            let mut output = String::new();
            output.push_str(&format!("model = {}\n", toml_string(model)));
            output.push_str("model_provider = \"substrate-managed-gateway\"\n");
            output.push_str("check_for_update_on_startup = false\n");
            output.push_str("cli_auth_credentials_store = \"ephemeral\"\n");
            output.push_str(&format!(
                "log_dir = {}\n\n",
                toml_string(&format!("{root_guest_absolute_path}/state/log"))
            ));
            output.push_str("[model_providers.substrate-managed-gateway]\n");
            output.push_str("name = \"Substrate managed gateway\"\n");
            output.push_str(&format!("base_url = {}\n", toml_string(codex_base_url)));
            output.push_str("wire_api = \"responses\"\n");
            output.push_str("requires_openai_auth = false\n");
            output.push_str("supports_websockets = false\n");
            output.push_str(&format!(
                "http_headers = {{ \"X-Substrate-Orchestration-Session\" = {}, \"X-Substrate-Participant\" = {}, \"X-Substrate-Projection\" = {} }}\n\n",
                toml_string(orchestration_session_id),
                toml_string(retained_participant_id),
                toml_string(identity_hash),
            ));
            output.push_str(&format!(
                "[projects.{}]\ntrust_level = \"untrusted\"\n",
                toml_string(workspace_physical_path)
            ));

            for server in mcp_servers {
                output.push_str(&format!(
                    "\n[mcp_servers.{}]\nenabled = {}\n",
                    toml_string(&server.server_id),
                    server.enabled
                ));
                match &server.transport {
                    LogicalMcpTransportV1::Stdio {
                        command,
                        args,
                        nonsecret_env,
                    } => {
                        validate_named_values(nonsecret_env)?;
                        output.push_str(&format!("command = {}\n", toml_string(command)));
                        output.push_str(&format!("args = {}\n", toml_array(args)));
                        if !nonsecret_env.is_empty() {
                            output.push_str(&format!("env = {}\n", toml_table(nonsecret_env)));
                        }
                    }
                    LogicalMcpTransportV1::StreamableHttp {
                        url,
                        nonsecret_headers,
                    } => {
                        if url.contains('#') || url.contains('@') {
                            return Err(ConfigProjectionFailureV1::Malformed);
                        }
                        validate_named_values(nonsecret_headers)?;
                        output.push_str(&format!("url = {}\n", toml_string(url)));
                        if !nonsecret_headers.is_empty() {
                            output.push_str(&format!(
                                "http_headers = {}\n",
                                toml_table(nonsecret_headers)
                            ));
                        }
                    }
                }
            }
            if !features.is_empty() {
                output.push_str("\n[features]\n");
                for LogicalFeatureV1 { name, enabled } in features {
                    if !known_feature(name) {
                        return Err(ConfigProjectionFailureV1::Malformed);
                    }
                    output.push_str(&format!("{} = {}\n", toml_string(name), enabled));
                }
            }
            Ok(output.into_bytes())
        }

        if effective.model != "codex"
            || effective.provider.provider_id != "substrate-managed-gateway"
            || effective.provider.wire_api != "responses"
            || effective.provider.requires_openai_auth
            || effective.provider.supports_websockets
            || effective.workspace_overlay != WorkspaceOverlayPostureV1::Disabled
            || effective.provider.gateway_intent_ref != managed_gateway.activation_intent_ref
            || identity.workspace_root.physical_path.is_empty()
        {
            return Err(ConfigProjectionFailureV1::UnsupportedConfiguration);
        }
        if !effective.mcp_servers.is_empty() || !effective.features.is_empty() {
            return Err(ConfigProjectionFailureV1::UnsupportedPolicySurface);
        }
        let expected_authority_path = format!(
            "authority-v1/agent-config-projection-v1/native-sources/{}/{}",
            identity.series_id, fence_id
        );
        let expected_guest_path = format!(
            "/run/substrate/member-config/{}/{}",
            identity.series_id, fence_id
        );
        if root.directory_mode != 0o700
            || root.authority_relative_path != expected_authority_path
            || root.guest_absolute_path != expected_guest_path
        {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }

        let environment = EffectiveEnvironmentV1 {
            inherited_names: Vec::new(),
            set: [
                (
                    "CODEX_HOME",
                    format!("{}/codex-home", root.guest_absolute_path),
                ),
                (
                    "CODEX_SQLITE_HOME",
                    format!("{}/state/sqlite", root.guest_absolute_path),
                ),
                ("HOME", format!("{}/home", root.guest_absolute_path)),
                ("LANG", "C.UTF-8".to_string()),
                ("LC_ALL", "C.UTF-8".to_string()),
                (
                    "PATH",
                    "/var/lib/substrate/world-deps/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"
                        .to_string(),
                ),
                ("RUST_LOG", "error".to_string()),
                ("TMPDIR", format!("{}/tmp", root.guest_absolute_path)),
            ]
            .into_iter()
            .map(|(name, value)| NamedValueV1 {
                name: name.to_string(),
                value,
            })
            .collect(),
            remove: [
                "ANTHROPIC_API_KEY",
                "CODEX_API_KEY",
                "CODEX_BINARY",
                "CODEX_OSS_BASE_URL",
                "CODEX_OSS_PORT",
                "OPENAI_ACCESS_TOKEN",
                "OPENAI_API_KEY",
                "OPENAI_BASE_URL",
                "OPENAI_ORGANIZATION",
                "OPENAI_PROJECT",
                "SUBSTRATE_E3_CODEX_LAUNCH_PLAN_FD",
                "SUBSTRATE_E3_NATIVE_REALIZATION_FD",
                "SUBSTRATE_E3_NATIVE_SOURCE_FD",
                "SUBSTRATE_E3_SYSTEM_EMPTY_FD",
                "SUBSTRATE_E3_WORLD_FS_INPUT_FD",
                "SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME",
                "SUBSTRATE_LLM_AUTH_BUNDLE_FD",
                "SUBSTRATE_WORLD_ENTRY_BINARY",
                "SUBSTRATE_WORLD_ENTRY_BINARY_FD",
                "SUBSTRATE_WORLD_ENTRY_CGROUP_PROCS_FD",
                "SUBSTRATE_WORLD_ENTRY_CGROUP_PROCS_PATH",
                "SUBSTRATE_WORLD_ENTRY_FINAL_EXEC_FD",
                "SUBSTRATE_WORLD_ENTRY_REQUIRE_CGROUP_ATTACH",
                "SUBSTRATE_WORLD_ENTRY_ROLE",
                "SUBSTRATE_WORLD_ENTRY_SETUP_READY_FD",
                "SUBSTRATE_WORLD_ENTRY_WORKING_DIR",
                "SUBSTRATE_WORLD_ENTRY_WORKING_DIR_FD",
            ]
            .into_iter()
            .map(str::to_string)
            .collect(),
        };
        if effective.environment != environment {
            return Err(ConfigProjectionFailureV1::UnsupportedConfiguration);
        }
        let config_bytes = render_config_toml(RenderConfigTomlInput {
            model: &effective.model,
            mcp_servers: &effective.mcp_servers,
            features: &effective.features,
            codex_base_url: &managed_gateway.codex_base_url,
            root_guest_absolute_path: &root.guest_absolute_path,
            workspace_physical_path: &identity.workspace_root.physical_path,
            orchestration_session_id: &identity.orchestration_session_id,
            retained_participant_id: &identity.retained_participant_id,
            identity_hash: &identity.identity_hash,
        })?;
        let config_hash = format!("{:x}", Sha256::digest(&config_bytes));
        let projected_config = loader_inputs
            .iter()
            .filter(|input| {
                input.layer == "User"
                    && input.relative_path == "config.toml"
                    && input.disposition == CodexLoaderInputDispositionV1::Projected
            })
            .collect::<Vec<_>>();
        if projected_config.len() != 1
            || projected_config[0].directory.physical_path
                != format!("{}/codex-home", root.guest_absolute_path)
            || projected_config[0].byte_length != Some(config_bytes.len() as u64)
            || projected_config[0].sha256.as_deref() != Some(config_hash.as_str())
        {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        let loader_source = CodexLoaderSourceIdentityV1 {
            codex_version: "0.125.0".to_string(),
            upstream_tag: "rust-v0.125.0".to_string(),
            config_loader_source_sha256:
                "f8e2eff1db4d4cc004dff849682e81e224acca742b5d38d94e2db4a713b560bc".to_string(),
            layer_io_source_sha256:
                "5b8aa76b0776370cc6db86b679efbb6bab361d2d474dc59768724f8a4c80a52a".to_string(),
            loader_model_source_sha256:
                "8c6573bc83f53396a31bae92371790997c30521e46d29c0662c44d58d570ea81".to_string(),
            exec_source_sha256: "b113fd23d8a0d264556b234fb067a4233ab8c3d5491dd393c0bc39b53c3170bd"
                .to_string(),
            cloud_requirements_source_sha256:
                "f43176f2889c54d19b65b0a669e98e22effc52712ca38710b9265ee22c804c77".to_string(),
            auth_storage_source_sha256:
                "31c05505d2ed91f852a225e154929db3083ae61ef8a662fb6aad09442c33650c".to_string(),
            config_types_source_sha256:
                "927c2a72b29136a5d8f627450a1f85a91173bc76e863351fd7233664b5ee32e4".to_string(),
            validator_schema_version: 1,
        };
        let fingerprint = ConfigProjectionCodecV1::domain_sha256(
            "substrate.e3.codex-0.125-loader-inputs.v1",
            &json!({"inputs": loader_inputs, "loader_source": loader_source}),
        )?;
        let ambient_closure = CodexAmbientConfigClosureV1 {
            loader_source,
            allowed_enabled_layers: vec!["System".to_string(), "User".to_string()],
            inputs: loader_inputs,
            forbidden_cli_overrides: vec![
                "--config".to_string(),
                "--ignore-rules".to_string(),
                "--ignore-user-config".to_string(),
                "--model".to_string(),
                "--oss".to_string(),
                "--profile".to_string(),
                "-c".to_string(),
            ],
            validated_loader_input_fingerprint: fingerprint,
        };
        Self::validate_loader_inputs(&ambient_closure)?;
        let project_inputs = ambient_closure
            .inputs
            .iter()
            .filter(|input| input.layer == "Project")
            .collect::<Vec<_>>();
        if project_inputs.len() != 3
            || project_inputs
                .iter()
                .any(|input| input.directory != identity.workspace_root)
        {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }

        let output_name_hash = ConfigProjectionCodecV1::domain_sha256(
            "substrate.e3.codex-output-last-message-name.v1",
            &json!({
                "fence_id": fence_id,
                "series_id": identity.series_id,
                "turn_id": identity.bootstrap_run_id,
            }),
        )?;
        let output_directory = format!("{}/tmp/output-last-message", root.guest_absolute_path);
        let output_path = format!("{output_directory}/olm_{output_name_hash}.txt");
        let fixed_prefix = vec![
            "codex".to_string(),
            "--dangerously-bypass-approvals-and-sandbox".to_string(),
            "exec".to_string(),
            "--color".to_string(),
            "never".to_string(),
            "--skip-git-repo-check".to_string(),
            "--json".to_string(),
            "--output-last-message".to_string(),
        ];
        let mut initial_argv = fixed_prefix.clone();
        initial_argv.push(output_path);

        let mut native = NativeAgentConfigProjectionV1 {
            projection_hash: String::new(),
            renderer: NativeRendererIdentityV1 {
                renderer_id: "substrate.codex.config-renderer".to_string(),
                renderer_schema_version: 1,
                codex_version: "0.125.0".to_string(),
            },
            root,
            files: vec![NativeProjectedFileV1 {
                role: NativeProjectedFileRoleV1::CodexConfigToml,
                relative_path: "codex-home/config.toml".to_string(),
                mode: 0o600,
                bytes_base64: STANDARD.encode(&config_bytes),
                byte_length: config_bytes.len() as u64,
                sha256: config_hash,
            }],
            directories: [
                ".",
                "system-empty",
                "home",
                "codex-home",
                "state",
                "state/sqlite",
                "state/log",
                "tmp",
                "tmp/output-last-message",
            ]
            .into_iter()
            .map(|relative_path| NativeProjectedDirectoryV1 {
                relative_path: relative_path.to_string(),
                mode: 0o700,
            })
            .collect(),
            environment,
            invocation: CodexNativeInvocationV1 {
                wrapper_argv: vec!["substrate-world-entry".to_string()],
                initial_codex_argv: initial_argv,
                resume_codex_argv_prefix: fixed_prefix,
                prompt_delivery: "stdin-lf-eof".to_string(),
                output_last_message_directory: output_directory,
                output_last_message_name_domain: "substrate.e3.codex-output-last-message-name.v1"
                    .to_string(),
                forbidden_arguments: vec![
                    "--config".to_string(),
                    "--ignore-rules".to_string(),
                    "--ignore-user-config".to_string(),
                    "--model".to_string(),
                    "--oss".to_string(),
                    "--profile".to_string(),
                    "-c".to_string(),
                ],
            },
            cwd: identity.workspace_root.clone(),
            ambient_closure,
        };
        let mut value =
            serde_json::to_value(&native).map_err(|_| ConfigProjectionFailureV1::Malformed)?;
        value
            .as_object_mut()
            .ok_or(ConfigProjectionFailureV1::Malformed)?
            .remove("projection_hash");
        native.projection_hash = ConfigProjectionCodecV1::domain_sha256(
            "",
            &json!({
                "domain": "substrate.e3.native-config-projection.v1",
                "projection": value,
            }),
        )?;
        Ok(native)
    }

    pub fn validate_loader_inputs(
        closure: &CodexAmbientConfigClosureV1,
    ) -> Result<(), ConfigProjectionFailureV1> {
        let pinned_source = CodexLoaderSourceIdentityV1 {
            codex_version: "0.125.0".to_string(),
            upstream_tag: "rust-v0.125.0".to_string(),
            config_loader_source_sha256:
                "f8e2eff1db4d4cc004dff849682e81e224acca742b5d38d94e2db4a713b560bc".to_string(),
            layer_io_source_sha256:
                "5b8aa76b0776370cc6db86b679efbb6bab361d2d474dc59768724f8a4c80a52a".to_string(),
            loader_model_source_sha256:
                "8c6573bc83f53396a31bae92371790997c30521e46d29c0662c44d58d570ea81".to_string(),
            exec_source_sha256: "b113fd23d8a0d264556b234fb067a4233ab8c3d5491dd393c0bc39b53c3170bd"
                .to_string(),
            cloud_requirements_source_sha256:
                "f43176f2889c54d19b65b0a669e98e22effc52712ca38710b9265ee22c804c77".to_string(),
            auth_storage_source_sha256:
                "31c05505d2ed91f852a225e154929db3083ae61ef8a662fb6aad09442c33650c".to_string(),
            config_types_source_sha256:
                "927c2a72b29136a5d8f627450a1f85a91173bc76e863351fd7233664b5ee32e4".to_string(),
            validator_schema_version: 1,
        };
        if closure.loader_source != pinned_source {
            return Err(ConfigProjectionFailureV1::UnsupportedRuntimeVersion);
        }
        if closure.allowed_enabled_layers != ["System", "User"]
            || closure.forbidden_cli_overrides
                != [
                    "--config",
                    "--ignore-rules",
                    "--ignore-user-config",
                    "--model",
                    "--oss",
                    "--profile",
                    "-c",
                ]
        {
            return Err(ConfigProjectionFailureV1::UnsupportedPolicySurface);
        }

        let mut previous: Option<(u8, &[u8])> = None;
        let mut projected_user_config = 0usize;
        let mut pseudo = [false; 3];
        let mut system = Vec::new();
        let mut user = Vec::new();
        let mut project = Vec::new();
        let mut system_directory = None;
        let mut user_directory = None;
        for input in &closure.inputs {
            let rank = match input.layer.as_str() {
                "System" => 0,
                "User" => 1,
                "Project" => 2,
                "McpCredentials" => 3,
                "Auth" => 4,
                "CloudRequirements" => 5,
                _ => return Err(ConfigProjectionFailureV1::UnsupportedPolicySurface),
            };
            if let Some((previous_rank, previous_locator)) = previous {
                if rank < previous_rank
                    || (rank == previous_rank && input.locator.as_bytes() <= previous_locator)
                {
                    return Err(ConfigProjectionFailureV1::Conflict);
                }
            }
            previous = Some((rank, input.locator.as_bytes()));
            if input.relative_path.is_empty()
                || input.relative_path.starts_with('/')
                || input.relative_path.split('/').any(|part| part == "..")
                || input.directory.physical_path.is_empty()
                || !matches!(
                    input.directory.physical_identity,
                    crate::DirectoryPhysicalIdentityV1::Linux {
                        device_id: 1..,
                        inode: 1..
                    }
                )
                || input.locator
                    != format!(
                        "{}/{}",
                        input.directory.physical_path.trim_end_matches('/'),
                        input.relative_path.trim_end_matches('/')
                    )
            {
                return Err(ConfigProjectionFailureV1::Malformed);
            }
            match input.layer.as_str() {
                "System" => {
                    system.push(input.relative_path.as_str());
                    if let Some(directory) = system_directory {
                        if directory != &input.directory {
                            return Err(ConfigProjectionFailureV1::WrongBinding);
                        }
                    } else {
                        system_directory = Some(&input.directory);
                    }
                    if input.disposition != CodexLoaderInputDispositionV1::ProvenAbsent
                        || input.device_id.is_some()
                        || input.inode.is_some()
                        || input.byte_length.is_some()
                        || input.sha256.is_some()
                    {
                        return Err(ConfigProjectionFailureV1::WrongBinding);
                    }
                }
                "User"
                    if input.relative_path == "config.toml"
                        && input.disposition == CodexLoaderInputDispositionV1::Projected =>
                {
                    user.push(input.relative_path.as_str());
                    user_directory = Some(&input.directory);
                    projected_user_config += 1;
                    if input.device_id.is_none()
                        || input.inode.is_none()
                        || input.byte_length.is_none()
                        || input.sha256.is_none()
                    {
                        return Err(ConfigProjectionFailureV1::WrongBinding);
                    }
                }
                "User" => {
                    user.push(input.relative_path.as_str());
                    if let Some(directory) = user_directory {
                        if directory != &input.directory {
                            return Err(ConfigProjectionFailureV1::WrongBinding);
                        }
                    } else {
                        user_directory = Some(&input.directory);
                    }
                    if input.disposition != CodexLoaderInputDispositionV1::ProvenAbsent {
                        return Err(ConfigProjectionFailureV1::WrongBinding);
                    }
                    if input.device_id.is_some()
                        || input.inode.is_some()
                        || input.byte_length.is_some()
                        || input.sha256.is_some()
                    {
                        return Err(ConfigProjectionFailureV1::WrongBinding);
                    }
                }
                "Project" => {
                    project.push(input);
                    if input.disposition != CodexLoaderInputDispositionV1::DisabledByTrust
                        || !matches!(
                            input.relative_path.as_str(),
                            ".codex/config.toml" | ".codex/rules" | ".codex/skills"
                        )
                    {
                        return Err(ConfigProjectionFailureV1::WrongBinding);
                    }
                    let present = input.device_id.is_some() as u8
                        + input.inode.is_some() as u8
                        + input.byte_length.is_some() as u8
                        + input.sha256.is_some() as u8;
                    if present != 0 && present != 4 {
                        return Err(ConfigProjectionFailureV1::WrongBinding);
                    }
                }
                "McpCredentials" => {
                    if input.relative_path != ".credentials.json"
                        || input.disposition
                            != CodexLoaderInputDispositionV1::DisabledByEmptyMcpSetAndProvenAbsent
                        || input.device_id.is_some()
                        || input.inode.is_some()
                        || input.byte_length.is_some()
                        || input.sha256.is_some()
                    {
                        return Err(ConfigProjectionFailureV1::WrongBinding);
                    }
                    pseudo[0] = true;
                }
                "Auth" => {
                    if input.relative_path != "auth.json"
                        || input.disposition
                            != CodexLoaderInputDispositionV1::DisabledByEphemeralCredentialStoreAndProvenAbsent
                        || input.device_id.is_some()
                        || input.inode.is_some()
                        || input.byte_length.is_some()
                        || input.sha256.is_some()
                    {
                        return Err(ConfigProjectionFailureV1::WrongBinding);
                    }
                    pseudo[1] = true;
                }
                "CloudRequirements" => {
                    if input.relative_path != "cloud-requirements-cache.json"
                        || input.disposition
                            != CodexLoaderInputDispositionV1::DisabledByNoEphemeralAuthAndProvenAbsent
                        || input.device_id.is_some()
                        || input.inode.is_some()
                        || input.byte_length.is_some()
                        || input.sha256.is_some()
                    {
                        return Err(ConfigProjectionFailureV1::WrongBinding);
                    }
                    pseudo[2] = true;
                }
                _ => unreachable!(),
            }
        }
        if system
            != [
                "config.toml",
                "managed_config.toml",
                "requirements.toml",
                "rules",
                "skills",
            ]
            || user
                != [
                    "config.toml",
                    "managed_config.toml",
                    "requirements.toml",
                    "rules",
                    "skills",
                ]
            || projected_user_config != 1
            || pseudo != [true; 3]
        {
            return Err(ConfigProjectionFailureV1::MissingPreparation);
        }
        let user_directory = user_directory.ok_or(ConfigProjectionFailureV1::MissingPreparation)?;
        if system_directory
            .filter(|directory| directory.physical_path == "/etc/codex")
            .is_none()
        {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        for input in closure.inputs.iter().filter(|input| {
            matches!(
                input.layer.as_str(),
                "McpCredentials" | "Auth" | "CloudRequirements"
            )
        }) {
            if &input.directory != user_directory {
                return Err(ConfigProjectionFailureV1::WrongBinding);
            }
        }
        if project.is_empty() || project.len() % 3 != 0 {
            return Err(ConfigProjectionFailureV1::MissingPreparation);
        }
        for group in project.chunks_exact(3) {
            if group[0].directory != group[1].directory
                || group[0].directory != group[2].directory
                || group.iter().map(|input| input.relative_path.as_str()).ne([
                    ".codex/config.toml",
                    ".codex/rules",
                    ".codex/skills",
                ])
            {
                return Err(ConfigProjectionFailureV1::WrongBinding);
            }
        }
        if ConfigProjectionCodecV1::domain_sha256(
            "substrate.e3.codex-0.125-loader-inputs.v1",
            &json!({"inputs": closure.inputs, "loader_source": closure.loader_source}),
        )? != closure.validated_loader_input_fingerprint
        {
            return Err(ConfigProjectionFailureV1::HashInvalid);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lower_hex(bytes: &[u8]) -> String {
        bytes.iter().map(|byte| format!("{byte:02x}")).collect()
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "the independent golden renderer keeps every contract input explicit"
    )]
    fn render_config_toml(
        model: &str,
        mcp_servers: &[crate::EffectiveMcpServerV1],
        features: &[LogicalFeatureV1],
        codex_base_url: &str,
        root_guest_absolute_path: &str,
        workspace_physical_path: &str,
        orchestration_session_id: &str,
        retained_participant_id: &str,
        identity_hash: &str,
    ) -> Result<Vec<u8>, ConfigProjectionFailureV1> {
        fn toml_string(value: &str) -> String {
            let mut rendered = String::from("\"");
            for character in value.chars() {
                match character {
                    '"' => rendered.push_str("\\\""),
                    '\\' => rendered.push_str("\\\\"),
                    '\u{0008}' => rendered.push_str("\\b"),
                    '\t' => rendered.push_str("\\t"),
                    '\n' => rendered.push_str("\\n"),
                    '\u{000c}' => rendered.push_str("\\f"),
                    '\r' => rendered.push_str("\\r"),
                    character if character <= '\u{001f}' || character == '\u{007f}' => {
                        rendered.push_str(&format!("\\u{:04x}", character as u32));
                    }
                    character => rendered.push(character),
                }
            }
            rendered.push('"');
            rendered
        }
        fn ensure_sorted_unique<'a>(
            values: impl Iterator<Item = &'a str>,
        ) -> Result<(), ConfigProjectionFailureV1> {
            let mut previous: Option<&[u8]> = None;
            for value in values {
                if value.is_empty() || previous.is_some_and(|item| value.as_bytes() <= item) {
                    return Err(ConfigProjectionFailureV1::Conflict);
                }
                previous = Some(value.as_bytes());
            }
            Ok(())
        }
        fn validate_named_values(values: &[NamedValueV1]) -> Result<(), ConfigProjectionFailureV1> {
            ensure_sorted_unique(values.iter().map(|value| value.name.as_str()))?;
            for value in values {
                let upper = value.name.to_ascii_uppercase();
                if upper.contains("AUTHORIZATION")
                    || upper.contains("COOKIE")
                    || upper.ends_with("_TOKEN")
                    || upper.ends_with("_SECRET")
                    || upper.ends_with("_KEY")
                {
                    return Err(ConfigProjectionFailureV1::Malformed);
                }
            }
            Ok(())
        }
        fn toml_array(values: &[String]) -> String {
            format!(
                "[{}]",
                values
                    .iter()
                    .map(|value| toml_string(value))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
        fn toml_table(values: &[NamedValueV1]) -> String {
            format!(
                "{{ {} }}",
                values
                    .iter()
                    .map(|value| format!(
                        "{} = {}",
                        toml_string(&value.name),
                        toml_string(&value.value)
                    ))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
        fn known_feature(name: &str) -> bool {
            matches!(
                name,
                "apps"
                    | "apply_patch_freeform"
                    | "artifact"
                    | "browser_use"
                    | "code_mode"
                    | "codex_hooks"
                    | "computer_use"
                    | "enable_request_compression"
                    | "fast_mode"
                    | "guardian_approval"
                    | "image_generation"
                    | "js_repl"
                    | "memories"
                    | "multi_agent"
                    | "personality"
                    | "request_permissions_tool"
                    | "runtime_metrics"
                    | "shell_snapshot"
                    | "shell_tool"
                    | "tool_call_mcp_elicitation"
                    | "tool_search"
                    | "tool_suggest"
                    | "unified_exec"
                    | "undo"
                    | "workspace_dependencies"
            )
        }

        if codex_base_url.contains('#')
            || codex_base_url.contains('@')
            || !(codex_base_url.starts_with("http://127.0.0.1:")
                || codex_base_url.starts_with("http://[::1]:"))
        {
            return Err(ConfigProjectionFailureV1::UnsupportedConfiguration);
        }
        ensure_sorted_unique(mcp_servers.iter().map(|server| server.server_id.as_str()))?;
        ensure_sorted_unique(features.iter().map(|feature| feature.name.as_str()))?;

        let mut output = String::new();
        output.push_str(&format!("model = {}\n", toml_string(model)));
        output.push_str("model_provider = \"substrate-managed-gateway\"\n");
        output.push_str("check_for_update_on_startup = false\n");
        output.push_str("cli_auth_credentials_store = \"ephemeral\"\n");
        output.push_str(&format!(
            "log_dir = {}\n\n",
            toml_string(&format!("{root_guest_absolute_path}/state/log"))
        ));
        output.push_str("[model_providers.substrate-managed-gateway]\n");
        output.push_str("name = \"Substrate managed gateway\"\n");
        output.push_str(&format!("base_url = {}\n", toml_string(codex_base_url)));
        output.push_str("wire_api = \"responses\"\n");
        output.push_str("requires_openai_auth = false\n");
        output.push_str("supports_websockets = false\n");
        output.push_str(&format!(
            "http_headers = {{ \"X-Substrate-Orchestration-Session\" = {}, \"X-Substrate-Participant\" = {}, \"X-Substrate-Projection\" = {} }}\n\n",
            toml_string(orchestration_session_id),
            toml_string(retained_participant_id),
            toml_string(identity_hash),
        ));
        output.push_str(&format!(
            "[projects.{}]\ntrust_level = \"untrusted\"\n",
            toml_string(workspace_physical_path)
        ));

        for server in mcp_servers {
            output.push_str(&format!(
                "\n[mcp_servers.{}]\nenabled = {}\n",
                toml_string(&server.server_id),
                server.enabled
            ));
            match &server.transport {
                LogicalMcpTransportV1::Stdio {
                    command,
                    args,
                    nonsecret_env,
                } => {
                    validate_named_values(nonsecret_env)?;
                    output.push_str(&format!("command = {}\n", toml_string(command)));
                    output.push_str(&format!("args = {}\n", toml_array(args)));
                    if !nonsecret_env.is_empty() {
                        output.push_str(&format!("env = {}\n", toml_table(nonsecret_env)));
                    }
                }
                LogicalMcpTransportV1::StreamableHttp {
                    url,
                    nonsecret_headers,
                } => {
                    if url.contains('#') || url.contains('@') {
                        return Err(ConfigProjectionFailureV1::Malformed);
                    }
                    validate_named_values(nonsecret_headers)?;
                    output.push_str(&format!("url = {}\n", toml_string(url)));
                    if !nonsecret_headers.is_empty() {
                        output.push_str(&format!(
                            "http_headers = {}\n",
                            toml_table(nonsecret_headers)
                        ));
                    }
                }
            }
        }
        if !features.is_empty() {
            output.push_str("\n[features]\n");
            for LogicalFeatureV1 { name, enabled } in features {
                if !known_feature(name) {
                    return Err(ConfigProjectionFailureV1::Malformed);
                }
                output.push_str(&format!("{} = {}\n", toml_string(name), enabled));
            }
        }
        Ok(output.into_bytes())
    }

    fn pinned_loader_source() -> CodexLoaderSourceIdentityV1 {
        CodexLoaderSourceIdentityV1 {
            codex_version: "0.125.0".to_string(),
            upstream_tag: "rust-v0.125.0".to_string(),
            config_loader_source_sha256:
                "f8e2eff1db4d4cc004dff849682e81e224acca742b5d38d94e2db4a713b560bc".to_string(),
            layer_io_source_sha256:
                "5b8aa76b0776370cc6db86b679efbb6bab361d2d474dc59768724f8a4c80a52a".to_string(),
            loader_model_source_sha256:
                "8c6573bc83f53396a31bae92371790997c30521e46d29c0662c44d58d570ea81".to_string(),
            exec_source_sha256: "b113fd23d8a0d264556b234fb067a4233ab8c3d5491dd393c0bc39b53c3170bd"
                .to_string(),
            cloud_requirements_source_sha256:
                "f43176f2889c54d19b65b0a669e98e22effc52712ca38710b9265ee22c804c77".to_string(),
            auth_storage_source_sha256:
                "31c05505d2ed91f852a225e154929db3083ae61ef8a662fb6aad09442c33650c".to_string(),
            config_types_source_sha256:
                "927c2a72b29136a5d8f627450a1f85a91173bc76e863351fd7233664b5ee32e4".to_string(),
            validator_schema_version: 1,
        }
    }

    fn loader_fingerprint(
        source: &CodexLoaderSourceIdentityV1,
        inputs: &[CodexLoaderInputAttestationV1],
    ) -> Result<String, ConfigProjectionFailureV1> {
        ConfigProjectionCodecV1::domain_sha256(
            "substrate.e3.codex-0.125-loader-inputs.v1",
            &json!({"inputs": inputs, "loader_source": source}),
        )
    }

    fn minimal_loader_inputs(
        config_byte_length: u64,
        config_sha256: String,
    ) -> Vec<CodexLoaderInputAttestationV1> {
        let directory = |path: &str, inode: u64| crate::CanonicalDirectoryV1 {
            physical_path: path.to_string(),
            physical_identity: crate::DirectoryPhysicalIdentityV1::Linux {
                device_id: 1,
                inode,
            },
        };
        let absent = |layer: &str,
                      base: &str,
                      relative_path: &str,
                      inode: u64,
                      disposition: CodexLoaderInputDispositionV1| {
            CodexLoaderInputAttestationV1 {
                layer: layer.to_string(),
                locator: format!("{base}/{relative_path}"),
                disposition,
                directory: directory(base, inode),
                relative_path: relative_path.to_string(),
                device_id: None,
                inode: None,
                byte_length: None,
                sha256: None,
            }
        };
        let mut inputs = Vec::new();
        for relative in [
            "config.toml",
            "managed_config.toml",
            "requirements.toml",
            "rules",
            "skills",
        ] {
            inputs.push(absent(
                "System",
                "/etc/codex",
                relative,
                10,
                CodexLoaderInputDispositionV1::ProvenAbsent,
            ));
        }
        inputs.push(CodexLoaderInputAttestationV1 {
            layer: "User".to_string(),
            locator: "/run/substrate/member-config/series/fence/codex-home/config.toml".to_string(),
            disposition: CodexLoaderInputDispositionV1::Projected,
            directory: directory("/run/substrate/member-config/series/fence/codex-home", 11),
            relative_path: "config.toml".to_string(),
            device_id: Some(1),
            inode: Some(12),
            byte_length: Some(config_byte_length),
            sha256: Some(config_sha256),
        });
        for relative in [
            "managed_config.toml",
            "requirements.toml",
            "rules",
            "skills",
        ] {
            inputs.push(absent(
                "User",
                "/run/substrate/member-config/series/fence/codex-home",
                relative,
                11,
                CodexLoaderInputDispositionV1::ProvenAbsent,
            ));
        }
        for relative in [".codex/config.toml", ".codex/rules", ".codex/skills"] {
            inputs.push(absent(
                "Project",
                "/workspace",
                relative,
                20,
                CodexLoaderInputDispositionV1::DisabledByTrust,
            ));
        }
        inputs.push(absent(
            "McpCredentials",
            "/run/substrate/member-config/series/fence/codex-home",
            ".credentials.json",
            11,
            CodexLoaderInputDispositionV1::DisabledByEmptyMcpSetAndProvenAbsent,
        ));
        inputs.push(absent(
            "Auth",
            "/run/substrate/member-config/series/fence/codex-home",
            "auth.json",
            11,
            CodexLoaderInputDispositionV1::DisabledByEphemeralCredentialStoreAndProvenAbsent,
        ));
        inputs.push(absent(
            "CloudRequirements",
            "/run/substrate/member-config/series/fence/codex-home",
            "cloud-requirements-cache.json",
            11,
            CodexLoaderInputDispositionV1::DisabledByNoEphemeralAuthAndProvenAbsent,
        ));
        inputs
    }

    #[test]
    fn e3c_toml_escaping_matches_the_pinned_golden_fragment() {
        let fragment = concat!(
            "[mcp_servers.\"mcp.é\"]\n",
            "enabled = true\n",
            "command = \"/bin/mcp\"\n",
            "args = [\"a\", \"line\\n\", \"é\"]\n",
            "env = { \"A\" = \"\\u0000\", \"Z\" = \"v\" }\n",
        );
        assert_eq!(
            lower_hex(&Sha256::digest(fragment.as_bytes())),
            "ae31989bc6eee464225d8f80fc68bce879f0b4d2d86b476aad51f01130a8a898"
        );
    }

    #[test]
    fn e3c_pinned_loader_identity_contains_all_seven_verified_hashes() {
        let identity = pinned_loader_source();
        assert_eq!(identity.codex_version, "0.125.0");
        assert_eq!(identity.upstream_tag, "rust-v0.125.0");
        assert_eq!(identity.validator_schema_version, 1);
        for hash in [
            identity.config_loader_source_sha256,
            identity.layer_io_source_sha256,
            identity.loader_model_source_sha256,
            identity.exec_source_sha256,
            identity.cloud_requirements_source_sha256,
            identity.auth_storage_source_sha256,
            identity.config_types_source_sha256,
        ] {
            assert_eq!(hash.len(), 64);
            assert!(hash.bytes().all(|byte| byte.is_ascii_hexdigit()));
        }
    }

    #[test]
    fn e3c_complete_minimal_document_is_byte_identical() {
        let expected = concat!(
            "model = \"codex\"\n",
            "model_provider = \"substrate-managed-gateway\"\n",
            "check_for_update_on_startup = false\n",
            "cli_auth_credentials_store = \"ephemeral\"\n",
            "log_dir = \"/run/native/state/log\"\n\n",
            "[model_providers.substrate-managed-gateway]\n",
            "name = \"Substrate managed gateway\"\n",
            "base_url = \"http://127.0.0.1:43123/v1\"\n",
            "wire_api = \"responses\"\n",
            "requires_openai_auth = false\n",
            "supports_websockets = false\n",
            "http_headers = { \"X-Substrate-Orchestration-Session\" = \"session\", \"X-Substrate-Participant\" = \"participant\", \"X-Substrate-Projection\" = \"identity\" }\n\n",
            "[projects.\"/workspace\"]\n",
            "trust_level = \"untrusted\"\n",
        );
        let first = render_config_toml(
            "codex",
            &[],
            &[],
            "http://127.0.0.1:43123/v1",
            "/run/native",
            "/workspace",
            "session",
            "participant",
            "identity",
        )
        .unwrap();
        let second = render_config_toml(
            "codex",
            &[],
            &[],
            "http://127.0.0.1:43123/v1",
            "/run/native",
            "/workspace",
            "session",
            "participant",
            "identity",
        )
        .unwrap();
        assert_eq!(first, second);
        assert_eq!(first, expected.as_bytes());
        assert_eq!(
            lower_hex(&Sha256::digest(&first)),
            "371d6ecc29d66f0c18653d753b9b9f15306cfd1f3e082431a04377c58124852a"
        );
    }

    #[test]
    fn e3c_complete_nonempty_document_is_byte_identical() {
        let servers = vec![
            crate::EffectiveMcpServerV1 {
                server_id: "a".to_string(),
                transport: LogicalMcpTransportV1::StreamableHttp {
                    url: "http://127.0.0.1:9000/mcp".to_string(),
                    nonsecret_headers: vec![
                        NamedValueV1 {
                            name: "A".to_string(),
                            value: "a".to_string(),
                        },
                        NamedValueV1 {
                            name: "B".to_string(),
                            value: "b".to_string(),
                        },
                    ],
                },
                enabled: false,
            },
            crate::EffectiveMcpServerV1 {
                server_id: "mcp.é".to_string(),
                transport: LogicalMcpTransportV1::Stdio {
                    command: "/bin/mcp".to_string(),
                    args: vec!["a".to_string(), "line\n".to_string(), "é".to_string()],
                    nonsecret_env: vec![
                        NamedValueV1 {
                            name: "A".to_string(),
                            value: "\0".to_string(),
                        },
                        NamedValueV1 {
                            name: "Z".to_string(),
                            value: "v".to_string(),
                        },
                    ],
                },
                enabled: true,
            },
        ];
        let features = vec![
            LogicalFeatureV1 {
                name: "apps".to_string(),
                enabled: false,
            },
            LogicalFeatureV1 {
                name: "undo".to_string(),
                enabled: true,
            },
        ];
        let expected = concat!(
            "model = \"codex\"\n",
            "model_provider = \"substrate-managed-gateway\"\n",
            "check_for_update_on_startup = false\n",
            "cli_auth_credentials_store = \"ephemeral\"\n",
            "log_dir = \"/run/native/state/log\"\n\n",
            "[model_providers.substrate-managed-gateway]\n",
            "name = \"Substrate managed gateway\"\n",
            "base_url = \"http://127.0.0.1:43123/v1\"\n",
            "wire_api = \"responses\"\n",
            "requires_openai_auth = false\n",
            "supports_websockets = false\n",
            "http_headers = { \"X-Substrate-Orchestration-Session\" = \"session\", \"X-Substrate-Participant\" = \"participant\", \"X-Substrate-Projection\" = \"identity\" }\n\n",
            "[projects.\"/workspace\"]\n",
            "trust_level = \"untrusted\"\n\n",
            "[mcp_servers.\"a\"]\n",
            "enabled = false\n",
            "url = \"http://127.0.0.1:9000/mcp\"\n",
            "http_headers = { \"A\" = \"a\", \"B\" = \"b\" }\n\n",
            "[mcp_servers.\"mcp.é\"]\n",
            "enabled = true\n",
            "command = \"/bin/mcp\"\n",
            "args = [\"a\", \"line\\n\", \"é\"]\n",
            "env = { \"A\" = \"\\u0000\", \"Z\" = \"v\" }\n\n",
            "[features]\n",
            "\"apps\" = false\n",
            "\"undo\" = true\n",
        );
        let bytes = render_config_toml(
            "codex",
            &servers,
            &features,
            "http://127.0.0.1:43123/v1",
            "/run/native",
            "/workspace",
            "session",
            "participant",
            "identity",
        )
        .unwrap();
        assert_eq!(bytes, expected.as_bytes());
        assert_eq!(
            lower_hex(&Sha256::digest(&bytes)),
            "1652faa8e5c923dba7b753ec35af34a9375a2b54160d734fc279da4ed0fd761d"
        );
    }

    #[test]
    fn e3c_render_is_deterministic_and_embeds_the_complete_golden_document() {
        let digest = "11".repeat(32);
        let commitment = serde_json::json!({
            "authority_store_id": "hsa_store",
            "commitment_id": "dpc_01890f3e-7b8c-7a11-8c55-0242ac120002",
            "exact_linkage_hash": digest,
        });
        let policy = serde_json::json!({
            "ref_id": format!("ao_{}", "22".repeat(16)),
            "object_kind": "policy",
            "schema_version": 1,
            "commitment": {"kind": "CanonicalSha256", "value": {"digest_hex": digest}},
        });
        let cap = serde_json::json!({
            "e2_activation_id": "e2a_test",
            "e2_launch_kind": "fresh_spawn",
            "commitment_ref": commitment,
            "commitment_subject": {"RetainedWorkerLaunch": {
                "retained_participant_id": "participant",
                "bootstrap_run_id": "bootstrap"
            }},
            "immutable_worker_cap_ref": commitment,
            "immutable_worker_cap_created_revision": 1,
            "immutable_worker_cap_application_revision": 1,
            "policy_snapshot_ref": policy,
            "policy_snapshot_hash": digest,
            "policy_snapshot_revision": "1",
            "request_id": "request",
            "idempotency_key": "idempotency",
            "caller_participant_id": "caller",
            "caller_backend_id": "cli:codex-world",
            "target_backend_id": "cli:codex-world",
            "target_world": {"world_id": "world", "world_generation": 1},
            "registry_publication_revision": 1
        });
        let support = serde_json::json!({
            "schema_version": 1,
            "support_policy_version": 1,
            "elf_execution_model": "StaticExec",
            "elf_interpreter": null,
            "dynamic_loader_cache": null,
            "ordered_elf_dependencies": [],
            "ordered_present_common_files": [],
            "system_config_mount_target": {
                "absolute_path": "/etc/codex", "device_id": 1, "inode": 2,
                "mode": 0o755, "owner_uid": 0, "owner_gid": 0,
                "ordered_entry_names": []
            },
            "manifest_hash": digest,
        });
        let artifact = |role: &str| {
            serde_json::json!({
                "role": role,
                "configured_absolute_path": "/artifact",
                "device_id": 1,
                "inode": 2,
                "file_type": "regular",
                "mode": 0o755,
                "owner_uid": 0,
                "byte_length": 1,
                "sha256": digest,
                "authority_ref": {
                    "authority_store_id": "cpa_01890f3e-7b8c-7a11-8c55-0242ac120001", "manifest_id": "ram_test",
                    "manifest_revision": 1, "manifest_entry_id": "rae_test",
                    "manifest_hash": digest, "entry_hash": digest
                },
                "provenance": {"OfficialCodexRelease": {
                    "version": "0.125.0", "target_triple": "x86_64-unknown-linux-musl",
                    "archive_name": "codex.tar.gz", "archive_url": "https://example.invalid/codex",
                    "archive_sha256": digest, "archive_entry_path": "codex",
                    "extracted_executable_sha256": digest
                }},
                "runtime_support": support,
            })
        };
        let canonical_directory = serde_json::json!({
            "physical_path": "/workspace",
            "physical_identity": {"Linux": {"device_id": 1, "inode": 20}}
        });
        let identity: ConfigProjectionIdentityV1 = serde_json::from_value(serde_json::json!({
            "schema_version": 1,
            "authority_store_id": "cpa_01890f3e-7b8c-7a11-8c55-0242ac120001",
            "series_id": "series",
            "accepted_home": canonical_directory,
            "workspace_root": canonical_directory,
            "orchestration_session_id": "session",
            "retained_participant_id": "participant",
            "bootstrap_run_id": "bootstrap",
            "backend_id": "cli:codex-world",
            "runtime_family": "codex",
            "world_id": "world",
            "world_generation": 1,
            "immutable_launch_cap": cap,
            "runtime_artifacts": {
                "codex": artifact("Codex0125"),
                "world_entry_wrapper": artifact("WorldEntryWrapper"),
                "managed_gateway": artifact("ManagedGateway")
            },
            "identity_hash": "identity"
        }))
        .unwrap();
        let intent = serde_json::json!({
            "authority_store_id": "cpa_01890f3e-7b8c-7a11-8c55-0242ac120001",
            "activation_intent_id": "gai_01890f3e-7b8c-7a11-8c55-0242ac120003",
            "intent_hash": digest
        });
        let root = NativeProjectionRootV1 {
            root_id: "root".to_string(),
            authority_relative_path:
                "authority-v1/agent-config-projection-v1/native-sources/series/fence".to_string(),
            guest_absolute_path: "/run/substrate/member-config/series/fence".to_string(),
            owner_uid: 1000,
            owner_gid: 1000,
            directory_mode: 0o700,
        };
        let expected = render_config_toml(
            "codex",
            &[],
            &[],
            "http://127.0.0.1:43123/v1",
            &root.guest_absolute_path,
            "/workspace",
            "session",
            "participant",
            "identity",
        )
        .unwrap();
        let environment = EffectiveEnvironmentV1 {
            inherited_names: Vec::new(),
            set: [
                ("CODEX_HOME", format!("{}/codex-home", root.guest_absolute_path)),
                (
                    "CODEX_SQLITE_HOME",
                    format!("{}/state/sqlite", root.guest_absolute_path),
                ),
                ("HOME", format!("{}/home", root.guest_absolute_path)),
                ("LANG", "C.UTF-8".to_string()),
                ("LC_ALL", "C.UTF-8".to_string()),
                ("PATH", "/var/lib/substrate/world-deps/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin".to_string()),
                ("RUST_LOG", "error".to_string()),
                ("TMPDIR", format!("{}/tmp", root.guest_absolute_path)),
            ]
            .into_iter()
            .map(|(name, value)| NamedValueV1 { name: name.to_string(), value })
            .collect(),
            remove: [
                "ANTHROPIC_API_KEY", "CODEX_API_KEY", "CODEX_BINARY", "CODEX_OSS_BASE_URL",
                "CODEX_OSS_PORT", "OPENAI_ACCESS_TOKEN", "OPENAI_API_KEY", "OPENAI_BASE_URL",
                "OPENAI_ORGANIZATION", "OPENAI_PROJECT", "SUBSTRATE_E3_CODEX_LAUNCH_PLAN_FD",
                "SUBSTRATE_E3_NATIVE_REALIZATION_FD", "SUBSTRATE_E3_NATIVE_SOURCE_FD",
                "SUBSTRATE_E3_SYSTEM_EMPTY_FD", "SUBSTRATE_E3_WORLD_FS_INPUT_FD",
                "SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME", "SUBSTRATE_LLM_AUTH_BUNDLE_FD",
                "SUBSTRATE_WORLD_ENTRY_BINARY", "SUBSTRATE_WORLD_ENTRY_BINARY_FD",
                "SUBSTRATE_WORLD_ENTRY_CGROUP_PROCS_FD", "SUBSTRATE_WORLD_ENTRY_CGROUP_PROCS_PATH",
                "SUBSTRATE_WORLD_ENTRY_FINAL_EXEC_FD", "SUBSTRATE_WORLD_ENTRY_REQUIRE_CGROUP_ATTACH",
                "SUBSTRATE_WORLD_ENTRY_ROLE", "SUBSTRATE_WORLD_ENTRY_SETUP_READY_FD",
                "SUBSTRATE_WORLD_ENTRY_WORKING_DIR", "SUBSTRATE_WORLD_ENTRY_WORKING_DIR_FD",
            ].into_iter().map(str::to_string).collect(),
        };
        let effective: EffectiveAgentConfigProjectionV1 =
            serde_json::from_value(serde_json::json!({
                "projection_hash": digest,
                "logical_projection_hash": digest,
                "accepted_policy": cap,
                "capabilities": [],
                "model": "codex",
                "provider": {
                    "provider_id": "substrate-managed-gateway", "wire_api": "responses",
                    "requires_openai_auth": false, "supports_websockets": false,
                    "gateway_intent_ref": intent
                },
                "mcp_servers": [], "features": [], "environment": environment,
                "workspace_overlay": "Disabled"
            }))
            .unwrap();
        let gateway: ManagedGatewayProjectionV1 =
            serde_json::from_value(serde_json::json!({
                "projection_hash": digest,
                "activation_intent_ref": intent,
                "expected_gateway_ref": {"authority_store_id": "cpa_01890f3e-7b8c-7a11-8c55-0242ac120001", "gateway_instance_id": "cgi_01890f3e-7b8c-7a11-8c55-0242ac120004", "gateway_identity_hash": digest},
                "codex_base_url": "http://127.0.0.1:43123/v1",
                "access_boundary_ref": {"authority_store_id": "cpa_01890f3e-7b8c-7a11-8c55-0242ac120001", "access_boundary_id": "gab_01890f3e-7b8c-7a11-8c55-0242ac120005", "revision": 1, "boundary_hash": digest},
                "activation_ack_ref": null,
                "posture": "Dormant"
            }))
            .unwrap();
        let inputs = minimal_loader_inputs(
            expected.len() as u64,
            format!("{:x}", Sha256::digest(&expected)),
        );
        let first = Codex0125ProjectionV1::render(
            &identity,
            &effective,
            &gateway,
            root.clone(),
            "fence",
            inputs.clone(),
        )
        .unwrap();
        let second =
            Codex0125ProjectionV1::render(&identity, &effective, &gateway, root, "fence", inputs)
                .unwrap();
        assert_eq!(first, second);
        assert_eq!(
            STANDARD.decode(&first.files[0].bytes_base64).unwrap(),
            expected
        );
        assert!(first
            .ambient_closure
            .inputs
            .iter()
            .all(|input| !input.locator.ends_with("/auth.json")
                || input.disposition
                    == CodexLoaderInputDispositionV1::DisabledByEphemeralCredentialStoreAndProvenAbsent));
    }

    #[test]
    fn e3c_loader_closure_rejects_every_extra_or_missing_locator() {
        let directory = |path: &str, inode: u64| crate::CanonicalDirectoryV1 {
            physical_path: path.to_string(),
            physical_identity: crate::DirectoryPhysicalIdentityV1::Linux {
                device_id: 1,
                inode,
            },
        };
        let absent = |layer: &str,
                      base: &str,
                      relative_path: &str,
                      inode: u64,
                      disposition: CodexLoaderInputDispositionV1| {
            CodexLoaderInputAttestationV1 {
                layer: layer.to_string(),
                locator: format!("{base}/{relative_path}"),
                disposition,
                directory: directory(base, inode),
                relative_path: relative_path.to_string(),
                device_id: None,
                inode: None,
                byte_length: None,
                sha256: None,
            }
        };
        let mut inputs = Vec::new();
        for relative in [
            "config.toml",
            "managed_config.toml",
            "requirements.toml",
            "rules",
            "skills",
        ] {
            inputs.push(absent(
                "System",
                "/etc/codex",
                relative,
                10,
                CodexLoaderInputDispositionV1::ProvenAbsent,
            ));
        }
        inputs.push(CodexLoaderInputAttestationV1 {
            layer: "User".to_string(),
            locator: "/run/native/codex-home/config.toml".to_string(),
            disposition: CodexLoaderInputDispositionV1::Projected,
            directory: directory("/run/native/codex-home", 11),
            relative_path: "config.toml".to_string(),
            device_id: Some(1),
            inode: Some(12),
            byte_length: Some(42),
            sha256: Some("11".repeat(32)),
        });
        for relative in [
            "managed_config.toml",
            "requirements.toml",
            "rules",
            "skills",
        ] {
            inputs.push(absent(
                "User",
                "/run/native/codex-home",
                relative,
                11,
                CodexLoaderInputDispositionV1::ProvenAbsent,
            ));
        }
        for relative in [".codex/config.toml", ".codex/rules", ".codex/skills"] {
            inputs.push(absent(
                "Project",
                "/workspace",
                relative,
                20,
                CodexLoaderInputDispositionV1::DisabledByTrust,
            ));
        }
        inputs.push(absent(
            "McpCredentials",
            "/run/native/codex-home",
            ".credentials.json",
            11,
            CodexLoaderInputDispositionV1::DisabledByEmptyMcpSetAndProvenAbsent,
        ));
        inputs.push(absent(
            "Auth",
            "/run/native/codex-home",
            "auth.json",
            11,
            CodexLoaderInputDispositionV1::DisabledByEphemeralCredentialStoreAndProvenAbsent,
        ));
        inputs.push(absent(
            "CloudRequirements",
            "/run/native/codex-home",
            "cloud-requirements-cache.json",
            11,
            CodexLoaderInputDispositionV1::DisabledByNoEphemeralAuthAndProvenAbsent,
        ));
        let loader_source = pinned_loader_source();
        let mut closure = CodexAmbientConfigClosureV1 {
            validated_loader_input_fingerprint: loader_fingerprint(&loader_source, &inputs)
                .unwrap(),
            loader_source,
            allowed_enabled_layers: vec!["System".to_string(), "User".to_string()],
            inputs,
            forbidden_cli_overrides: vec![
                "--config",
                "--ignore-rules",
                "--ignore-user-config",
                "--model",
                "--oss",
                "--profile",
                "-c",
            ]
            .into_iter()
            .map(str::to_string)
            .collect(),
        };
        Codex0125ProjectionV1::validate_loader_inputs(&closure).unwrap();

        closure.inputs.insert(
            1,
            absent(
                "System",
                "/etc/codex",
                "extra.toml",
                10,
                CodexLoaderInputDispositionV1::ProvenAbsent,
            ),
        );
        closure.validated_loader_input_fingerprint =
            loader_fingerprint(&closure.loader_source, &closure.inputs).unwrap();
        assert_eq!(
            Codex0125ProjectionV1::validate_loader_inputs(&closure),
            Err(ConfigProjectionFailureV1::MissingPreparation)
        );
    }
}
