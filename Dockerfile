FROM rust:1-slim-buster as build

RUN cargo new --bin app
WORKDIR /app
COPY Cargo.toml /app/
COPY Cargo.lock /app/
COPY src /app/src

RUN cargo build --release

FROM debian:buster-slim

COPY --from=build /app/target/release/rinha_backend_2024_Q1 /app/rinha
CMD "/app/rinha"