//! Comparison logic for detecting divergences between original and replayed executions

use anyhow::Result;
use similar::{ChangeTag, TextDiff};
use substrate_common::FsDiff;

use crate::{
    DivergenceReport, DivergenceSeverity, DivergenceType, ReplayConfig,
    replay::ExecutionResult,
    state::TraceSpan,
};

/// Result of comparing original and replayed executions
pub struct ComparisonResult {
    pub divergence: Option<DivergenceReport>,
    pub warnings: Vec<String>,
}

impl ComparisonResult {
    /// Check if the replay matched the original
    pub fn is_match(&self) -> bool {
        self.divergence.is_none()
    }
}

/// Compare execution results with original span
pub fn compare_execution(
    original: &TraceSpan,
    replay: &ExecutionResult,
    config: &ReplayConfig,
) -> Result<ComparisonResult> {
    let mut warnings = Vec::new();
    
    // First check exit code - this is critical
    if let Some(orig_exit) = original.exit_code {
        if orig_exit != replay.exit_code {
            return Ok(ComparisonResult {
                divergence: Some(DivergenceReport {
                    divergence_type: DivergenceType::ExitCode,
                    description: format!(
                        "Exit code diverged: expected {}, got {}",
                        orig_exit, replay.exit_code
                    ),
                    expected: orig_exit.to_string(),
                    actual: replay.exit_code.to_string(),
                    severity: DivergenceSeverity::Critical,
                }),
                warnings,
            });
        }
    }
    
    // Check stdout if captured in original
    if let Some(orig_stdout) = &original.stdout {
        let replay_stdout = String::from_utf8_lossy(&replay.stdout);
        let divergence = compare_output(
            orig_stdout,
            &replay_stdout,
            config.max_output_compare,
            DivergenceType::StdoutMismatch,
        );
        
        if let Some(div) = divergence {
            if config.strict || div.severity as u8 >= DivergenceSeverity::High as u8 {
                return Ok(ComparisonResult {
                    divergence: Some(div),
                    warnings,
                });
            } else {
                warnings.push(format!("Minor stdout difference: {}", div.description));
            }
        }
    }
    
    // Check stderr if captured in original
    if let Some(orig_stderr) = &original.stderr {
        let replay_stderr = String::from_utf8_lossy(&replay.stderr);
        let divergence = compare_output(
            orig_stderr,
            &replay_stderr,
            config.max_output_compare,
            DivergenceType::StderrMismatch,
        );
        
        if let Some(div) = divergence {
            if config.strict || div.severity as u8 >= DivergenceSeverity::High as u8 {
                return Ok(ComparisonResult {
                    divergence: Some(div),
                    warnings,
                });
            } else {
                warnings.push(format!("Minor stderr difference: {}", div.description));
            }
        }
    }
    
    // Check filesystem differences
    if let (Some(orig_diff), Some(replay_diff)) = (&original.fs_diff, &replay.fs_diff) {
        if let Some(div) = compare_fs_diff(orig_diff, replay_diff) {
            if config.strict || div.severity as u8 >= DivergenceSeverity::High as u8 {
                return Ok(ComparisonResult {
                    divergence: Some(div),
                    warnings,
                });
            } else {
                warnings.push(format!("Filesystem difference: {}", div.description));
            }
        }
    }
    
    // Check network scopes
    if let Some(orig_scopes) = &original.scopes_used {
        let div = compare_scopes(orig_scopes, &replay.scopes_used);
        if let Some(d) = div {
            if config.strict {
                return Ok(ComparisonResult {
                    divergence: Some(d),
                    warnings,
                });
            } else {
                warnings.push(format!("Network scope difference: {}", d.description));
            }
        }
    }
    
    // Check timing if not ignored
    if !config.ignore_timing {
        if let Some(orig_duration) = original.duration_ms {
            let timing_diff = (orig_duration as i64 - replay.duration_ms as i64).abs();
            let threshold = orig_duration.max(100) * 2; // 2x threshold or at least 200ms
            
            if timing_diff > threshold as i64 {
                warnings.push(format!(
                    "Timing difference: {}ms vs {}ms ({}ms delta)",
                    orig_duration, replay.duration_ms, timing_diff
                ));
            }
        }
    }
    
    Ok(ComparisonResult {
        divergence: None,
        warnings,
    })
}

