use serde_json::{json, Value};
use substrate_broker::world_fs_policy;
use transport_api_types::{
    PlatformBootstrapMappingV1, PlatformInstanceIdentityV1, PlatformPrincipalV1,
    PlatformTransportIdentityV1,
};

fn mapping_transport_string(mapping: &PlatformBootstrapMappingV1) -> String {
    match &mapping.realized_transport {
        PlatformTransportIdentityV1::Wsl { pipe_path, .. } => format!("pipe:{pipe_path}"),
        PlatformTransportIdentityV1::Lima { host_socket, .. } => format!("unix:{host_socket}"),
    }
}

fn mapping_json(mapping: &PlatformBootstrapMappingV1) -> Value {
    let platform_instance = match &mapping.platform_instance {
        PlatformInstanceIdentityV1::Wsl {
            distro_name,
            guest_machine_id,
        } => json!({
            "kind": "wsl",
            "distro_name": distro_name,
            "guest_machine_id": guest_machine_id,
        }),
        PlatformInstanceIdentityV1::Lima {
            vm_name,
            guest_machine_id,
        } => json!({
            "kind": "lima",
            "vm_name": vm_name,
            "guest_machine_id": guest_machine_id,
        }),
    };
    let realized_principal = match &mapping.realized_principal {
        PlatformPrincipalV1::Unix { account, uid } => json!({
            "kind": "unix",
            "account": account,
            "uid": uid,
        }),
        PlatformPrincipalV1::Windows { account, sid } => json!({
            "kind": "windows",
            "account": account,
            "sid": sid,
        }),
    };
    let realized_transport = match &mapping.realized_transport {
        PlatformTransportIdentityV1::Wsl {
            pipe_path,
            guest_socket,
        } => json!({
            "kind": "wsl",
            "pipe_path": pipe_path,
            "guest_socket": guest_socket,
        }),
        PlatformTransportIdentityV1::Lima {
            host_socket,
            guest_socket,
        } => json!({
            "kind": "lima",
            "host_socket": host_socket,
            "guest_socket": guest_socket,
        }),
    };

    json!({
        "host_context_commitment": mapping.host_context_commitment,
        "platform_instance": platform_instance,
        "host_platform_control_root": mapping.host_platform_control_root,
        "realized_substrate_home": mapping.realized_substrate_home,
        "realized_principal": realized_principal,
        "realized_transport": realized_transport,
        "lifecycle_prerequisite": "r3_unmet",
    })
}

pub(crate) fn host_doctor_main(
    json_mode: bool,
    world_enabled: bool,
    world_disable_attribution: Option<&crate::execution::config_model::DoctorDisableAttribution>,
) -> i32 {
    let fs_policy = world_fs_policy();
    let mapping_result = crate::execution::pw::detect().and_then(|ctx| {
        #[cfg(not(test))]
        {
            ctx.bootstrap_mapping
                .ok_or_else(|| anyhow::anyhow!("Windows platform world mapping is unavailable"))
        }
        #[cfg(test)]
        {
            let _ = ctx;
            Err(anyhow::anyhow!(
                "Windows platform world mapping is unavailable"
            ))
        }
    });
    let mapping_ok = mapping_result.is_ok();
    let top_level_ok = world_enabled && mapping_ok;
    let host_error = mapping_result.as_ref().err().map(|err| format!("{err:#}"));
    let mapping_value = mapping_result
        .as_ref()
        .ok()
        .map(mapping_json)
        .unwrap_or(Value::Null);

    if json_mode {
        let mut out = json!({
            "schema_version": 1,
            "platform": "windows",
            "world_enabled": world_enabled,
            "ok": top_level_ok,
            "host": {
                "platform": "windows",
                "ok": mapping_ok,
                "world_fs_mode": fs_policy.mode.as_str(),
                "world_fs_isolation": fs_policy.isolation.as_str(),
                "world_fs_require_world": fs_policy.require_world,
                "status": if mapping_ok { "mapped" } else { "incoherent" },
                "mapping": mapping_value,
                "error": host_error,
            }
        });
        if let Some(attribution) = world_disable_attribution {
            out["world_disable_reason"] = json!(attribution.reason);
            out["world_disable_source"] = json!(attribution.source);
        }
        println!("{}", serde_json::to_string_pretty(&out).unwrap());
    } else {
        println!("== substrate host doctor ==");
        if !world_enabled {
            if let Some(attribution) = world_disable_attribution {
                println!("FAIL  | {}", attribution.reason);
            }
        }
        if let Some(mapping) = mapping_result.as_ref().ok() {
            println!(
                "INFO  | mapping commitment: {}...",
                &mapping.host_context_commitment[..mapping.host_context_commitment.len().min(12)]
            );
            if let PlatformInstanceIdentityV1::Wsl {
                distro_name,
                guest_machine_id,
            } = &mapping.platform_instance
            {
                println!("INFO  | distro: {distro_name}");
                println!("INFO  | guest machine id: {guest_machine_id}");
            }
            println!(
                "INFO  | control root: {}",
                mapping.host_platform_control_root
            );
            if let PlatformPrincipalV1::Unix { account, uid } = &mapping.realized_principal {
                println!("INFO  | guest principal: {account} ({uid})");
            }
            println!("INFO  | guest home: {}", mapping.realized_substrate_home);
            println!("INFO  | transport: {}", mapping_transport_string(mapping));
            println!("WARN  | provisioning/lifecycle activation unavailable here: R3 prerequisite unmet.");
            println!("PASS  | authenticated WSL bootstrap mapping coherent");
        } else if let Some(err) = host_error.as_deref() {
            println!("FAIL  | authenticated WSL bootstrap mapping incoherent ({err})");
        } else {
            println!("FAIL  | authenticated WSL bootstrap mapping unavailable");
        }
    }
    if !world_enabled || !mapping_ok {
        4
    } else {
        0
    }
}

