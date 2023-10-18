//! src/routes/subscriptions.rs

use crate::domain::{NewSubscriber, SubscriberEmail, SubscriberName};
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
/// This is a request handler for the `POST /subscriptions` endpoint.
///
/// An orchestrator function which calls the required routines and translates their output
/// into a proper HTTP response to the incoming HTTP request.
/// We retrieve a connection from the application state (which is defined at startup).
#[allow(clippy::async_yields_async)]
#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, pool),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
pub async fn subscribe(
    web::Form(form): web::Form<FormData>,
    pool: web::Data<PgPool>,
) -> HttpResponse {
    let new_subscriber = match NewSubscriber::try_from(form) {
        Ok(new_subscriber) => new_subscriber,

        // Return early with 400 Bad Request if the new subscriber is invalid
        Err(_) => return HttpResponse::BadRequest().finish(),
    };

    match insert_subscriber(&new_subscriber, &pool).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

/// Converts form data into `NewSubscriber`.
///
/// Converts data from our *wire format* (the URL-decoded data obtained from a web (HTML) form)
/// to our *domain model*, `NewSubscriber`.
fn parse_subscriber(form: FormData) -> Result<NewSubscriber, String> {
    let email = SubscriberEmail::parse(form.email)?;
    let name = SubscriberName::parse(form.name)?;
    Ok(NewSubscriber { email, name })
}

impl TryFrom<FormData> for NewSubscriber {
    type Error = String;

    /// Converts form data into `NewSubscriber`.
    ///
    /// Converts data from our *wire format* (the URL-decoded data obtained from a web (HTML) form)
    /// to our *domain model*, `NewSubscriber`.
    fn try_from(form: FormData) -> Result<Self, Self::Error> {
        let email = SubscriberEmail::parse(form.email)?;
        let name = SubscriberName::parse(form.name)?;

        Ok(NewSubscriber { email, name })
    }
}

/// Insert the new subscriber details in a Postgres database
///
/// This function doesn't depend, nor is aware, of a potentially surrounding (web) framework,
/// which is good. The input parameters are not necessarily of a web-type.
/// This function just executes a DB query.
/// This is a quasi-DAL, although still not fully-independent (not fully-abstract).
/// It is specialized for the Postgres database, though, so still not 100% generic,
/// but at least it only knows about a DB and only works with a DB.
/// Sure enough, if it were fully-abstract, it would be abstracted away from a DB as well,
/// because data in general do not necessarily have to be persisted in a DB.
/// So, there is room for improvement, for even better abstraction and separation of concerns,
/// for even looser coupling, but is a step in the right direction.
/// We could add a true DAL, because this is more of a concrete data-layer implementation than a DAL.
#[tracing::instrument(
    name = "Saving the new subscriber details in the database",
    skip(new_subscriber, pool)
)]
async fn insert_subscriber(
    new_subscriber: &NewSubscriber,
    pool: &PgPool,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
            INSERT INTO subscriptions (id, email, name, subscribed_at)
            VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        Utc::now()
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: '{:?}'.", e);
        e
    })?;

    Ok(())
}
