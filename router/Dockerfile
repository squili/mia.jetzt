# syntax = docker/dockerfile:1.4

FROM rust:1.63-slim-bullseye AS builder

WORKDIR /app
COPY . .
RUN --mount=type=cache,target=/app/target \
    --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/usr/local/rustup \
    set -eux; \
    rustup install stable; \
    cargo build --release; \
    objcopy --compress-debug-sections target/release/router ./router

FROM debian:11.3-slim

WORKDIR /app

COPY --from=builder /app/router ./router
CMD ["./router"]

EXPOSE 8080
