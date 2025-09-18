# Common build stage for dependencies  
FROM rust:latest AS dependencies

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Install Node.js for TailwindCSS
RUN curl -fsSL https://deb.nodesource.com/setup_lts.x | bash - && \
    apt-get install -y nodejs

# Install cargo tools (use latest stable version)
RUN cargo install cargo-watch --locked

WORKDIR /app

# Copy dependency files first
COPY Cargo.toml Leptos.toml ./
COPY app/Cargo.toml ./app/

# Create dummy source files to cache dependencies
RUN mkdir -p app/src && \
    echo "fn main() {}" > app/src/main.rs && \
    echo "" > app/src/lib.rs

# Build dependencies (cached layer)
RUN cargo build --release
RUN rm -rf app/src

# Development stage
FROM dependencies AS development

# Install TailwindCSS
RUN npm install -g tailwindcss @tailwindcss/forms @tailwindcss/typography

# Copy package files and install npm dependencies
COPY package*.json ./
RUN npm install

WORKDIR /app

# Set development environment variables
ENV LEPTOS_RELOAD_PORT=3001
ENV RUST_LOG=debug
ENV LEPTOS_SITE_ADDR="0.0.0.0:3030"

EXPOSE 3000 3001

# Use cargo-watch for hot-reload development
CMD ["cargo", "watch", "-x", "run --bin app --features ssr", "-w", "app/src", "-w"]

# Builder stage for production
FROM dependencies AS builder

# Copy all source code
COPY . .

# Install npm dependencies and build CSS
RUN npm install
RUN npm run build-css

# Build the application
RUN cargo build --release --bin app --features ssr

# Production stage
FROM debian:bookworm-slim AS production

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary and assets
COPY --from=builder /app/target/release/app /app/
COPY --from=builder /app/public /app/public/

ENV LEPTOS_SITE_ROOT="/app/site"
ENV LEPTOS_SITE_ADDR="0.0.0.0:3030"
ENV RUST_LOG=info

EXPOSE 3030

CMD ["./app"]
