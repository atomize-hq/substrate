//! Regression detection and reporting

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{DivergenceSeverity, DivergenceType, ReplayResult};

/// Report summarizing regression testing results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionReport {
    /// Total number of spans tested
    pub total_spans: usize,
    /// Number of spans that matched exactly
    pub matched: usize,
    /// Number of spans with divergences
    pub diverged: usize,
    /// Number of spans that failed to replay
    pub failed: usize,
    /// Overall pass rate
    pub pass_rate: f64,
    /// Breakdown by divergence type
    pub divergence_breakdown: HashMap<String, usize>,
    /// Breakdown by severity
    pub severity_breakdown: HashMap<String, usize>,
    /// List of critical failures
    pub critical_failures: Vec<CriticalFailure>,
    /// Summary statistics
    pub statistics: RegressionStatistics,
    /// Recommended actions
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriticalFailure {
    pub span_id: String,
    pub command: String,
    pub divergence_type: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionStatistics {
    /// Average execution time for replays
    pub avg_duration_ms: f64,
    /// Most common divergence type
    pub most_common_divergence: Option<String>,
    /// Percentage of non-deterministic failures
    pub non_deterministic_rate: f64,
    /// Commands with highest failure rate
    pub problematic_commands: Vec<(String, f64)>,
}

/// Analyze replay results and generate regression report
pub fn analyze_results(results: Vec<ReplayResult>) -> Result<RegressionReport> {
    let total_spans = results.len();

    if total_spans == 0 {
        return Ok(RegressionReport {
            total_spans: 0,
            matched: 0,
            diverged: 0,
            failed: 0,
            pass_rate: 0.0,
            divergence_breakdown: HashMap::new(),
            severity_breakdown: HashMap::new(),
            critical_failures: Vec::new(),
            statistics: RegressionStatistics {
                avg_duration_ms: 0.0,
                most_common_divergence: None,
                non_deterministic_rate: 0.0,
                problematic_commands: Vec::new(),
            },
            recommendations: vec!["No spans were tested".to_string()],
        });
    }

    let mut matched = 0;
    let mut diverged = 0;
    let mut failed = 0;
    let mut divergence_breakdown: HashMap<String, usize> = HashMap::new();
    let mut severity_breakdown: HashMap<String, usize> = HashMap::new();
    let mut critical_failures = Vec::new();
    let mut command_failures: HashMap<String, (usize, usize)> = HashMap::new();

    for result in &results {
        if result.matched {
            matched += 1;
        } else if let Some(div) = &result.divergence {
            diverged += 1;

            // Update divergence type breakdown
            let div_type = format!("{:?}", div.divergence_type);
            *divergence_breakdown.entry(div_type.clone()).or_insert(0) += 1;

            // Update severity breakdown
            let severity = format!("{:?}", div.severity);
            *severity_breakdown.entry(severity).or_insert(0) += 1;

            // Track critical failures
            if matches!(div.severity, DivergenceSeverity::Critical) {
                critical_failures.push(CriticalFailure {
                    span_id: result.span_id.clone(),
                    command: extract_command(&result.span_id), // Would need actual command
                    divergence_type: div_type,
                    description: div.description.clone(),
                });
            }
        } else {
            failed += 1;
        }

        // Track command failure rates
        let cmd = extract_command(&result.span_id);
        let entry = command_failures.entry(cmd).or_insert((0, 0));
        entry.0 += 1; // Total
        if !result.matched {
            entry.1 += 1; // Failed
        }
    }

    // Calculate statistics
    let pass_rate = (matched as f64 / total_spans as f64) * 100.0;

    let most_common_divergence = divergence_breakdown
        .iter()
        .max_by_key(|(_, count)| *count)
        .map(|(div_type, _)| div_type.clone());

    // Count non-deterministic failures (timing, PIDs, etc.)
    let non_deterministic_count = results
        .iter()
        .filter(|r| {
            r.divergence.as_ref().is_some_and(|d| {
                matches!(
                    d.divergence_type,
                    DivergenceType::TimingDrift | DivergenceType::EnvironmentChange
                )
            })
        })
        .count();

    let non_deterministic_rate = if diverged > 0 {
        (non_deterministic_count as f64 / diverged as f64) * 100.0
    } else {
        0.0
    };

    // Find problematic commands
    let mut problematic_commands: Vec<(String, f64)> = command_failures
        .into_iter()
        .filter(|(_, (_total, failed))| *failed > 0)
        .map(|(cmd, (total, failed))| (cmd, (failed as f64 / total as f64) * 100.0))
        .collect();

    problematic_commands.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    problematic_commands.truncate(5); // Top 5

    // Generate recommendations
    let recommendations = generate_recommendations(
        pass_rate,
        &severity_breakdown,
        &divergence_breakdown,
        non_deterministic_rate,
    );

    Ok(RegressionReport {
        total_spans,
        matched,
        diverged,
        failed,
        pass_rate,
        divergence_breakdown,
        severity_breakdown,
        critical_failures,
        statistics: RegressionStatistics {
            avg_duration_ms: 0.0, // Would need to track timing
            most_common_divergence,
            non_deterministic_rate,
            problematic_commands,
        },
        recommendations,
    })
}

/// Generate recommendations based on analysis
fn generate_recommendations(
    pass_rate: f64,
    severity_breakdown: &HashMap<String, usize>,
    divergence_breakdown: &HashMap<String, usize>,
    non_deterministic_rate: f64,
) -> Vec<String> {
    let mut recommendations = Vec::new();

    if pass_rate < 90.0 {
        recommendations.push(format!(
            "Pass rate is {:.1}% - investigate failures before deployment",
            pass_rate
        ));
    }

    if let Some(critical_count) = severity_breakdown.get("Critical") {
        if *critical_count > 0 {
            recommendations.push(format!(
                "{} critical failures detected - these must be fixed",
                critical_count
            ));
        }
    }

    if non_deterministic_rate > 20.0 {
        recommendations.push(format!(
            "{:.1}% of failures are non-deterministic - consider relaxing timing constraints",
            non_deterministic_rate
        ));
    }

    if let Some(exit_code_failures) = divergence_breakdown.get("ExitCode") {
        if *exit_code_failures > 0 {
            recommendations.push(format!(
                "{} commands failed with different exit codes - check for environment dependencies",
                exit_code_failures
            ));
        }
    }

    if let Some(fs_failures) = divergence_breakdown.get("FilesystemDiff") {
        if *fs_failures > 5 {
            recommendations.push(
                "Many filesystem differences detected - verify file paths are correct".to_string(),
            );
        }
    }

    if recommendations.is_empty() && pass_rate == 100.0 {
        recommendations.push("All replays passed successfully!".to_string());
    }

    recommendations
}

/// Extract command from span_id (placeholder - would need actual implementation)
fn extract_command(_span_id: &str) -> String {
    // In real implementation, would look up the command from the span
    "unknown".to_string()
}

/// Generate a detailed HTML report
pub fn generate_html_report(report: &RegressionReport) -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Substrate Replay Regression Report</title>
    <style>
        body {{ font-family: sans-serif; margin: 20px; }}
        h1 {{ color: #333; }}
        .summary {{ background: #f0f0f0; padding: 15px; border-radius: 5px; }}
        .pass {{ color: green; font-weight: bold; }}
        .fail {{ color: red; font-weight: bold; }}
        .warning {{ color: orange; }}
        table {{ border-collapse: collapse; width: 100%; margin: 20px 0; }}
        th, td {{ border: 1px solid #ddd; padding: 8px; text-align: left; }}
        th {{ background: #4CAF50; color: white; }}
        .critical {{ background: #ffcccc; }}
        .recommendation {{ background: #fffacd; padding: 10px; margin: 10px 0; }}
    </style>
</head>
<body>
    <h1>Substrate Replay Regression Report</h1>
    
    <div class="summary">
        <h2>Summary</h2>
        <p>Total Spans Tested: {}</p>
        <p>Pass Rate: <span class="{}">{:.1}%</span></p>
        <p>Matched: {} | Diverged: {} | Failed: {}</p>
    </div>
    
    <h2>Divergence Breakdown</h2>
    <table>
        <tr><th>Type</th><th>Count</th></tr>
        {}
    </table>
    
    <h2>Critical Failures</h2>
    {}
    
    <h2>Recommendations</h2>
    <div class="recommendation">
        {}
    </div>
</body>
</html>"#,
        report.total_spans,
        if report.pass_rate >= 90.0 {
            "pass"
        } else {
            "fail"
        },
        report.pass_rate,
        report.matched,
        report.diverged,
        report.failed,
        report
            .divergence_breakdown
            .iter()
            .map(|(k, v)| format!("<tr><td>{}</td><td>{}</td></tr>", k, v))
            .collect::<Vec<_>>()
            .join("\n"),
        if report.critical_failures.is_empty() {
            "<p>No critical failures detected</p>".to_string()
        } else {
            format!(
                "<table class='critical'><tr><th>Span ID</th><th>Description</th></tr>{}</table>",
                report
                    .critical_failures
                    .iter()
                    .map(|f| format!("<tr><td>{}</td><td>{}</td></tr>", f.span_id, f.description))
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        },
        report
            .recommendations
            .iter()
            .map(|r| format!("<li>{}</li>", r))
            .collect::<Vec<_>>()
            .join("\n")
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DivergenceReport, DivergenceSeverity, DivergenceType};

    #[test]
    fn test_analyze_empty_results() {
        let report = analyze_results(Vec::new()).unwrap();
        assert_eq!(report.total_spans, 0);
        assert_eq!(report.pass_rate, 0.0);
    }

    #[test]
    fn test_analyze_all_passed() {
        let results = vec![
            ReplayResult {
                span_id: "test1".to_string(),
                exit_code: 0,
                stdout: Vec::new(),
                stderr: Vec::new(),
                fs_diff: None,
                scopes_used: Vec::new(),
                matched: true,
                divergence: None,
                warnings: Vec::new(),
            },
            ReplayResult {
                span_id: "test2".to_string(),
                exit_code: 0,
                stdout: Vec::new(),
                stderr: Vec::new(),
                fs_diff: None,
                scopes_used: Vec::new(),
                matched: true,
                divergence: None,
                warnings: Vec::new(),
            },
        ];

        let report = analyze_results(results).unwrap();
        assert_eq!(report.total_spans, 2);
        assert_eq!(report.matched, 2);
        assert_eq!(report.pass_rate, 100.0);
        assert!(report
            .recommendations
            .contains(&"All replays passed successfully!".to_string()));
    }

    #[test]
    fn test_analyze_with_failures() {
        let results = vec![
            ReplayResult {
                span_id: "test1".to_string(),
                exit_code: 0,
                stdout: Vec::new(),
                stderr: Vec::new(),
                fs_diff: None,
                scopes_used: Vec::new(),
                matched: true,
                divergence: None,
                warnings: Vec::new(),
            },
            ReplayResult {
                span_id: "test2".to_string(),
                exit_code: 1,
                stdout: Vec::new(),
                stderr: Vec::new(),
                fs_diff: None,
                scopes_used: Vec::new(),
                matched: false,
                divergence: Some(DivergenceReport {
                    divergence_type: DivergenceType::ExitCode,
                    description: "Exit code mismatch".to_string(),
                    expected: "0".to_string(),
                    actual: "1".to_string(),
                    severity: DivergenceSeverity::Critical,
                }),
                warnings: Vec::new(),
            },
        ];

        let report = analyze_results(results).unwrap();
        assert_eq!(report.total_spans, 2);
        assert_eq!(report.matched, 1);
        assert_eq!(report.diverged, 1);
        assert_eq!(report.pass_rate, 50.0);
        assert_eq!(report.critical_failures.len(), 1);
    }
}
