#[cfg(unix)]
mod unix {
    use std::{collections::BTreeMap, fs, time::Duration};

    use claude_code::{ClaudeClient, ClaudeHomeLayout, ClaudeHomeSeedLevel};
    use tempfile::TempDir;

    fn parse_env_dump(s: &str) -> BTreeMap<String, String> {
        let mut out = BTreeMap::new();
        for line in s.lines() {
            let Some((k, v)) = line.split_once('=') else {
                continue;
            };
            out.insert(k.to_string(), v.to_string());
        }
        out
    }

    #[tokio::test]
    async fn claude_home_injects_env_vars() {
        use std::os::unix::fs::PermissionsExt;

        let tmp = TempDir::new().expect("temp dir");
        let script_path = tmp.path().join("fake-claude");
        let home_root = tmp.path().join("claude-home");

        let script = r#"#!/bin/sh
set -eu
if [ "${1:-}" != "--version" ]; then
  echo "expected --version, got: ${1:-<none>}" >&2
  exit 10
fi
echo "CLAUDE_HOME=${CLAUDE_HOME:-}"
echo "HOME=${HOME:-}"
echo "XDG_CONFIG_HOME=${XDG_CONFIG_HOME:-}"
echo "XDG_DATA_HOME=${XDG_DATA_HOME:-}"
echo "XDG_CACHE_HOME=${XDG_CACHE_HOME:-}"
"#;

        fs::write(&script_path, script).expect("write script");
        let mut perms = fs::metadata(&script_path).expect("metadata").permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms).expect("chmod");

        let client = ClaudeClient::builder()
            .binary(&script_path)
            .claude_home(&home_root)
            .timeout(Some(Duration::from_secs(2)))
            .build();

        let out = client.version().await.expect("version");
        assert!(out.status.success());
        let envs = parse_env_dump(&String::from_utf8_lossy(&out.stdout));

        let layout = ClaudeHomeLayout::new(&home_root);
        assert_eq!(
            envs.get("CLAUDE_HOME").map(String::as_str),
            Some(layout.root().to_str().unwrap())
        );
        assert_eq!(
            envs.get("HOME").map(String::as_str),
            Some(layout.root().to_str().unwrap())
        );
        assert_eq!(
            envs.get("XDG_CONFIG_HOME").map(String::as_str),
            Some(layout.xdg_config_home().to_str().unwrap())
        );
        assert_eq!(
            envs.get("XDG_DATA_HOME").map(String::as_str),
            Some(layout.xdg_data_home().to_str().unwrap())
        );
        assert_eq!(
            envs.get("XDG_CACHE_HOME").map(String::as_str),
            Some(layout.xdg_cache_home().to_str().unwrap())
        );
    }

    #[test]
    fn claude_home_materializes_dirs_during_build() {
        let tmp = TempDir::new().expect("temp dir");
        let home_root = tmp.path().join("claude-home");
        assert!(!home_root.exists());

        let _client = ClaudeClient::builder().claude_home(&home_root).build();

        let layout = ClaudeHomeLayout::new(&home_root);
        assert!(layout.root().is_dir(), "home root should exist");
        assert!(layout.xdg_config_home().is_dir(), "xdg config should exist");
        assert!(layout.xdg_data_home().is_dir(), "xdg data should exist");
        assert!(layout.xdg_cache_home().is_dir(), "xdg cache should exist");
    }

    #[test]
    fn seed_minimal_copies_expected_artifacts() {
        let seed = TempDir::new().expect("seed temp dir");
        let target = TempDir::new().expect("target temp dir");

        fs::write(seed.path().join(".claude.json"), "{}").expect("write .claude.json");
        fs::create_dir_all(seed.path().join(".claude").join("plugins")).expect("mkdir plugins");
        fs::write(
            seed.path().join(".claude").join("settings.json"),
            r#"{"a":1}"#,
        )
        .expect("write settings.json");
        fs::write(
            seed.path().join(".claude").join("settings.local.json"),
            r#"{"b":2}"#,
        )
        .expect("write settings.local.json");
        fs::write(
            seed.path()
                .join(".claude")
                .join("plugins")
                .join("config.json"),
            r#"{"p":true}"#,
        )
        .expect("write plugin config");

        // Excluded artifacts (should not be copied by MinimalAuth).
        fs::write(seed.path().join(".claude").join("history.jsonl"), "history")
            .expect("write history");

        let layout = ClaudeHomeLayout::new(target.path().join("home"));
        let outcome = layout
            .seed_from_user_home(seed.path(), ClaudeHomeSeedLevel::MinimalAuth)
            .expect("seed minimal");

        assert!(layout.root().join(".claude.json").is_file());
        assert!(layout
            .root()
            .join(".claude")
            .join("settings.json")
            .is_file());
        assert!(layout
            .root()
            .join(".claude")
            .join("settings.local.json")
            .is_file());
        assert!(layout
            .root()
            .join(".claude")
            .join("plugins")
            .join("config.json")
            .is_file());

        assert!(
            !layout.root().join(".claude").join("history.jsonl").exists(),
            "history should not be copied"
        );

        assert!(
            !outcome.copied_paths.is_empty(),
            "expected at least one copied path"
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn seed_full_profile_copies_macos_app_support() {
        let seed = TempDir::new().expect("seed temp dir");
        let target = TempDir::new().expect("target temp dir");

        let src = seed
            .path()
            .join("Library")
            .join("Application Support")
            .join("Claude");
        fs::create_dir_all(&src).expect("mkdir app support");
        fs::write(src.join("Preferences"), "prefs").expect("write prefs");

        let layout = ClaudeHomeLayout::new(target.path().join("home"));
        layout
            .seed_from_user_home(seed.path(), ClaudeHomeSeedLevel::FullProfile)
            .expect("seed full");

        let dst = layout
            .root()
            .join("Library")
            .join("Application Support")
            .join("Claude")
            .join("Preferences");
        assert!(dst.is_file(), "expected full profile to be copied");
    }
}
