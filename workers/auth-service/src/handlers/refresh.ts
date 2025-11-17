/**
 * POST /api/auth/refresh
 * Refresh access token using refresh token
 */

import { Env, RefreshRequest, RefreshResponse } from '../types';
import { verifyToken, createAccessToken } from '../utils/jwt';
import { getUser } from '../utils/storage';

export async function handleRefresh(request: Request, env: Env): Promise<Response> {
  try {
    // Parse request body
    const body = await request.json() as RefreshRequest;

    if (!body.refreshToken) {
      return new Response(
        JSON.stringify({
          error: 'Bad Request',
          message: 'Refresh token is required',
          statusCode: 400,
        }),
        {
          status: 400,
          headers: { 'Content-Type': 'application/json' },
        }
      );
    }

    // Verify refresh token
    let payload;
    try {
      payload = await verifyToken(body.refreshToken, env);
    } catch (error) {
      return new Response(
        JSON.stringify({
          error: 'Unauthorized',
          message: 'Invalid or expired refresh token',
          statusCode: 401,
        }),
        {
          status: 401,
          headers: { 'Content-Type': 'application/json' },
        }
      );
    }

    // Get user data
    const user = await getUser(payload.sub, env);

    if (!user) {
      return new Response(
        JSON.stringify({
          error: 'Unauthorized',
          message: 'User not found',
          statusCode: 401,
        }),
        {
          status: 401,
          headers: { 'Content-Type': 'application/json' },
        }
      );
    }

    // Create new access token
    const accessToken = await createAccessToken(
      {
        id: user.id,
        email: user.email,
        name: user.name,
        avatar: user.avatar,
        provider: user.provider,
      },
      user.tier,
      env
    );

    // Return new access token
    const response: RefreshResponse = {
      accessToken,
      expiresIn: 3600, // 1 hour
    };

    return new Response(JSON.stringify(response), {
      status: 200,
      headers: { 'Content-Type': 'application/json' },
    });
  } catch (error) {
    console.error('Refresh token error:', error);

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
