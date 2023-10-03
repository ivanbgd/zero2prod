//! tests/health_check.rs
//!
//! Run with:
//! `cargo test --test health_check`

use rstest::rstest;
use std::net::TcpListener;

/// Spin up an instance of our application in the background and return its address (i.e., `http:://127.0.0.1:XXXX`)
fn spawn_app() -> String {
    let addr = "127.0.0.1";
    let addr_port = format!("{}:0", addr);
    let listener = TcpListener::bind(addr_port)
        .expect("Failed to bind a random port.");
    let port = listener.local_addr()
        .expect("Failed to unwrap listener's local address.").port();

    // We are not propagating errors like in `main()`, because this is a test function. We can simply panic instead.
    let server = zero2prod::run(listener)
        .unwrap_or_else(|_| panic!("Failed to bind the address {}:{}", addr, port));

    // Launch the server as a background task
    tokio::spawn(server);

    format!("http://{}:{}", addr, port)
}

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
    let addr = spawn_app();
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(format!("{}/health_check", addr))
        .send()
        .await
        .expect("Failed to send request to '/health_check'.");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_200_for_valid_form_data() {
    // Arrange
    let app_addr = spawn_app();
    let client = reqwest::Client::new();

    // Act
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(&format!("{}/subscriptions", &app_addr))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to send request to '/subscriptions'.");

    // Assert
    assert_eq!(200, response.status().as_u16());
}

#[tokio::test]
async fn subscribe_returns_400_when_data_is_missing() {
    // Arrange
    let app_addr = spawn_app();
    let client = reqwest::Client::new();
    let test_cases = [
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        // Act
        let response = client
            .post(format!("{}/subscriptions", app_addr))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to send request to '/subscriptions'.");

        // Assert
        assert_eq!(400, response.status().as_u16(),
                   "The API did not fail with 400 Bad Request when payload was {}.", error_message);
    }
}

#[rstest(
    invalid_body, error_message,
    case::missing_email("name=le%20guin", "missing the email"),
    case::missing_name("email=ursula_le_guin%40gmail.com", "missing the name"),
    case::missing_both_name_and_email("", "missing both name and email")
)]
#[tokio::test]
async fn subscribe_returns_400_when_data_is_missing_parameterized(invalid_body: &'static str, error_message: &str) {
    // Arrange
    let app_addr = spawn_app();
    let client = reqwest::Client::new();

    // Act
    let response = client
        .post(format!("{}/subscriptions", app_addr))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(invalid_body)
        .send()
        .await
        .expect("Failed to send request to '/subscriptions'.");

    // Assert
    assert_eq!(400, response.status().as_u16(),
               "The API did not fail with 400 Bad Request when payload was {}.", error_message);
}
