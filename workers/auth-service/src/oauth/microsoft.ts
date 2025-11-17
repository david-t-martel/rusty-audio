/**
 * Microsoft OAuth 2.0 Configuration
 * https://learn.microsoft.com/en-us/azure/active-directory/develop/v2-oauth2-auth-code-flow
 */

import { Env, OAuthConfig, UserProfile } from '../types';

export function getMicrosoftOAuthConfig(env: Env): OAuthConfig {
  return {
    authorizationEndpoint: 'https://login.microsoftonline.com/common/oauth2/v2.0/authorize',
    tokenEndpoint: 'https://login.microsoftonline.com/common/oauth2/v2.0/token',
    userInfoEndpoint: 'https://graph.microsoft.com/v1.0/me',
    scopes: ['openid', 'email', 'profile', 'User.Read'],
    clientId: env.MICROSOFT_CLIENT_ID,
    clientSecret: env.MICROSOFT_CLIENT_SECRET,
  };
}

export async function getMicrosoftUserProfile(accessToken: string): Promise<UserProfile> {
  const response = await fetch('https://graph.microsoft.com/v1.0/me', {
    headers: {
      'Authorization': `Bearer ${accessToken}`,
    },
  });

  if (!response.ok) {
    throw new Error(`Failed to fetch Microsoft user profile: ${response.statusText}`);
  }

  const data = await response.json() as {
    id: string;
    userPrincipalName: string;
    displayName: string;
    mail?: string;
  };

  return {
    id: `microsoft_${data.id}`,
    email: data.mail || data.userPrincipalName,
    name: data.displayName,
    provider: 'microsoft',
  };
}
