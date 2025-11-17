#!/bin/bash
# Setup KV namespaces for Rusty Audio Auth Service
# Usage: ./scripts/setup-kv.sh [dev|production]

set -e

ENVIRONMENT=${1:-dev}

echo "📦 Setting up KV namespaces for $ENVIRONMENT..."

# Check if wrangler is installed
if ! command -v wrangler &> /dev/null; then
    echo "❌ Error: wrangler CLI not found"
    echo "Install with: npm install -g wrangler"
    exit 1
fi

# Create KV namespaces
if [ "$ENVIRONMENT" = "production" ]; then
    echo "🌐 Creating PRODUCTION KV namespaces..."

    SESSIONS_ID=$(wrangler kv:namespace create "SESSIONS" --env production | grep "id =" | cut -d'"' -f2)
    USERS_ID=$(wrangler kv:namespace create "USERS" --env production | grep "id =" | cut -d'"' -f2)
    RATE_LIMIT_ID=$(wrangler kv:namespace create "RATE_LIMIT" --env production | grep "id =" | cut -d'"' -f2)

    echo "✅ KV namespaces created:"
    echo "   SESSIONS: $SESSIONS_ID"
    echo "   USERS: $USERS_ID"
    echo "   RATE_LIMIT: $RATE_LIMIT_ID"

    echo ""
    echo "📝 Update wrangler.toml [env.production] with these IDs:"
    echo ""
    echo "[[env.production.kv_namespaces]]"
    echo "binding = \"SESSIONS\""
    echo "id = \"$SESSIONS_ID\""
    echo ""
    echo "[[env.production.kv_namespaces]]"
    echo "binding = \"USERS\""
    echo "id = \"$USERS_ID\""
    echo ""
    echo "[[env.production.kv_namespaces]]"
    echo "binding = \"RATE_LIMIT\""
    echo "id = \"$RATE_LIMIT_ID\""

elif [ "$ENVIRONMENT" = "dev" ]; then
    echo "🧪 Creating DEVELOPMENT KV namespaces..."

    SESSIONS_ID=$(wrangler kv:namespace create "SESSIONS" --env dev | grep "id =" | cut -d'"' -f2)
    USERS_ID=$(wrangler kv:namespace create "USERS" --env dev | grep "id =" | cut -d'"' -f2)
    RATE_LIMIT_ID=$(wrangler kv:namespace create "RATE_LIMIT" --env dev | grep "id =" | cut -d'"' -f2)

    echo "✅ KV namespaces created:"
    echo "   SESSIONS: $SESSIONS_ID"
    echo "   USERS: $USERS_ID"
    echo "   RATE_LIMIT: $RATE_LIMIT_ID"

    echo ""
    echo "📝 Update wrangler.toml [env.dev] with these IDs:"
    echo ""
    echo "[[env.dev.kv_namespaces]]"
    echo "binding = \"SESSIONS\""
    echo "id = \"$SESSIONS_ID\""
    echo ""
    echo "[[env.dev.kv_namespaces]]"
    echo "binding = \"USERS\""
    echo "id = \"$USERS_ID\""
    echo ""
    echo "[[env.dev.kv_namespaces]]"
    echo "binding = \"RATE_LIMIT\""
    echo "id = \"$RATE_LIMIT_ID\""

else
    echo "❌ Invalid environment: $ENVIRONMENT"
    echo "Usage: ./scripts/setup-kv.sh [dev|production]"
    exit 1
fi

echo ""
echo "🎉 KV namespace setup complete!"
