use std::{env, time::Duration};

use codex::CodexClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Usage: cargo run -p unified-agent-api-codex --example send_prompt -- <prompt> [model]
    let mut args = env::args().skip(1);
    let prompt = args
        .next()
        .expect("missing prompt argument. usage: send_prompt <prompt> [model]");
    let maybe_model = args.next();

    let mut builder = CodexClient::builder().timeout(Duration::from_secs(90));
    if let Some(model) = maybe_model {
        builder = builder.model(model);
    }

    let client = builder.build();
    let response = client.send_prompt(prompt).await?;
    println!("{response}");
    Ok(())
}
