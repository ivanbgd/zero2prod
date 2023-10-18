//! src/routes/health_check.rs

use actix_web::HttpResponse;

/// Health check
///
/// This is a request handler for the `GET /health_check` endpoint.
pub async fn health_check() -> HttpResponse {
    tracing::debug!("Health check is working!");
    HttpResponse::Ok().finish()
}
