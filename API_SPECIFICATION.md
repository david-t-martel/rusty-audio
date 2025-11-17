# API Specification - Rusty Audio Backend

## Overview

This document specifies the REST API for the Rusty Audio web application backend, deployed as Cloudflare Workers with Cloudflare D1 database.

**Base URL:** `https://api.rusty-audio.example.com`

**Authentication:** Bearer token (JWT) in `Authorization` header

**Content Type:** `application/json`

---

## OpenAPI Specification

```yaml
openapi: 3.0.3
info:
  title: Rusty Audio API
  description: Backend API for Rusty Audio web application
  version: 0.2.0
  contact:
    name: Rusty Audio Team
    url: https://github.com/yourusername/rusty-audio

servers:
  - url: https://api.rusty-audio.example.com
    description: Production server
  - url: https://api-staging.rusty-audio.example.com
    description: Staging server
  - url: http://localhost:8787
    description: Local development server

components:
  securitySchemes:
    bearerAuth:
      type: http
      scheme: bearer
      bearerFormat: JWT

  schemas:
    Error:
      type: object
      properties:
        error:
          type: string
          description: Error message
        code:
          type: string
          description: Error code
      required:
        - error
        - code

    User:
      type: object
      properties:
        id:
          type: string
          format: uuid
          example: "550e8400-e29b-41d4-a716-446655440000"
        email:
          type: string
          format: email
          example: "user@gmail.com"
        name:
          type: string
          example: "John Doe"
        avatar_url:
          type: string
          format: uri
          nullable: true
          example: "https://lh3.googleusercontent.com/..."
        tier:
          type: string
          enum: [Free, Premium]
          example: "Free"
        created_at:
          type: string
          format: date-time
          example: "2024-01-01T00:00:00Z"
      required:
        - id
        - email
        - name
        - tier
        - created_at

    AudioPreset:
      type: object
      properties:
        id:
          type: string
          format: uuid
        user_id:
          type: string
          format: uuid
        name:
          type: string
          maxLength: 100
          example: "Bass Boost"
        description:
          type: string
          maxLength: 500
          nullable: true
          example: "Heavy bass with crisp highs"
        eq_settings:
          type: array
          items:
            $ref: '#/components/schemas/EqBand'
          minItems: 8
          maxItems: 8
        effects_config:
          type: object
          nullable: true
          properties:
            reverb:
              type: number
              minimum: 0
              maximum: 1
            delay:
              type: number
              minimum: 0
              maximum: 1
        is_public:
          type: boolean
          default: false
        created_at:
          type: string
          format: date-time
        updated_at:
          type: string
          format: date-time
      required:
        - id
        - user_id
        - name
        - eq_settings
        - is_public
        - created_at
        - updated_at

    EqBand:
      type: object
      properties:
        frequency:
          type: number
          format: float
          example: 60.0
          description: "Frequency in Hz"
        gain:
          type: number
          format: float
          minimum: -12
          maximum: 12
          example: 3.0
          description: "Gain in dB"
        q:
          type: number
          format: float
          minimum: 0.1
          maximum: 10
          example: 1.0
          description: "Q factor (bandwidth)"
      required:
        - frequency
        - gain
        - q

    UsageMetric:
      type: object
      properties:
        id:
          type: string
          format: uuid
        user_id:
          type: string
          format: uuid
        event_type:
          type: string
          enum: [playback, preset_save, preset_load, eq_adjust, export]
          example: "playback"
        event_data:
          type: object
          nullable: true
        timestamp:
          type: string
          format: date-time
      required:
        - id
        - user_id
        - event_type
        - timestamp

paths:
  # ==================== Authentication ====================

  /api/auth/callback:
    post:
      summary: Exchange OAuth authorization code for session token
      description: |
        Called after OAuth redirect from provider.
        Exchanges authorization code + PKCE verifier for session token.
      tags:
        - Authentication
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
              properties:
                code:
                  type: string
                  description: OAuth authorization code from provider
                code_verifier:
                  type: string
                  description: PKCE code verifier (43-128 chars)
                provider:
                  type: string
                  enum: [Google, GitHub, Microsoft]
              required:
                - code
                - code_verifier
                - provider
      responses:
        '200':
          description: Authentication successful
          content:
            application/json:
              schema:
                type: object
                properties:
                  session_token:
                    type: string
                    description: JWT session token (valid for 30 days)
                  user:
                    $ref: '#/components/schemas/User'
                  expires_at:
                    type: integer
                    format: int64
                    description: Unix timestamp when token expires
                required:
                  - session_token
                  - user
                  - expires_at
        '400':
          description: Invalid request (missing parameters, invalid provider)
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Error'
        '401':
          description: Authentication failed (invalid code, PKCE verification failed)
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Error'
        '500':
          description: Internal server error
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Error'

  /api/auth/refresh:
    post:
      summary: Refresh session token
      description: |
        Refresh an existing session token before expiration.
        Returns a new token with extended validity.
      tags:
        - Authentication
      security:
        - bearerAuth: []
      responses:
        '200':
          description: Token refreshed successfully
          content:
            application/json:
              schema:
                type: object
                properties:
                  session_token:
                    type: string
                    description: New JWT session token
                  expires_at:
                    type: integer
                    format: int64
                    description: Unix timestamp when new token expires
                required:
                  - session_token
                  - expires_at
        '401':
          description: Invalid or expired token
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Error'

  /api/auth/logout:
    post:
      summary: Logout and invalidate session
      description: |
        Invalidate the current session token.
        Client should clear local storage after successful logout.
      tags:
        - Authentication
      security:
        - bearerAuth: []
      responses:
        '200':
          description: Logout successful
          content:
            application/json:
              schema:
                type: object
                properties:
                  success:
                    type: boolean
                    example: true
        '401':
          description: Invalid token
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Error'

  /api/auth/user:
    get:
      summary: Get current user profile
      description: Get authenticated user's profile information
      tags:
        - Authentication
      security:
        - bearerAuth: []
      responses:
        '200':
          description: User profile retrieved successfully
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/User'
        '401':
          description: Not authenticated
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Error'

  # ==================== Audio Presets ====================

  /api/presets:
    get:
      summary: List user's audio presets
      description: |
        Get all presets owned by the authenticated user.
        Optionally include public presets.
      tags:
        - Presets
      security:
        - bearerAuth: []
      parameters:
        - name: include_public
          in: query
          description: Include public presets from other users
          schema:
            type: boolean
            default: false
        - name: limit
          in: query
          description: Maximum number of presets to return
          schema:
            type: integer
            minimum: 1
            maximum: 100
            default: 50
        - name: offset
          in: query
          description: Pagination offset
          schema:
            type: integer
            minimum: 0
            default: 0
      responses:
        '200':
          description: Presets retrieved successfully
          content:
            application/json:
              schema:
                type: object
                properties:
                  presets:
                    type: array
                    items:
                      $ref: '#/components/schemas/AudioPreset'
                  total:
                    type: integer
                    description: Total number of presets
                  limit:
                    type: integer
                  offset:
                    type: integer
        '401':
          description: Not authenticated
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Error'

    post:
      summary: Create new audio preset
      description: |
        Save a new audio preset for the authenticated user.
        Free tier users limited to 10 presets, Premium unlimited.
      tags:
        - Presets
      security:
        - bearerAuth: []
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
              properties:
                name:
                  type: string
                  maxLength: 100
                  example: "My Custom EQ"
                description:
                  type: string
                  maxLength: 500
                  nullable: true
                  example: "Perfect for rock music"
                eq_settings:
                  type: array
                  items:
                    $ref: '#/components/schemas/EqBand'
                  minItems: 8
                  maxItems: 8
                effects_config:
                  type: object
                  nullable: true
                is_public:
                  type: boolean
                  default: false
              required:
                - name
                - eq_settings
      responses:
        '201':
          description: Preset created successfully
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/AudioPreset'
        '400':
          description: Invalid request (validation errors)
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Error'
        '401':
          description: Not authenticated
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Error'
        '403':
          description: Preset limit reached (Free tier)
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Error'
              example:
                error: "Preset limit reached. Upgrade to Premium for unlimited presets."
                code: "PRESET_LIMIT_REACHED"

  /api/presets/{preset_id}:
    get:
      summary: Get preset by ID
      description: Get a specific preset (must be owned by user or public)
      tags:
        - Presets
      security:
        - bearerAuth: []
      parameters:
        - name: preset_id
          in: path
          required: true
          schema:
            type: string
            format: uuid
      responses:
        '200':
          description: Preset retrieved successfully
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/AudioPreset'
        '401':
          description: Not authenticated
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Error'
        '403':
          description: Access denied (private preset owned by another user)
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Error'
        '404':
          description: Preset not found
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Error'

    put:
      summary: Update preset
      description: Update an existing preset (must be owned by user)
      tags:
        - Presets
      security:
        - bearerAuth: []
      parameters:
        - name: preset_id
          in: path
          required: true
          schema:
            type: string
            format: uuid
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
              properties:
                name:
                  type: string
                  maxLength: 100
                description:
                  type: string
                  maxLength: 500
                  nullable: true
                eq_settings:
                  type: array
                  items:
                    $ref: '#/components/schemas/EqBand'
                  minItems: 8
                  maxItems: 8
                effects_config:
                  type: object
                  nullable: true
                is_public:
                  type: boolean
      responses:
        '200':
          description: Preset updated successfully
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/AudioPreset'
        '400':
          description: Invalid request
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Error'
        '401':
          description: Not authenticated
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Error'
        '403':
          description: Not authorized (not owner)
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Error'
        '404':
          description: Preset not found
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Error'

    delete:
      summary: Delete preset
      description: Delete a preset (must be owned by user)
      tags:
        - Presets
      security:
        - bearerAuth: []
      parameters:
        - name: preset_id
          in: path
          required: true
          schema:
            type: string
            format: uuid
      responses:
        '204':
          description: Preset deleted successfully
        '401':
          description: Not authenticated
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Error'
        '403':
          description: Not authorized (not owner)
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Error'
        '404':
          description: Preset not found
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Error'

  # ==================== Usage Metrics (Premium) ====================

  /api/usage:
    get:
      summary: Get usage metrics
      description: |
        Get usage analytics for the authenticated user.
        Premium tier only.
      tags:
        - Usage
      security:
        - bearerAuth: []
      parameters:
        - name: start_date
          in: query
          description: Start date for metrics (ISO 8601)
          schema:
            type: string
            format: date-time
        - name: end_date
          in: query
          description: End date for metrics (ISO 8601)
          schema:
            type: string
            format: date-time
        - name: event_type
          in: query
          description: Filter by event type
          schema:
            type: string
            enum: [playback, preset_save, preset_load, eq_adjust, export]
      responses:
        '200':
          description: Metrics retrieved successfully
          content:
            application/json:
              schema:
                type: object
                properties:
                  metrics:
                    type: array
                    items:
                      $ref: '#/components/schemas/UsageMetric'
                  total_events:
                    type: integer
                  summary:
                    type: object
                    properties:
                      total_playback_time:
                        type: integer
                        description: Total playback time in seconds
                      total_presets:
                        type: integer
                      most_used_preset:
                        type: string
                        nullable: true
        '401':
          description: Not authenticated
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Error'
        '403':
          description: Premium tier required
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Error'
              example:
                error: "Usage metrics are a Premium feature"
                code: "PREMIUM_REQUIRED"

    post:
      summary: Log usage event
      description: |
        Log a usage event for analytics.
        Called by client to track user actions.
      tags:
        - Usage
      security:
        - bearerAuth: []
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
              properties:
                event_type:
                  type: string
                  enum: [playback, preset_save, preset_load, eq_adjust, export]
                event_data:
                  type: object
                  nullable: true
                  description: Additional event-specific data
              required:
                - event_type
      responses:
        '201':
          description: Event logged successfully
          content:
            application/json:
              schema:
                type: object
                properties:
                  id:
                    type: string
                    format: uuid
        '401':
          description: Not authenticated
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Error'

  # ==================== Feedback ====================

  /api/feedback:
    post:
      summary: Submit user feedback
      description: |
        Submit feedback or bug reports.
        Available to all users (authenticated or anonymous).
      tags:
        - Feedback
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
              properties:
                type:
                  type: string
                  enum: [bug, feature_request, general]
                  example: "bug"
                message:
                  type: string
                  maxLength: 5000
                  example: "The EQ doesn't save properly on mobile"
                email:
                  type: string
                  format: email
                  nullable: true
                  description: Optional contact email for follow-up
              required:
                - type
                - message
      responses:
        '201':
          description: Feedback submitted successfully
          content:
            application/json:
              schema:
                type: object
                properties:
                  id:
                    type: string
                    format: uuid
                  message:
                    type: string
                    example: "Thank you for your feedback!"
        '400':
          description: Invalid request
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Error'

  # ==================== Health Check ====================

  /api/health:
    get:
      summary: Health check endpoint
      description: Check API health status
      tags:
        - System
      responses:
        '200':
          description: API is healthy
          content:
            application/json:
              schema:
                type: object
                properties:
                  status:
                    type: string
                    example: "healthy"
                  version:
                    type: string
                    example: "0.2.0"
                  timestamp:
                    type: string
                    format: date-time

tags:
  - name: Authentication
    description: OAuth 2.0 authentication and session management
  - name: Presets
    description: Audio preset management
  - name: Usage
    description: Usage analytics (Premium tier)
  - name: Feedback
    description: User feedback and bug reports
  - name: System
    description: System health and status
```

