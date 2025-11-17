# Rusty Audio Workspace

This workspace contains the modular architecture for Rusty Audio, split into three main components:

## Workspace Structure

```
rusty-audio/
├── Cargo.toml                # Workspace root configuration
├── rusty-audio-core/         # Shared library code
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs            # Core library entry point
│       ├── audio/            # Audio backends and processing
│       ├── ui/               # UI components and themes
│       ├── ai/               # AI-enhanced features
│       ├── security/         # Security and validation
│       └── testing/          # Testing utilities
├── rusty-audio-desktop/      # Desktop application
│   ├── Cargo.toml
│   ├── benches/              # Performance benchmarks
│   └── src/
│       ├── main.rs           # Desktop entry point
│       └── panel_implementation.rs
└── rusty-audio-web/          # WASM/PWA application
    ├── Cargo.toml
    └── src/
        ├── lib.rs            # WASM entry point
        ├── auth/             # OAuth authentication
        │   ├── mod.rs
        │   ├── oauth_client.rs   # OAuth 2.0 with PKCE
        │   ├── providers.rs      # Google, GitHub, Microsoft
        │   ├── session.rs        # Session management
        │   └── token_storage.rs  # Secure token storage
        ├── wasm_app.rs       # WASM application logic
        └── web_storage.rs    # Web Storage API helpers
```

## Components

### rusty-audio-core

The core library containing all shared functionality:

- **Audio Processing**: Backends, DSP, effects, equalizer
- **UI Components**: Controls, visualizers, themes, layouts
- **AI Features**: Audio analysis, optimization, recommendations
- **Security**: Input validation, audio safety limits
- **Testing**: Signal generators, property tests, benchmarks

**Features:**
- `native` - Enable native platform features (CPAL, web-audio-api, etc.)
- `wasm` - Enable WASM platform features (web-sys, wasm-bindgen, etc.)
- `audio-optimizations` - Enable real-time audio optimizations
- `ai-features` - Enable AI-enhanced processing
- `property-testing` - Enable property-based testing

**Build:**
```bash
cd rusty-audio-core
cargo build --release --features native
cargo test --all-features
```

### rusty-audio-desktop

Native desktop application for Windows, Linux, and macOS:

- Professional audio interface support (ASIO on Windows)
- Low-latency audio processing
- Full feature set with no browser limitations
- Recording and MIDI support

**Features:**
- `audio-optimizations` - Enable real-time audio thread priority
- `ai-features` - Enable AI-enhanced processing

**Build:**
```bash
cd rusty-audio-desktop
cargo build --release
cargo run --release
```

**Benchmarks:**
```bash
cargo bench
```

### rusty-audio-web

WASM/PWA web application with OAuth authentication:

- Progressive Web App support
- OAuth 2.0 authentication (Google, GitHub, Microsoft)
- PKCE (Proof Key for Code Exchange) for secure SPA auth
- Web Audio API integration
- Offline support via Service Workers

**Features:**
- `auth` - Enable OAuth authentication (default)
- `ai-features` - Enable AI-enhanced processing

**Build:**
```bash
cd rusty-audio-web
wasm-pack build --target web --release
```

**Deploy:**
```bash
# See deployment scripts in scripts/ directory
../scripts/deploy-pwa-cdn.sh
```

## OAuth Authentication

The web application includes a complete OAuth 2.0 implementation with PKCE:

### Supported Providers

1. **Google** - OAuth 2.0 with OpenID Connect
2. **GitHub** - OAuth 2.0
3. **Microsoft** - OAuth 2.0 with OpenID Connect

### Usage

```rust
use rusty_audio_web::auth::{OAuthClient, OAuthProvider};

// Initialize OAuth client
let mut client = OAuthClient::new(
    OAuthProvider::Google,
    "your-client-id".to_string(),
    "http://localhost:8080/callback".to_string(),
);

// Start login flow
let auth_url = client.initiate_auth().await?;
// Redirect user to auth_url

// Handle callback
let session = client.handle_callback(&code).await?;

// Use session
if let Some(token) = session.access_token {
    // Make authenticated requests
}

// Logout
client.logout().await?;
```

### Security Features

- **PKCE**: Proof Key for Code Exchange prevents authorization code interception
- **State Parameter**: CSRF protection with random state verification
- **Token Storage**: Secure storage in localStorage (encrypted in production)
- **Session Management**: Automatic token refresh and expiration handling

