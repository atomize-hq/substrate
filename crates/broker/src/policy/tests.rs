use super::*;

#[test]
fn test_default_policy() {
    let policy = Policy::default();
    assert_eq!(policy.id, "default");
    assert!(!policy.cmd_allowed.is_empty() || !policy.cmd_denied.is_empty());
}

#[test]
fn test_policy_merge() {
    let mut base = Policy {
        fs_write: vec!["/tmp/*".to_string()],
        ..Default::default()
    };

    let addon = Policy {
        id: "addon".to_string(),
        name: "Addon Policy".to_string(),
        fs_write: vec!["/var/tmp/*".to_string()],
        cmd_denied: vec!["sudo rm -rf /".to_string()],
        ..Default::default()
    };

    base.merge(&addon);

    assert_eq!(base.fs_write.len(), 2);
    assert!(base.fs_write.contains(&"/tmp/*".to_string()));
    assert!(base.fs_write.contains(&"/var/tmp/*".to_string()));
    assert!(base.cmd_denied.contains(&"sudo rm -rf /".to_string()));
}

#[test]
fn test_path_checks() {
    let policy = Policy {
        fs_read: vec!["/home/*".to_string(), "/tmp/*".to_string()],
        fs_write: vec!["/tmp/*".to_string()],
        ..Default::default()
    };

    assert!(policy.is_path_readable("/home/user/file.txt"));
    assert!(policy.is_path_readable("/tmp/test.txt"));
    assert!(!policy.is_path_readable("/etc/passwd"));

    assert!(policy.is_path_writable("/tmp/test.txt"));
    assert!(!policy.is_path_writable("/home/user/file.txt"));
}

#[test]
fn test_host_allowed() {
    let policy = Policy {
        net_allowed: vec!["github.com".to_string(), "*.example.com".to_string()],
        ..Default::default()
    };

    assert!(policy.is_host_allowed("github.com"));
    assert!(policy.is_host_allowed("api.github.com"));
    assert!(policy.is_host_allowed("test.example.com"));
    assert!(!policy.is_host_allowed("evil.com"));
}
