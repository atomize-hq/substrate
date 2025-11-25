use super::*;
use tempfile::tempdir;

#[test]
fn test_profile_detection() {
    let temp = tempdir().unwrap();
    let project_dir = temp.path().join("project");
    let sub_dir = project_dir.join("src").join("lib");
    std::fs::create_dir_all(&sub_dir).unwrap();

    // Create a profile file in project root
    let profile_path = project_dir.join(PROFILE_FILENAME);
    std::fs::write(&profile_path, "test").unwrap();

    let mut detector = ProfileDetector::new();

    // Should find profile from subdirectory
    let result = detector.find_profile(&sub_dir).unwrap();
    assert!(result.is_some());
    // Compare canonical paths to handle symlinks
    let found = result.unwrap().canonicalize().unwrap();
    let expected = profile_path.canonicalize().unwrap();
    assert_eq!(found, expected);

    // Should use cache on second call
    let result2 = detector.find_profile(&sub_dir).unwrap();
    assert!(result2.is_some());
}

#[test]
fn test_no_profile() {
    let temp = tempdir().unwrap();
    let mut detector = ProfileDetector::new();

    let result = detector.find_profile(temp.path()).unwrap();
    assert_eq!(result, None);
}

#[test]
fn test_profile_directory() {
    let temp = tempdir().unwrap();
    let profile_dir = temp.path().join(PROFILE_DIR_FILENAME);
    std::fs::create_dir(&profile_dir).unwrap();

    let policy_file = profile_dir.join("default.yaml");
    std::fs::write(&policy_file, "id: test\nname: Test").unwrap();

    let mut detector = ProfileDetector::new();
    let result = detector.find_profile(temp.path()).unwrap();
    assert!(result.is_some());
    // Compare canonical paths to handle symlinks
    let found = result.unwrap().canonicalize().unwrap();
    let expected = policy_file.canonicalize().unwrap();
    assert_eq!(found, expected);
}

#[test]
fn test_sample_profile_creation() {
    let temp = tempdir().unwrap();
    let profile_path = temp.path().join(".substrate-profile");

    create_sample_profile(&profile_path).unwrap();
    assert!(profile_path.exists());

    // Verify it can be parsed
    let content = std::fs::read_to_string(&profile_path).unwrap();
    let policy: crate::Policy = serde_yaml::from_str(&content).unwrap();
    assert_eq!(policy.id, "project-policy");
}
