use serde_json::json;

pub(crate) fn world_doctor_main(json_mode: bool) -> i32 {
    if json_mode {
        let out = json!({
            "platform": std::env::consts::OS,
            "ok": true,
            "message": "world doctor for Windows not yet implemented"
        });
        println!("{}", serde_json::to_string_pretty(&out).unwrap());
    } else {
        eprintln!("substrate world doctor is not yet implemented on Windows");
    }
    0
}
