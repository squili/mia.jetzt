# syntax = docker/dockerfile:1.4

FROM rust:1.63-slim-bullseye AS builder

RUN apt update; \
    apt install -y make g++
WORKDIR /app
COPY . .
RUN --mount=type=cache,target=/app/target \
    --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/usr/local/rustup \
    set -eux; \
    rustup install stable; \
    cargo build --release; \
    objcopy --compress-debug-sections target/release/site ./site

FROM debian:bullseye-slim

WORKDIR /app
COPY --from=builder /app/site ./site
COPY --from=builder /app/templates ./templates
CMD ["./site"]

EXPOSE 8080
