/**
 * KV Storage Utilities
 */

import { Env, StoredUser, StoredSession } from '../types';
import { generateUUID } from './crypto';

// TTL Constants (in seconds)
const SESSION_TTL = 2592000; // 30 days
const USER_TTL = 31536000; // 1 year (refresh on login)

/**
 * Store user in KV
 */
export async function storeUser(user: StoredUser, env: Env): Promise<void> {
  const key = `user:${user.id}`;
  await env.USERS.put(key, JSON.stringify(user), {
    expirationTtl: USER_TTL,
  });

  // Also store email -> userId mapping for lookup
  const emailKey = `email:${user.email.toLowerCase()}`;
  await env.USERS.put(emailKey, user.id, {
    expirationTtl: USER_TTL,
  });
}

/**
 * Get user from KV by ID
 */
export async function getUser(userId: string, env: Env): Promise<StoredUser | null> {
  const key = `user:${userId}`;
  const data = await env.USERS.get(key);

  if (!data) {
    return null;
  }

  return JSON.parse(data) as StoredUser;
}

/**
 * Get user by email
 */
export async function getUserByEmail(email: string, env: Env): Promise<StoredUser | null> {
  const emailKey = `email:${email.toLowerCase()}`;
  const userId = await env.USERS.get(emailKey);

  if (!userId) {
    return null;
  }

  return getUser(userId, env);
}

/**
 * Update user last login time
 */
export async function updateUserLastLogin(userId: string, env: Env): Promise<void> {
  const user = await getUser(userId, env);
  if (user) {
    user.lastLoginAt = Date.now();
    await storeUser(user, env);
  }
}

/**
 * Create and store session
 */
export async function createSession(
  session: Omit<StoredSession, 'createdAt'>,
  env: Env
): Promise<string> {
  const sessionId = generateUUID();
  const sessionData: StoredSession = {
    ...session,
    createdAt: Date.now(),
  };

  const key = `session:${sessionId}`;
  await env.SESSIONS.put(key, JSON.stringify(sessionData), {
    expirationTtl: SESSION_TTL,
  });

  // Store userId -> sessionId mapping for quick lookup
  const userSessionKey = `user_session:${session.userId}`;
  await env.SESSIONS.put(userSessionKey, sessionId, {
    expirationTtl: SESSION_TTL,
  });

  return sessionId;
}

/**
 * Get session from KV
 */
export async function getSession(sessionId: string, env: Env): Promise<StoredSession | null> {
  const key = `session:${sessionId}`;
  const data = await env.SESSIONS.get(key);

  if (!data) {
    return null;
  }

  return JSON.parse(data) as StoredSession;
}

/**
 * Get session by user ID
 */
export async function getSessionByUserId(userId: string, env: Env): Promise<StoredSession | null> {
  const userSessionKey = `user_session:${userId}`;
  const sessionId = await env.SESSIONS.get(userSessionKey);

  if (!sessionId) {
    return null;
  }

  return getSession(sessionId, env);
}

/**
 * Delete session
 */
export async function deleteSession(sessionId: string, env: Env): Promise<void> {
  const session = await getSession(sessionId, env);

  if (session) {
    // Delete session
    const key = `session:${sessionId}`;
    await env.SESSIONS.delete(key);

    // Delete user session mapping
    const userSessionKey = `user_session:${session.userId}`;
    await env.SESSIONS.delete(userSessionKey);
  }
}

/**
 * Delete all sessions for a user
 */
export async function deleteUserSessions(userId: string, env: Env): Promise<void> {
  const userSessionKey = `user_session:${userId}`;
  const sessionId = await env.SESSIONS.get(userSessionKey);

  if (sessionId) {
    await deleteSession(sessionId, env);
  }
}

/**
 * Update session tokens
 */
export async function updateSessionTokens(
  sessionId: string,
  accessToken: string,
  refreshToken: string | undefined,
  expiresAt: number,
  env: Env
): Promise<void> {
  const session = await getSession(sessionId, env);

  if (session) {
    session.accessToken = accessToken;
    if (refreshToken) {
      session.refreshToken = refreshToken;
    }
    session.expiresAt = expiresAt;

    const key = `session:${sessionId}`;
    await env.SESSIONS.put(key, JSON.stringify(session), {
      expirationTtl: SESSION_TTL,
    });
  }
}
