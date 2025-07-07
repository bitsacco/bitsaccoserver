# Quick Start Guide

This guide will help you get the Bitsacco Server up and running quickly.

## Prerequisites

- Docker and Docker Compose
- Node.js and npm (for local development)
- Rust toolchain (for local development)

## One-Command Setup

```bash
# Install all dependencies
chmod +x install.sh && ./install.sh
```

## Development Workflows

### 1. Full Docker Development (Recommended)

This runs everything in Docker with hot reload support:

```bash
# Start all services (PostgreSQL, Keycloak, and App with hot reload)
npm run dev

# View logs in another terminal
npm run logs

# Stop everything
npm run stop
```

**Advantages:**
- ✅ No need to install Rust locally
- ✅ Consistent environment
- ✅ Hot reload works inside Docker
- ✅ All services managed together

### 2. Hybrid Development

Run databases in Docker, but the app locally:

```bash
# Start only PostgreSQL and Keycloak
npm run services

# In another terminal, run the app locally
npm run cargo:dev

# Stop services when done
npm run services:stop
```

**Advantages:**
- ✅ Faster compilation
- ✅ Better for debugging
- ✅ Direct access to cargo commands

### 3. Production Build

#### Using Docker:
```bash
# Build and run production containers
npm run prod

# Or just build the production image
npm run prod:build
```

#### Local Build:
```bash
# Build production binary locally
npm run cargo:build
```

## Script Reference

| Command | Description |
|---------|-------------|
| **Local Development** |
| `npm run cargo:dev` | Run development server locally |
| `npm run cargo:build` | Build production binary locally |
| `npm run cargo:fmt` | Format all Rust code |
| `npm run cargo:fmt:check` | Check code formatting |
| `npm run cargo:lint` | Run Clippy linter |
| `npm run cargo:test` | Run all tests |
| `npm run cargo:check` | Check code compilation |
| `npm run cargo:clean` | Clean build artifacts |
| **Docker Development** |
| `npm run dev` | Start all services in dev mode |
| `npm run dev:build` | Build development image |
| `npm run dev:rebuild` | Rebuild without cache |
| **Docker Production** |
| `npm run prod` | Start production services |
| `npm run prod:build` | Build production image |
| **Service Management** |
| `npm run services` | Start DB and auth only |
| `npm run stop` | Stop all containers |
| `npm run logs` | View app logs |
| **Utilities** |
| `npm run build-css` | Watch TailwindCSS |

## Environment Variables

The system uses these key environment variables:

```bash
# Database
DATABASE_URL=postgres://bitsaccoserver:password@localhost:5432/bitsaccoserver

# Docker build target
BUILD_TARGET=development  # or 'production'

# Logging
LOG_LEVEL=debug
ENVIRONMENT=development
```

## Common Tasks

### Rebuild After Dependency Changes

```bash
# For Docker development
npm run dev:rebuild

# For local development
cargo clean && npm run cargo:dev
```

### Access Services

- **Application**: http://localhost:3000
- **Keycloak Admin**: http://localhost:8080
  - Username: `admin`
  - Password: `admin`

### View Database

```bash
# Connect to PostgreSQL
docker exec -it postgres psql -U bitsaccoserver -d bitsaccoserver

# List tables
\dt

# Exit
\q
```

### Debug Issues

```bash
# Check container status
docker compose ps

# View all logs
docker compose logs

# View specific service logs
docker compose logs postgres
docker compose logs keycloak
docker compose logs app

# Rebuild everything
npm run dev:rebuild
```

## Tips

1. **First Time Setup**: The initial build takes longer due to dependency compilation. Subsequent builds are much faster due to Docker layer caching.

2. **Hot Reload**: When running `npm run dev`, changes to Rust files automatically trigger a rebuild and browser refresh.

3. **Port Conflicts**: If you get port conflicts, ensure no other services are using ports 3000, 3001, 5432, or 8080.

4. **Memory Usage**: The development environment with all services can use 2-4GB RAM. Close other applications if you experience slowness.

5. **Clean Slate**: To completely reset:
   ```bash
   npm run stop
   docker compose down -v  # This removes volumes too
   npm run dev:rebuild
   ```