---

## Error Codes

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `AUTH_FAILED` | 401 | Authentication failed (invalid credentials) |
| `INVALID_TOKEN` | 401 | Invalid or expired session token |
| `NOT_AUTHENTICATED` | 401 | No authentication provided |
| `INVALID_STATE` | 401 | Invalid OAuth state parameter (CSRF) |
| `PKCE_VERIFICATION_FAILED` | 401 | PKCE code verifier doesn't match challenge |
| `ACCESS_DENIED` | 403 | User not authorized for this resource |
| `PREMIUM_REQUIRED` | 403 | Premium tier required for this feature |
| `PRESET_LIMIT_REACHED` | 403 | Free tier preset limit (10) reached |
| `NOT_FOUND` | 404 | Resource not found |
| `INVALID_REQUEST` | 400 | Invalid request parameters |
| `VALIDATION_ERROR` | 400 | Request validation failed |
| `RATE_LIMIT_EXCEEDED` | 429 | Too many requests |
| `INTERNAL_ERROR` | 500 | Internal server error |
| `DATABASE_ERROR` | 500 | Database operation failed |
| `PROVIDER_ERROR` | 500 | OAuth provider error |

---

## Rate Limiting

### Limits by Endpoint

| Endpoint | Free Tier | Premium Tier |
|----------|-----------|--------------|
| `/api/auth/*` | 10 req/min | 20 req/min |
| `/api/presets` (GET) | 60 req/min | 300 req/min |
| `/api/presets` (POST/PUT/DELETE) | 10 req/min | 100 req/min |
| `/api/usage` | N/A (Premium only) | 60 req/min |
| `/api/feedback` | 5 req/hour | 10 req/hour |

