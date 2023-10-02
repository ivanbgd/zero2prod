//! tests/health_check.rs

// `#[tokio::test]` is the testing equivalent of `#[tokio::main]`.
// It also saves us from having to specify the `#[test]` attribute.
//
// We can inspect code that gets generated using
// `cargo expand --test health_check` (<- name of the test file).

// When a tokio runtime is shut down all tasks spawned on it are dropped.
// `tokio::test` spins up a new runtime at the beginning of each
// test case, and they shut down at the end of each test case.
// Hence, there is no need to implement any clean up logic to avoid leaking resources between test runs.

/// Test health check
///
/// `spawn_app()` is the only piece that will, reasonably, depend on our application code.
/// Everything else is completely decoupled from the underlying implementation details.
///
/// Additionally, the test covers a full range of properties we are interested in checking:
/// - the verb used is GET,
/// - the endpoint is `/health_check`,
/// - the endpoint always returns `200 OK`,
/// - the response has no body.
#[tokio::test]
async fn health_check_works() {
    // Arrange
    spawn_app();
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get("http://127.0.0.1:8000/health_check")
        .send()
        .await
        .expect("Failed to send request to '/health_check'.");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

/// Launch our application in the background
fn spawn_app() {
    // We are not propagating errors like in `main()`, because this is a test function.
    // We can simply panic instead.
    let server = zero2prod::run().expect("Failed to bind address.");

    // Launch the server as a background task
    tokio::spawn(server);
}