/// Compare text output and determine severity of differences
fn compare_output(
    original: &str,
    replay: &str,
    max_bytes: usize,
    divergence_type: DivergenceType,
) -> Option<DivergenceReport> {
    // Truncate for comparison if needed
    let orig_truncated = if original.len() > max_bytes {
        &original[..max_bytes]
    } else {
        original
    };
    
    let replay_truncated = if replay.len() > max_bytes {
        &replay[..max_bytes]
    } else {
        replay
    };
    
    if orig_truncated == replay_truncated {
        return None;
    }
    
    // Normalize whitespace and compare again
    let orig_normalized = normalize_whitespace(orig_truncated);
    let replay_normalized = normalize_whitespace(replay_truncated);
    
    if orig_normalized == replay_normalized {
        // Only whitespace differences
        return None;
    }
    
    // Check for timestamp/PID differences (common non-deterministic elements)
    if looks_like_timestamp_difference(&orig_normalized, &replay_normalized) {
        return Some(DivergenceReport {
            divergence_type,
            description: "Output differs in timestamps or PIDs (non-deterministic)".to_string(),
            expected: truncate_for_report(orig_truncated, 200),
            actual: truncate_for_report(replay_truncated, 200),
            severity: DivergenceSeverity::Low,
        });
    }
    
    // Calculate diff to determine severity
    let diff = TextDiff::from_lines(orig_truncated, replay_truncated);
    let changes: Vec<_> = diff.iter_all_changes()
        .filter(|c| c.tag() != ChangeTag::Equal)
        .collect();
    
    let change_ratio = changes.len() as f32 / diff.iter_all_changes().count().max(1) as f32;
    
    let severity = if change_ratio > 0.5 {
        DivergenceSeverity::High
    } else if change_ratio > 0.1 {
        DivergenceSeverity::Medium
    } else {
        DivergenceSeverity::Low
    };
    
    Some(DivergenceReport {
        divergence_type,
        description: format!(
            "Output differs significantly ({:.0}% changed)",
            change_ratio * 100.0
        ),
        expected: truncate_for_report(orig_truncated, 200),
        actual: truncate_for_report(replay_truncated, 200),
        severity,
    })
}

/// Compare filesystem diffs
fn compare_fs_diff(original: &FsDiff, replay: &FsDiff) -> Option<DivergenceReport> {
    use std::collections::HashSet;
    
    // Extract file paths from both diffs
    let orig_files: HashSet<_> = extract_paths_from_diff(original).into_iter().collect();
    let replay_files: HashSet<_> = extract_paths_from_diff(replay).into_iter().collect();
    
    let missing: Vec<_> = orig_files.difference(&replay_files).cloned().collect();
    let extra: Vec<_> = replay_files.difference(&orig_files).cloned().collect();
    
    if missing.is_empty() && extra.is_empty() {
        return None;
    }
    
    let severity = if missing.len() + extra.len() > 5 {
        DivergenceSeverity::High
    } else {
        DivergenceSeverity::Medium
    };
    
    Some(DivergenceReport {
        divergence_type: DivergenceType::FilesystemDiff,
        description: format!(
            "Filesystem changes differ: {} missing, {} extra files",
            missing.len(),
            extra.len()
        ),
        expected: format!("{:?}", orig_files),
        actual: format!("{:?}", replay_files),
        severity,
    })
}

/// Compare network scopes accessed
fn compare_scopes(original: &[String], replay: &[String]) -> Option<DivergenceReport> {
    use std::collections::HashSet;
    
    let orig_set: HashSet<_> = original.iter().cloned().collect();
    let replay_set: HashSet<_> = replay.iter().cloned().collect();
    
    if orig_set == replay_set {
        return None;
    }
    
    let missing: Vec<_> = orig_set.difference(&replay_set).cloned().collect();
    let extra: Vec<_> = replay_set.difference(&orig_set).cloned().collect();
    
    Some(DivergenceReport {
        divergence_type: DivergenceType::NetworkScope,
        description: format!(
            "Network scopes differ: {} missing, {} extra",
            missing.len(),
            extra.len()
        ),
        expected: format!("{:?}", original),
        actual: format!("{:?}", replay),
        severity: DivergenceSeverity::Medium,
    })
}

/// Helper to normalize whitespace for comparison
fn normalize_whitespace(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Check if differences look like timestamps or PIDs
fn looks_like_timestamp_difference(s1: &str, s2: &str) -> bool {
    // Simple heuristic: if strings are similar length and contain numbers
    if (s1.len() as i32 - s2.len() as i32).abs() > 50 {
        return false;
    }
    
    // Check if both contain timestamp-like patterns
    let timestamp_pattern = regex::Regex::new(r"\d{4}-\d{2}-\d{2}|\d{2}:\d{2}:\d{2}").unwrap();
    let pid_pattern = regex::Regex::new(r"\[\d+\]|\bpid=\d+").unwrap();
    
    (timestamp_pattern.is_match(s1) && timestamp_pattern.is_match(s2)) ||
    (pid_pattern.is_match(s1) && pid_pattern.is_match(s2))
}

/// Truncate string for report display
fn truncate_for_report(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len])
    }
}

/// Extract file paths from filesystem diff
fn extract_paths_from_diff(diff: &FsDiff) -> Vec<String> {
    let mut paths = Vec::new();
    
    // Add all writes
    paths.extend(diff.writes.iter().map(|p| p.to_string_lossy().to_string()));
    
    // Add all modifications
    paths.extend(diff.mods.iter().map(|p| p.to_string_lossy().to_string()));
    
    // Add all deletes
    paths.extend(diff.deletes.iter().map(|p| p.to_string_lossy().to_string()));
    
    paths
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_normalize_whitespace() {
        assert_eq!(
            normalize_whitespace("hello   world\n\ttab"),
            "hello world tab"
        );
    }
    
    #[test]
    fn test_looks_like_timestamp_difference() {
        assert!(looks_like_timestamp_difference(
            "2024-01-01 10:00:00 Starting",
            "2024-01-01 10:00:01 Starting"
        ));
        
        assert!(looks_like_timestamp_difference(
            "Process[1234] ready",
            "Process[5678] ready"
        ));
        
        assert!(!looks_like_timestamp_difference(
            "hello world",
            "goodbye world"
        ));
    }
}