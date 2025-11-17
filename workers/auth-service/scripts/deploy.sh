#!/bin/bash
# Deployment script for Rusty Audio Auth Service
# Usage: ./scripts/deploy.sh [dev|production]

set -e

ENVIRONMENT=${1:-dev}

echo "🚀 Deploying Rusty Audio Auth Service to $ENVIRONMENT..."

# Check if wrangler is installed
if ! command -v wrangler &> /dev/null; then
    echo "❌ Error: wrangler CLI not found"
    echo "Install with: npm install -g wrangler"
    exit 1
fi

# Navigate to worker directory
cd "$(dirname "$0")/.."

# Install dependencies
echo "📦 Installing dependencies..."
npm ci

# Type check
echo "🔍 Type checking..."
npm run type-check

# Deploy based on environment
if [ "$ENVIRONMENT" = "production" ]; then
    echo "🌐 Deploying to PRODUCTION..."

    # Prompt for confirmation
    read -p "⚠️  Deploy to PRODUCTION? (yes/no): " CONFIRM
    if [ "$CONFIRM" != "yes" ]; then
        echo "❌ Deployment cancelled"
        exit 1
    fi

    wrangler deploy --env production

    echo "✅ Production deployment complete"
    echo "🔗 URL: https://api.rusty-audio.com"

    # Health check
    echo "🏥 Running health check..."
    sleep 5
    if curl -f https://api.rusty-audio.com/health > /dev/null 2>&1; then
        echo "✅ Health check passed"
    else
        echo "⚠️  Health check failed - please verify deployment"
    fi

elif [ "$ENVIRONMENT" = "dev" ]; then
    echo "🧪 Deploying to DEVELOPMENT..."

    wrangler deploy --env dev

    echo "✅ Development deployment complete"
    echo "🔗 URL: Check wrangler output for worker URL"

else
    echo "❌ Invalid environment: $ENVIRONMENT"
    echo "Usage: ./scripts/deploy.sh [dev|production]"
    exit 1
fi

echo "🎉 Deployment successful!"
