/**
 * POST /api/auth/logout
 * Logout user and invalidate session
 */

import { Env, LogoutRequest, LogoutResponse } from '../types';
import { verifyToken } from '../utils/jwt';
import { deleteUserSessions } from '../utils/storage';

export async function handleLogout(request: Request, env: Env): Promise<Response> {
  try {
    // Parse request body
    const body = await request.json() as LogoutRequest;

    if (!body.accessToken) {
      return new Response(
        JSON.stringify({
          error: 'Bad Request',
          message: 'Access token is required',
          statusCode: 400,
        }),
        {
          status: 400,
          headers: { 'Content-Type': 'application/json' },
        }
      );
    }

    // Verify token
    let payload;
    try {
      payload = await verifyToken(body.accessToken, env);
    } catch (error) {
      // Even if token is invalid, return success (idempotent logout)
      const response: LogoutResponse = { success: true };
      return new Response(JSON.stringify(response), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      });
    }

    // Delete all user sessions
    await deleteUserSessions(payload.sub, env);

    // Return success
    const response: LogoutResponse = { success: true };

    return new Response(JSON.stringify(response), {
      status: 200,
      headers: { 'Content-Type': 'application/json' },
    });
  } catch (error) {
    console.error('Logout error:', error);

    return new Response(
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
  }
}
