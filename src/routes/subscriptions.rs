//! src/routes/subscriptions.rs

use actix_web::{web, HttpResponse};
use chrono::Utc;
use sqlx::PgPool;
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
        "Adding a new subscriber.",
        %request_id,
        subscriber_email = %form.email,
        subscriber_name = %form.name
    );
    let _request_span_guard = request_span.enter();

    tracing::info!(
        r#"Request ID {} - Adding "{}" "{}" as a new subscriber."#,
        request_id,
        form.email,
        form.name
    );
    tracing::info!(
        "Request ID {} - Saving new subscriber details in the database.",
        request_id
    );
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
    .await
    {
        Ok(_) => {
            tracing::info!(
                r#"Request ID {} - New subscriber details have been saved, "{}" "{}"."#,
                request_id,
                form.email,
                form.name
            );
            HttpResponse::Ok().finish()
        }
        Err(e) => {
            tracing::error!(
                r#"Request ID {} - Failed to execute query: "{:?}"."#,
                request_id,
                e
            );
            HttpResponse::InternalServerError().finish()
        }
    }
}
