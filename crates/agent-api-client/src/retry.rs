//! Retry logic for agent communication.

use anyhow::Result;
use std::time::Duration;
use tokio::time::sleep;

/// Retry configuration for agent requests.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts.
    pub max_attempts: u32,
    /// Initial delay between retries.
    pub initial_delay: Duration,
    /// Maximum delay between retries.
    pub max_delay: Duration,
    /// Multiplier for exponential backoff.
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 2.0,
        }
    }
}

impl RetryConfig {
    /// Create a retry config with no retries.
    pub fn no_retry() -> Self {
        Self {
            max_attempts: 1,
            initial_delay: Duration::ZERO,
            max_delay: Duration::ZERO,
            backoff_multiplier: 1.0,
        }
    }

    /// Create a retry config with aggressive retries.
    pub fn aggressive() -> Self {
        Self {
            max_attempts: 5,
            initial_delay: Duration::from_millis(50),
            max_delay: Duration::from_secs(5),
            backoff_multiplier: 1.5,
        }
    }
}

/// Retry a future with exponential backoff.
pub async fn retry_with_backoff<F, Fut, T, E>(
    config: &RetryConfig,
    mut operation: F,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Debug,
{
    let mut delay = config.initial_delay;
    let mut last_error = None;
    let max_attempts = std::cmp::max(1, config.max_attempts);

    for attempt in 1..=max_attempts {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(error) => {
                last_error = Some(error);

                if attempt < max_attempts {
                    tracing::debug!("Attempt {} failed, retrying in {:?}", attempt, delay);

                    sleep(delay).await;

                    // Exponential backoff with max delay
                    delay = std::cmp::min(
                        Duration::from_secs_f64(delay.as_secs_f64() * config.backoff_multiplier),
                        config.max_delay,
                    );
                }
            }
        }
    }

    match last_error {
        Some(err) => Err(err),
        None => unreachable!("retry_with_backoff executed without attempts"),
    }
}

/// Determine if an error is retryable.
pub fn is_retryable_error(error: &anyhow::Error) -> bool {
    let error_str = error.to_string().to_lowercase();

    // Network-related errors that might be transient
    error_str.contains("connection refused")
        || error_str.contains("connection reset")
        || error_str.contains("timeout")
        || error_str.contains("temporary failure")
        || error_str.contains("service unavailable")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_config_defaults() {
        let config = RetryConfig::default();
        assert_eq!(config.max_attempts, 3);
        assert_eq!(config.initial_delay, Duration::from_millis(100));
        assert_eq!(config.max_delay, Duration::from_secs(10));
        assert_eq!(config.backoff_multiplier, 2.0);
    }

    #[test]
    fn test_no_retry_config() {
        let config = RetryConfig::no_retry();
        assert_eq!(config.max_attempts, 1);
        assert_eq!(config.initial_delay, Duration::ZERO);
    }

    #[test]
    fn test_aggressive_retry_config() {
        let config = RetryConfig::aggressive();
        assert_eq!(config.max_attempts, 5);
        assert_eq!(config.initial_delay, Duration::from_millis(50));
    }

    #[tokio::test]
    async fn test_retry_success_on_first_attempt() {
        let config = RetryConfig::default();
        let result = retry_with_backoff(&config, || async { Ok::<i32, &'static str>(42) }).await;

        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_retry_success_after_failures() {
        let config = RetryConfig::default();
        let counter = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));

        let result = retry_with_backoff(&config, || {
            let counter = counter.clone();
            async move {
                let attempt = counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
                if attempt < 3 {
                    Err("temporary failure")
                } else {
                    Ok(42)
                }
            }
        })
        .await;

        assert_eq!(result.unwrap(), 42);
        assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_exhausted() {
        let config = RetryConfig::no_retry();
        let result = retry_with_backoff(&config, || async {
            Err::<i32, &'static str>("permanent failure")
        })
        .await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "permanent failure");
    }

    #[test]
    fn test_retryable_error_detection() {
        let retryable_errors = [
            anyhow::anyhow!("Connection refused"),
            anyhow::anyhow!("Connection reset by peer"),
            anyhow::anyhow!("Request timeout"),
            anyhow::anyhow!("Temporary failure in name resolution"),
            anyhow::anyhow!("Service unavailable"),
        ];

        for error in &retryable_errors {
            assert!(
                is_retryable_error(error),
                "Expected {} to be retryable",
                error
            );
        }

        let non_retryable_errors = [
            anyhow::anyhow!("Invalid request format"),
            anyhow::anyhow!("Authentication failed"),
            anyhow::anyhow!("Permission denied"),
            anyhow::anyhow!("Not found"),
        ];

        for error in &non_retryable_errors {
            assert!(
                !is_retryable_error(error),
                "Expected {} to not be retryable",
                error
            );
        }
    }
}
