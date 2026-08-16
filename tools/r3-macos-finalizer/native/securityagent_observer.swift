import AppKit
import CoreGraphics
import CryptoKit
import Foundation

private let schemaOwner = "substrate.r3-macos-securityagent-observation"
private let schemaVersion = 1
private let sampleCount = 101
private let sampleIntervalSeconds = 0.05
private let securityAgentBundleIdentifier = "com.apple.SecurityAgent"
private let securityAgentExecutableName = "SecurityAgent"

private struct ProcessObservation: Codable, Hashable {
    let pid: Int32
    let bundleIdentifier: String?
    let executablePathSha256: String?
    let launchDateUnixNanoseconds: UInt64?
    let activationPolicy: Int
    let active: Bool
    let hidden: Bool
    let terminated: Bool
}

private struct WindowObservation: Codable, Hashable {
    let ownerPid: Int32
    let ownerNameSha256: String
    let windowNumber: Int
    let layer: Int
    let onScreen: Bool
    let alphaMilli: Int
    let boundsX: Int
    let boundsY: Int
    let boundsWidth: Int
    let boundsHeight: Int
}

private struct SampleObservation: Codable, Hashable {
    let ordinal: Int
    let monotonicNanoseconds: UInt64
    let processes: [ProcessObservation]
    let windows: [WindowObservation]
}

private struct ObservationReport: Codable {
    let schemaOwner: String
    let schemaVersion: Int
    let sampleCount: Int
    let sampleIntervalMilliseconds: Int
    let baseline: SampleObservation
    let distinctProcesses: [ProcessObservation]
    let distinctWindows: [WindowObservation]
    let newProcessAfterBaseline: Bool
    let activeTransitionAfterBaseline: Bool
    let windowAfterBaseline: Bool
    let unexpectedUiObserved: Bool
}

private func sha256(_ value: String) -> String {
    SHA256.hash(data: Data(value.utf8)).map { String(format: "%02x", $0) }.joined()
}

private func unixNanoseconds(_ date: Date?) -> UInt64? {
    guard let date else { return nil }
    let nanoseconds = date.timeIntervalSince1970 * 1_000_000_000
    guard nanoseconds >= 0, nanoseconds <= Double(UInt64.max) else { return nil }
    return UInt64(nanoseconds.rounded(.down))
}

private func isSecurityAgent(_ application: NSRunningApplication) -> Bool {
    application.bundleIdentifier == securityAgentBundleIdentifier
        || application.executableURL?.lastPathComponent == securityAgentExecutableName
}

private func processObservations() -> [ProcessObservation] {
    NSWorkspace.shared.runningApplications
        .filter(isSecurityAgent)
        .map { application in
            ProcessObservation(
                pid: application.processIdentifier,
                bundleIdentifier: application.bundleIdentifier,
                executablePathSha256: application.executableURL.map { sha256($0.path) },
                launchDateUnixNanoseconds: unixNanoseconds(application.launchDate),
                activationPolicy: application.activationPolicy.rawValue,
                active: application.isActive,
                hidden: application.isHidden,
                terminated: application.isTerminated
            )
        }
        .sorted {
            ($0.pid, $0.launchDateUnixNanoseconds ?? 0) < ($1.pid, $1.launchDateUnixNanoseconds ?? 0)
        }
}

private func integer(_ value: Any?) -> Int? {
    (value as? NSNumber)?.intValue
}

private func bool(_ value: Any?) -> Bool? {
    (value as? NSNumber)?.boolValue
}

private func double(_ value: Any?) -> Double? {
    (value as? NSNumber)?.doubleValue
}

private func windowObservations(securityAgentPids: Set<Int32>) -> [WindowObservation] {
    guard let raw = CGWindowListCopyWindowInfo(
        [.optionOnScreenOnly, .excludeDesktopElements],
        kCGNullWindowID
    ) as? [[String: Any]] else {
        return []
    }
    return raw.compactMap { window in
        guard
            let ownerPidValue = integer(window[kCGWindowOwnerPID as String]),
            ownerPidValue >= Int(Int32.min),
            ownerPidValue <= Int(Int32.max),
            let ownerName = window[kCGWindowOwnerName as String] as? String,
            securityAgentPids.contains(Int32(ownerPidValue))
                || ownerName == securityAgentExecutableName,
            let windowNumber = integer(window[kCGWindowNumber as String]),
            let layer = integer(window[kCGWindowLayer as String]),
            let onScreen = bool(window[kCGWindowIsOnscreen as String]),
            let alpha = double(window[kCGWindowAlpha as String]),
            let boundsDictionary = window[kCGWindowBounds as String] as? NSDictionary,
            let bounds = CGRect(dictionaryRepresentation: boundsDictionary)
        else {
            return nil
        }
        return WindowObservation(
            ownerPid: Int32(ownerPidValue),
            ownerNameSha256: sha256(ownerName),
            windowNumber: windowNumber,
            layer: layer,
            onScreen: onScreen,
            alphaMilli: Int((alpha * 1_000).rounded()),
            boundsX: Int(bounds.origin.x.rounded()),
            boundsY: Int(bounds.origin.y.rounded()),
            boundsWidth: Int(bounds.size.width.rounded()),
            boundsHeight: Int(bounds.size.height.rounded())
        )
    }.sorted {
        ($0.ownerPid, $0.windowNumber) < ($1.ownerPid, $1.windowNumber)
    }
}

