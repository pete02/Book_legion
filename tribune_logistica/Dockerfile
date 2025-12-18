# Minimal Rust builder image
FROM rust:slim-bullseye AS builder

# Install any system dependencies your backend might need
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev build-essential && \
    rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy source code
COPY . .


# Default command: start an interactive shell
RUN cargo build --release


FROM debian:bullseye-slim

# Optional: Install SSL certs if backend uses HTTPS
RUN apt-get update && apt-get install -y ca-certificates ffmpeg && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy the release binary from builder
COPY --from=builder /app/target/release/tribune_logistica .

# Expose port (adjust to your backend port)
EXPOSE 8000

# Run the backend
CMD ["./tribune_logistica"]