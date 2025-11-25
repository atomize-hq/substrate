use std::env;
use std::sync::Mutex;

// Global mutex to ensure tests that modify environment run sequentially
pub(crate) static TEST_ENV_MUTEX: Mutex<()> = Mutex::new(());

// Helper to run tests with TEST_MODE set
pub(crate) fn with_test_mode<F: FnOnce()>(f: F) {
    // Lock the mutex to ensure exclusive access to environment
    let _guard = TEST_ENV_MUTEX.lock().unwrap();

    // Save original value if it exists
    let original = env::var("TEST_MODE").ok();

    env::set_var("TEST_MODE", "1");
    f();

    // Restore original value or remove
    match original {
        Some(val) => env::set_var("TEST_MODE", val),
        None => env::remove_var("TEST_MODE"),
    }
}
