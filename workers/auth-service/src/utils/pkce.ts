/**
 * PKCE (Proof Key for Code Exchange) Utilities
 * RFC 7636: https://tools.ietf.org/html/rfc7636
 */

import { generateRandomString, base64UrlEncode, sha256 } from './crypto';
import { PKCEState } from '../types';

/**
 * Generate PKCE code verifier
 * Must be 43-128 characters long
 */
export function generateCodeVerifier(): string {
  return generateRandomString(64); // 128 hex characters = 64 bytes
}

/**
 * Generate PKCE code challenge from verifier
 * Uses S256 method (SHA-256 hash)
 */
export async function generateCodeChallenge(verifier: string): Promise<string> {
  const hash = await sha256(verifier);
  return base64UrlEncode(hash);
}

/**
 * Verify PKCE code challenge
 */
export async function verifyCodeChallenge(
  verifier: string,
  challenge: string
): Promise<boolean> {
  const computedChallenge = await generateCodeChallenge(verifier);
  return computedChallenge === challenge;
}

/**
 * Generate state parameter for OAuth flow
 */
export function generateState(): string {
  return generateRandomString(32);
}

/**
 * Create complete PKCE state object
 */
export async function createPKCEState(provider: string): Promise<PKCEState> {
  const codeVerifier = generateCodeVerifier();
  const codeChallenge = await generateCodeChallenge(codeVerifier);
  const state = generateState();

  return {
    state,
    codeVerifier,
    codeChallenge,
    provider: provider as any,
    createdAt: Date.now(),
  };
}

/**
 * Validate PKCE state (check expiration)
 */
export function isValidPKCEState(pkceState: PKCEState, maxAgeMs: number = 600000): boolean {
  const now = Date.now();
  const age = now - pkceState.createdAt;
  return age <= maxAgeMs; // Default 10 minutes
}
