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
    log::info!(
        r#"Request ID {} - Adding "{}" "{}" as a new subscriber."#,
        request_id,
        form.email,
        form.name
    );
    log::info!(
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
            log::info!(
                r#"Request ID {} - New subscriber details have been saved, "{}" "{}"."#,
                request_id,
                form.email,
                form.name
            );
            HttpResponse::Ok().finish()
        }
        Err(e) => {
            log::error!(
                r#"Request ID {} - Failed to execute query: "{:?}"."#,
                request_id,
                e
            );
            HttpResponse::InternalServerError().finish()
        }
    }
}
