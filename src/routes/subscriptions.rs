//! src/routes/subscriptions.rs

use actix_web::{web, HttpResponse};
use chrono::Utc;
use sqlx::PgPool;
use tracing::Instrument;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

/// Subscribe a new member
///
/// We retrieve a connection from the application state.
pub async fn subscribe(
    web::Form(form): web::Form<FormData>,
    pool: web::Data<PgPool>,
) -> HttpResponse {
    let request_id = Uuid::new_v4();
    let request_span = tracing::info_span!(
        "Adding a new subscriber",
        %request_id,
        subscriber_email = %form.email,
        subscriber_name = %form.name
    );
    let _request_span_guard = request_span.enter();

    // We are keeping this old log message and converting it to a trace info for the sake of comparison and education.
    // It is generally not needed anymore, after we've stopped using logging and switched to tracing, so it can be
    // removed in production-ready code. Namely, the above message contains the same data.
    tracing::info!(
        "Request ID {} - Adding '{}' '{}' as a new subscriber",
        request_id,
        form.email,
        form.name
    );

    let query_span = tracing::info_span!("Saving new subscriber details in the database");
    match sqlx::query!(
        r#"
            INSERT INTO subscriptions (id, email, name, subscribed_at)
            VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(pool.get_ref())
    .instrument(query_span)
    .await
    {
        Ok(_) => {
            tracing::info!(
                "New subscriber details have been saved, '{}' '{}'.",
                form.email,
                form.name
            );
            HttpResponse::Ok().finish()
        }
        Err(e) => {
            tracing::error!("Failed to execute query: '{:?}'.", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
