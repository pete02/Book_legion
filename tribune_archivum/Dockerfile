# ----------------------
# Build stage (static binary)
# ----------------------
FROM rust:latest AS builder

WORKDIR /app

# musl for static linking
RUN apt-get update && apt-get install -y musl-tools

# add musl target
RUN rustup target add x86_64-unknown-linux-musl

# copy sources
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# build static binary
RUN cargo build --release --target x86_64-unknown-linux-musl

# ----------------------
# Runtime stage (Debian + Calibre)
# ----------------------
FROM debian:bookworm-slim

WORKDIR /app

# install calibre (provides ebook-convert)
RUN apt-get update && apt-get install -y \
    calibre \
 && rm -rf /var/lib/apt/lists/*

# copy static binary
COPY --from=builder \
  /app/target/x86_64-unknown-linux-musl/release/tribune_archivum \
  /app/tribune_archivum

# create data dirs
RUN mkdir -p /data /out /errs /onboard

ENTRYPOINT ["/app/tribune_archivum"]