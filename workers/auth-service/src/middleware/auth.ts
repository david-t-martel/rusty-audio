/**
 * Authentication Middleware
 */

import { Env, JWTPayload } from '../types';
import { verifyToken, extractBearerToken } from '../utils/jwt';

/**
 * Verify JWT from request
 */
export async function verifyRequestAuth(
  request: Request,
  env: Env
): Promise<JWTPayload | null> {
  const authHeader = request.headers.get('Authorization');
  const token = extractBearerToken(authHeader);

  if (!token) {
    return null;
  }

  try {
    const payload = await verifyToken(token, env);
    return payload;
  } catch (error) {
    console.error('Token verification failed:', error);
    return null;
  }
}

/**
 * Require authentication middleware
 */
export async function requireAuth(
  request: Request,
  env: Env
): Promise<{ payload: JWTPayload } | { error: Response }> {
  const payload = await verifyRequestAuth(request, env);

  if (!payload) {
    return {
      error: new Response(
        JSON.stringify({
          error: 'Unauthorized',
          message: 'Valid authentication token required',
          statusCode: 401,
        }),
        {
          status: 401,
          headers: { 'Content-Type': 'application/json' },
        }
      ),
    };
  }

  return { payload };
}
