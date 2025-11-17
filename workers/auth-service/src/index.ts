/**
 * Rusty Audio Auth Service - Main Worker
 * Cloudflare Workers-based OAuth 2.0 authentication service
 */

import { Env } from './types';
import { handleCorsPreFlight, addCorsHeaders } from './middleware/cors';
import { enforceRateLimit } from './middleware/ratelimit';
import { handleInitiate } from './handlers/initiate';
import { handleCallback } from './handlers/callback';
import { handleRefresh } from './handlers/refresh';
import { handleLogout } from './handlers/logout';
import { handleGetUser } from './handlers/user';

export default {
  async fetch(request: Request, env: Env, ctx: ExecutionContext): Promise<Response> {
    const url = new URL(request.url);
    const path = url.pathname;
    const method = request.method;
    const origin = request.headers.get('Origin');

    try {
      // Handle CORS preflight
      if (method === 'OPTIONS') {
        return handleCorsPreFlight(origin, env);
      }

      // Health check endpoint
      if (path === '/health' || path === '/api/health') {
        const response = new Response(
          JSON.stringify({
            status: 'healthy',
            service: 'rusty-audio-auth',
            timestamp: new Date().toISOString(),
          }),
          {
            status: 200,
            headers: { 'Content-Type': 'application/json' },
          }
        );
        return addCorsHeaders(response, origin, env);
      }

      // Route handling with rate limiting
      let response: Response;

      if (path === '/api/auth/initiate' && method === 'POST') {
        // Check rate limit
        const rateLimitResponse = await enforceRateLimit(request, path, env);
        if (rateLimitResponse) {
          return addCorsHeaders(rateLimitResponse, origin, env);
        }

        response = await handleInitiate(request, env);
      } else if (path === '/api/auth/callback' && method === 'POST') {
        // Check rate limit
        const rateLimitResponse = await enforceRateLimit(request, path, env);
        if (rateLimitResponse) {
          return addCorsHeaders(rateLimitResponse, origin, env);
        }

        response = await handleCallback(request, env);
      } else if (path === '/api/auth/refresh' && method === 'POST') {
        // Check rate limit
        const rateLimitResponse = await enforceRateLimit(request, path, env);
        if (rateLimitResponse) {
          return addCorsHeaders(rateLimitResponse, origin, env);
        }

        response = await handleRefresh(request, env);
      } else if (path === '/api/auth/logout' && method === 'POST') {
        // Check rate limit
        const rateLimitResponse = await enforceRateLimit(request, path, env);
        if (rateLimitResponse) {
          return addCorsHeaders(rateLimitResponse, origin, env);
        }

        response = await handleLogout(request, env);
      } else if (path === '/api/auth/user' && method === 'GET') {
        // Check rate limit
        const rateLimitResponse = await enforceRateLimit(request, path, env);
        if (rateLimitResponse) {
          return addCorsHeaders(rateLimitResponse, origin, env);
        }

        response = await handleGetUser(request, env);
      } else {
        // 404 Not Found
        response = new Response(
          JSON.stringify({
            error: 'Not Found',
            message: `Endpoint not found: ${method} ${path}`,
            statusCode: 404,
          }),
          {
            status: 404,
            headers: { 'Content-Type': 'application/json' },
          }
        );
      }

      // Add CORS headers to response
      return addCorsHeaders(response, origin, env);
    } catch (error) {
      console.error('Worker error:', error);

      const errorResponse = new Response(
        JSON.stringify({
          error: 'Internal Server Error',
          message: error instanceof Error ? error.message : 'Unknown error',
          statusCode: 500,
        }),
        {
          status: 500,
          headers: { 'Content-Type': 'application/json' },
        }
      );

      return addCorsHeaders(errorResponse, origin, env);
    }
  },
};
