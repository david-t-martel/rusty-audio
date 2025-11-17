/**
 * OAuth Provider Registry
 */

import { Env, OAuthConfig, OAuthProvider, UserProfile } from '../types';
import { getGoogleOAuthConfig, getGoogleUserProfile } from './google';
import { getGitHubOAuthConfig, getGitHubUserProfile } from './github';
import { getMicrosoftOAuthConfig, getMicrosoftUserProfile } from './microsoft';

export function getOAuthConfig(provider: OAuthProvider, env: Env): OAuthConfig {
  switch (provider) {
    case 'google':
      return getGoogleOAuthConfig(env);
    case 'github':
      return getGitHubOAuthConfig(env);
    case 'microsoft':
      return getMicrosoftOAuthConfig(env);
    default:
      throw new Error(`Unsupported OAuth provider: ${provider}`);
  }
}

export async function getUserProfile(
  provider: OAuthProvider,
  accessToken: string
): Promise<UserProfile> {
  switch (provider) {
    case 'google':
      return getGoogleUserProfile(accessToken);
    case 'github':
      return getGitHubUserProfile(accessToken);
    case 'microsoft':
      return getMicrosoftUserProfile(accessToken);
    default:
      throw new Error(`Unsupported OAuth provider: ${provider}`);
  }
}

export { getGoogleOAuthConfig } from './google';
export { getGitHubOAuthConfig } from './github';
export { getMicrosoftOAuthConfig } from './microsoft';
