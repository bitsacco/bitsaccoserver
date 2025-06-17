import { NextRequest, NextResponse } from 'next/server';
import { ServiceRole } from '@bitsaccoserver/types';

// API configuration
const API_BASE_URL = process.env.API_URL || 'http://localhost:4000/api/v1';

// Rate limiting (simple in-memory store for demo)
const rateLimitStore = new Map<string, { count: number; resetTime: number }>();
const RATE_LIMIT = 100; // requests per minute
const RATE_LIMIT_WINDOW = 60 * 1000; // 1 minute in ms

/**
 * Simple rate limiting check
 */
function checkRateLimit(clientIp: string): boolean {
  const now = Date.now();
  const key = clientIp;
  const record = rateLimitStore.get(key);

  if (!record || now > record.resetTime) {
    rateLimitStore.set(key, { count: 1, resetTime: now + RATE_LIMIT_WINDOW });
    return true;
  }

  if (record.count >= RATE_LIMIT) {
    return false;
  }

  record.count++;
  return true;
}

/**
 * Get client IP address
 */
function getClientIp(request: NextRequest): string {
  const forwarded = request.headers.get('x-forwarded-for');
  const realIp = request.headers.get('x-real-ip');
  const clientIp = forwarded ? forwarded.split(',')[0] : realIp;
  return clientIp || 'unknown';
}

/**
 * Decode JWT token without verification (for role checking)
 * In production, this should verify the token signature
 */
function decodeJWT(token: string): { serviceRole?: ServiceRole } | null {
  try {
    const base64Url = token.split('.')[1];
    if (!base64Url) return null;

    const base64 = base64Url.replace(/-/g, '+').replace(/_/g, '/');
    const jsonPayload = decodeURIComponent(
      atob(base64)
        .split('')
        .map((c) => '%' + ('00' + c.charCodeAt(0).toString(16)).slice(-2))
        .join(''),
    );

    const payload = JSON.parse(jsonPayload);

    // Extract service role - check direct serviceRole field first (server-generated tokens)
    let serviceRole: ServiceRole = ServiceRole.MEMBER; // default role

    // If the token already has a serviceRole field (server-generated token), use it
    if (payload.serviceRole) {
      serviceRole = payload.serviceRole;
    } else {
      // For Keycloak tokens, check for realm roles
      if (payload.realm_access?.roles) {
        if (payload.realm_access.roles.includes('system_admin')) {
          serviceRole = ServiceRole.SYSTEM_ADMIN;
        } else if (payload.realm_access.roles.includes('admin')) {
          serviceRole = ServiceRole.ADMIN;
        }
      }
    }

    return { serviceRole };
  } catch (error) {
    console.error('Failed to decode JWT:', error);
    return null;
  }
}

/**
 * Check if user has required admin role for accessing admin endpoints
 */
function hasAdminAccess(request: NextRequest, path: string[]): boolean {
  // Allow non-admin endpoints
  if (!path || path.length === 0 || path[0] !== 'admin') {
    return true;
  }

  const authHeader = request.headers.get('authorization');
  if (!authHeader || !authHeader.startsWith('Bearer ')) {
    return false;
  }

  const token = authHeader.slice(7);
  const payload = decodeJWT(token);

  if (!payload || !payload.serviceRole) {
    return false;
  }

  // Allow SYSTEM_ADMIN and ADMIN roles for admin endpoints
  return (
    payload.serviceRole === ServiceRole.SYSTEM_ADMIN ||
    payload.serviceRole === ServiceRole.ADMIN
  );
}

/**
 * Validate and sanitize the API path
 */
function validateApiPath(path: string[]): string {
  if (!path || path.length === 0) {
    throw new Error('Invalid API path');
  }

  // Remove any potentially dangerous path segments
  const sanitizedPath = path
    .filter(
      (segment) => segment && !segment.includes('..') && !segment.includes('~'),
    )
    .join('/');

  if (!sanitizedPath) {
    throw new Error('Invalid API path after sanitization');
  }

  return sanitizedPath;
}

/**
 * Create proper error response
 */
function createErrorResponse(
  message: string,
  status: number = 500,
): NextResponse {
  return NextResponse.json(
    { error: message, timestamp: new Date().toISOString() },
    {
      status,
      headers: {
        'Content-Type': 'application/json',
        'X-Content-Type-Options': 'nosniff',
        'X-Frame-Options': 'DENY',
        'X-XSS-Protection': '1; mode=block',
      },
    },
  );
}

/**
 * Main API proxy handler with security and best practices
 */
