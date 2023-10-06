//! src/startup.rs

use crate::routes::{health_check, subscribe};
use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
use sqlx::PgPool;
use std::net::TcpListener;

/// Run the application - the web server - concurrently
///
/// Spin up a worker process for each available CPU core.
/// Each worker runs its own copy of the application.
pub fn run(listener: TcpListener, db_pool: PgPool) -> Result<Server, std::io::Error> {
    let db_pool = web::Data::new(db_pool);
    let server = HttpServer::new(move || {
        App::new()
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .app_data(db_pool.clone())  // Get a pointer copy and attach it to the application state.
    })
        .listen(listener)?
        .run();

    Ok(server)
}
