# ----------------------
# Build stage (dynamic binary)
# ----------------------
FROM rust:latest AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# normal build (no musl)
RUN cargo build --release

# ----------------------
# Runtime stage
# ----------------------
FROM ubuntu:24.04

WORKDIR /app

RUN apt-get update && apt-get install -y \
    calibre curl iputils-ping zip unzip\
 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/tribune_archivum /app/tribune_archivum

RUN mkdir -p /data /out /errs /onboard

ENTRYPOINT ["/app/tribune_archivum"]