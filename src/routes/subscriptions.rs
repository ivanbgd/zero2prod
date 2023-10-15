//! src/routes/subscriptions.rs

use crate::consts::{FORBIDDEN_NAME_CHARACTERS, MAX_NAME_LEN};
use actix_web::{web, HttpResponse};
use chrono::Utc;
use sqlx::PgPool;
use unicode_segmentation::UnicodeSegmentation;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

/// Subscribe a new member
///
/// An orchestrator function which calls the required routines and translates their output
/// into a proper HTTP response.
/// We retrieve a connection from the application state (which is defined at startup).
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
    if !is_valid_name(&form.name) {
        return HttpResponse::BadRequest().finish();
    }

    match insert_subscriber(&form, &pool).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

/// Checks validity of a new user's name
///
/// Returns `true` if **ALL** validation constraints are satisfied,
/// `false` otherwise.
pub fn is_valid_name(name: &str) -> bool {
    let is_empty_or_whitespace = name.trim().is_empty();

    let is_too_long = name.graphemes(true).count() > MAX_NAME_LEN;

    let contains_a_forbidden_character =
        name.chars().any(|c| FORBIDDEN_NAME_CHARACTERS.contains(&c));

    !(is_empty_or_whitespace || is_too_long || contains_a_forbidden_character)
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
    skip(form, pool)
)]
pub async fn insert_subscriber(form: &FormData, pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
            INSERT INTO subscriptions (id, email, name, subscribed_at)
            VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        form.email,
        form.name,
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

#[cfg(test)]
mod tests {
    use super::*;
    use once_cell::sync::Lazy;
    use rstest::rstest;

    static VALID_MAX_LONG_NAME: Lazy<String> = Lazy::new(|| "å".repeat(MAX_NAME_LEN));
    static TOO_LONG_NAME: Lazy<String> = Lazy::new(|| "a".repeat(MAX_NAME_LEN + 1));

    #[rstest(
        valid_name,
        case::first_name("John"),
        case::first_last("John Doe"),
        case::first_last_whitespace("  \t \n  John  \t \n  Doe \t \n  "),
        case::non_ascii("å"),
        case::non_ascii_max_long(&VALID_MAX_LONG_NAME),
        case::punctuation(". , ? ! : ;"),
    )]
    fn is_valid_name_passes_valid_names(valid_name: &'static str) {
        let is_valid = is_valid_name(valid_name);
        assert_eq!(true, is_valid, "Rejected a valid name '{}'.", valid_name);
    }

    #[rstest(
        invalid_name,
        error_message,
        case::empty_name("", "empty"),
        case::whitespace_name(" \t \r \n   ", "whitespace"),
        case::too_long(&TOO_LONG_NAME, "too long"),
        case::forward_slash("/", "forward slash"),
        case::open_parenthesis("(", "open parenthesis"),
        case::close_parenthesis(")", "close parenthesis"),
        case::double_quote(r#"""#, "double quote"),
        case::open_angle_bracket("<", "open angle bracket"),
        case::close_angle_bracket(">", "close angle bracket"),
        case::back_slash("\\", "back slash"),
        case::open_curly_brace("{", "open curly brace"),
        case::close_curly_brace("}", "close curly brace"),
    )]
    fn is_valid_name_rejects_invalid_names(invalid_name: &'static str, error_message: &str) {
        let is_valid = is_valid_name(invalid_name);
        assert_eq!(
            false, is_valid,
            "Didn't reject the invalid name '{}' (name is {}).",
            invalid_name, error_message
        );
    }
}
