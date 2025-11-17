#!/bin/bash
# Feature flag deployment script for gradual rollout of multithreading

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Configuration
CLOUDFLARE_ACCOUNT_ID="${CLOUDFLARE_ACCOUNT_ID:-}"
CLOUDFLARE_API_TOKEN="${CLOUDFLARE_API_TOKEN:-}"
KV_NAMESPACE_ID="${KV_NAMESPACE_ID:-}"

# Feature flag percentages (0-100)
THREADING_ROLLOUT_PERCENTAGE="${THREADING_ROLLOUT_PERCENTAGE:-0}"

usage() {
    cat << USAGE
Feature Flag Deployment Script

Usage: $0 [OPTIONS]

Options:
    -p, --percentage <0-100>    Percentage of users to enable threading (default: 0)
    -e, --enable                Enable threading for all users (100%)
    -d, --disable               Disable threading for all users (0%)
    -s, --status                Show current rollout status
    -h, --help                  Show this help message

Environment Variables:
    CLOUDFLARE_ACCOUNT_ID       Cloudflare account ID
    CLOUDFLARE_API_TOKEN        Cloudflare API token with KV edit permissions
    KV_NAMESPACE_ID             Cloudflare KV namespace ID for feature flags

Examples:
    # Enable threading for 10% of users
    $0 --percentage 10

    # Enable for all users
    $0 --enable

    # Disable for all users
    $0 --disable

    # Check current status
    $0 --status
USAGE
}

check_dependencies() {
    local deps=("curl" "jq")
    for cmd in "${deps[@]}"; do
        if ! command -v "$cmd" &> /dev/null; then
            echo "Error: Required command '$cmd' not found"
            exit 1
        fi
    done
}

check_env() {
    if [[ -z "$CLOUDFLARE_ACCOUNT_ID" ]] || [[ -z "$CLOUDFLARE_API_TOKEN" ]] || [[ -z "$KV_NAMESPACE_ID" ]]; then
        echo "Error: Missing required environment variables"
        echo "Please set: CLOUDFLARE_ACCOUNT_ID, CLOUDFLARE_API_TOKEN, KV_NAMESPACE_ID"
        exit 1
    fi
}

set_feature_flag() {
    local percentage=$1
    
    if [[ $percentage -lt 0 ]] || [[ $percentage -gt 100 ]]; then
        echo "Error: Percentage must be between 0 and 100"
        exit 1
    fi

    echo "Setting threading rollout to ${percentage}%..."

    local response=$(curl -s -X PUT \
        "https://api.cloudflare.com/client/v4/accounts/$CLOUDFLARE_ACCOUNT_ID/storage/kv/namespaces/$KV_NAMESPACE_ID/values/threading_rollout" \
        -H "Authorization: Bearer $CLOUDFLARE_API_TOKEN" \
        -H "Content-Type: application/json" \
        -d "{\"percentage\": $percentage, \"updated_at\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"}")

    if echo "$response" | jq -e '.success' > /dev/null 2>&1; then
        echo "✅ Feature flag updated successfully"
        echo "Threading enabled for ${percentage}% of users"
    else
        echo "❌ Failed to update feature flag"
        echo "$response" | jq '.'
        exit 1
    fi
}

get_feature_flag() {
    local response=$(curl -s -X GET \
        "https://api.cloudflare.com/client/v4/accounts/$CLOUDFLARE_ACCOUNT_ID/storage/kv/namespaces/$KV_NAMESPACE_ID/values/threading_rollout" \
        -H "Authorization: Bearer $CLOUDFLARE_API_TOKEN")

    if [[ -n "$response" ]]; then
        echo "Current Threading Rollout Status:"
        echo "$response" | jq '.'
    else
        echo "No feature flag found (threading disabled)"
    fi
}

# Parse command line arguments
PERCENTAGE="${THREADING_ROLLOUT_PERCENTAGE}"

while [[ $# -gt 0 ]]; do
    case $1 in
        -p|--percentage)
            PERCENTAGE="$2"
            shift 2
            ;;
        -e|--enable)
            PERCENTAGE=100
            shift
            ;;
        -d|--disable)
            PERCENTAGE=0
            shift
            ;;
        -s|--status)
            check_dependencies
            check_env
            get_feature_flag
            exit 0
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            usage
            exit 1
            ;;
    esac
done

# Main execution
check_dependencies
check_env
set_feature_flag "$PERCENTAGE"

echo ""
echo "Deployment complete. Users will receive threading based on rollout percentage."
echo "Monitor adoption at: https://dash.cloudflare.com/analytics"
