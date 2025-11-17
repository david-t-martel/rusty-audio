/**
 * GitHub OAuth 2.0 Configuration
 * https://docs.github.com/en/apps/oauth-apps/building-oauth-apps
 */

import { Env, OAuthConfig, UserProfile } from '../types';

export function getGitHubOAuthConfig(env: Env): OAuthConfig {
  return {
    authorizationEndpoint: 'https://github.com/login/oauth/authorize',
    tokenEndpoint: 'https://github.com/login/oauth/access_token',
    userInfoEndpoint: 'https://api.github.com/user',
    scopes: ['read:user', 'user:email'],
    clientId: env.GITHUB_CLIENT_ID,
    clientSecret: env.GITHUB_CLIENT_SECRET,
  };
}

export async function getGitHubUserProfile(accessToken: string): Promise<UserProfile> {
  // Fetch user profile
  const userResponse = await fetch('https://api.github.com/user', {
    headers: {
      'Authorization': `Bearer ${accessToken}`,
      'User-Agent': 'Rusty-Audio-Auth',
      'Accept': 'application/vnd.github.v3+json',
    },
  });

  if (!userResponse.ok) {
    throw new Error(`Failed to fetch GitHub user profile: ${userResponse.statusText}`);
  }

  const userData = await userResponse.json() as {
    id: number;
    login: string;
    name: string | null;
    email: string | null;
    avatar_url: string;
  };

  // If email is not public, fetch from emails endpoint
  let email = userData.email;
  if (!email) {
    const emailsResponse = await fetch('https://api.github.com/user/emails', {
      headers: {
        'Authorization': `Bearer ${accessToken}`,
        'User-Agent': 'Rusty-Audio-Auth',
        'Accept': 'application/vnd.github.v3+json',
      },
    });

    if (emailsResponse.ok) {
      const emails = await emailsResponse.json() as Array<{
        email: string;
        primary: boolean;
        verified: boolean;
      }>;
      const primaryEmail = emails.find(e => e.primary && e.verified);
      email = primaryEmail?.email || emails[0]?.email || '';
    }
  }

  return {
    id: `github_${userData.id}`,
    email: email || '',
    name: userData.name || userData.login,
    avatar: userData.avatar_url,
    provider: 'github',
  };
}
