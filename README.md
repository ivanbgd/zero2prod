# Zero to Production in Rust

[![Rust](https://github.com/ivanbgd/zero2prod/actions/workflows/general.yml/badge.svg)](https://github.com/ivanbgd/zero2prod/actions/workflows/general.yml)
[![Security audit](https://github.com/ivanbgd/zero2prod/actions/workflows/audit.yml/badge.svg)](https://github.com/ivanbgd/zero2prod/actions/workflows/audit.yml)

This is my implementation of the book [Zero to Production in Rust](https://www.zero2prod.com) by Luca Palmieri.

Code is similar to the original, but not exactly the same. It is not a copy-paste of the original code from the book.

Some improvements have been made by me.

One of the most notable improvements that I've made is the usage of the [rstest](https://docs.rs/rstest/latest/rstest/) crate for **testing**. The crate helps us in writing tests.  
It allows for test parameterization, for example. The feature allows for having unique test names and customized error messages per test case.  
Files in the [src/domain](src/domain) directory contain some examples and alternative implementations.  
So, some things have been implemented two or even three times, but in a different way, for the sake of example.

The [src/domain](src/domain) directory also contains some useful notes about input *name* and *email* **validation** (similar, but different notes).

### The Most Notable Crates Used
- [Actix Web](https://actix.rs/) (actix-web), as web framework
- [Tokio](https://tokio.rs/), as an asynchronous runtime
- [Docker](https://www.docker.com/), for containerization
- [PostgreSQL](https://www.postgresql.org/), as RDBMS
- [sqlx](https://docs.rs/sqlx/latest/sqlx/), as an async SQL toolkit for Rust
- [reqwest](https://docs.rs/reqwest/latest/reqwest/), as an HTTP client for sending e-mails and for integration testing
- [tracing](https://docs.rs/tracing/latest/tracing/index.html), for collecting scoped, structured, event-based diagnostic information
- [rstest](https://docs.rs/rstest/latest/rstest/), for testing
- [config](https://docs.rs/config/latest/config/), for configuration
- [secrecy](https://docs.rs/secrecy/latest/secrecy/), for more careful handling of secret values
- [validator](https://crates.io/crates/validator), for input validation
- [fake](https://crates.io/crates/fake), for generating fake data for testing
- [quickcheck](https://crates.io/crates/quickcheck), for property-based testing

A full list of crates used can be found in [Cargo.toml](Cargo.toml).
