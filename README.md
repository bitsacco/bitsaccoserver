# Bitsacco Server - Rust Implementation

[Bitsacco Server](https://github.com/minmoto/bitsaccoserver) is an open source model for running Savings and Credit Cooperatives (SACCO) on Bitcoin

## Technology Stack

- **Backend**: Axum web framework
- **Frontend**: Leptos (SSR + Hydration)
- **Database**: PostgreSQL with advanced caching (no Redis dependency)
- **Authentication**: Keycloak
- **Styling**: TailwindCSS
- **Build Tool**: cargo-leptos

## Documentation

- [JWT Deployment Guide](docs/JWT_DEPLOYMENT_GUIDE.md) - Complete guide for JWT configuration in production
- [Environment Variables](docs/ENVIRONMENT_VARIABLES.md) - Reference for all configuration options

## Quick Start

### Prerequisites

- Rust 1.82+
- Docker and Docker Compose
- Node.js (for TailwindCSS)

### One-Command Installation

```bash
# Install all dependencies and tools
chmod +x install.sh && ./install.sh
```

### Development

#### Local Development (without Docker)
```bash
# Start database and auth services
npm run services

# Run development server with hot reload
npm run cargo:dev
```

#### Docker Development (with hot reload)
```bash
# Build and run everything in development mode
npm run dev

# View logs
npm run logs
```

Open http://localhost:3000

### Production

```bash
# Build and run in production mode
npm run prod

# Build production binary locally
npm run cargo:build
```

### Available Scripts

#### Local Cargo Commands
- `npm run cargo:dev` - Run development server locally with cargo-leptos watch
- `npm run cargo:build` - Build production binary locally
- `npm run cargo:fmt` - Format all Rust code
- `npm run cargo:fmt:check` - Check if Rust code is formatted (CI friendly)

#### Docker Development Commands
- `npm run dev` - Start all services in development mode with hot reload
- `npm run dev:build` - Build development Docker image
- `npm run dev:rebuild` - Rebuild development image without cache
- `npm run logs` - View app container logs

#### Docker Production Commands
- `npm run prod` - Start all services in production mode
- `npm run prod:build` - Build production Docker image

#### Service Management
- `npm run services` - Start PostgreSQL and Keycloak only
- `npm run services:stop` - Stop all services
- `npm run stop` - Stop all Docker containers

#### Other Commands
- `npm run build-css` - Watch and rebuild TailwindCSS styles

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
npm run cargo:fmt

# Format everything (Rust + CSS)
npm run fmt

# Check formatting without changes
npm run cargo:fmt:check
```

### Linting and Testing
```bash
# Run Clippy linter
npm run cargo:lint

# Run all tests
npm run cargo:test

# Quick compilation check
npm run cargo:check
```

### Git Hooks
```bash
# Set up pre-commit hooks
npm run setup:hooks

# Run pre-commit checks manually
npm run precommit
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
