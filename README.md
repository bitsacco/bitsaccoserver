# Bitsacco Server - Rust Implementation

[Bitsacco Server](https://github.com/minmoto/bitsaccoserver) is an open source model for running Savings and Credit Cooperatives (SACCO) on Bitcoin

## Technology Stack

- **Backend**: Axum web framework
- **Frontend**: Leptos (SSR + Hydration)
- **Database**: PostgreSQL with advanced caching (no Redis dependency)
- **Authentication**: JWT-based with NestJS integration
- **Styling**: TailwindCSS
- **Build Tool**: Just command runner + Cargo

## Documentation

- [JWT Deployment Guide](docs/JWT_DEPLOYMENT_GUIDE.md) - Complete guide for JWT configuration in production
- [Environment Variables](docs/ENVIRONMENT_VARIABLES.md) - Reference for all configuration options

## Quick Start

### Prerequisites

- Rust 1.82+
- [Just](https://crates.io/crates/just) command runner: `cargo install just`
- Docker and Docker Compose
- Node.js (for TailwindCSS)

### Setup

```bash
# Install dependencies and setup environment
just setup

# Check system dependencies
just check-deps
```

### Development

#### Admin Dashboard (Recommended)
```bash
# Start admin dashboard with NestJS backend
just admin

# Start with file watching for development
just admin-dev

# Start with debug logging
just admin-debug
```

#### Docker Development
```bash
# Build and run everything in development mode
just docker-dev

# View logs
just logs
```

Open http://localhost:3030 for admin dashboard

### Production

```bash
# Build and run in production mode
just prod

# Build production binary locally
just build
```

### Available Commands

View all available commands:
```bash
just --list
```

#### Main Commands
- `just admin` - Start admin dashboard
- `just admin-dev` - Start with file watching
- `just admin-debug` - Start with debug logging
- `just build` - Build production binary
- `just test` - Run all tests
- `just test-e2e` - Run end-to-end authentication tests

#### Development Commands
- `just cargo-dev` - Development server with hot reload
- `just cargo-fmt` - Format all Rust code
- `just cargo-lint` - Run clippy lints
- `just cargo-test` - Run unit tests
- `just build-css` - Build Tailwind CSS
- `just build-css-watch` - Watch and rebuild CSS

#### Docker Commands
- `just docker-dev` - Start Docker development environment
- `just prod` - Start production environment
- `just stop` - Stop all containers
- `just logs` - View container logs

#### Maintenance Commands
- `just setup` - Install dependencies and setup
- `just status` - Show project status
- `just fmt` - Format code (Rust + CSS)
- `just clean-rebuild` - Clean and rebuild everything

## Project Structure

```
bsr/
├── app/              # Main application (Leptos + Axum)
├── entity/           # Database entities (SeaORM)
├── migration/        # Database migrations
├── style/            # CSS and styling
├── public/           # Static assets
├── tests/            # Integration tests
└── keycloak/         # Keycloak configuration
```

## Development Status

This is Phase 1 of the migration plan. Currently implemented:

- ✅ Project structure setup
- ✅ Cargo workspace configuration
- ✅ Docker Compose environment
- ✅ Basic Leptos application
- ✅ Configuration management
- ✅ Logging and tracing
- ✅ TailwindCSS integration
- ✅ Health check endpoints
- ✅ Testing framework setup

## Next Steps (Phase 2)

- Database schema design
- SeaORM entities implementation
- PostgreSQL caching strategies
- Migration system

## API Endpoints

- `GET /` - Home page
- `GET /health` - Health check page
- `GET /api/health` - Health check API
- `GET /api/info` - API information

## Code Quality

### Formatting
```bash
# Format all Rust code
just cargo-fmt

# Format everything (Rust + CSS)
just fmt

# Check formatting without changes
just cargo-fmt-check
```

### Linting and Testing
```bash
# Run Clippy linter
just cargo-lint

# Run all tests
just test

# Run unit tests only
just cargo-test

# Run e2e tests
just test-e2e

# Quick compilation check
just cargo-check
```

### Git Hooks
```bash
# Set up pre-commit hooks
just setup-hooks

# Run pre-commit checks manually
just precommit
```

## Contributing

Please refer to the migration plan document for development guidelines and phase objectives. All code must be formatted and pass linting checks before committing.

## Support

Bitsacco Server is MIT-licensed. We grow thanks to the sponsors and support by the amazing backers. If you'd like to join them, please [read more here](https://bitsacco.com/opensource).

## Stay in touch

- Website - [https://bitsacco.com](https://bitsacco.com/)
- Twitter - [@bitsacco](https://twitter.com/bitsacco)
- Maintainer - [Jodom](https://twitter.com/okjodom)

## License

Bitsacco Server is [MIT licensed](https://github.com/minmoto/bitsaccoserver/blob/main/LICENSE).
