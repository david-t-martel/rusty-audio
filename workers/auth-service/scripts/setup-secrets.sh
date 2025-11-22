#!/bin/bash
# Setup secrets for Rusty Audio Auth Service
# Usage: ./scripts/setup-secrets.sh [dev|production]

set -e

ENVIRONMENT=${1:-dev}

echo "🔐 Setting up secrets for $ENVIRONMENT..."

# Check if wrangler is installed
if ! command -v wrangler &> /dev/null; then
    echo "❌ Error: wrangler CLI not found"
    echo "Install with: npm install -g wrangler"
    exit 1
fi

# Function to set secret
set_secret() {
    local SECRET_NAME=$1
    local SECRET_PROMPT=$2

    echo ""
    read -sp "$SECRET_PROMPT: " SECRET_VALUE
    echo ""

    if [ -n "$SECRET_VALUE" ]; then
        echo "$SECRET_VALUE" | wrangler secret put "$SECRET_NAME" --env "$ENVIRONMENT"
        echo "✅ $SECRET_NAME set"
    else
        echo "⚠️  Skipping $SECRET_NAME (empty)"
    fi
}

echo "🔐 Setting up OAuth provider secrets..."

# Google OAuth
set_secret "GOOGLE_CLIENT_ID" "Enter Google Client ID"
set_secret "GOOGLE_CLIENT_SECRET" "Enter Google Client Secret"

# GitHub OAuth
set_secret "GITHUB_CLIENT_ID" "Enter GitHub Client ID"
set_secret "GITHUB_CLIENT_SECRET" "Enter GitHub Client Secret"

# Microsoft OAuth
set_secret "MICROSOFT_CLIENT_ID" "Enter Microsoft Client ID"
set_secret "MICROSOFT_CLIENT_SECRET" "Enter Microsoft Client Secret"

# JWT Secret
echo ""
echo "🔑 JWT Secret..."
echo "⚠️  Important: Use a strong, random secret (at least 32 characters)"
echo "You can generate one with: openssl rand -base64 32"
set_secret "JWT_SECRET" "Enter JWT Secret"

echo ""
echo "🎉 Secrets setup complete!"
echo ""
echo "📝 Next steps:"
echo "1. Verify secrets with: wrangler secret list --env $ENVIRONMENT"
echo "2. Test deployment with: npm run deploy"
