# BitSacco Server - Just Command Runner
# Replaces npm scripts with just for better cross-platform support

# Default recipe - show available commands
default:
    @just --list

# Environment variables
export NESTJS_API_URL := "http://localhost:4000/v1"

# ============================================================================
# CSS & Frontend Build Commands
# ============================================================================

# Build Tailwind CSS styles
build-css:
    npx tailwindcss -i ./style/tailwind.css -o ./public/styles.css

# Watch and rebuild CSS on changes
build-css-watch:
    npx tailwindcss -i ./style/tailwind.css -o ./public/styles.css --watch

# ============================================================================
# Cargo Development Commands
# ============================================================================

# Development build with CSS compilation
cargo-dev:
    just build-css
    cargo watch -x 'run --bin app --features ssr' -w app/src -w entity/src -w migration/src

# Release build with CSS compilation
cargo-build:
    just build-css
    cargo build --release --bin app --features ssr

# Format all Rust code
cargo-fmt:
    cargo fmt --all

# Check Rust code formatting
cargo-fmt-check:
    cargo fmt --all -- --check

# Run clippy lints
cargo-lint:
    cargo clippy --all-targets --all-features -- -D warnings

# Run all tests
cargo-test:
    cargo test --all

# Check compilation without building
cargo-check:
    cargo check --all

# Clean build artifacts
cargo-clean:
    cargo clean

# ============================================================================
# Admin Dashboard Commands
# ============================================================================

# Run admin dashboard with NestJS backend
admin:
    API_BACKEND=nestjs cargo run --bin app

# Run admin dashboard in development mode with file watching
admin-dev:
    just build-css
    API_BACKEND=nestjs cargo watch -x 'run --bin app --features ssr' -w app/src -w entity/src -w migration/src

# Show admin dashboard info
admin-info:
    @echo "Admin dashboard will run on http://localhost:3030"
    @echo "Backend API: {{NESTJS_API_URL}}"
    @echo "To run: just admin"

# Run admin with debug logging
admin-debug:
    RUST_LOG=debug API_BACKEND=nestjs cargo run --bin app

# ============================================================================
# Docker Commands
# ============================================================================

# Start development environment with Docker
docker-start:
    BUILD_TARGET=development docker compose up --build

# Build development Docker image
docker-build:
    BUILD_TARGET=development docker compose build app

# Rebuild development Docker image (no cache)
docker-rebuild:
    BUILD_TARGET=development docker compose build --no-cache app

# Start production environment
docker-prod:
    BUILD_TARGET=production docker compose up -d

# Build production Docker image
prod-build:
    BUILD_TARGET=production docker compose build app

# Stop all Docker containers
stop:
    docker compose down

# Show Docker logs
logs:
    docker compose logs -f app

# ============================================================================
# Git & Development Tools
# ============================================================================

# Setup git hooks
setup-hooks:
    cp scripts/pre-commit.sh .git/hooks/pre-commit
    chmod +x .git/hooks/pre-commit

# Format code (Rust + CSS)
fmt:
    just cargo-fmt
    just build-css

# Run pre-commit checks
precommit:
    ./scripts/pre-commit.sh

# ============================================================================
# Testing Commands
# ============================================================================

# Run end-to-end tests
test-e2e:
    cd e2e && npm test

# Run end-to-end tests with debug output
test-e2e-debug:
    cd e2e && npm run test:debug || true

# Run all tests (unit + e2e)
test-all:
    just cargo-test
    just test-e2e

# ============================================================================
# Database Commands (if applicable)
# ============================================================================

# Run database migrations
migrate:
    cargo run --bin migration

# Reset database
db-reset:
    cargo run --bin migration -- reset

# ============================================================================
# Maintenance Commands
# ============================================================================

# Full clean and rebuild
clean-rebuild:
    just cargo-clean
    cargo build
    just build-css

# Install dependencies and setup environment
setup:
    cargo build
    cd e2e && npm install
    just setup-hooks
    @echo "Setup complete! Run 'just admin' to start the admin dashboard."

# Check system dependencies
check-deps:
    @echo "Checking system dependencies..."
    @which cargo > /dev/null && echo "✅ cargo found" || echo "❌ cargo not found - install Rust"
    @which just > /dev/null && echo "✅ just found" || echo "❌ just not found - run: cargo install just"
    @which node > /dev/null && echo "✅ node found" || echo "❌ node not found - install Node.js"
    @which docker > /dev/null && echo "✅ docker found" || echo "❌ docker not found - install Docker"
    @test -f node_modules/.bin/tailwindcss && echo "✅ tailwindcss found" || echo "❌ tailwindcss not found - run: npm install"
    @echo "✅ Dependency check complete"

# Show project status
status:
    @echo "📊 BitSacco Server Status"
    @echo "=========================="
    @echo "🦀 Rust version: $(cargo --version)"
    @echo "📦 Project: $(cargo metadata --format-version 1 --no-deps | jq -r '.packages[0].name + " v" + .packages[0].version')"
    @echo "🎨 CSS: $([ -f public/styles.css ] && echo '✅ Built' || echo '❌ Not built')"
    @echo "🔧 Git hooks: $([ -f .git/hooks/pre-commit ] && echo '✅ Installed' || echo '❌ Not installed')"
    @echo ""
    @echo "🚀 Quick commands:"
    @echo "   just admin      - Start admin dashboard"
    @echo "   just admin-dev  - Start with file watching"
    @echo "   just test-e2e   - Run authentication tests"

# ============================================================================
# Platform-specific commands
# ============================================================================

# Watch logs on macOS/Linux
[unix]
watch-logs:
    tail -f logs/*.log || echo "No log files found"

# Install system dependencies on Ubuntu/Debian
[linux]
install-deps-ubuntu:
    sudo apt update
    sudo apt install -y build-essential pkg-config libssl-dev
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    npm install -g tailwindcss

# ============================================================================
# Aliases for common commands (similar to npm run)
# ============================================================================

# Aliases matching npm script names for easy migration
alias start := docker-start
alias build := cargo-build
alias test := test-all
alias lint := cargo-lint
