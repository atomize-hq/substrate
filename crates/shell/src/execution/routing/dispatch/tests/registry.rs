use super::*;

// Registry helpers
#[test]
fn parse_demo_burst_command_defaults() {
    assert_eq!(parse_demo_burst_command(":demo-burst"), Some((4, 400, 0)));
    assert_eq!(
        parse_demo_burst_command(":demo-burst 3 10 5"),
        Some((3, 10, 5))
    );
    assert!(parse_demo_burst_command(":other").is_none());
}