### Rate Limit Headers

```
X-RateLimit-Limit: 60
X-RateLimit-Remaining: 45
X-RateLimit-Reset: 1704067200
```

### Rate Limit Exceeded Response

```json
HTTP/1.1 429 Too Many Requests
Content-Type: application/json

{
  "error": "Rate limit exceeded. Try again in 30 seconds.",
  "code": "RATE_LIMIT_EXCEEDED",
  "retry_after": 30
}
```

---

## CORS Configuration

### Allowed Origins

**Production:**
- `https://rusty-audio.example.com`
- `https://www.rusty-audio.example.com`

**Development:**
- `http://localhost:8080`
- `http://localhost:3000`

### CORS Headers

```
Access-Control-Allow-Origin: https://rusty-audio.example.com
Access-Control-Allow-Methods: GET, POST, PUT, DELETE, OPTIONS
Access-Control-Allow-Headers: Content-Type, Authorization
Access-Control-Max-Age: 86400
```

---

## Pagination

All list endpoints support pagination via `limit` and `offset` query parameters.

**Example Request:**
```
GET /api/presets?limit=20&offset=40
```

**Example Response:**
```json
{
  "presets": [...],
  "total": 157,
  "limit": 20,
  "offset": 40,
  "has_more": true
}
```

---

## Webhooks (Future)

