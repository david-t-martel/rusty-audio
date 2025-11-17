/**
 * Google OAuth 2.0 Configuration
 * https://developers.google.com/identity/protocols/oauth2
 */

import { Env, OAuthConfig, UserProfile } from '../types';

export function getGoogleOAuthConfig(env: Env): OAuthConfig {
  return {
    authorizationEndpoint: 'https://accounts.google.com/o/oauth2/v2/auth',
    tokenEndpoint: 'https://oauth2.googleapis.com/token',
    userInfoEndpoint: 'https://www.googleapis.com/oauth2/v2/userinfo',
    scopes: ['openid', 'email', 'profile'],
    clientId: env.GOOGLE_CLIENT_ID,
    clientSecret: env.GOOGLE_CLIENT_SECRET,
  };
}

export async function getGoogleUserProfile(accessToken: string): Promise<UserProfile> {
  const response = await fetch('https://www.googleapis.com/oauth2/v2/userinfo', {
    headers: {
      'Authorization': `Bearer ${accessToken}`,
    },
  });

  if (!response.ok) {
    throw new Error(`Failed to fetch Google user profile: ${response.statusText}`);
  }

  const data = await response.json() as {
    id: string;
    email: string;
    name: string;
    picture?: string;
  };

  return {
    id: `google_${data.id}`,
    email: data.email,
    name: data.name,
    avatar: data.picture,
    provider: 'google',
  };
}
