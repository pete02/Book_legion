# Stage 1: Build the Dioxus WASM frontend
FROM rust:latest as builder

# Install wasm32 target
RUN rustup target add wasm32-unknown-unknown

# Install Dioxus CLI
RUN cargo install dioxus-cli

# Create app directory
WORKDIR /app

# Copy source
COPY . .

# Build the frontend in release mode
# Assumes your frontend is in the default "src" folder
RUN dx build --release --web --output-path dist

# Stage 2: Serve the frontend with nginx
FROM nginx:alpine

# Remove default nginx content
RUN rm -rf /usr/share/nginx/html/*

# Copy built frontend from builder
COPY --from=builder /app/dist /usr/share/nginx/html

# Copy a default nginx configuration (optional)
# COPY nginx.conf /etc/nginx/conf.d/default.conf

# Expose port 80
EXPOSE 80

# Run nginx
CMD ["nginx", "-g", "daemon off;"]
