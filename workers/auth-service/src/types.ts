/**
 * Type definitions for Rusty Audio Auth Service
 */

// Environment bindings
export interface Env {
  // KV Namespaces
  SESSIONS: KVNamespace;
  USERS: KVNamespace;
  RATE_LIMIT: KVNamespace;

  // OAuth Provider Credentials
  GOOGLE_CLIENT_ID: string;
  GOOGLE_CLIENT_SECRET: string;
  GITHUB_CLIENT_ID: string;
  GITHUB_CLIENT_SECRET: string;
  MICROSOFT_CLIENT_ID: string;
  MICROSOFT_CLIENT_SECRET: string;

  // JWT Secret
  JWT_SECRET: string;

  // Application Configuration
  APP_URL: string;
  REDIRECT_URI: string;
  ENVIRONMENT: 'development' | 'production';
}

// OAuth Providers
export type OAuthProvider = 'google' | 'github' | 'microsoft';

// User Profile
export interface UserProfile {
  id: string;
  email: string;
  name: string;
  avatar?: string;
  provider: OAuthProvider;
}

// User Tier
export type UserTier = 'free' | 'premium';

// Stored User (in KV)
export interface StoredUser {
  id: string;
  email: string;
  name: string;
  avatar?: string;
  provider: OAuthProvider;
  tier: UserTier;
  createdAt: number;
  lastLoginAt: number;
}

// Session Data (in KV)
export interface StoredSession {
  userId: string;
  provider: OAuthProvider;
  accessToken: string;
  refreshToken?: string;
  expiresAt: number;
  createdAt: number;
}

// OAuth Configuration
export interface OAuthConfig {
  authorizationEndpoint: string;
  tokenEndpoint: string;
  userInfoEndpoint: string;
  scopes: string[];
  clientId: string;
  clientSecret: string;
}

// PKCE State
export interface PKCEState {
  state: string;
  codeVerifier: string;
  codeChallenge: string;
  provider: OAuthProvider;
  createdAt: number;
}

// JWT Payload
export interface JWTPayload {
  sub: string; // User ID
  email: string;
  name: string;
  provider: OAuthProvider;
  tier: UserTier;
  iat: number;
  exp: number;
}

// API Request/Response Types

export interface InitiateAuthRequest {
  provider: OAuthProvider;
}

export interface InitiateAuthResponse {
  authUrl: string;
  state: string;
  codeVerifier: string;
}

export interface CallbackRequest {
  code: string;
  state: string;
  codeVerifier: string;
  provider: OAuthProvider;
}

export interface CallbackResponse {
  accessToken: string;
  refreshToken?: string;
  expiresIn: number;
  user: UserProfile;
}

export interface RefreshRequest {
  refreshToken: string;
}

export interface RefreshResponse {
  accessToken: string;
  expiresIn: number;
}

export interface LogoutRequest {
  accessToken: string;
}

export interface LogoutResponse {
  success: boolean;
}

// OAuth Token Response
export interface OAuthTokenResponse {
  access_token: string;
  refresh_token?: string;
  expires_in: number;
  token_type: string;
  scope?: string;
}

// Error Response
export interface ErrorResponse {
  error: string;
  message: string;
  statusCode: number;
}

// Rate Limit Configuration
export interface RateLimitConfig {
  requests: number;
  window: number; // seconds
}

// Rate Limit Entry (in KV)
export interface RateLimitEntry {
  count: number;
  resetAt: number;
}
