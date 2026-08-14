const PLIST: &str =
    include_str!("../../../scripts/mac/com.atomize.substrate.r3-macos-evidence-finalizer.v2.plist");

#[test]
fn launchd_route_is_the_exact_root_socket_contract_without_extra_arguments() {
    const EXPECTED: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>com.atomize.substrate.r3-macos-evidence-finalizer.v2</string>
  <key>Program</key>
  <string>/Library/PrivilegedHelperTools/com.atomize.substrate.r3-macos-evidence-finalizer.v2</string>
  <key>UserName</key>
  <string>root</string>
  <key>StandardInPath</key>
  <string>/dev/null</string>
  <key>WorkingDirectory</key>
  <string>/</string>
  <key>Sockets</key>
  <dict>
    <key>Listener</key>
    <dict>
      <key>SockPathName</key>
      <string>/private/var/run/com.atomize.substrate.r3-macos-evidence-finalizer.v2.sock</string>
      <key>SockPathOwner</key>
      <integer>0</integer>
      <key>SockPathGroup</key>
      <integer>20</integer>
      <key>SockPathMode</key>
      <integer>432</integer>
    </dict>
  </dict>
</dict>
</plist>
"#;

    assert_eq!(PLIST, EXPECTED);
    assert!(!PLIST.contains("ProgramArguments"));
}
