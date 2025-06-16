FROM mcr.microsoft.com/devcontainers/rust:latest

RUN apt-get update && apt-get install -y clang pkg-config

RUN cargo install --locked cargo-leptos \
    && cargo install leptosfmt \
    && rustup target add wasm32-unknown-unknown
