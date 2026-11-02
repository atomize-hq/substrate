use super::super::test_support::{prelude::*, *};
use super::super::*;

#[tokio::test]
async fn app_server_launch_can_enable_analytics_flag() {
    let (dir, script) = write_fake_app_server();
    let log_path = dir.path().join("argv.json");

    let mut config = test_config(script);
    config.app_server_analytics_default_enabled = true;
    config.env.push((
        OsString::from("ARGV_LOG"),
        OsString::from(log_path.as_os_str()),
    ));

    let client = test_client();
    let server = CodexAppServer::start(config, client)
        .await
        .expect("spawn app server");

    let mut argv_line = None;
    for _ in 0..50 {
        if let Ok(contents) = fs::read_to_string(&log_path) {
            argv_line = contents.lines().next().map(str::to_string);
            break;
        }
        tokio::time::sleep(Duration::from_millis(5)).await;
    }

    let argv_line = argv_line.expect("argv log should be written");
    let argv: Vec<String> = serde_json::from_str(&argv_line).expect("argv json");
    assert_eq!(argv, vec!["app-server", "--analytics-default-enabled"]);

    server.shutdown().await.expect("shutdown server");
}
