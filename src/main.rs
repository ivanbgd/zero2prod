//! src/main.rs

use zero2prod::run;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    // Bubble-up the io::Error if we fail to bind the address.
    // Otherwise, call `.await` on our Server.
    run()?.await
}
