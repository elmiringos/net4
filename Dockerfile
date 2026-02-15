FROM rust:1.85-bookworm AS builder

RUN apt-get update && apt-get install -y \
    clang \
    libclang-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src/ src/

RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/net4 /usr/local/bin/net4
CMD ["net4"]
