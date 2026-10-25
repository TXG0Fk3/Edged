#!/usr/bin/env bash
set -ex

# ==============================================================================
# Edged - Arch Linux Container Build & Packaging Script
# ==============================================================================

VERSION="${VERSION:-v$(date -u +'%Y.%m.%d').1}"
CLEAN_VERSION="${CLEAN_VERSION:-$(date -u +'%Y.%m.%d').1}"

echo "==> Building Edged ${VERSION} inside Arch Linux environment..."

# 1. Update Arch Linux and install packaging toolchain
pacman -Syu --noconfirm rust cargo git zstd binutils desktop-file-utils cargo-generate-rpm wget file

# 2. Run automated test suite
cargo test --release

# 3. Build optimized release binary
cargo build --release --locked
strip target/release/edged

# Ensure output directory exists
mkdir -p dist

# 4. Standalone Binaries (executable without packaging)
cp target/release/edged dist/edged
cp target/release/edged dist/edged-standalone-x86_64

# 5. Linux Tarball (.tar.gz with automated installer)
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

# 6. Arch Linux Native Package (.pkg.tar.zst)
ARCH_DIR="arch_pkg"
mkdir -p "${ARCH_DIR}/usr/bin" \
         "${ARCH_DIR}/usr/share/applications" \
         "${ARCH_DIR}/usr/share/icons/hicolor/256x256/apps" \
         "${ARCH_DIR}/usr/share/icons/hicolor/scalable/apps" \
         "${ARCH_DIR}/usr/share/doc/edged" \
         "${ARCH_DIR}/usr/share/licenses/edged"

cp target/release/edged "${ARCH_DIR}/usr/bin/"
cp edged.desktop "${ARCH_DIR}/usr/share/applications/"
cp assets/edged.png "${ARCH_DIR}/usr/share/icons/hicolor/256x256/apps/edged.png"
cp assets/edged.svg "${ARCH_DIR}/usr/share/icons/hicolor/scalable/apps/edged.svg"
cp config.example.toml "${ARCH_DIR}/usr/share/doc/edged/"
cp README.md "${ARCH_DIR}/usr/share/doc/edged/"
cp LICENSE "${ARCH_DIR}/usr/share/doc/edged/"
cp LICENSE "${ARCH_DIR}/usr/share/licenses/edged/"

cat << EOF > "${ARCH_DIR}/.PKGINFO"
pkgname = edged
pkgbase = edged
pkgver = ${CLEAN_VERSION}-1
pkgdesc = High-quality PowerToys Screen Ruler replica for Wayland
url = https://github.com/TXG0Fk3/Edged
builddate = $(date +%s)
packager = TXG0Fk3 <https://github.com/TXG0Fk3/Edged>
size = $(du -sb "${ARCH_DIR}" | awk '{print $1}')
arch = x86_64
license = MIT
depend = glibc
optdepend = grim: Wayland screencopy screenshot utility
optdepend = hyprshot: Hyprland screenshot utility
optdepend = wl-clipboard: Wayland clipboard support
EOF

(cd "${ARCH_DIR}" && bsdtar --uid 0 --gid 0 -cf - .PKGINFO usr | zstd -c -T0 -19 > "../dist/edged-${CLEAN_VERSION}-1-x86_64.pkg.tar.zst")
rm -rf "${ARCH_DIR}"

# 7. Fedora / RHEL Package (.rpm via official cargo-generate-rpm on Arch)
cargo generate-rpm
cp target/generate-rpm/*.rpm "dist/edged-${CLEAN_VERSION}-1.x86_64.rpm"

# 8. Debian / Ubuntu Package (.deb via native ar + tar)
DEB_DIR="deb_pkg"
mkdir -p "${DEB_DIR}/DEBIAN" \
         "${DEB_DIR}/usr/bin" \
         "${DEB_DIR}/usr/share/applications" \
         "${DEB_DIR}/usr/share/icons/hicolor/256x256/apps" \
         "${DEB_DIR}/usr/share/icons/hicolor/scalable/apps" \
         "${DEB_DIR}/usr/share/doc/edged"

cp target/release/edged "${DEB_DIR}/usr/bin/"
cp edged.desktop "${DEB_DIR}/usr/share/applications/"
cp assets/edged.png "${DEB_DIR}/usr/share/icons/hicolor/256x256/apps/edged.png"
cp assets/edged.svg "${DEB_DIR}/usr/share/icons/hicolor/scalable/apps/edged.svg"
cp config.example.toml "${DEB_DIR}/usr/share/doc/edged/"
cp README.md "${DEB_DIR}/usr/share/doc/edged/"
cp LICENSE "${DEB_DIR}/usr/share/doc/edged/"

INSTALLED_SIZE=$(du -sk "${DEB_DIR}/usr" | awk '{print $1}')
cat << EOF > "${DEB_DIR}/DEBIAN/control"
Package: edged
Version: ${CLEAN_VERSION}
Section: utils
Priority: optional
Architecture: amd64
Installed-Size: ${INSTALLED_SIZE}
Maintainer: TXG0Fk3 <txg0fk3@proton.me>
Depends: libc6 (>= 2.31)
Recommends: grim, wl-clipboard
Homepage: https://github.com/TXG0Fk3/Edged
Description: High-quality PowerToys Screen Ruler replica for Wayland
 Fast, pixel-accurate screen measuring tool for Wayland desktops.
EOF

echo "2.0" > "${DEB_DIR}/debian-binary"
(cd "${DEB_DIR}/DEBIAN" && tar -czf ../control.tar.gz control)
(cd "${DEB_DIR}" && tar -czf data.tar.gz usr)
ar rcs "dist/edged_${CLEAN_VERSION}_amd64.deb" "${DEB_DIR}/debian-binary" "${DEB_DIR}/control.tar.gz" "${DEB_DIR}/data.tar.gz"
rm -rf "${DEB_DIR}"

# 9. Universal AppImage (.AppImage)
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

if [ ! -f "/usr/local/bin/appimagetool" ]; then
    wget -q -O /usr/local/bin/appimagetool "https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-x86_64.AppImage"
    chmod +x /usr/local/bin/appimagetool
fi

ARCH=x86_64 /usr/local/bin/appimagetool --appimage-extract-and-run "${APPDIR}" "dist/edged-${VERSION}-x86_64.AppImage"
rm -rf "${APPDIR}"

# 10. Generate Checksums
(cd dist && sha256sum * > SHA256SUMS.txt)

echo "==> Packaging complete! Generated assets in dist/:"
ls -lh dist/
