# ===== Stage 1: Builder =====
FROM rust:slim-bullseye AS builder

# Install build dependencies
RUN apt-get update && \
    apt-get install -y git curl build-essential pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

# Install Dioxus CLI
RUN cargo install dioxus-cli

# Set working directory
WORKDIR /app

# Copy source code
COPY . .

# Build web release bundle
# Note: if wasm-opt errors, the build still produces output
RUN dx bundle --platform web --release

# ===== Stage 2: Minimal nginx =====
FROM nginx:alpine

# Remove default nginx content
RUN rm -rf /usr/share/nginx/html/*

# Copy only the release bundle from builder
# Adjust the path based on your log:
# /app/target/dx/lector/release/web/public
COPY --from=builder /app/target/dx/lector/release/web/public /usr/share/nginx/html

COPY nginx.conf /etc/nginx/conf.d/default.conf

# Ensure correct MIME type for WASM
RUN echo "types { application/wasm wasm; }" >> /etc/nginx/mime.types

# Expose default HTTP port
EXPOSE 80

# Run nginx in foreground
CMD ["nginx", "-g", "daemon off;"]
