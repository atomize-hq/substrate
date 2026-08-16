#!/bin/zsh
set -euo pipefail

readonly REPOSITORY="$(/usr/bin/git rev-parse --show-toplevel)"
readonly SWIFTC="/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/swiftc"
readonly XCRUN="/usr/bin/xcrun"
readonly PYTHON="/usr/bin/python3"
readonly FIXED_PATH="/usr/bin:/bin:/usr/sbin:/sbin"
readonly SDKROOT="$(${XCRUN} --sdk macosx --show-sdk-path)"
readonly WORK_ROOT="$(/usr/bin/mktemp -d /private/tmp/r3-securityagent-observer-positive.XXXXXX)"
readonly OBSERVER="${WORK_ROOT}/substrate-r3-macos-securityagent-observer"
readonly OBSERVER_STDOUT="${WORK_ROOT}/observer.stdout"
readonly OBSERVER_STDERR="${WORK_ROOT}/observer.stderr"
readonly FIXTURE_APP="${WORK_ROOT}/SecurityAgentFixture.app"
readonly FIXTURE_EXECUTABLE="${FIXTURE_APP}/Contents/MacOS/observer-positive-fixture"
readonly FIXTURE_STDOUT="${WORK_ROOT}/fixture.stdout"
readonly FIXTURE_STDERR="${WORK_ROOT}/fixture.stderr"

typeset observer_pid=""
typeset fixture_pid=""

cleanup() {
    if [[ -n "${fixture_pid}" ]] && /bin/kill -0 "${fixture_pid}" 2>/dev/null; then
        /bin/kill "${fixture_pid}" 2>/dev/null || true
        wait "${fixture_pid}" 2>/dev/null || true
    fi
    if [[ -n "${observer_pid}" ]] && /bin/kill -0 "${observer_pid}" 2>/dev/null; then
        /bin/kill "${observer_pid}" 2>/dev/null || true
    fi
    /bin/rm -rf "${WORK_ROOT}"
}
trap cleanup EXIT INT TERM HUP

fail() {
    print -u2 -- "R3 SecurityAgent observer positive-detection regression failed: $*"
    if [[ -f "${OBSERVER_STDOUT}" ]]; then
        print -u2 -- "observer stdout:"
        /bin/cat "${OBSERVER_STDOUT}" >&2
    fi
    if [[ -f "${OBSERVER_STDERR}" ]]; then
        print -u2 -- "observer stderr:"
        /bin/cat "${OBSERVER_STDERR}" >&2
    fi
    if [[ -f "${FIXTURE_STDERR}" ]]; then
        print -u2 -- "fixture stderr:"
        /bin/cat "${FIXTURE_STDERR}" >&2
    fi
    exit 1
}

