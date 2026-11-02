use std::{env, ffi::OsString, path::PathBuf};

use codex::{AppServerCodegenRequest, CodexClient};

fn usage() {
    eprintln!(
        "usage: app_server_codegen <ts|json> <OUT_DIR> [--experimental] [--prettier <PATH>] [--profile <PROFILE>]"
    );
    eprintln!("examples:");
    eprintln!("  app_server_codegen ts ./gen/app --prettier ./node_modules/.bin/prettier");
    eprintln!("  app_server_codegen json ./gen/app");
    eprintln!("  app_server_codegen json ./gen/app --experimental");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Maps to:
    // - codex app-server generate-ts --out <OUT_DIR> [--prettier <PATH>]
    // - codex app-server generate-json-schema --out <OUT_DIR>
    let mut args: Vec<OsString> = env::args_os().skip(1).collect();
    if args.len() < 2 {
        usage();
        return Ok(());
    }

    let target = args.remove(0);
    let out_dir = PathBuf::from(args.remove(0));
    let mut request = match target.to_string_lossy().to_ascii_lowercase().as_str() {
        "ts" | "typescript" => AppServerCodegenRequest::typescript(out_dir),
        "json" | "schema" | "json-schema" => AppServerCodegenRequest::json_schema(out_dir),
        _ => {
            usage();
            return Ok(());
        }
    };

    let mut prettier: Option<PathBuf> = None;
    let mut profile: Option<String> = None;
    let mut experimental = false;

    let mut index = 0;
    while index < args.len() {
        match args[index].to_string_lossy().as_ref() {
            "--experimental" => {
                experimental = true;
            }
            "--prettier" => {
                if let Some(path) = args.get(index + 1) {
                    prettier = Some(PathBuf::from(path));
                    index += 1;
                } else {
                    eprintln!("missing value for --prettier");
                    usage();
                    return Ok(());
                }
            }
            "--profile" => {
                if let Some(value) = args.get(index + 1) {
                    profile = Some(value.to_string_lossy().to_string());
                    index += 1;
                } else {
                    eprintln!("missing value for --profile");
                    usage();
                    return Ok(());
                }
            }
            other => {
                eprintln!("unknown flag: {other}");
                usage();
                return Ok(());
            }
        }
        index += 1;
    }

    if experimental {
        request = request.experimental(true);
    }

    if let Some(path) = prettier {
        request = request.prettier(path);
    }

    if let Some(profile) = profile {
        request = request.profile(profile);
    }

    let client = CodexClient::builder()
        .mirror_stdout(false)
        .quiet(true)
        .build();

    let output = client.generate_app_server_bindings(request).await?;
    println!("app-server exit code: {:?}", output.status.code());
    println!("output dir: {}", output.out_dir.display());

    if !output.stdout.is_empty() {
        println!("stdout:\n{}", output.stdout);
    }

    if !output.stderr.is_empty() {
        eprintln!("stderr:\n{}", output.stderr);
    }

    Ok(())
}
