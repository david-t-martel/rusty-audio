/**
 * Rate Limiting Middleware
 */

import { Env, RateLimitConfig, RateLimitEntry } from '../types';

// Rate limit configurations per endpoint
const RATE_LIMITS: Record<string, RateLimitConfig> = {
  '/api/auth/initiate': { requests: 10, window: 60 }, // 10 requests per minute
  '/api/auth/callback': { requests: 5, window: 60 },  // 5 requests per minute
  '/api/auth/refresh': { requests: 20, window: 60 },  // 20 requests per minute
  '/api/auth/logout': { requests: 10, window: 60 },   // 10 requests per minute
  '/api/auth/user': { requests: 30, window: 60 },     // 30 requests per minute
};

/**
 * Get client identifier (IP address)
 */
function getClientId(request: Request): string {
  // Try to get real IP from Cloudflare headers
  const cfConnectingIp = request.headers.get('CF-Connecting-IP');
  if (cfConnectingIp) {
    return cfConnectingIp;
  }

  // Fallback to X-Forwarded-For
  const forwardedFor = request.headers.get('X-Forwarded-For');
  if (forwardedFor) {
    return forwardedFor.split(',')[0].trim();
  }

  // Last resort
  return 'unknown';
}

/**
 * Generate rate limit key
 */
function getRateLimitKey(clientId: string, path: string): string {
  return `ratelimit:${path}:${clientId}`;
}

/**
 * Check rate limit
 */
export async function checkRateLimit(
  request: Request,
  path: string,
  env: Env
): Promise<{ allowed: boolean; retryAfter?: number }> {
  const config = RATE_LIMITS[path];
  if (!config) {
    // No rate limit configured, allow
    return { allowed: true };
  }

  const clientId = getClientId(request);
  const key = getRateLimitKey(clientId, path);
  const now = Date.now();

  // Get current rate limit entry
  const entryData = await env.RATE_LIMIT.get(key);
  let entry: RateLimitEntry;

  if (entryData) {
    entry = JSON.parse(entryData) as RateLimitEntry;

    // Check if window has expired
    if (now >= entry.resetAt) {
      // Reset window
      entry = {
        count: 1,
        resetAt: now + config.window * 1000,
      };
    } else {
      // Within window, increment count
      entry.count += 1;

      if (entry.count > config.requests) {
        // Rate limit exceeded
        const retryAfter = Math.ceil((entry.resetAt - now) / 1000);
        return { allowed: false, retryAfter };
      }
    }
  } else {
    // First request in window
    entry = {
      count: 1,
      resetAt: now + config.window * 1000,
    };
  }

  // Update entry
  const ttl = Math.ceil((entry.resetAt - now) / 1000);
  await env.RATE_LIMIT.put(key, JSON.stringify(entry), {
    expirationTtl: ttl,
  });

  return { allowed: true };
}

/**
 * Rate limit middleware
 */
export async function enforceRateLimit(
  request: Request,
  path: string,
  env: Env
): Promise<Response | null> {
  const { allowed, retryAfter } = await checkRateLimit(request, path, env);

  if (!allowed) {
    return new Response(
      JSON.stringify({
        error: 'Too Many Requests',
        message: 'Rate limit exceeded',
        statusCode: 429,
        retryAfter,
      }),
      {
        status: 429,
        headers: {
          'Content-Type': 'application/json',
          'Retry-After': retryAfter?.toString() || '60',
        },
      }
    );
  }

  return null;
}