Future enhancement: Webhook notifications for premium users.

**Planned Events:**
- `preset.created` - New preset created
- `preset.updated` - Preset updated
- `tier.upgraded` - User upgraded to Premium
- `usage.threshold` - Usage threshold reached

---

## Versioning Strategy

**Current Version:** v1 (implicit in `/api/*`)

**Future Versions:** `/api/v2/*` for breaking changes

**Deprecation Policy:**
- Announce deprecations 6 months in advance
- Support deprecated endpoints for 12 months
- Provide migration guides

---

## Client Implementation Examples

### Rust (reqwest + wasm)

```rust
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct Preset {
    id: String,
    name: String,
    eq_settings: Vec<EqBand>,
}

async fn get_presets(session_token: &str) -> Result<Vec<Preset>, Box<dyn std::error::Error>> {
    let response = Request::get("/api/presets")
        .header("Authorization", &format!("Bearer {}", session_token))
        .send()
        .await?
        .json::<PresetsResponse>()
        .await?;

    Ok(response.presets)
}

async fn create_preset(
    session_token: &str,
    name: &str,
    eq_settings: Vec<EqBand>,
) -> Result<Preset, Box<dyn std::error::Error>> {
    let body = CreatePresetRequest {
        name: name.to_string(),
        eq_settings,
        is_public: false,
    };

    let preset = Request::post("/api/presets")
        .header("Authorization", &format!("Bearer {}", session_token))
        .json(&body)?
        .send()
        .await?
        .json::<Preset>()
        .await?;

    Ok(preset)
}
```

