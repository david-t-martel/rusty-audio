/**
 * GET /api/auth/user
 * Get authenticated user profile
 */

import { Env, UserProfile } from '../types';
import { requireAuth } from '../middleware/auth';
import { getUser } from '../utils/storage';

export async function handleGetUser(request: Request, env: Env): Promise<Response> {
  try {
    // Require authentication
    const authResult = await requireAuth(request, env);

    if ('error' in authResult) {
      return authResult.error;
    }

    const { payload } = authResult;

    // Get full user data from storage
    const user = await getUser(payload.sub, env);

    if (!user) {
      return new Response(
        JSON.stringify({
          error: 'Not Found',
          message: 'User not found',
          statusCode: 404,
        }),
        {
          status: 404,
          headers: { 'Content-Type': 'application/json' },
        }
      );
    }

    // Return user profile
    const userProfile: UserProfile & { tier: string } = {
      id: user.id,
      email: user.email,
      name: user.name,
      avatar: user.avatar,
      provider: user.provider,
      tier: user.tier,
    };

    return new Response(JSON.stringify(userProfile), {
      status: 200,
      headers: { 'Content-Type': 'application/json' },
    });
  } catch (error) {
    console.error('Get user error:', error);

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
