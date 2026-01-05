use std::path::{Path, PathBuf};

fn playbook_paths() -> Vec<PathBuf> {
    let next_dir =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/project_management/next");
    let mut out = Vec::new();
    collect_playbooks(&next_dir, &mut out);
    out
}

fn collect_playbooks(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_playbooks(&path, out);
            continue;
        }
        if path
            .file_name()
            .is_some_and(|name| name == "manual_testing_playbook.md")
        {
            out.push(path);
        }
    }
}

fn parse_heredoc_delimiter(line: &str) -> Option<String> {
    let prefix = "cat > .substrate-profile <<";
    let start = line.find(prefix)?;
    let after = line[start + prefix.len()..].trim();
    let token = after.split_whitespace().next()?.trim_end_matches(';');
    let token = token
        .strip_prefix('\'')
        .and_then(|value| value.strip_suffix('\''))
        .or_else(|| {
            token
                .strip_prefix('\"')
                .and_then(|value| value.strip_suffix('\"'))
        })
        .unwrap_or(token);
    if token.is_empty() {
        None
    } else {
        Some(token.to_string())
    }
}

#[test]
fn manual_testing_playbook_substrate_profile_snippets_include_required_id_and_name() {
    let playbooks = playbook_paths();
    assert!(
        !playbooks.is_empty(),
        "expected at least one manual testing playbook under docs/project_management/next/"
    );

    for playbook_path in playbooks {
        let contents = std::fs::read_to_string(&playbook_path)
            .unwrap_or_else(|err| panic!("failed to read {playbook_path:?}: {err}"));

        let mut snippets = Vec::new();
        let mut lines = contents.lines().enumerate().peekable();
        while let Some((line_no, line)) = lines.next() {
            let Some(delimiter) = parse_heredoc_delimiter(line) else {
                continue;
            };

            let mut snippet = Vec::new();
            for (_, body_line) in lines.by_ref() {
                if body_line.trim_end() == delimiter {
                    break;
                }
                snippet.push(body_line);
            }

            snippets.push((line_no + 1, snippet.join("\n")));
        }

        if snippets.is_empty() {
            continue;
        }

        for (start_line, snippet) in snippets {
            let value: serde_yaml::Value = serde_yaml::from_str(&snippet).unwrap_or_else(|err| {
                panic!(
                    "failed to parse `.substrate-profile` snippet from {playbook_path:?}:{start_line} as YAML: {err}\n---\n{snippet}\n---"
                )
            });

            let mapping = value.as_mapping().unwrap_or_else(|| {
                panic!(
                    "expected YAML mapping at top-level for `.substrate-profile` snippet from {playbook_path:?}:{start_line}\n---\n{snippet}\n---"
                )
            });

            let has_id = mapping.contains_key(serde_yaml::Value::String("id".to_string()));
            let has_name = mapping.contains_key(serde_yaml::Value::String("name".to_string()));
            assert!(
                has_id == has_name,
                "expected `.substrate-profile` snippet from {playbook_path:?}:{start_line} to include either both top-level keys `id` and `name`, or neither\n---\n{snippet}\n---"
            );
        }
    }
}
