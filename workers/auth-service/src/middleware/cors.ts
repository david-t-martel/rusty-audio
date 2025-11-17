/**
 * CORS Middleware
 */

import { Env } from '../types';

const ALLOWED_ORIGINS = [
  'http://localhost:8080',
  'http://localhost:3000',
  'https://rusty-audio.pages.dev',
  'https://rusty-audio.com',
];

/**
 * Get CORS headers
 */
export function getCorsHeaders(origin: string | null, env: Env): Record<string, string> {
  // Check if origin is allowed
  const allowedOrigin = origin && ALLOWED_ORIGINS.includes(origin) ? origin : ALLOWED_ORIGINS[0];

  return {
    'Access-Control-Allow-Origin': allowedOrigin,
    'Access-Control-Allow-Methods': 'GET, POST, OPTIONS',
    'Access-Control-Allow-Headers': 'Content-Type, Authorization',
    'Access-Control-Max-Age': '86400', // 24 hours
    'Access-Control-Allow-Credentials': 'true',
  };
}

/**
 * Handle CORS preflight request
 */
export function handleCorsPreFlight(origin: string | null, env: Env): Response {
  return new Response(null, {
    status: 204,
    headers: getCorsHeaders(origin, env),
  });
}

/**
 * Add CORS headers to response
 */
export function addCorsHeaders(response: Response, origin: string | null, env: Env): Response {
  const corsHeaders = getCorsHeaders(origin, env);

  // Create new response with CORS headers
  const newResponse = new Response(response.body, {
    status: response.status,
    statusText: response.statusText,
    headers: new Headers(response.headers),
  });

  // Add CORS headers
  Object.entries(corsHeaders).forEach(([key, value]) => {
    newResponse.headers.set(key, value);
  });

  return newResponse;
}
