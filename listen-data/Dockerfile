FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
WORKDIR /app/listen-tracing
RUN ls -la
WORKDIR /app/listen-data
RUN ls -la
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
WORKDIR /app
COPY . .
WORKDIR /app/listen-data
COPY --from=planner /app/listen-data/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
RUN cargo build --release --bin indexer

EXPOSE 6969

FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y \
	ca-certificates \
	openssl \
	libssl3 \
	&& rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/listen-data/target/release/indexer /usr/local/bin
ENTRYPOINT ["/usr/local/bin/indexer"]