### JavaScript (fetch)

```javascript
// Get presets
async function getPresets(sessionToken) {
  const response = await fetch('/api/presets', {
    headers: {
      'Authorization': `Bearer ${sessionToken}`
    }
  });

  if (!response.ok) {
    throw new Error('Failed to fetch presets');
  }

  const data = await response.json();
  return data.presets;
}

// Create preset
async function createPreset(sessionToken, name, eqSettings) {
  const response = await fetch('/api/presets', {
    method: 'POST',
    headers: {
      'Authorization': `Bearer ${sessionToken}`,
      'Content-Type': 'application/json'
    },
    body: JSON.stringify({
      name,
      eq_settings: eqSettings,
      is_public: false
    })
  });

  if (!response.ok) {
    const error = await response.json();
    throw new Error(error.error);
  }

  return await response.json();
}
```

---

## Testing the API

### Using cURL

```bash
# Get health status
curl https://api.rusty-audio.example.com/api/health

# Create preset (authenticated)
curl -X POST https://api.rusty-audio.example.com/api/presets \
  -H "Authorization: Bearer YOUR_SESSION_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "My EQ Preset",
    "eq_settings": [
      {"frequency": 60, "gain": 3.0, "q": 1.0},
      {"frequency": 170, "gain": 2.0, "q": 1.0},
      {"frequency": 310, "gain": 0.0, "q": 1.0},
      {"frequency": 600, "gain": -1.0, "q": 1.0},
      {"frequency": 1000, "gain": 0.0, "q": 1.0},
      {"frequency": 3000, "gain": 2.0, "q": 1.0},
      {"frequency": 6000, "gain": 4.0, "q": 1.0},
      {"frequency": 12000, "gain": 3.0, "q": 1.0}
    ]
  }'
```

### Postman Collection

Import the OpenAPI spec into Postman for interactive testing:
1. File → Import → Paste OpenAPI YAML
2. Configure environment variables for `base_url` and `session_token`
3. Test all endpoints

---

## Production Deployment

### Cloudflare Workers Configuration

```toml
# wrangler.toml
name = "rusty-audio-api"
main = "src/index.ts"
compatibility_date = "2024-01-01"

[env.production]
vars = { ENVIRONMENT = "production" }
route = "api.rusty-audio.example.com/*"

[[d1_databases]]
binding = "DB"
database_name = "rusty-audio-production"
database_id = "YOUR_D1_DATABASE_ID"

[[kv_namespaces]]
binding = "SESSIONS"
id = "YOUR_KV_NAMESPACE_ID"

[vars]
JWT_SECRET = "YOUR_JWT_SECRET"
GOOGLE_CLIENT_ID = "YOUR_GOOGLE_CLIENT_ID"
GITHUB_CLIENT_ID = "YOUR_GITHUB_CLIENT_ID"
MICROSOFT_CLIENT_ID = "YOUR_MICROSOFT_CLIENT_ID"
```

### Deploy Commands

```bash
# Deploy to production
wrangler deploy

# Run migrations
wrangler d1 migrations apply rusty-audio-production

# View logs
wrangler tail

# Test locally
wrangler dev
```

---

## Monitoring and Observability

### Key Metrics

- **Request Rate:** Requests per second
- **Error Rate:** 4xx and 5xx responses
- **Latency:** p50, p95, p99 response times
- **Auth Success Rate:** OAuth callback success rate
- **Active Sessions:** Number of active user sessions

### Alerts

- Error rate > 5% for 5 minutes
- p99 latency > 2 seconds
- Database connection failures
- OAuth provider downtime

---

## Changelog

### Version 0.2.0 (2024-12-01)
- Initial API specification
- OAuth 2.0 authentication
- Audio preset management
- Usage metrics (Premium tier)
- Feedback submission

### Future Versions

**0.3.0 (Planned)**
- Preset sharing and discovery
- Social features (likes, comments)
- Preset categories and tags
- Advanced search

**0.4.0 (Planned)**
- Collaborative presets
- Real-time sync across devices
- Export/import functionality
- Backup and restore

---

## Support

- **Documentation:** https://docs.rusty-audio.example.com
- **API Status:** https://status.rusty-audio.example.com
- **GitHub Issues:** https://github.com/yourusername/rusty-audio/issues
- **Discord:** https://discord.gg/rusty-audio