export async function handleApiProxy(
  request: NextRequest,
  { path }: { path?: string[] },
): Promise<NextResponse> {
  try {
    // Rate limiting
    const clientIp = getClientIp(request);
    if (!checkRateLimit(clientIp)) {
      return createErrorResponse('Rate limit exceeded', 429);
    }

    // Validate and sanitize path
    const sanitizedPath = validateApiPath(path || []);

    // Check admin access for admin endpoints
    if (!hasAdminAccess(request, path || [])) {
      return createErrorResponse(
        'Insufficient permissions for admin access',
        403,
      );
    }

    // Construct target URL
    const targetUrl = `${API_BASE_URL}/${sanitizedPath}`;

    // Validate target URL
    try {
      new URL(targetUrl);
    } catch {
      return createErrorResponse('Invalid target URL', 400);
    }

    // Prepare headers for upstream request
    const upstreamHeaders = new Headers();

    // Copy safe headers from the original request
    const safeHeaders = [
      'authorization',
      'content-type',
      'accept',
      'user-agent',
      'accept-language',
      'accept-encoding',
    ];

    safeHeaders.forEach((headerName) => {
      const value = request.headers.get(headerName);
      if (value) {
        upstreamHeaders.set(headerName, value);
      }
    });

    // Add proxy headers
    upstreamHeaders.set('X-Forwarded-For', clientIp);
    upstreamHeaders.set('X-Forwarded-Proto', 'https');
    upstreamHeaders.set('X-Forwarded-Host', request.headers.get('host') || '');

    // Prepare request body
    let body: BodyInit | null = null;
    if (request.method !== 'GET' && request.method !== 'HEAD') {
      try {
        // Clone request to read body
        const clonedRequest = request.clone();
        body = await clonedRequest.arrayBuffer();
      } catch (error) {
        console.error('Failed to read request body:', error);
        return createErrorResponse('Invalid request body', 400);
      }
    }

    // Make upstream request with timeout
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), 30000); // 30 second timeout

    let upstreamResponse: Response;
    try {
      upstreamResponse = await fetch(targetUrl, {
        method: request.method,
        headers: upstreamHeaders,
        body,
        signal: controller.signal,
        // Disable automatic redirect following for security
        redirect: 'manual',
      });
    } catch (error) {
      clearTimeout(timeoutId);
      console.error('Upstream request failed:', error);

      if (error instanceof Error && error.name === 'AbortError') {
        return createErrorResponse('Request timeout', 504);
      }

      return createErrorResponse('Service unavailable', 503);
    } finally {
      clearTimeout(timeoutId);
    }

    // Handle redirects manually for security
    if (upstreamResponse.status >= 300 && upstreamResponse.status < 400) {
      return createErrorResponse('Redirect not allowed', 400);
    }

    // Read response body
    let responseBody: ArrayBuffer;
    try {
      responseBody = await upstreamResponse.arrayBuffer();
    } catch (error) {
      console.error('Failed to read upstream response:', error);
      return createErrorResponse('Failed to process upstream response', 502);
    }

    // Create response with security headers
    const response = new NextResponse(responseBody, {
      status: upstreamResponse.status,
      statusText: upstreamResponse.statusText,
    });

    // Copy safe response headers
    const safeResponseHeaders = [
      'content-type',
      'content-length',
      'cache-control',
      'expires',
      'last-modified',
      'etag',
    ];

    safeResponseHeaders.forEach((headerName) => {
      const value = upstreamResponse.headers.get(headerName);
      if (value) {
        response.headers.set(headerName, value);
      }
    });

    // Add security headers
    response.headers.set('X-Content-Type-Options', 'nosniff');
    response.headers.set('X-Frame-Options', 'DENY');
    response.headers.set('X-XSS-Protection', '1; mode=block');
    response.headers.set('Referrer-Policy', 'strict-origin-when-cross-origin');

    // CORS headers for API responses
    response.headers.set('Access-Control-Allow-Origin', 'same-origin');
    response.headers.set('Access-Control-Allow-Credentials', 'true');

    return response;
  } catch (error) {
    console.error('API proxy error:', error);
    return createErrorResponse('Internal proxy error', 500);
  }
}

/**
 * Handle OPTIONS requests for CORS preflight
 */
export function handleOptionsRequest(): NextResponse {
  return new NextResponse(null, {
    status: 204,
    headers: {
      'Access-Control-Allow-Origin': 'same-origin',
      'Access-Control-Allow-Methods': 'GET, POST, PUT, DELETE, PATCH, OPTIONS',
      'Access-Control-Allow-Headers': 'Content-Type, Authorization, Accept',
      'Access-Control-Allow-Credentials': 'true',
      'Access-Control-Max-Age': '86400', // 24 hours
      'X-Content-Type-Options': 'nosniff',
      'X-Frame-Options': 'DENY',
    },
  });
}
