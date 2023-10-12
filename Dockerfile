FROM rust:1.72.0
LABEL authors="Ivan"

WORKDIR /app

RUN apt update && apt install lld clang -y

COPY . .

RUN cargo build --release

ENTRYPOINT ["./target/release/zero2prod"]
