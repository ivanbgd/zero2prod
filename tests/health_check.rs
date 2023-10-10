//! tests/health_check.rs
//!
//! Run with:
//! `cargo test --test health_check`

use once_cell::sync::Lazy;
use rstest::rstest;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;
use uuid::Uuid;
use zero2prod::configuration::{get_configuration, DatabaseSettings};
use zero2prod::startup::run;
use zero2prod::telemetry::{get_subscriber, init_subscriber};

// Ensure that the `tracing` stack is initialized only once by using `once_cell`
static TRACING: Lazy<()> = Lazy::new(|| {
    let subscriber_name = "test";
    let default_log_level = "debug";
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_log_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_log_level, std::io::sink);
        init_subscriber(subscriber);
    }
});

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

/// Spin up an instance of our application in the background and return a `TestApp` struct
/// with the app's address (i.e., `http:://127.0.0.1:XXXX`) and a handle to the connection pool.
async fn spawn_app() -> TestApp {
    // The code in `TRACING` is executed only the first time `spawn_app` is invoked.
    // All other invocations will skip its execution.
    // This means that subscriber initialization happens only once.
    Lazy::force(&TRACING);

    let addr = "127.0.0.1";
    let addr_port = format!("{}:0", addr);
    let listener = TcpListener::bind(addr_port).expect("Failed to bind a random port.");
    let port = listener
        .local_addr()
        .expect("Failed to unwrap listener's local address.")
        .port();
    let address = format!("http://{}:{}", addr, port);

    let mut configuration = get_configuration().expect("Failed to read configuration.");
    configuration.database.database_name = Uuid::new_v4().to_string();
    let db_pool = configure_database(&configuration.database).await;

    // We are not propagating errors like in `main()`, because this is a test function. We can simply panic instead.
    let server = run(listener, db_pool.clone())
        .unwrap_or_else(|_| panic!("Failed to bind the address '{}'.", address));

    // Launch the server as a background task
    tokio::spawn(server);

    TestApp { address, db_pool }
}

async fn configure_database(db_settings: &DatabaseSettings) -> PgPool {
    let connection_string = db_settings.get_connection_string_without_database_name();

    // Create database
    let mut connection = PgConnection::connect(&connection_string)
        .await
        .expect("Failed to connect to Postgres.");
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, db_settings.database_name).as_str())
        .await
        .expect("Failed to create database.");

    // Migrate database
    let db_pool = PgPool::connect(&db_settings.get_connection_string())
        .await
        .expect("Failed to create a new connection pool and to connect to Postgres.");
    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .expect("Failed to migrate the database.");

    db_pool
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
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(&format!("{}/health_check", &app.address))
        .send()
        .await
        .expect("Failed to send request to '/health_check'.");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_200_for_valid_form_data() {
    println!("Hello from subscribe_returns_200_for_valid_form_data!!!"); // REMOVE!!!

    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    // Act
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(&format!("{}/subscriptions", &app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to send request to '/subscriptions'.");

    // Assert
    assert_eq!(200, response.status().as_u16());

    // Act
    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");

    // Assert
    assert_eq!("ursula_le_guin@gmail.com", saved.email);
    assert_eq!("le guin", saved.name);
}

#[tokio::test]
async fn subscribe_returns_400_when_data_is_missing() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = [
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        // Act
        let response = client
            .post(format!("{}/subscriptions", app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to send request to '/subscriptions'.");

        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when payload was {}.",
            error_message
        );
    }
}

#[rstest(
    invalid_body,
    error_message,
    case::missing_email("name=le%20guin", "missing the email"),
    case::missing_name("email=ursula_le_guin%40gmail.com", "missing the name"),
    case::missing_both_name_and_email("", "missing both name and email")
)]
#[tokio::test]
async fn subscribe_returns_400_when_data_is_missing_parameterized(
    invalid_body: &'static str,
    error_message: &str,
) {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    // Act
    let response = client
        .post(format!("{}/subscriptions", app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(invalid_body)
        .send()
        .await
        .expect("Failed to send request to '/subscriptions'.");

    // Assert
    assert_eq!(
        400,
        response.status().as_u16(),
        "The API did not fail with 400 Bad Request when payload was {}.",
        error_message
    );
}
