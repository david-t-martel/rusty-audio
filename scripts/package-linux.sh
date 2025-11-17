#!/usr/bin/env bash
# Linux Packaging Script for Rusty Audio
# Creates AppImage, .deb, and .rpm packages

set -euo pipefail

# Configuration
VERSION="${VERSION:-0.1.0}"
ARCH="x86_64"
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST_DIR="${PROJECT_ROOT}/dist/linux"
TARGET_DIR="${PROJECT_ROOT}/target/x86_64-unknown-linux-gnu/release"
BINARY_NAME="rusty-audio_native"
OUTPUT_NAME="rusty-audio"

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
echo -e "${CYAN}  Rusty Audio - Linux Packaging${NC}"
echo -e "${CYAN}========================================${NC}"
echo ""
echo -e "${YELLOW}Version: $VERSION${NC}"
echo -e "${YELLOW}Architecture: $ARCH${NC}"
echo ""

# Create distribution directory
print_info "[1/7] Creating distribution directory..."
mkdir -p "$DIST_DIR"
print_success "Created: $DIST_DIR"

# Build release binary if not exists
BINARY_PATH="${TARGET_DIR}/${BINARY_NAME}"
if [ ! -f "$BINARY_PATH" ]; then
    print_info "[2/7] Building release binary..."
    cargo build --release --features native-binary --target x86_64-unknown-linux-gnu

    if [ $? -ne 0 ]; then
        print_error "Build failed!"
        exit 1
    fi
else
    print_warning "[2/7] Using existing binary: $BINARY_PATH"
fi

# Verify binary
print_info "[3/7] Verifying binary..."
if [ ! -f "$BINARY_PATH" ]; then
    print_error "Binary not found: $BINARY_PATH"
    exit 1
fi

BINARY_SIZE=$(stat -c%s "$BINARY_PATH")
BINARY_SIZE_MB=$(echo "scale=2; $BINARY_SIZE / 1024 / 1024" | bc)
print_success "Binary size: ${BINARY_SIZE_MB} MB"

# Copy binary to dist
DIST_BINARY="${DIST_DIR}/${OUTPUT_NAME}"
cp "$BINARY_PATH" "$DIST_BINARY"
chmod +x "$DIST_BINARY"
print_success "Copied to: $DIST_BINARY"

# Create tarball distribution
print_info "[4/7] Creating tarball distribution..."
TARBALL_NAME="rusty-audio-${VERSION}-linux-${ARCH}.tar.gz"
TARBALL_PATH="${DIST_DIR}/${TARBALL_NAME}"

tar -czf "$TARBALL_PATH" -C "$DIST_DIR" "$OUTPUT_NAME"
print_success "Tarball created: $TARBALL_PATH"
print_success "Size: $(stat -c%s "$TARBALL_PATH" | numfmt --to=iec-i --suffix=B)"

# Create AppImage (requires appimagetool)
print_info "[5/7] Creating AppImage..."

if command -v appimagetool >/dev/null 2>&1; then
    APPDIR="${DIST_DIR}/RustyAudio.AppDir"
    mkdir -p "$APPDIR"
    mkdir -p "$APPDIR/usr/bin"
    mkdir -p "$APPDIR/usr/share/applications"
    mkdir -p "$APPDIR/usr/share/icons/hicolor/256x256/apps"

    # Copy binary
    cp "$DIST_BINARY" "$APPDIR/usr/bin/"

    # Create desktop entry
    cat > "$APPDIR/rusty-audio.desktop" <<EOF
[Desktop Entry]
Type=Application
Name=Rusty Audio
Comment=Professional audio player built in Rust
Exec=rusty-audio
Icon=rusty-audio
Categories=AudioVideo;Audio;Player;
Terminal=false
EOF

    # Copy desktop entry to standard location
    cp "$APPDIR/rusty-audio.desktop" "$APPDIR/usr/share/applications/"

    # Create AppRun script
    cat > "$APPDIR/AppRun" <<'EOF'
