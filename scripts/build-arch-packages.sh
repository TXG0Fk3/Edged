#!/usr/bin/env bash
set -ex

# ==============================================================================
# Edged - Arch Linux Build & Distro Packaging Script
# ==============================================================================

# Ensure version variables exist
if [ -z "$VERSION" ]; then
    TODAY=$(date -u +'%Y.%m%d')
    TODAY_RELEASES=$(git tag -l "v${TODAY}*" 2>/dev/null | wc -l || echo 0)
    RELEASE_COUNT=$((TODAY_RELEASES + 1))
    VERSION="v${TODAY}.${RELEASE_COUNT}"
    CLEAN_VERSION="${TODAY}.${RELEASE_COUNT}"
fi

echo "==> Building Edged ${VERSION} (${CLEAN_VERSION}) on Arch Linux..."

# Update Cargo.toml version string for consistent packaging across all formats
sed -i "s/^version = \".*\"/version = \"${CLEAN_VERSION}\"/" Cargo.toml

# 1. Run automated test suite
cargo test --release

# 2. Build optimized release binary
cargo build --release
strip target/release/edged

# Prepare output directory
mkdir -p dist

# 3. Standalone Executables (Ready to chmod +x and run)
cp target/release/edged dist/edged
cp target/release/edged dist/edged-standalone-x86_64

# 4. Linux Tarball (.tar.gz with automated installer)
TAR_DIR="edged-${VERSION}-x86_64"
mkdir -p "${TAR_DIR}/assets"
cp target/release/edged "${TAR_DIR}/"
cp install.sh "${TAR_DIR}/"
cp edged.desktop "${TAR_DIR}/"
cp config.example.toml "${TAR_DIR}/"
cp README.md "${TAR_DIR}/"
cp LICENSE "${TAR_DIR}/"
cp assets/* "${TAR_DIR}/assets/"
tar -czvf "dist/edged-${VERSION}-x86_64-linux.tar.gz" "${TAR_DIR}"
rm -rf "${TAR_DIR}"

# 5. Arch Linux Native Package (.pkg.tar.zst via canonical makepkg)
cat << EOF > PKGBUILD
# Maintainer: TXG0Fk3 <https://github.com/TXG0Fk3/Edged>
pkgname=edged
pkgver=${CLEAN_VERSION}
pkgrel=1
pkgdesc="High-quality PowerToys Screen Ruler replica natively built for Wayland"
arch=('x86_64')
url="https://github.com/TXG0Fk3/Edged"
license=('MIT')
depends=('glibc' 'gcc-libs')
optdepends=(
    'grim: Wayland screencopy screenshot utility'
    'hyprshot: Hyprland screenshot utility'
    'wl-clipboard: Wayland clipboard support'
)
options=('!debug')

package() {
    install -Dm755 "\${srcdir}/../target/release/edged" "\${pkgdir}/usr/bin/edged"
    install -Dm644 "\${srcdir}/../edged.desktop" "\${pkgdir}/usr/share/applications/edged.desktop"
    install -Dm644 "\${srcdir}/../assets/edged.png" "\${pkgdir}/usr/share/icons/hicolor/256x256/apps/edged.png"
    install -Dm644 "\${srcdir}/../assets/edged.svg" "\${pkgdir}/usr/share/icons/hicolor/scalable/apps/edged.svg"
    install -Dm644 "\${srcdir}/../config.example.toml" "\${pkgdir}/usr/share/doc/edged/config.example.toml"
    install -Dm644 "\${srcdir}/../README.md" "\${pkgdir}/usr/share/doc/edged/README.md"
    install -Dm644 "\${srcdir}/../LICENSE" "\${pkgdir}/usr/share/licenses/edged/LICENSE"
}
EOF

# If running as root, run makepkg as the builder user (makepkg refuses root)
if [ "$(id -u)" -eq 0 ] && id "user" &>/dev/null; then
    chown -R user:user .
    sudo -u user makepkg -f --noconfirm
else
    makepkg -f --noconfirm
fi
cp edged-${CLEAN_VERSION}-1-x86_64.pkg.tar.zst dist/
rm -f PKGBUILD edged-${CLEAN_VERSION}-1-x86_64.pkg.tar.zst
rm -rf src pkg

# 6. Debian / Ubuntu Package (.deb via official cargo-deb)
cargo deb --no-build
cp target/debian/*.deb "dist/edged_${CLEAN_VERSION}_amd64.deb"

# 7. Fedora / RHEL Package (.rpm via official cargo-generate-rpm)
cargo generate-rpm
cp target/generate-rpm/*.rpm "dist/edged-${CLEAN_VERSION}-1.x86_64.rpm"

# 8. Universal AppImage (.AppImage via appimagetool extracted runtime)
APPDIR="AppDir"
mkdir -p "${APPDIR}/usr/bin" \
         "${APPDIR}/usr/share/applications" \
         "${APPDIR}/usr/share/icons/hicolor/256x256/apps"

cp target/release/edged "${APPDIR}/usr/bin/"
cp edged.desktop "${APPDIR}/"
cp edged.desktop "${APPDIR}/usr/share/applications/"
cp assets/edged.png "${APPDIR}/"
cp assets/edged.png "${APPDIR}/usr/share/icons/hicolor/256x256/apps/edged.png"

cat << 'EOF' > "${APPDIR}/AppRun"
#!/usr/bin/env sh
SELF=$(readlink -f "$0")
HERE=${SELF%/*}
export PATH="${HERE}/usr/bin:${PATH}"
exec "${HERE}/usr/bin/edged" "$@"
EOF
chmod +x "${APPDIR}/AppRun"

# Download and extract appimagetool to work cleanly in containers without FUSE
if [ ! -d "/tmp/appimagetool-extracted" ]; then
    wget -q -O /tmp/appimagetool.AppImage "https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-x86_64.AppImage"
    chmod +x /tmp/appimagetool.AppImage
    (cd /tmp && ./appimagetool.AppImage --appimage-extract && mv squashfs-root appimagetool-extracted && rm -f appimagetool.AppImage)
fi

ARCH=x86_64 /tmp/appimagetool-extracted/AppRun "${APPDIR}" "dist/edged-${VERSION}-x86_64.AppImage"
rm -rf "${APPDIR}"

# 9. Checksums
(cd dist && sha256sum * > SHA256SUMS.txt)

echo "==> Build and packaging completed successfully!"
ls -lh dist/
