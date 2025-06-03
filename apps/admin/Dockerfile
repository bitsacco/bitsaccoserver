FROM node:lts-alpine AS base

# Development stage with hot reload
FROM base AS development

RUN apk update
RUN apk add --no-cache libc6-compat

WORKDIR /app

# Install dependencies
COPY package.json yarn.lock ./
COPY apps/admin/package.json ./apps/admin/package.json
COPY packages/ ./packages/
RUN yarn install

# Install dependencies in the app directory
WORKDIR /app/apps/admin
RUN yarn install

# Don't run as root in development (but keep current user for volume mounts)
RUN addgroup --system --gid 1001 nodejs
RUN adduser --system --uid 1001 nextjs
RUN chown -R nextjs:nodejs /app
# USER nextjs  # Commented out for development to match host user

# Expose port
EXPOSE 3001

# Start development server with hot reload
CMD ["sh", "-c", "yarn dev"]

FROM base AS builder

RUN apk update
RUN apk add --no-cache libc6-compat

WORKDIR /app
RUN yarn global add turbo

COPY . .

RUN turbo prune --scope="@bitsacco/admin" --docker

# Add lockfile and package.json's of isolated subworkspace
FROM base AS installer

RUN apk update
RUN apk add --no-cache libc6-compat

WORKDIR /app

# First install the dependencies (as they change less often)
COPY --from=builder /app/out/json/ .
RUN yarn install

# Build the project
COPY --from=builder /app/out/full/ .

# Set environment variables for build
ENV NODE_ENV=production

RUN yarn turbo build --filter=@bitsacco/admin...

FROM base AS runner

WORKDIR /app

# Don't run production as root
RUN addgroup --system --gid 1001 nodejs
RUN adduser --system --uid 1001 nextjs
USER nextjs

# Automatically leverage output traces to reduce image size
# https://nextjs.org/docs/advanced-features/output-file-tracing
COPY --from=installer --chown=nextjs:nodejs /app/apps/admin/.next/standalone ./
COPY --from=installer --chown=nextjs:nodejs /app/apps/admin/.next/static ./apps/admin/.next/static
COPY --from=installer --chown=nextjs:nodejs /app/apps/admin/public ./apps/admin/public

ENV NODE_ENV=production
ENV PORT=3001

EXPOSE 3001

CMD ["node", "apps/admin/server.js"]