private func sample(_ ordinal: Int, origin: ContinuousClock.Instant) -> SampleObservation {
    let processes = processObservations()
    let pids = Set(processes.map(\.pid))
    let elapsed = origin.duration(to: .now)
    let seconds = UInt64(max(0, elapsed.components.seconds))
    let attoseconds = UInt64(max(0, elapsed.components.attoseconds))
    let monotonicNanoseconds = seconds &* 1_000_000_000 &+ attoseconds / 1_000_000_000
    return SampleObservation(
        ordinal: ordinal,
        monotonicNanoseconds: monotonicNanoseconds,
        processes: processes,
        windows: windowObservations(securityAgentPids: pids)
    )
}

private func fail(_ message: String) -> Never {
    FileHandle.standardError.write(Data("securityagent-observer: \(message)\n".utf8))
    exit(78)
}

private func emitReport(_ samples: [SampleObservation], forcedUnexpected: Bool) {
    guard let baseline = samples.first else {
        fail("fixed sample set is empty")
    }
    let baselinePids = Set(baseline.processes.map(\.pid))
    let baselineActive = Dictionary(
        uniqueKeysWithValues: baseline.processes.map { ($0.pid, $0.active) }
    )
    let distinctProcesses = Array(Set(samples.flatMap(\.processes))).sorted {
        ($0.pid, $0.launchDateUnixNanoseconds ?? 0, $0.active ? 1 : 0)
            < ($1.pid, $1.launchDateUnixNanoseconds ?? 0, $1.active ? 1 : 0)
    }
    let distinctWindows = Array(Set(samples.flatMap(\.windows))).sorted {
        ($0.ownerPid, $0.windowNumber) < ($1.ownerPid, $1.windowNumber)
    }
    let newProcess = samples.dropFirst().contains { current in
        current.processes.contains { !baselinePids.contains($0.pid) }
    }
    let activeTransition = samples.dropFirst().contains { current in
        current.processes.contains { process in
            process.active && baselineActive[process.pid] != true
        }
    }
    let windowAfterBaseline = samples.dropFirst().contains { !$0.windows.isEmpty }
    let report = ObservationReport(
        schemaOwner: schemaOwner,
        schemaVersion: schemaVersion,
        sampleCount: samples.count,
        sampleIntervalMilliseconds: Int((sampleIntervalSeconds * 1_000).rounded()),
        baseline: baseline,
        distinctProcesses: distinctProcesses,
        distinctWindows: distinctWindows,
        newProcessAfterBaseline: newProcess,
        activeTransitionAfterBaseline: activeTransition,
        windowAfterBaseline: windowAfterBaseline,
        unexpectedUiObserved: forcedUnexpected || newProcess || activeTransition || windowAfterBaseline
    )
    let encoder = JSONEncoder()
    encoder.outputFormatting = [.sortedKeys, .withoutEscapingSlashes]
    do {
        let encoded = try encoder.encode(report)
        FileHandle.standardOutput.write(encoded)
        FileHandle.standardOutput.write(Data([0x0a]))
    } catch {
        fail("encode report: \(error)")
    }
}

private func emitAlertAndExit(_ samples: [SampleObservation]) -> Never {
    emitReport(samples, forcedUnexpected: true)
    FileHandle.standardError.write(Data("ALERT\n".utf8))
    exit(86)
}

@main
private enum SecurityAgentObserver {
    static func main() {
        guard CommandLine.arguments.count == 1 else {
            fail("no argv is accepted")
        }
        guard FileManager.default.currentDirectoryPath == "/" else {
            fail("cwd must be the fixed root directory")
        }
        guard !ProcessInfo.processInfo.environment.keys.contains(where: { $0.hasPrefix("SUBSTRATE_") }) else {
            fail("ambient SUBSTRATE_* input is rejected")
        }

        let origin = ContinuousClock.now
        var samples: [SampleObservation] = []
        samples.reserveCapacity(sampleCount)
        for ordinal in 0..<sampleCount {
            let current = sample(ordinal, origin: origin)
            samples.append(current)
            if ordinal == 0 {
                if current.processes.contains(where: { $0.active }) || !current.windows.isEmpty {
                    emitAlertAndExit(samples)
                }
                FileHandle.standardError.write(Data("READY\n".utf8))
            } else if let baseline = samples.first {
                let baselinePids = Set(baseline.processes.map(\.pid))
                let baselineActive = Dictionary(
                    uniqueKeysWithValues: baseline.processes.map { ($0.pid, $0.active) }
                )
                let unexpected = current.processes.contains {
                    !baselinePids.contains($0.pid) || ($0.active && baselineActive[$0.pid] != true)
                } || !current.windows.isEmpty
                if unexpected {
                    emitAlertAndExit(samples)
                }
            }
            if ordinal + 1 < sampleCount {
                Thread.sleep(forTimeInterval: sampleIntervalSeconds)
            }
        }

        emitReport(samples, forcedUnexpected: false)
    }
}