#!/bin/bash
SELF=$(readlink -f "$0")
HERE=${SELF%/*}
export PATH="${HERE}/usr/bin:${PATH}"
export LD_LIBRARY_PATH="${HERE}/usr/lib:${LD_LIBRARY_PATH}"
exec "${HERE}/usr/bin/rusty-audio" "$@"
EOF
    chmod +x "$APPDIR/AppRun"

    # Create placeholder icon (you should replace with actual icon)
    if [ -f "${PROJECT_ROOT}/static/icons/icon-512.png" ]; then
        cp "${PROJECT_ROOT}/static/icons/icon-512.png" \
           "$APPDIR/usr/share/icons/hicolor/256x256/apps/rusty-audio.png"
        cp "${PROJECT_ROOT}/static/icons/icon-512.png" \
           "$APPDIR/rusty-audio.png"
    else
        print_warning "Icon not found, creating placeholder"
        # Create minimal placeholder icon
        if command -v convert >/dev/null 2>&1; then
            convert -size 256x256 xc:blue \
                -pointsize 48 -fill white -gravity center -annotate +0+0 'RA' \
                "$APPDIR/rusty-audio.png"
            cp "$APPDIR/rusty-audio.png" \
               "$APPDIR/usr/share/icons/hicolor/256x256/apps/rusty-audio.png"
        fi
    fi

    # Build AppImage
    APPIMAGE_PATH="${DIST_DIR}/RustyAudio-${VERSION}-${ARCH}.AppImage"
    appimagetool "$APPDIR" "$APPIMAGE_PATH"

    if [ -f "$APPIMAGE_PATH" ]; then
        print_success "AppImage created: $APPIMAGE_PATH"
        print_success "Size: $(stat -c%s "$APPIMAGE_PATH" | numfmt --to=iec-i --suffix=B)"
    else
        print_warning "AppImage creation failed"
    fi

    # Clean up AppDir
    rm -rf "$APPDIR"
else
    print_warning "appimagetool not found. Skipping AppImage creation."
    print_info "Install from: https://github.com/AppImage/AppImageKit/releases"
fi

# Create .deb package (requires dpkg-deb)
print_info "[6/7] Creating .deb package..."

if command -v dpkg-deb >/dev/null 2>&1; then
    DEB_DIR="${DIST_DIR}/rusty-audio_${VERSION}_amd64"
    mkdir -p "$DEB_DIR/DEBIAN"
    mkdir -p "$DEB_DIR/usr/bin"
    mkdir -p "$DEB_DIR/usr/share/applications"
    mkdir -p "$DEB_DIR/usr/share/icons/hicolor/256x256/apps"
    mkdir -p "$DEB_DIR/usr/share/doc/rusty-audio"

    # Copy binary
    cp "$DIST_BINARY" "$DEB_DIR/usr/bin/"

    # Create control file
    INSTALLED_SIZE=$(du -sk "$DEB_DIR" | cut -f1)
    cat > "$DEB_DIR/DEBIAN/control" <<EOF
Package: rusty-audio
Version: ${VERSION}
Section: sound
Priority: optional
Architecture: amd64
Depends: libasound2 (>= 1.0.16), libc6 (>= 2.31)
Maintainer: Rusty Audio Team <rusty-audio@example.com>
Installed-Size: ${INSTALLED_SIZE}
Description: Professional audio player built in Rust
 Rusty Audio is a car-stereo-style audio player featuring:
 - 8-band parametric equalizer
 - Real-time spectrum analyzer
 - Signal generator
 - Audio recording
 - Multiple themes
 .
 Built with Rust, egui, and wgpu for high performance.
EOF

    # Create desktop entry
    cat > "$DEB_DIR/usr/share/applications/rusty-audio.desktop" <<EOF
[Desktop Entry]
Type=Application
Name=Rusty Audio
Comment=Professional audio player built in Rust
Exec=/usr/bin/rusty-audio
Icon=rusty-audio
Categories=AudioVideo;Audio;Player;
Terminal=false
EOF

    # Copy icon
    if [ -f "${PROJECT_ROOT}/static/icons/icon-512.png" ]; then
        cp "${PROJECT_ROOT}/static/icons/icon-512.png" \
           "$DEB_DIR/usr/share/icons/hicolor/256x256/apps/rusty-audio.png"
    fi

    # Create copyright file
    cat > "$DEB_DIR/usr/share/doc/rusty-audio/copyright" <<EOF
Format: https://www.debian.org/doc/packaging-manuals/copyright-format/1.0/
Upstream-Name: rusty-audio
Source: https://github.com/david-t-martel/rusty-audio

Files: *
Copyright: 2024 Rusty Audio Team
License: MIT or Apache-2.0
EOF

    # Create changelog
    cat > "$DEB_DIR/usr/share/doc/rusty-audio/changelog.Debian" <<EOF
rusty-audio (${VERSION}) unstable; urgency=medium

  * Release version ${VERSION}

 -- Rusty Audio Team <rusty-audio@example.com>  $(date -R)
EOF
    gzip -9 -n "$DEB_DIR/usr/share/doc/rusty-audio/changelog.Debian"

    # Set permissions
    chmod 755 "$DEB_DIR/usr/bin/rusty-audio"
    chmod 644 "$DEB_DIR/usr/share/applications/rusty-audio.desktop"

    # Build .deb
    DEB_PATH="${DIST_DIR}/rusty-audio_${VERSION}_amd64.deb"
    dpkg-deb --build "$DEB_DIR" "$DEB_PATH"

    if [ -f "$DEB_PATH" ]; then
        print_success ".deb package created: $DEB_PATH"
        print_success "Size: $(stat -c%s "$DEB_PATH" | numfmt --to=iec-i --suffix=B)"
    else
        print_warning ".deb creation failed"
    fi

    # Clean up
    rm -rf "$DEB_DIR"
else
    print_warning "dpkg-deb not found. Skipping .deb creation."
fi

# Create .rpm package (requires rpmbuild)
print_info "[7/7] Creating .rpm package..."

if command -v rpmbuild >/dev/null 2>&1; then
    RPM_BUILD_DIR="${DIST_DIR}/rpmbuild"
    mkdir -p "$RPM_BUILD_DIR"/{BUILD,RPMS,SOURCES,SPECS,SRPMS}

    # Create spec file
    cat > "$RPM_BUILD_DIR/SPECS/rusty-audio.spec" <<EOF
Name:           rusty-audio
Version:        ${VERSION}
Release:        1%{?dist}
Summary:        Professional audio player built in Rust

License:        MIT or Apache-2.0
URL:            https://github.com/david-t-martel/rusty-audio
Source0:        rusty-audio-${VERSION}.tar.gz

BuildRequires:  alsa-lib-devel
Requires:       alsa-lib

%description
Rusty Audio is a car-stereo-style audio player featuring:
- 8-band parametric equalizer
- Real-time spectrum analyzer
- Signal generator
- Audio recording
- Multiple themes

Built with Rust, egui, and wgpu for high performance.

%prep
# No prep needed for pre-built binary

%build
# No build needed for pre-built binary

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}%{_bindir}
mkdir -p %{buildroot}%{_datadir}/applications
mkdir -p %{buildroot}%{_datadir}/icons/hicolor/256x256/apps

install -m 755 ${DIST_BINARY} %{buildroot}%{_bindir}/rusty-audio

cat > %{buildroot}%{_datadir}/applications/rusty-audio.desktop <<'DESKTOP_EOF'
[Desktop Entry]
Type=Application
Name=Rusty Audio
Comment=Professional audio player built in Rust
Exec=rusty-audio
Icon=rusty-audio
Categories=AudioVideo;Audio;Player;
Terminal=false
DESKTOP_EOF

%files
%{_bindir}/rusty-audio
%{_datadir}/applications/rusty-audio.desktop

%changelog
* $(date "+%a %b %d %Y") Rusty Audio Team <rusty-audio@example.com> - ${VERSION}-1
- Release version ${VERSION}
EOF

    # Build RPM
    RPM_PATH=$(rpmbuild --define "_topdir $RPM_BUILD_DIR" \
                        -bb "$RPM_BUILD_DIR/SPECS/rusty-audio.spec" 2>&1 | \
               grep "Wrote:" | awk '{print $2}')

    if [ -f "$RPM_PATH" ]; then
        RPM_FINAL="${DIST_DIR}/$(basename "$RPM_PATH")"
        mv "$RPM_PATH" "$RPM_FINAL"
        print_success ".rpm package created: $RPM_FINAL"
        print_success "Size: $(stat -c%s "$RPM_FINAL" | numfmt --to=iec-i --suffix=B)"
    else
        print_warning ".rpm creation failed"
    fi

    # Clean up
    rm -rf "$RPM_BUILD_DIR"
else
    print_warning "rpmbuild not found. Skipping .rpm creation."
fi

# Generate checksums
print_info "Generating checksums..."
CHECKSUM_FILE="${DIST_DIR}/SHA256SUMS.txt"
rm -f "$CHECKSUM_FILE"

cd "$DIST_DIR"
for file in *.tar.gz *.AppImage *.deb *.rpm "$OUTPUT_NAME"; do
    if [ -f "$file" ]; then
        sha256sum "$file" >> "$CHECKSUM_FILE"
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
for file in "$DIST_DIR"/*.{tar.gz,AppImage,deb,rpm} "$DIST_BINARY"; do
    if [ -f "$file" ]; then
        size=$(stat -c%s "$file" | numfmt --to=iec-i --suffix=B)
        echo -e "${CYAN}  - $(basename "$file") ($size)${NC}"
    fi
done
echo ""
echo -e "${YELLOW}Next steps:${NC}"
echo -e "${NC}  1. Test the packages on target systems${NC}"
echo -e "${NC}  2. Upload to GitHub Releases${NC}"
echo -e "${NC}  3. Submit to package repositories (optional)${NC}"
echo ""
