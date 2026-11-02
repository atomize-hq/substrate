use std::{
    env,
    io::{self, Write},
    thread,
    time::Duration,
};

const SYSTEM_INIT: &str = include_str!("../../tests/fixtures/stream_json/v1/system_init.jsonl");
const USER_MESSAGE: &str = include_str!("../../tests/fixtures/stream_json/v1/user_message.jsonl");

fn first_nonempty_line(text: &str) -> &str {
    text.lines()
        .find(|line| !line.chars().all(|ch| ch.is_whitespace()))
        .expect("fixture contains a non-empty line")
}

fn write_line(out: &mut impl Write, line: &str) -> io::Result<()> {
    out.write_all(line.as_bytes())?;
    out.flush()?;
    Ok(())
}

fn main() -> io::Result<()> {
    // This is a tiny, cross-platform test binary used by `claude_code` integration tests.
    //
    // It emulates `claude --print --output-format stream-json` by emitting JSONL lines to stdout.
    // Scenario is selected via env var so tests can validate framing + redaction + incrementality.
    let scenario = env::var("FAKE_CLAUDE_SCENARIO").unwrap_or_else(|_| "two_events_delayed".into());

    let init = first_nonempty_line(SYSTEM_INIT);
    let user = first_nonempty_line(USER_MESSAGE);

    let mut out = io::stdout().lock();

    match scenario.as_str() {
        "crlf_blank_lines" => {
            write_line(&mut out, "\r\n")?;
            write_line(&mut out, "   \r\n")?;
            write_line(&mut out, &format!("{init}\r\n"))?;
            write_line(&mut out, "\r\n")?;
            write_line(&mut out, &format!("{user}\r\n"))?;
        }
        "parse_error_redaction" => {
            let secret = "VERY_SECRET_SHOULD_NOT_APPEAR";
            write_line(&mut out, &format!("not json {secret}\n"))?;
            write_line(&mut out, &format!("{init}\n"))?;
        }
        // Default: prove incrementality by pausing between events while keeping the process alive.
        _ => {
            write_line(&mut out, &format!("{init}\n"))?;
            thread::sleep(Duration::from_millis(250));
            write_line(&mut out, &format!("{user}\n"))?;
        }
    }

    Ok(())
}
