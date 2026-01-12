use serde_json::{json, Value};
use substrate_broker::world_fs_policy;

pub(crate) fn host_doctor_main(json_mode: bool, world_enabled: bool) -> i32 {
    let fs_policy = world_fs_policy();
    if json_mode {
        let out = json!({
            "schema_version": 1,
            "platform": "windows",
            "world_enabled": world_enabled,
            "ok": false,
            "host": {
                "platform": "windows",
                "ok": false,
                "world_fs_mode": fs_policy.mode.as_str(),
                "world_fs_isolation": fs_policy.isolation.as_str(),
                "world_fs_require_world": fs_policy.require_world,
                "status": "unsupported",
                "message": "host doctor is not yet implemented on Windows (use `substrate world doctor --json` for WSL backend diagnostics)",
            }
        });
        println!("{}", serde_json::to_string_pretty(&out).unwrap());
    } else {
        println!("== substrate host doctor ==");
        println!("FAIL  | host doctor is not yet implemented on Windows");
    }
    4
}

pub(crate) fn world_doctor_main(json_mode: bool, world_enabled: bool) -> i32 {
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

    let ctx = crate::execution::pw::detect();
    let transport = ctx
        .as_ref()
        .ok()
        .map(|c| c.transport.to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let (host_ok, host_error) = if !world_enabled {
        (
            false,
            Some("world disabled by effective config".to_string()),
        )
    } else {
        match ctx.as_ref() {
            Ok(c) => match (c.ensure_ready.as_ref())() {
                Ok(()) => (true, None),
                Err(err) => (false, Some(err.to_string())),
            },
            Err(err) => (false, Some(err.to_string())),
        }
    };

    let host_error_json = host_error.clone();
    let host_value = json!({
        "platform": "windows",
        "ok": host_ok,
        "world_fs_mode": fs_policy.mode.as_str(),
        "world_fs_isolation": fs_policy.isolation.as_str(),
        "world_fs_require_world": fs_policy.require_world,
        "transport": transport,
        "error": host_error_json,
    });

    let mut exit_code = 4;
    let world_value = if !world_enabled {
        json!({"status": "disabled", "ok": false})
    } else if !host_ok {
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

                exit_code = if host_ok { 0 } else { 4 };
                value
            }
            Err(_) => {
                exit_code = 3;
                json!({"status": "unreachable", "ok": false})
            }
        }
    };

    let ok = host_ok && world_value.get("ok").and_then(Value::as_bool) == Some(true);

    if json_mode {
        let out = json!({
            "schema_version": 1,
            "platform": "windows",
            "world_enabled": world_enabled,
            "ok": ok,
            "host": host_value,
            "world": world_value,
        });
        println!("{}", serde_json::to_string_pretty(&out).unwrap());
    } else {
        println!("== substrate world doctor ==");
        println!("== Host ==");
        if !world_enabled {
            fail("world isolation disabled by effective config (--no-world)");
        }

        info(&format!("transport: {}", transport));

        if host_ok {
            pass("world backend: ready");
        } else if let Some(err) = host_error {
            fail(&format!("world backend: not ready ({err})"));
        } else {
            fail("world backend: not ready");
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
