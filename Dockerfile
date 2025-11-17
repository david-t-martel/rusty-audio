# Multi-stage Dockerfile for Rusty Audio
# Supports cross-compilation for multiple targets
# Usage: docker build --target <target> -t rusty-audio:<target> .

# ====================
# Stage 1: Base Builder
# ====================
FROM rust:1.75-slim as base-builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libasound2-dev \
    build-essential \
    curl \
    git \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src
COPY benches ./benches
COPY tests ./tests

# Copy submodule (web-audio-api-rs)
COPY web-audio-api-rs ./web-audio-api-rs

# ====================
# Stage 2: Desktop Build (Linux x64)
# ====================
FROM base-builder as desktop-linux

RUN cargo build --release --features native-binary --target x86_64-unknown-linux-gnu

RUN mkdir -p /output && \
    cp target/x86_64-unknown-linux-gnu/release/rusty-audio_native /output/rusty-audio && \
    chmod +x /output/rusty-audio

# ====================
# Stage 3: WASM Build
# ====================
FROM base-builder as wasm-builder

# Install wasm-pack and binaryen
RUN rustup target add wasm32-unknown-unknown && \
    cargo install wasm-pack --locked && \
    apt-get update && apt-get install -y binaryen && \
    rm -rf /var/lib/apt/lists/*

# Build WASM
RUN wasm-pack build \
    --target web \
    --out-dir /output/pkg \
    --release \
    -- --no-default-features

# Optimize WASM
RUN wasm-opt -Oz \
    --enable-simd \
    --enable-bulk-memory \
    --strip-debug \
    --vacuum \
    /output/pkg/rusty_audio_bg.wasm \
    -o /output/pkg/rusty_audio_bg.wasm

# ====================
# Stage 4: Testing Environment
# ====================
FROM base-builder as testing

# Install additional testing tools
RUN cargo install cargo-tarpaulin cargo-audit cargo-deny --locked

# Run tests
RUN cargo test --all-features --all-targets

# Generate coverage report
RUN cargo tarpaulin --all-features --workspace --timeout 120 \
    --exclude-files "benches/*" "examples/*" \
    --out Xml --output-dir /coverage/

# Security audit
RUN cargo audit && cargo deny check

# ====================
# Stage 5: Cross-Compilation for Windows
# ====================
FROM base-builder as cross-windows

# Install MinGW toolchain for Windows cross-compilation
RUN apt-get update && apt-get install -y \
    mingw-w64 \
    && rm -rf /var/lib/apt/lists/*

RUN rustup target add x86_64-pc-windows-gnu

# Configure Cargo for cross-compilation
RUN mkdir -p ~/.cargo && \
    echo '[target.x86_64-pc-windows-gnu]\nlinker = "x86_64-w64-mingw32-gcc"' > ~/.cargo/config.toml

RUN cargo build --release --features native-binary --target x86_64-pc-windows-gnu

RUN mkdir -p /output && \
    cp target/x86_64-pc-windows-gnu/release/rusty-audio_native.exe /output/

# ====================
# Stage 6: Runtime Image (Linux Desktop)
# ====================
FROM debian:bookworm-slim as runtime-linux

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libasound2 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 rustyaudio

WORKDIR /app

# Copy binary from builder
COPY --from=desktop-linux /output/rusty-audio /usr/local/bin/rusty-audio

USER rustyaudio

# Set environment variables
ENV RUST_LOG=info
ENV RUST_BACKTRACE=1

ENTRYPOINT ["rusty-audio"]

# ====================
# Stage 7: WASM Server (for local testing)
# ====================
FROM nginx:alpine as wasm-server

# Copy WASM build output
COPY --from=wasm-builder /output /usr/share/nginx/html

# Copy static assets
COPY static /usr/share/nginx/html
COPY index.html /usr/share/nginx/html

# Configure nginx for WASM
RUN cat > /etc/nginx/conf.d/default.conf <<'EOF'
server {
    listen 80;
    server_name localhost;
    root /usr/share/nginx/html;
    index index.html;

    # Security headers
    add_header X-Content-Type-Options nosniff always;
    add_header X-Frame-Options DENY always;
    add_header Referrer-Policy no-referrer always;
    add_header Cross-Origin-Opener-Policy same-origin always;
    add_header Cross-Origin-Embedder-Policy require-corp always;

    # CSP for WASM
    add_header Content-Security-Policy "default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'self' https:; media-src 'self' blob:; worker-src 'self'; object-src 'none'; frame-ancestors 'none'" always;

    # WASM MIME type
    location ~ \.wasm$ {
        add_header Content-Type application/wasm;
        add_header Cache-Control "public, max-age=31536000, immutable";
    }

    # Service worker (no cache)
    location = /service-worker.js {
        add_header Cache-Control "no-store";
    }

    # Manifest
    location = /manifest.webmanifest {
        add_header Content-Type "application/manifest+json; charset=utf-8";
    }

    # SPA routing
    location / {
        try_files $uri $uri/ /index.html;
    }

    # Gzip compression
    gzip on;
    gzip_vary on;
    gzip_proxied any;
    gzip_comp_level 6;
    gzip_types text/plain text/css text/xml application/json application/javascript application/wasm;
}
EOF

EXPOSE 80

# ====================
# Stage 8: Development Environment
# ====================
FROM base-builder as development

# Install development tools
RUN cargo install cargo-watch cargo-edit cargo-outdated sccache --locked

# Install wasm tools
RUN rustup target add wasm32-unknown-unknown && \
    cargo install wasm-pack trunk --locked

# Install system tools
RUN apt-get update && apt-get install -y \
    vim \
    tmux \
    htop \
    binaryen \
    && rm -rf /var/lib/apt/lists/*

# Configure sccache
ENV RUSTC_WRAPPER=sccache
ENV SCCACHE_DIR=/sccache

WORKDIR /workspace

# Development entrypoint
CMD ["bash"]

# ====================
# Build Arguments
# ====================
# docker build --target desktop-linux -t rusty-audio:linux .
# docker build --target wasm-server -t rusty-audio:wasm .
# docker build --target testing -t rusty-audio:test .
# docker build --target development -t rusty-audio:dev .
