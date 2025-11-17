#!/usr/bin/env bash
# macOS Packaging Script for Rusty Audio
# Creates .app bundle and .dmg installer for macOS (Intel + Apple Silicon)

set -euo pipefail

# Configuration
VERSION="${VERSION:-0.1.0}"
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST_DIR="${PROJECT_ROOT}/dist/macos"
APP_NAME="Rusty Audio"
BUNDLE_ID="dev.david.rusty-audio"
BINARY_NAME="rusty-audio_native"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

print_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
print_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
print_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
print_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Header
echo -e "${CYAN}========================================${NC}"
echo -e "${CYAN}  Rusty Audio - macOS Packaging${NC}"
echo -e "${CYAN}========================================${NC}"
echo ""
echo -e "${YELLOW}Version: $VERSION${NC}"
echo ""

# Check if running on macOS
if [[ "$OSTYPE" != "darwin"* ]]; then
    print_error "This script must be run on macOS"
    exit 1
fi

# Create distribution directory
print_info "[1/6] Creating distribution directory..."
mkdir -p "$DIST_DIR"
print_success "Created: $DIST_DIR"

# Build for both architectures
print_info "[2/6] Building universal binary..."

X86_BINARY="${PROJECT_ROOT}/target/x86_64-apple-darwin/release/${BINARY_NAME}"
ARM_BINARY="${PROJECT_ROOT}/target/aarch64-apple-darwin/release/${BINARY_NAME}"

# Build x86_64
if [ ! -f "$X86_BINARY" ]; then
    print_info "Building x86_64 binary..."
    cargo build --release --features native-binary --target x86_64-apple-darwin
else
    print_warning "Using existing x86_64 binary"
fi

# Build ARM64
if [ ! -f "$ARM_BINARY" ]; then
    print_info "Building ARM64 binary..."
    cargo build --release --features native-binary --target aarch64-apple-darwin
else
    print_warning "Using existing ARM64 binary"
fi

# Create universal binary
UNIVERSAL_BINARY="${DIST_DIR}/rusty-audio"
print_info "Creating universal binary..."
lipo -create "$X86_BINARY" "$ARM_BINARY" -output "$UNIVERSAL_BINARY"
chmod +x "$UNIVERSAL_BINARY"

print_success "Universal binary created"
print_success "Size: $(stat -f%z "$UNIVERSAL_BINARY" | numfmt --to=iec-i --suffix=B 2>/dev/null || stat -f%z "$UNIVERSAL_BINARY")"

# Verify architectures
print_info "Verifying architectures..."
lipo -info "$UNIVERSAL_BINARY"

# Create .app bundle
print_info "[3/6] Creating .app bundle..."

APP_BUNDLE="${DIST_DIR}/${APP_NAME}.app"
APP_CONTENTS="${APP_BUNDLE}/Contents"
APP_MACOS="${APP_CONTENTS}/MacOS"
APP_RESOURCES="${APP_CONTENTS}/Resources"

mkdir -p "$APP_MACOS"
mkdir -p "$APP_RESOURCES"

# Copy binary
cp "$UNIVERSAL_BINARY" "${APP_MACOS}/rusty-audio"
chmod +x "${APP_MACOS}/rusty-audio"

# Create Info.plist
cat > "${APP_CONTENTS}/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>en</string>
    <key>CFBundleDisplayName</key>
    <string>${APP_NAME}</string>
    <key>CFBundleExecutable</key>
    <string>rusty-audio</string>
    <key>CFBundleIconFile</key>
    <string>AppIcon</string>
    <key>CFBundleIdentifier</key>
    <string>${BUNDLE_ID}</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>${APP_NAME}</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>${VERSION}</string>
    <key>CFBundleVersion</key>
    <string>${VERSION}</string>
    <key>LSMinimumSystemVersion</key>
    <string>11.0</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSHumanReadableCopyright</key>
    <string>Copyright © 2024 Rusty Audio Team</string>
    <key>NSMicrophoneUsageDescription</key>
    <string>Rusty Audio needs access to your microphone for audio recording features.</string>
    <key>LSApplicationCategoryType</key>
    <string>public.app-category.music</string>
    <key>CFBundleDocumentTypes</key>
    <array>
        <dict>
            <key>CFBundleTypeName</key>
            <string>Audio Files</string>
            <key>CFBundleTypeRole</key>
            <string>Viewer</string>
            <key>LSItemContentTypes</key>
            <array>
                <string>public.audio</string>
                <string>public.mp3</string>
                <string>public.aifc-audio</string>
                <string>public.aiff-audio</string>
                <string>com.microsoft.waveform-audio</string>
            </array>
        </dict>
    </array>
</dict>
</plist>
EOF

# Create icon (placeholder - you should replace with actual .icns)
if [ -f "${PROJECT_ROOT}/static/icons/AppIcon.icns" ]; then
    cp "${PROJECT_ROOT}/static/icons/AppIcon.icns" "${APP_RESOURCES}/"
else
    print_warning "AppIcon.icns not found. Create one from PNG using:"
    print_info "  mkdir AppIcon.iconset"
    print_info "  sips -z 16 16     icon.png --out AppIcon.iconset/icon_16x16.png"
    print_info "  sips -z 32 32     icon.png --out AppIcon.iconset/icon_16x16@2x.png"
    print_info "  # ... (repeat for all sizes)"
    print_info "  iconutil -c icns AppIcon.iconset"
fi

print_success ".app bundle created: ${APP_BUNDLE}"

# Create DMG
print_info "[4/6] Creating DMG installer..."

DMG_NAME="RustyAudio-${VERSION}-macOS-universal.dmg"
DMG_PATH="${DIST_DIR}/${DMG_NAME}"
DMG_TEMP="${DIST_DIR}/dmg-temp"

