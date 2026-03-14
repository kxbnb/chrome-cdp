#!/bin/sh
set -e

BINARY="webact"
OLD_BINS="webact-mcp webact-rs"

echo "Uninstalling ${BINARY}..."

REMOVED=""

# Determine which dirs to clean
if [ "$1" = "--global" ]; then
  DIRS="/usr/local/bin $HOME/.local/bin"
else
  DIRS="$HOME/.local/bin"
fi

# --- Remove MCP configs + data (handled by binary) ---

if command -v "$BINARY" >/dev/null 2>&1; then
  "$BINARY" uninstall
  REMOVED="${REMOVED}configs, "
fi

# --- Remove binaries ---

for dir in $DIRS; do
  for bin in $BINARY $OLD_BINS; do
    if [ -x "$dir/${bin}" ]; then
      if [ -w "$dir" ]; then
        rm "$dir/${bin}"
      else
        sudo rm "$dir/${bin}"
      fi
      echo "Removed $dir/${bin}"
      REMOVED="${REMOVED}${bin}, "
    fi
  done
done

# --- Remove PATH entry from shell rc ---

for rc in "$HOME/.zshrc" "$HOME/.bashrc" "$HOME/.bash_profile"; do
  if [ -f "$rc" ]; then
    for marker in "# Added by webact installer" "# Added by webact-mcp installer"; do
      if grep -q "$marker" "$rc" 2>/dev/null; then
        sed -i.bak "/$marker/{ N; d; }" "$rc" 2>/dev/null || \
          sed -i '' "/$marker/{ N; d; }" "$rc"
        rm -f "${rc}.bak"
        echo "Removed PATH entry ($marker) from $rc"
        REMOVED="${REMOVED}PATH, "
      fi
    done
  fi
done

echo ""
if [ -z "$REMOVED" ]; then
  echo "Nothing to uninstall — ${BINARY} was not found."
else
  echo "Done! ${BINARY} has been uninstalled."
fi