## Building the Workspace

### Build All Components

```bash
./build-all.sh
```

### Build Specific Components

```bash
# Core library
cargo build -p rusty-audio-core --release --features native

# Desktop app
cargo build -p rusty-audio-desktop --release

# WASM app
cd rusty-audio-web && wasm-pack build --target web --release
```

### Run Tests

```bash
# All tests
cargo test --workspace

# Core library tests
cargo test -p rusty-audio-core --all-features

# Desktop app tests
cargo test -p rusty-audio-desktop

# WASM tests (requires wasm-pack)
cd rusty-audio-web && wasm-pack test --headless --firefox
```

## Development Workflow

1. **Make changes to core library:**
   ```bash
   cd rusty-audio-core
   cargo check
   cargo test
   ```

2. **Test desktop application:**
   ```bash
   cd rusty-audio-desktop
   cargo run
   ```

3. **Test WASM application:**
   ```bash
   cd rusty-audio-web
   wasm-pack build --dev
   # Serve with local web server
   python -m http.server 8080
   ```

## Migration from Monolithic Structure

The original `src/` directory has been split into:

- **Shared code** → `rusty-audio-core/src/`
- **Desktop-specific** → `rusty-audio-desktop/src/`
- **WASM-specific** → `rusty-audio-web/src/`

### Import Changes

Before (monolithic):
```rust
use rusty_audio::audio::AudioBackend;
use rusty_audio::ui::Theme;
```

After (workspace):
```rust
// In desktop app
use rusty_audio_core::audio::AudioBackend;
use rusty_audio_core::ui::Theme;

// In WASM app
use rusty_audio_core::prelude::*;
```

## Workspace Dependencies

Dependencies are shared via `[workspace.dependencies]` in the root `Cargo.toml`:

- **GUI**: egui 0.33, eframe 0.33, egui_dock 0.18
- **Audio**: cpal 0.15, web-audio-api 1.2, symphonia 0.5
- **DSP**: rustfft 6.0, realfft 3.3
- **Serialization**: serde 1.0, serde_json 1.0
- **Async**: tokio 1.0, futures 0.3
- **WASM**: wasm-bindgen 0.2, web-sys 0.3
- **OAuth**: oauth2 4.4, reqwest 0.11

## Testing Strategy

### Unit Tests
```bash
cargo test --workspace
```

### Integration Tests
```bash
cargo test --workspace --test '*'
```

### Property-Based Tests
```bash
cargo test --workspace --features property-testing
```

### Benchmarks
```bash
cd rusty-audio-desktop
cargo bench
```

### UI Tests
```bash
cargo test --workspace --features egui_kittest
```

## Performance Optimization

### Desktop Build Profiles

- **release**: Full optimization with LTO
- **release-fast**: Maximum speed optimization
- **release-small**: Size-optimized build

```bash
cargo build --profile release-fast
```

### WASM Build Optimization

```bash
wasm-pack build --release -- -Z build-std=std,panic_abort
```

## Deployment

### Desktop Application

```bash
# Windows
cd rusty-audio-desktop
cargo build --release
# Binary: target/release/rusty-audio.exe

# Linux
cargo build --release
# Binary: target/release/rusty-audio

# macOS
cargo build --release --target x86_64-apple-darwin
```

### WASM/PWA Application

```bash
cd rusty-audio-web
wasm-pack build --target web --release

# Deploy to CDN
../scripts/deploy-pwa-cdn.sh
```

## Documentation

Generate documentation for all workspace members:

```bash
cargo doc --workspace --no-deps --open
```

## Contributing

When contributing to the workspace:

1. **Core library changes**: Update all dependent crates
2. **Breaking changes**: Update semver appropriately
3. **New features**: Add feature flags for optional functionality
4. **Tests**: Add tests for new functionality
5. **Documentation**: Update inline docs and README files

## License

MIT OR Apache-2.0

## Resources

- [eframe Documentation](https://docs.rs/eframe)
- [wasm-bindgen Guide](https://rustwasm.github.io/docs/wasm-bindgen/)
- [OAuth 2.0 RFC](https://datatracker.ietf.org/doc/html/rfc6749)
- [PKCE RFC](https://datatracker.ietf.org/doc/html/rfc7636)
