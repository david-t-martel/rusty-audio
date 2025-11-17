/**
 * POST /api/auth/initiate
 * Initiate OAuth flow
 */

import { Env, InitiateAuthRequest, InitiateAuthResponse, OAuthProvider } from '../types';
import { getOAuthConfig } from '../oauth';
import { createPKCEState } from '../utils/pkce';

export async function handleInitiate(request: Request, env: Env): Promise<Response> {
  try {
    // Parse request body
    const body = await request.json() as InitiateAuthRequest;

    if (!body.provider) {
      return new Response(
        JSON.stringify({
          error: 'Bad Request',
          message: 'Provider is required',
          statusCode: 400,
        }),
        {
          status: 400,
          headers: { 'Content-Type': 'application/json' },
        }
      );
    }

    // Validate provider
    const validProviders: OAuthProvider[] = ['google', 'github', 'microsoft'];
    if (!validProviders.includes(body.provider)) {
      return new Response(
        JSON.stringify({
          error: 'Bad Request',
          message: `Invalid provider. Must be one of: ${validProviders.join(', ')}`,
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

    // Generate PKCE parameters
    const pkceState = await createPKCEState(body.provider);

    // Build authorization URL
    const authUrl = new URL(oauthConfig.authorizationEndpoint);
    authUrl.searchParams.set('client_id', oauthConfig.clientId);
    authUrl.searchParams.set('redirect_uri', env.REDIRECT_URI);
    authUrl.searchParams.set('response_type', 'code');
    authUrl.searchParams.set('scope', oauthConfig.scopes.join(' '));
    authUrl.searchParams.set('state', pkceState.state);
    authUrl.searchParams.set('code_challenge', pkceState.codeChallenge);
    authUrl.searchParams.set('code_challenge_method', 'S256');

    // Provider-specific parameters
    if (body.provider === 'google') {
      authUrl.searchParams.set('access_type', 'offline');
      authUrl.searchParams.set('prompt', 'consent');
    } else if (body.provider === 'microsoft') {
      authUrl.searchParams.set('response_mode', 'query');
    }

    // Return auth URL and code verifier (client stores verifier)
    const response: InitiateAuthResponse = {
      authUrl: authUrl.toString(),
      state: pkceState.state,
      codeVerifier: pkceState.codeVerifier,
    };

    return new Response(JSON.stringify(response), {
      status: 200,
      headers: { 'Content-Type': 'application/json' },
    });
  } catch (error) {
    console.error('Initiate auth error:', error);

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
