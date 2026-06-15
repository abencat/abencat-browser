#!/usr/bin/env bash
# Download the CloakBrowser fingerprint kernel (© CloakHQ) for the current OS.
# The kernel is NOT redistributed in this repo; it is fetched from the official
# CloakBrowser releases. https://github.com/CloakHQ/CloakBrowser
set -euo pipefail

VERSION="${CLOAKBROWSER_VERSION:-chromium-v146.0.7680.177.5}"
DEST="${1:-/opt/cloakbrowser}"
BASE="https://github.com/CloakHQ/CloakBrowser/releases/download/${VERSION}"

case "$(uname -s)-$(uname -m)" in
  Linux-x86_64)  ASSET="cloakbrowser-linux-x64.tar.gz" ;;
  Linux-aarch64) ASSET="cloakbrowser-linux-arm64.tar.gz" ;;
  Darwin-*)      ASSET="cloakbrowser-macos.tar.gz" ;;
  *) echo "Unsupported platform: $(uname -s)-$(uname -m)"; exit 1 ;;
esac

echo "Downloading ${ASSET} (${VERSION}) → ${DEST}"
mkdir -p "${DEST}"
curl -fSL "${BASE}/${ASSET}" -o "/tmp/${ASSET}"
tar xzf "/tmp/${ASSET}" -C "${DEST}"
chmod +x "${DEST}/chrome" 2>/dev/null || true
rm -f "/tmp/${ASSET}"

echo ""
echo "Done. Point the app at the kernel:"
echo "  export CLOAKBROWSER_BINARY_PATH=${DEST}/chrome"
