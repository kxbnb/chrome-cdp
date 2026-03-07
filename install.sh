#!/bin/sh
set -e

REPO="kilospark/webact"
BINARY="webact-mcp"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"

# Detect OS and architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Darwin) PLATFORM="darwin" ;;
  Linux)  PLATFORM="linux" ;;
  *)      echo "Unsupported OS: $OS"; exit 1 ;;
esac

case "$ARCH" in
  arm64|aarch64) ARCH_NAME="arm64" ;;
  x86_64|amd64)  ARCH_NAME="x64" ;;
  *)              echo "Unsupported architecture: $ARCH"; exit 1 ;;
esac

ASSET="${BINARY}-${PLATFORM}-${ARCH_NAME}"

# Get latest release tag if not specified
if [ -z "$VERSION" ]; then
  VERSION="$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | cut -d'"' -f4)"
fi

if [ -z "$VERSION" ]; then
  echo "Failed to determine latest version"
  exit 1
fi

URL="https://github.com/${REPO}/releases/download/${VERSION}/${ASSET}.tar.gz"

echo "Installing ${BINARY} ${VERSION} (${PLATFORM}/${ARCH_NAME})..."

TMPDIR="$(mktemp -d)"
trap 'rm -rf "$TMPDIR"' EXIT

curl -fsSL "$URL" | tar xz -C "$TMPDIR"

if [ -w "$INSTALL_DIR" ]; then
  mv "$TMPDIR/${ASSET}" "${INSTALL_DIR}/${BINARY}"
else
  sudo mv "$TMPDIR/${ASSET}" "${INSTALL_DIR}/${BINARY}"
fi

chmod +x "${INSTALL_DIR}/${BINARY}"

echo "Installed ${BINARY} to ${INSTALL_DIR}/${BINARY}"
echo ""
echo "Add to your MCP client config:"
echo '  { "command": "webact-mcp" }'
