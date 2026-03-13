#!/bin/sh
set -e

REPO="kilospark/webact"
BINARY="webact"

# Default: user-local install. Use --global for /usr/local/bin.
if [ "$1" = "--global" ] || [ "$INSTALL_DIR" = "/usr/local/bin" ]; then
  INSTALL_DIR="/usr/local/bin"
elif [ -n "$INSTALL_DIR" ]; then
  : # custom INSTALL_DIR from env
else
  INSTALL_DIR="$HOME/.local/bin"
fi

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

URL="https://webact.space/download/${VERSION}/${ASSET}.tar.gz"

echo ""
echo "  webact ${VERSION}"
echo "  ${PLATFORM}/${ARCH_NAME}"
echo ""

TMPDIR="$(mktemp -d)"
trap 'rm -rf "$TMPDIR"' EXIT

curl -fsSL "$URL" | tar xz -C "$TMPDIR"

mkdir -p "$INSTALL_DIR"

if [ -w "$INSTALL_DIR" ]; then
  mv "$TMPDIR/${ASSET}" "${INSTALL_DIR}/${BINARY}"
else
  sudo mv "$TMPDIR/${ASSET}" "${INSTALL_DIR}/${BINARY}"
fi

chmod +x "${INSTALL_DIR}/${BINARY}"

echo "Installed ${BINARY} to ${INSTALL_DIR}/${BINARY}"

# Clean up old webact-mcp binary if present
for dir in /usr/local/bin "$HOME/.local/bin"; do
  if [ -x "$dir/webact-mcp" ]; then
    if [ -w "$dir" ]; then
      rm -f "$dir/webact-mcp"
      echo "Removed old $dir/webact-mcp (now use: webact mcp)"
    fi
  fi
done

# Auto-add install dir to PATH in shell rc if needed
case ":$PATH:" in
  *":${INSTALL_DIR}:"*) ;;
  *)
    PATH_LINE="export PATH=\"${INSTALL_DIR}:\$PATH\""
    if [ -f "$HOME/.zshrc" ]; then
      RC_FILE="$HOME/.zshrc"
    elif [ -f "$HOME/.bashrc" ]; then
      RC_FILE="$HOME/.bashrc"
    elif [ -f "$HOME/.bash_profile" ]; then
      RC_FILE="$HOME/.bash_profile"
    else
      RC_FILE=""
    fi
    if [ -n "$RC_FILE" ]; then
      if ! grep -q "${INSTALL_DIR}" "$RC_FILE" 2>/dev/null; then
        echo "" >> "$RC_FILE"
        echo "# Added by webact installer" >> "$RC_FILE"
        echo "$PATH_LINE" >> "$RC_FILE"
        echo "Added ${INSTALL_DIR} to PATH in ${RC_FILE}"
      fi
    else
      echo "WARNING: ${INSTALL_DIR} is not in your PATH. Add it with:"
      echo "  $PATH_LINE"
    fi
    export PATH="${INSTALL_DIR}:$PATH"
    ;;
esac

# Configure MCP clients (handled natively by the binary)
"${INSTALL_DIR}/${BINARY}" setup
