/**
 * POST /api/auth/callback
 * Handle OAuth callback and exchange code for tokens
 */

import { Env, CallbackRequest, CallbackResponse, OAuthTokenResponse, StoredUser } from '../types';
import { getOAuthConfig, getUserProfile } from '../oauth';
import { verifyCodeChallenge } from '../utils/pkce';
import { createAccessToken, createRefreshToken } from '../utils/jwt';
import { storeUser, getUserByEmail, updateUserLastLogin, createSession } from '../utils/storage';

export async function handleCallback(request: Request, env: Env): Promise<Response> {
  try {
    // Parse request body
    const body = await request.json() as CallbackRequest;

    // Validate required fields
    if (!body.code || !body.state || !body.codeVerifier || !body.provider) {
      return new Response(
        JSON.stringify({
          error: 'Bad Request',
          message: 'Missing required fields: code, state, codeVerifier, provider',
          statusCode: 400,
        }),
        {
          status: 400,
          headers: { 'Content-Type': 'application/json' },
        }
      );
    }

    // Get OAuth configuration
    const oauthConfig = getOAuthConfig(body.provider, env);

    // Exchange authorization code for tokens
    const tokenParams = new URLSearchParams({
      grant_type: 'authorization_code',
      code: body.code,
      redirect_uri: env.REDIRECT_URI,
      client_id: oauthConfig.clientId,
      client_secret: oauthConfig.clientSecret,
      code_verifier: body.codeVerifier,
    });

    const tokenResponse = await fetch(oauthConfig.tokenEndpoint, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/x-www-form-urlencoded',
        'Accept': 'application/json',
      },
      body: tokenParams.toString(),
    });

    if (!tokenResponse.ok) {
      const errorText = await tokenResponse.text();
      console.error('Token exchange failed:', errorText);

      return new Response(
        JSON.stringify({
          error: 'OAuth Error',
          message: 'Failed to exchange authorization code for tokens',
          statusCode: 400,
        }),
        {
          status: 400,
          headers: { 'Content-Type': 'application/json' },
        }
      );
    }

    const tokenData = await tokenResponse.json() as OAuthTokenResponse;

    // Fetch user profile
    const userProfile = await getUserProfile(body.provider, tokenData.access_token);

    // Check if user already exists
    let storedUser = await getUserByEmail(userProfile.email, env);

    if (storedUser) {
      // Update last login
      await updateUserLastLogin(storedUser.id, env);
    } else {
      // Create new user
      storedUser = {
        id: userProfile.id,
        email: userProfile.email,
        name: userProfile.name,
        avatar: userProfile.avatar,
        provider: userProfile.provider,
        tier: 'free',
        createdAt: Date.now(),
        lastLoginAt: Date.now(),
      };

      await storeUser(storedUser, env);
    }

    // Create JWT tokens
    const accessToken = await createAccessToken(userProfile, storedUser.tier, env);
    const refreshToken = await createRefreshToken(storedUser.id, env);

    // Store session
    const expiresAt = Date.now() + (tokenData.expires_in * 1000);
    await createSession(
      {
        userId: storedUser.id,
        provider: body.provider,
        accessToken: tokenData.access_token,
        refreshToken: tokenData.refresh_token,
        expiresAt,
      },
      env
    );

    // Return response
    const response: CallbackResponse = {
      accessToken,
      refreshToken,
      expiresIn: 3600, // 1 hour
      user: userProfile,
    };

    return new Response(JSON.stringify(response), {
      status: 200,
      headers: { 'Content-Type': 'application/json' },
    });
  } catch (error) {
    console.error('Callback error:', error);

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
