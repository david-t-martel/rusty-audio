# Rusty Audio Workspace - Quick Start Guide

## Build All Components

```bash
./build-all.sh
```

## Individual Builds

### Core Library
```bash
cd rusty-audio-core
cargo build --release --features native
cargo test --all-features
```

### Desktop Application
```bash
cd rusty-audio-desktop
cargo run --release
```

### WASM Application
```bash
cd rusty-audio-web
wasm-pack build --target web --release
```

## OAuth Usage (WASM)

```rust
use rusty_audio_web::auth::{OAuthClient, OAuthProvider};

// Google login
let client = OAuthClient::new(
    OAuthProvider::Google,
    "your-client-id".to_string(),
    "http://localhost:8080/callback".to_string(),
);

let auth_url = client.initiate_auth().await?;
// Redirect to auth_url

let session = client.handle_callback(&code).await?;
// User authenticated!
```

## Features

### Core Library
- `native` - Native platform support
- `wasm` - WASM platform support
- `audio-optimizations` - Real-time optimizations
- `ai-features` - AI-enhanced processing

### Desktop App
- `audio-optimizations` (default)
- `ai-features`

### WASM App
- `auth` (default) - OAuth authentication
- `ai-features`

## Documentation

- `WORKSPACE_README.md` - Complete guide
- `WORKSPACE_IMPLEMENTATION_COMPLETE.md` - Implementation details
- `WORKSPACE_SEPARATION_SUMMARY.md` - Executive summary

## Status

✅ Core library compiles
✅ Desktop app compiles and runs
✅ OAuth fully implemented
⚠️  WASM needs feature gating

## Next Steps

1. Fix WASM build (feature gates)
2. Address clippy warnings
3. Add integration tests
4. Deploy demo

---

For detailed information, see `WORKSPACE_README.md`