[[ $# -eq 0 ]] || fail "this regression accepts no arguments"
[[ "$(/usr/bin/uname -s)" == "Darwin" ]] || fail "requires macOS"
[[ -x "${SWIFTC}" ]] || fail "fixed Swift compiler is unavailable"

/bin/mkdir -p "${FIXTURE_APP}/Contents/MacOS"
/bin/cat > "${WORK_ROOT}/fixture.swift" <<'SWIFT'
import AppKit
import CoreGraphics
import Foundation

private let expectedBundleIdentifier = "com.atomize.substrate.r3-securityagent-observer-positive-fixture"
private let expectedExecutableName = "observer-positive-fixture"
private let expectedWindowOwnerName = "SecurityAgent"

private func fail(_ message: String) -> Never {
    FileHandle.standardError.write(Data("fixture: \(message)\n".utf8))
    exit(78)
}

private func ownerNameForVisibleWindow(pid: Int32) -> String? {
    guard let windows = CGWindowListCopyWindowInfo(
        [.optionOnScreenOnly, .excludeDesktopElements],
        kCGNullWindowID
    ) as? [[String: Any]] else {
        return nil
    }
    return windows.first { window in
        (window[kCGWindowOwnerPID as String] as? NSNumber)?.int32Value == pid
    }?[kCGWindowOwnerName as String] as? String
}

@main
private enum SecurityAgentObserverPositiveFixture {
    static func main() {
        let application = NSApplication.shared
        application.setActivationPolicy(.accessory)
        let window = NSWindow(
            contentRect: NSRect(x: 16, y: 16, width: 24, height: 24),
            styleMask: [.borderless],
            backing: .buffered,
            defer: false
        )
        window.backgroundColor = NSColor.clear
        window.alphaValue = 0.01
        window.level = .floating
        window.orderFrontRegardless()

        let deadline = Date().addingTimeInterval(2)
        var ownerName: String?
        repeat {
            RunLoop.current.run(until: Date().addingTimeInterval(0.01))
            ownerName = ownerNameForVisibleWindow(pid: getpid())
        } while ownerName == nil && Date() < deadline

        let running = NSRunningApplication(processIdentifier: getpid())
        guard Bundle.main.bundleIdentifier == expectedBundleIdentifier,
              running?.bundleIdentifier == expectedBundleIdentifier,
              running?.executableURL?.lastPathComponent == expectedExecutableName,
              ownerName == expectedWindowOwnerName else {
            fail(
                "fixture did not reproduce the false-negative shape: "
                    + "bundle=\(running?.bundleIdentifier ?? "nil") "
                    + "executable=\(running?.executableURL?.lastPathComponent ?? "nil") "
                    + "owner=\(ownerName ?? "nil")"
            )
        }

        FileHandle.standardOutput.write(Data("READY\n".utf8))
        application.run()
    }
}
SWIFT

/bin/cat > "${FIXTURE_APP}/Contents/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>observer-positive-fixture</string>
    <key>CFBundleIdentifier</key>
    <string>com.atomize.substrate.r3-securityagent-observer-positive-fixture</string>
    <key>CFBundleName</key>
    <string>SecurityAgent</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>LSUIElement</key>
    <true/>
</dict>
</plist>
PLIST

env -i HOME="${HOME}" PATH="${FIXED_PATH}" LANG=C LC_ALL=C TZ=UTC SDKROOT="${SDKROOT}" \
    "${SWIFTC}" -parse-as-library -O -whole-module-optimization \
    -target arm64-apple-macos15.0 \
    -o "${OBSERVER}" \
    "${REPOSITORY}/tools/r3-macos-finalizer/native/securityagent_observer.swift"
env -i HOME="${HOME}" PATH="${FIXED_PATH}" LANG=C LC_ALL=C TZ=UTC SDKROOT="${SDKROOT}" \
    "${SWIFTC}" -parse-as-library -O -whole-module-optimization \
    -target arm64-apple-macos15.0 \
    -o "${FIXTURE_EXECUTABLE}" "${WORK_ROOT}/fixture.swift"

(
    cd /
    exec env -i PATH="${FIXED_PATH}" LANG=C LC_ALL=C TZ=UTC \
        "${OBSERVER}" > "${OBSERVER_STDOUT}" 2> "${OBSERVER_STDERR}"
) &
observer_pid=$!

for _ in {1..300}; do
    /usr/bin/grep -qx 'READY' "${OBSERVER_STDERR}" 2>/dev/null && break
    /bin/kill -0 "${observer_pid}" 2>/dev/null || fail "observer exited before READY"
    /bin/sleep 0.01
done
/usr/bin/grep -qx 'READY' "${OBSERVER_STDERR}" 2>/dev/null \
    || fail "observer did not emit READY"

"${FIXTURE_EXECUTABLE}" > "${FIXTURE_STDOUT}" 2> "${FIXTURE_STDERR}" &
fixture_pid=$!

for _ in {1..300}; do
    /usr/bin/grep -qx 'READY' "${FIXTURE_STDOUT}" 2>/dev/null && break
    /bin/kill -0 "${fixture_pid}" 2>/dev/null || fail "fixture exited before reproducing the window shape"
    /bin/sleep 0.01
done
/usr/bin/grep -qx 'READY' "${FIXTURE_STDOUT}" 2>/dev/null \
    || fail "fixture did not expose its visible SecurityAgent-named window"

for _ in {1..200}; do
    /bin/kill -0 "${observer_pid}" 2>/dev/null || break
    /bin/sleep 0.01
done
/bin/kill -0 "${observer_pid}" 2>/dev/null \
    && fail "observer missed the positive SecurityAgent window"

typeset observer_status=0
wait "${observer_pid}" || observer_status=$?
observer_pid=""
[[ "${observer_status}" == "86" ]] || fail "observer exited ${observer_status}, expected ALERT exit 86"
[[ "$(/usr/bin/grep -c '^ALERT$' "${OBSERVER_STDERR}")" == "1" ]] \
    || fail "observer did not emit exactly one ALERT"

env -i PATH="${FIXED_PATH}" LANG=C LC_ALL=C TZ=UTC \
    "${PYTHON}" - "${OBSERVER_STDOUT}" <<'PY'
import hashlib
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
lines = path.read_bytes().splitlines()
if len(lines) != 1:
    raise SystemExit("observer did not emit exactly one report")
report = json.loads(lines[0])
if report.get("unexpectedUiObserved") is not True:
    raise SystemExit("positive observation did not classify unexpected UI")
if report.get("windowAfterBaseline") is not True:
    raise SystemExit("positive observation did not classify a post-baseline window")
expected_owner = hashlib.sha256(b"SecurityAgent").hexdigest()
windows = report.get("distinctWindows")
if not isinstance(windows, list) or not any(
    window.get("ownerNameSha256") == expected_owner for window in windows
):
    raise SystemExit("positive observation omitted the SecurityAgent-named window")
PY

print -- "R3 macOS SecurityAgent observer positive-detection regression: PASS"
