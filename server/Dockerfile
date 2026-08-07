# syntax=docker/dockerfile:1

FROM rust:1.96-bookworm AS builder

WORKDIR /workspace
COPY server ./server
COPY openapi ./openapi

WORKDIR /workspace/server
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/workspace/server/target \
    cargo build --locked --release && \
    cp target/release/server /usr/local/bin/server

FROM debian:bookworm-slim AS runtime

RUN apt-get update && \
    apt-get install --yes --no-install-recommends ca-certificates && \
    rm -rf /var/lib/apt/lists/* && \
    useradd --create-home --uid 10001 app

COPY --from=builder /usr/local/bin/server /usr/local/bin/server

USER app
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/server"]
