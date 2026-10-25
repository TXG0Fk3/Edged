#!/usr/bin/env bash
# ==============================================================================
# Edged - Nix Flake Sources Auto-Updater
# Automatically updates nix/sources.json to point to the latest GitHub release.
# ==============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
SOURCES_FILE="${SCRIPT_DIR}/sources.json"
REPO="TXG0Fk3/Edged"

TARGET_TAG="${1:-}"

if [ -z "${TARGET_TAG}" ]; then
  echo "==> Fetching latest release tag from GitHub API for ${REPO}..."
  LATEST_JSON=$(curl -sSL -H "Accept: application/vnd.github+json" "https://api.github.com/repos/${REPO}/releases/latest")
  TARGET_TAG=$(echo "${LATEST_JSON}" | grep -Po '"tag_name":\s*"\K[^"]+' || true)
fi

if [ -z "${TARGET_TAG}" ]; then
  echo "Error: Could not determine latest release tag." >&2
  exit 1
fi

CLEAN_VERSION="${TARGET_TAG#v}"
TAR_NAME="edged-${TARGET_TAG}-x86_64-linux.tar.gz"
DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${TARGET_TAG}/${TAR_NAME}"

echo "==> Target Release: ${TARGET_TAG} (Clean: ${CLEAN_VERSION})"
echo "==> Download URL:   ${DOWNLOAD_URL}"

# Attempt to fetch sha256 from SHA256SUMS.txt on release assets first
SHA256=""
SHA_URL="https://github.com/${REPO}/releases/download/${TARGET_TAG}/SHA256SUMS.txt"
if SHA_CONTENT=$(curl -sSL -f "${SHA_URL}" 2>/dev/null); then
  SHA256=$(echo "${SHA_CONTENT}" | grep -E "${TAR_NAME}$" | awk '{print $1}' || true)
fi

# Fallback: calculate hash by downloading or streaming tarball
if [ -z "${SHA256}" ]; then
  echo "==> SHA256SUMS.txt not found or did not list ${TAR_NAME}. Streaming hash directly..."
  if command -v nix-prefetch-url >/dev/null 2>&1; then
    SHA256=$(nix-prefetch-url --type sha256 "${DOWNLOAD_URL}")
  else
    SHA256=$(curl -sSL -f "${DOWNLOAD_URL}" | sha256sum | awk '{print $1}')
  fi
fi

if [ -z "${SHA256}" ]; then
  echo "Error: Failed to obtain SHA-256 hash for ${TAR_NAME}." >&2
  exit 1
fi

echo "==> SHA256:         ${SHA256}"

cat << EOF > "${SOURCES_FILE}"
{
  "version": "${TARGET_TAG}",
  "clean_version": "${CLEAN_VERSION}",
  "url": "${DOWNLOAD_URL}",
  "sha256": "${SHA256}"
}
EOF

echo "==> Successfully updated ${SOURCES_FILE} to ${TARGET_TAG}!"
