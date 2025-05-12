FROM mcr.microsoft.com/devcontainers/rust:latest

RUN apt-get update && apt-get install clang pkg-config

RUN cargo install trunk \
    && cargo install leptosfmt \
    && rustup target add wasm32-unknown-unknown