pub(crate) fn world_doctor_main(
    json_mode: bool,
    world_enabled: bool,
    world_disable_attribution: Option<&crate::execution::config_model::DoctorDisableAttribution>,
) -> i32 {
    // Helpers
    fn pass(msg: &str) {
        println!("PASS  | {}", msg);
    }
    fn warn(msg: &str) {
        println!("WARN  | {}", msg);
    }
    fn fail(msg: &str) {
        println!("FAIL  | {}", msg);
    }
    fn info(msg: &str) {
        println!("INFO  | {}", msg);
    }

    let fs_policy = world_fs_policy();

    let ctx = if let Some(ctx) = crate::execution::pw::get_context() {
        Ok(ctx)
    } else {
        crate::execution::pw::detect().and_then(|detected| {
            crate::execution::pw::store_context_globally(detected);
            crate::execution::pw::get_context()
                .ok_or_else(|| anyhow::anyhow!("Windows platform world context did not persist"))
        })
    };
    let mapping_result = ctx.and_then(|ctx| {
        #[cfg(not(test))]
        {
            ctx.bootstrap_mapping
                .clone()
                .ok_or_else(|| anyhow::anyhow!("Windows platform world mapping is unavailable"))
        }
        #[cfg(test)]
        {
            let _ = ctx;
            Err(anyhow::anyhow!(
                "Windows platform world mapping is unavailable"
            ))
        }
    });
    let transport = mapping_result
        .as_ref()
        .ok()
        .map(mapping_transport_string)
        .unwrap_or_else(|| "unknown".to_string());
    let mapping_ok = mapping_result.is_ok();
    let host_error = mapping_result.as_ref().err().map(|err| format!("{err:#}"));
    let mapping_value = mapping_result
        .as_ref()
        .ok()
        .map(mapping_json)
        .unwrap_or(Value::Null);

    if json_mode {
        if let Some(err) = &host_error {
            eprintln!("substrate world doctor (windows): backend not ready: {err:#}");
        }
    }

    let host_error_json = host_error.clone();
    let host_value = json!({
        "platform": "windows",
        "ok": mapping_ok,
        "world_fs_mode": fs_policy.mode.as_str(),
        "world_fs_isolation": fs_policy.isolation.as_str(),
        "world_fs_require_world": fs_policy.require_world,
        "status": if mapping_ok { "mapped" } else { "incoherent" },
        "transport": transport,
        "mapping": mapping_value,
        "error": host_error_json,
    });

    let mut exit_code = 4;
    let world_value = if !world_enabled {
        json!({"status": "disabled", "ok": false})
    } else if !mapping_ok {
        exit_code = 3;
        json!({"status": "unreachable", "ok": false})
    } else {
        let report = match tokio::runtime::Runtime::new() {
            Ok(rt) => rt.block_on(async {
                let client = crate::execution::pw::windows::build_agent_client()?;
                client.doctor_world().await.map_err(|e| anyhow::anyhow!(e))
            }),
            Err(err) => Err(anyhow::anyhow!(
                "failed to create tokio runtime for world doctor: {err}"
            )),
        };

        match report {
            Ok(report) => {
                let mut value = serde_json::to_value(report).unwrap_or_else(|_| json!({}));
                let enforcement_ok = value.get("ok").and_then(Value::as_bool).unwrap_or(false);
                if let Some(obj) = value.as_object_mut() {
                    obj.insert("enforcement_ok".to_string(), json!(enforcement_ok));
                    // On Windows (WSL backend), treat successful doctor retrieval as backend readiness:
                    // `ok=true` means the backend is reachable, while `enforcement_ok` carries the
                    // stricter in-world enforcement signal from the agent report.
                    obj.insert("ok".to_string(), json!(true));
                    obj.insert("status".to_string(), json!("ok"));
                }

                exit_code = if mapping_ok { 0 } else { 4 };
                value
            }
            Err(err) => {
                if json_mode {
                    eprintln!(
                        "substrate world doctor (windows): agent doctor request failed: {err:#}"
                    );
                }
                exit_code = 3;
                json!({"status": "unreachable", "ok": false})
            }
        }
    };

    let ok =
        world_enabled && mapping_ok && world_value.get("ok").and_then(Value::as_bool) == Some(true);

    if json_mode {
        let mut out = json!({
            "schema_version": 1,
            "platform": "windows",
            "world_enabled": world_enabled,
            "ok": ok,
            "host": host_value,
            "world": world_value,
        });
        if let Some(attribution) = world_disable_attribution {
            out["world_disable_reason"] = json!(attribution.reason);
            out["world_disable_source"] = json!(attribution.source);
        }
        println!("{}", serde_json::to_string_pretty(&out).unwrap());
    } else {
        println!("== substrate world doctor ==");
        println!("== Host ==");
        if !world_enabled {
            if let Some(attribution) = world_disable_attribution {
                fail(attribution.reason);
            }
        }

        info(&format!("transport: {}", transport));

        if mapping_ok {
            pass("authenticated WSL bootstrap mapping coherent");
        } else if let Some(err) = host_error {
            fail(&format!(
                "authenticated WSL bootstrap mapping incoherent ({err})"
            ));
        } else {
            fail("authenticated WSL bootstrap mapping unavailable");
        }
        println!("== World ==");
        match world_value.get("status").and_then(Value::as_str) {
            Some("disabled") => fail("world doctor disabled (world isolation is off)"),
            Some("unreachable") => fail("world backend unreachable (agent did not respond)"),
            Some("missing_prereqs") | Some("ok") => {
                let ok = world_value
                    .get("ok")
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
                let enforcement_ok = world_value
                    .get("enforcement_ok")
                    .and_then(Value::as_bool)
                    .unwrap_or(true);
                if ok {
                    pass("world doctor: ok");
                } else {
                    warn("world doctor: ok=false");
                }
                if !enforcement_ok {
                    warn("world doctor: enforcement_ok=false (non-fatal on Windows WSL backend)");
                }
            }
            _ => fail("world doctor: unknown status"),
        }
    }

    exit_code
}

#[cfg(test)]
mod source_tests {
    #[test]
    fn world_doctor_main_persists_detected_context_before_client_selection() {
        let source = include_str!("windows.rs");
        let start = source
            .find("pub(crate) fn world_doctor_main(")
            .expect("world_doctor_main");
        let end = source[start..]
            .find("#[cfg(test)]\nmod source_tests")
            .map(|offset| start + offset)
            .expect("source test module boundary");
        let section = &source[start..end];

        let detect_pos = section
            .find("crate::execution::pw::detect().and_then(|detected| {")
            .expect("detect path");
        let store_pos = section
            .find("crate::execution::pw::store_context_globally(detected);")
            .expect("context store");
        let client_pos = section
            .find("crate::execution::pw::windows::build_agent_client()?")
            .expect("typed client build");

        assert!(
            detect_pos < store_pos && store_pos < client_pos,
            "Windows world doctor must persist the detected context before building the typed client so host diagnostics and world doctor share one mapping"
        );
    }
}
