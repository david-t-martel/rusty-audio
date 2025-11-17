/**
 * JWT (JSON Web Token) Utilities
 * Using the jose library for secure JWT operations
 */

import { SignJWT, jwtVerify } from 'jose';
import { Env, JWTPayload, UserProfile, UserTier } from '../types';

/**
 * Get JWT signing key
 */
function getJWTKey(env: Env): Uint8Array {
  return new TextEncoder().encode(env.JWT_SECRET);
}

/**
 * Create JWT access token
 */
export async function createAccessToken(
  user: UserProfile,
  tier: UserTier,
  env: Env
): Promise<string> {
  const key = getJWTKey(env);
  const now = Math.floor(Date.now() / 1000);

  const payload: Omit<JWTPayload, 'iat' | 'exp'> = {
    sub: user.id,
    email: user.email,
    name: user.name,
    provider: user.provider,
    tier,
  };

  const jwt = await new SignJWT(payload as any)
    .setProtectedHeader({ alg: 'HS256', typ: 'JWT' })
    .setIssuedAt(now)
    .setExpirationTime(now + 3600) // 1 hour
    .setIssuer('rusty-audio-auth')
    .setAudience('rusty-audio-app')
    .sign(key);

  return jwt;
}

/**
 * Create JWT refresh token (longer expiration)
 */
export async function createRefreshToken(
  userId: string,
  env: Env
): Promise<string> {
  const key = getJWTKey(env);
  const now = Math.floor(Date.now() / 1000);

  const jwt = await new SignJWT({ sub: userId })
    .setProtectedHeader({ alg: 'HS256', typ: 'JWT' })
    .setIssuedAt(now)
    .setExpirationTime(now + 2592000) // 30 days
    .setIssuer('rusty-audio-auth')
    .setAudience('rusty-audio-app')
    .sign(key);

  return jwt;
}

/**
 * Verify and decode JWT token
 */
export async function verifyToken(
  token: string,
  env: Env
): Promise<JWTPayload> {
  try {
    const key = getJWTKey(env);

    const { payload } = await jwtVerify(token, key, {
      issuer: 'rusty-audio-auth',
      audience: 'rusty-audio-app',
    });

    return payload as JWTPayload;
  } catch (error) {
    throw new Error(`Invalid token: ${error instanceof Error ? error.message : 'Unknown error'}`);
  }
}

/**
 * Extract token from Authorization header
 */
export function extractBearerToken(authHeader: string | null): string | null {
  if (!authHeader) {
    return null;
  }

  const parts = authHeader.split(' ');
  if (parts.length !== 2 || parts[0] !== 'Bearer') {
    return null;
  }

  return parts[1];
}

/**
 * Check if token is expired
 */
export function isTokenExpired(payload: JWTPayload): boolean {
  const now = Math.floor(Date.now() / 1000);
  return payload.exp < now;
}

/**
 * Get token expiration time in seconds from now
 */
export function getTokenExpiresIn(payload: JWTPayload): number {
  const now = Math.floor(Date.now() / 1000);
  return Math.max(0, payload.exp - now);
}
