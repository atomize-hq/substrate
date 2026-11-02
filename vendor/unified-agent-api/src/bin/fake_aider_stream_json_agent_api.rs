use std::{
    env, fs,
    io::{self, Write},
    thread,
    time::Duration,
};

fn write_line(out: &mut impl Write, line: &str) -> io::Result<()> {
    out.write_all(line.as_bytes())?;
    out.flush()?;
    Ok(())
}

fn capture_invocation() {
    let capture_path = match env::var("FAKE_AIDER_CAPTURE") {
        Ok(path) => path,
        Err(_) => return,
    };

    let payload = serde_json::json!({
        "argv": env::args().skip(1).collect::<Vec<_>>(),
        "cwd": env::current_dir().ok().map(|path| path.display().to_string()),
    });
    fs::write(
        capture_path,
        serde_json::to_vec_pretty(&payload).expect("serialize capture"),
    )
    .expect("write capture");
}

fn main() -> io::Result<()> {
    capture_invocation();

    let scenario =
        env::var("FAKE_AIDER_SCENARIO").unwrap_or_else(|_| "three_events_delayed".to_string());
    let mut out = io::stdout().lock();

    match scenario.as_str() {
        "capture_args" => {
            write_line(
                &mut out,
                "{\"type\":\"init\",\"session_id\":\"session-aider-1\",\"model\":\"sonnet\"}\n",
            )?;
            write_line(
                &mut out,
                "{\"type\":\"message\",\"role\":\"assistant\",\"content\":\"Hello from fake aider\",\"delta\":true}\n",
            )?;
            write_line(
                &mut out,
                "{\"type\":\"result\",\"status\":\"success\",\"stats\":{\"turns\":1}}\n",
            )?;
        }
        "crlf_blank_lines" => {
            write_line(&mut out, "\r\n")?;
            write_line(&mut out, "   \r\n")?;
            write_line(
                &mut out,
                "{\"type\":\"init\",\"session_id\":\"session-aider-1\",\"model\":\"sonnet\"}\r\n",
            )?;
            write_line(&mut out, "\r\n")?;
            write_line(
                &mut out,
                "{\"type\":\"message\",\"role\":\"assistant\",\"content\":\"Hello from fake aider\",\"delta\":true}\r\n",
            )?;
            write_line(
                &mut out,
                "{\"type\":\"result\",\"status\":\"success\",\"stats\":{\"turns\":1}}\r\n",
            )?;
        }
        "parse_error_redaction" => {
            let secret = "VERY_SECRET_SHOULD_NOT_APPEAR";
            write_line(&mut out, &format!("not json {secret}\n"))?;
            write_line(
                &mut out,
                "{\"type\":\"init\",\"session_id\":\"session-aider-1\",\"model\":\"sonnet\"}\n",
            )?;
            write_line(
                &mut out,
                "{\"type\":\"message\",\"role\":\"assistant\",\"content\":\"Hello from fake aider\",\"delta\":true}\n",
            )?;
            write_line(
                &mut out,
                "{\"type\":\"result\",\"status\":\"success\",\"stats\":{\"turns\":1}}\n",
            )?;
        }
        "tool_roundtrip" => {
            write_line(
                &mut out,
                "{\"type\":\"init\",\"session_id\":\"session-aider-1\",\"model\":\"sonnet\"}\n",
            )?;
            write_line(
                &mut out,
                "{\"type\":\"tool_use\",\"tool_name\":\"shell\",\"tool_id\":\"tool-aider-1\",\"parameters\":{\"cmd\":\"pwd\"}}\n",
            )?;
            write_line(
                &mut out,
                "{\"type\":\"tool_result\",\"tool_id\":\"tool-aider-1\",\"status\":\"success\",\"output\":\"/tmp\"}\n",
            )?;
            write_line(
                &mut out,
                "{\"type\":\"message\",\"role\":\"assistant\",\"content\":\"done\",\"delta\":true}\n",
            )?;
            write_line(
                &mut out,
                "{\"type\":\"result\",\"status\":\"success\",\"stats\":{\"turns\":1}}\n",
            )?;
        }
        "slow_until_killed" => {
            write_line(
                &mut out,
                "{\"type\":\"init\",\"session_id\":\"session-aider-1\",\"model\":\"sonnet\"}\n",
            )?;
            thread::sleep(Duration::from_secs(30));
        }
        "turn_limit_exceeded" => {
            write_line(
                &mut out,
                "{\"type\":\"init\",\"session_id\":\"session-aider-1\",\"model\":\"sonnet\"}\n",
            )?;
            write_line(
                &mut out,
                "{\"type\":\"message\",\"role\":\"assistant\",\"content\":\"partial\",\"delta\":true}\n",
            )?;
            write_line(
                &mut out,
                "{\"type\":\"result\",\"status\":\"error\",\"error\":{\"type\":\"turn_limit\",\"message\":\"Turn limit exceeded\"}}\n",
            )?;
            std::process::exit(53);
        }
        "invalid_input" => {
            write_line(
                &mut out,
                "{\"type\":\"result\",\"status\":\"error\",\"error\":{\"type\":\"input_error\",\"message\":\"Prompt rejected\"}}\n",
            )?;
            std::process::exit(42);
        }
        _ => {
            write_line(
                &mut out,
                "{\"type\":\"init\",\"session_id\":\"session-aider-1\",\"model\":\"sonnet\"}\n",
            )?;
            thread::sleep(Duration::from_millis(150));
            write_line(
                &mut out,
                "{\"type\":\"message\",\"role\":\"assistant\",\"content\":\"Hello from fake aider\",\"delta\":true}\n",
            )?;
            thread::sleep(Duration::from_millis(150));
            write_line(
                &mut out,
                "{\"type\":\"result\",\"status\":\"success\",\"stats\":{\"turns\":1}}\n",
            )?;
        }
    }

    Ok(())
}