mkdir -p "$DMG_TEMP"

# Copy .app bundle to temp directory
cp -R "$APP_BUNDLE" "$DMG_TEMP/"

# Create symbolic link to Applications
ln -s /Applications "$DMG_TEMP/Applications"

# Create README
cat > "$DMG_TEMP/README.txt" <<EOF
Rusty Audio v${VERSION}
=======================

Professional audio player built in Rust

INSTALLATION
------------
1. Drag "Rusty Audio.app" to the Applications folder
2. Launch from Applications or Spotlight

SYSTEM REQUIREMENTS
-------------------
- macOS 11.0 or later
- 4GB RAM minimum, 8GB recommended
- Intel or Apple Silicon processor

FIRST LAUNCH
------------
If you see "App is damaged" or security warning:
1. Open System Preferences > Security & Privacy
2. Click "Open Anyway"

Or disable Gatekeeper temporarily:
  sudo spctl --master-disable

FEATURES
--------
- 8-band parametric equalizer
- Real-time spectrum analyzer
- Signal generator
- Audio recording
- Multiple themes

SUPPORT
-------
GitHub: https://github.com/david-t-martel/rusty-audio
Issues: https://github.com/david-t-martel/rusty-audio/issues

COPYRIGHT
---------
Copyright © 2024 Rusty Audio Team
Licensed under MIT or Apache-2.0
EOF

# Create DS_Store for nice DMG appearance (optional)
# This would require additional tools like dmgbuild

# Create DMG
if command -v hdiutil >/dev/null 2>&1; then
    # Remove existing DMG
    rm -f "$DMG_PATH"

    # Create DMG
    hdiutil create -volname "Rusty Audio ${VERSION}" \
                   -srcfolder "$DMG_TEMP" \
                   -ov \
                   -format UDZO \
                   -imagekey zlib-level=9 \
                   "$DMG_PATH"

    print_success "DMG created: $DMG_PATH"
    print_success "Size: $(stat -f%z "$DMG_PATH" | numfmt --to=iec-i --suffix=B 2>/dev/null || stat -f%z "$DMG_PATH")"

    # Clean up temp directory
    rm -rf "$DMG_TEMP"
else
    print_error "hdiutil not found (should be available on macOS)"
    exit 1
fi

# Code signing (optional, requires Apple Developer account)
print_info "[5/6] Code signing..."

if command -v codesign >/dev/null 2>&1; then
    # Check if signing identity is available
    IDENTITY=$(security find-identity -v -p codesigning | grep "Developer ID Application" | head -1 | awk '{print $2}')

    if [ -n "$IDENTITY" ]; then
        print_info "Signing with identity: $IDENTITY"

        # Sign the binary
        codesign --force --deep --sign "$IDENTITY" "${APP_MACOS}/rusty-audio"

        # Sign the app bundle
        codesign --force --deep --sign "$IDENTITY" "$APP_BUNDLE"

        # Verify signature
        codesign --verify --deep --strict "$APP_BUNDLE"
        print_success "Code signing complete"
    else
        print_warning "No code signing identity found. Skipping signing."
        print_info "Users will need to right-click and choose 'Open' on first launch."
    fi
else
    print_warning "codesign not found. Skipping signing."
fi

# Notarization (optional, requires Apple Developer account and app-specific password)
print_info "[6/6] Notarization..."
print_warning "Notarization requires Apple Developer account and app-specific password"
print_info "To notarize manually:"
print_info "  1. Create app-specific password at appleid.apple.com"
print_info "  2. Run: xcrun notarytool submit ${DMG_NAME} --apple-id your@email.com --password @keychain:AC_PASSWORD --team-id TEAMID"
print_info "  3. Wait for email confirmation"
print_info "  4. Run: xcrun stapler staple ${APP_BUNDLE}"

# Generate checksums
print_info "Generating checksums..."
CHECKSUM_FILE="${DIST_DIR}/SHA256SUMS.txt"
rm -f "$CHECKSUM_FILE"

cd "$DIST_DIR"
for file in *.dmg *.tar.gz rusty-audio; do
    if [ -f "$file" ]; then
        shasum -a 256 "$file" >> "$CHECKSUM_FILE"
    fi
done
cd - > /dev/null

print_success "Checksums written to: $CHECKSUM_FILE"

# Summary
echo ""
echo -e "${CYAN}========================================${NC}"
echo -e "${GREEN}  Packaging Complete!${NC}"
echo -e "${CYAN}========================================${NC}"
echo ""
echo -e "${YELLOW}Output directory: $DIST_DIR${NC}"
echo ""
echo -e "${YELLOW}Files created:${NC}"
for file in "$DIST_DIR"/*.dmg "$DIST_DIR"/*.tar.gz "$UNIVERSAL_BINARY"; do
    if [ -f "$file" ]; then
        size=$(stat -f%z "$file" | numfmt --to=iec-i --suffix=B 2>/dev/null || stat -f%z "$file")
        echo -e "${CYAN}  - $(basename "$file") ($size)${NC}"
    fi
done
echo ""
echo -e "${YELLOW}.app bundle:${NC}"
echo -e "${CYAN}  - ${APP_BUNDLE}${NC}"
echo ""
echo -e "${YELLOW}Next steps:${NC}"
echo -e "${NC}  1. Test the .app bundle on macOS 11+ (Intel and Apple Silicon)${NC}"
echo -e "${NC}  2. Test the DMG installer${NC}"
echo -e "${NC}  3. Code sign and notarize (if you have Apple Developer account)${NC}"
echo -e "${NC}  4. Upload to GitHub Releases${NC}"
echo ""
