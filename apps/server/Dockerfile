# Build stage
FROM node:lts-alpine AS base

# Development stage with hot reload
FROM base AS development

WORKDIR /workspace

# Install curl for health checks
RUN apk add --no-cache curl

# Copy workspace root files
COPY package.json yarn.lock ./

# Copy packages for dependency resolution
COPY packages/types/package.json ./packages/types/package.json

# Copy bitsacco server package.json for dependency resolution
COPY apps/server/package.json ./apps/server/package.json

# Install all dependencies (including dev dependencies)
RUN yarn install

# Copy and build packages
COPY packages ./packages
RUN yarn workspace @bitsaccoserver/types build

# Create non-root user
RUN addgroup -g 1001 -S nodejs
RUN adduser -S nestjs -u 1001

# Change ownership of workspace
RUN chown -R nestjs:nodejs /workspace
USER nestjs

# Expose port
EXPOSE 4000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:4000/api/v1/ || exit 1

# Working directory for bitsacco server
WORKDIR /workspace/apps/server

# Start in development mode with hot reload
CMD ["yarn", "start:dev"]

FROM base AS builder

WORKDIR /workspace

# Copy workspace root files
COPY package.json yarn.lock ./

# Copy packages for dependency resolution
COPY packages/types/package.json ./packages/types/package.json

# Copy bitsacco server package.json for dependency resolution
COPY apps/server/package.json ./apps/server/package.json

# Install dependencies for the entire workspace
RUN yarn install --frozen-lockfile

# Copy and build packages
COPY packages ./packages
RUN yarn workspace @bitsaccoserver/types build

# Copy bitsacco server source code
COPY apps/server ./apps/server

# Build the bitsacco server
WORKDIR /workspace/apps/server
RUN yarn build

# Production stage
FROM base AS production

WORKDIR /app

# Install curl for health checks
RUN apk add --no-cache curl

# Create non-root user
RUN addgroup -g 1001 -S nodejs
RUN adduser -S nestjs -u 1001

# Copy only the built application and necessary files
COPY --from=builder --chown=nestjs:nodejs /workspace/apps/server/dist ./dist
COPY --from=builder --chown=nestjs:nodejs /workspace/apps/server/package.json ./package.json

# Install only production dependencies directly in production stage
RUN npm install --omit=dev

# Expose port
EXPOSE 4000

# Switch to non-root user
USER nestjs

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:4000/api/v1/ || exit 1

# Start the application
CMD ["node", "dist/main.js"]