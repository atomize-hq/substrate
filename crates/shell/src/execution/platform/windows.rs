use serde_json::json;

pub(crate) fn host_doctor_main(json_mode: bool, world_enabled: bool) -> i32 {
    if json_mode {
        let out = json!({
            "schema_version": 1,
            "platform": "windows",
            "world_enabled": world_enabled,
            "ok": false,
            "host": {
                "platform": "windows",
                "ok": false,
                "status": "unsupported",
                "message": "host doctor is not yet implemented on Windows",
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
    if json_mode {
        let out = json!({
            "schema_version": 1,
            "platform": "windows",
            "world_enabled": world_enabled,
            "ok": false,
            "host": {
                "platform": "windows",
                "ok": false,
                "status": "unsupported",
                "message": "host doctor is not yet implemented on Windows",
            },
            "world": {
                "status": "unsupported",
                "ok": false,
            }
        });
        println!("{}", serde_json::to_string_pretty(&out).unwrap());
    } else {
        println!("== substrate world doctor ==");
        println!("== Host ==");
        println!("FAIL  | host doctor is not yet implemented on Windows");
        println!("== World ==");
        println!("FAIL  | world doctor is not yet implemented on Windows");
    }
    4
}
