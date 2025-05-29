#!/usr/bin/env bash

set -e

REPO="hitblast/cutler"
BINARY="cutler"
INSTALL_DIR="/usr/local/bin"
MAN_DIR="/usr/local/share/man/man1"
OS="$(uname -s)"
ARCH="$(uname -m)"

# Only macOS is supported
if [[ "$OS" != "Darwin" ]]; then
  echo "❌ cutler only supports macOS. Detected: $OS"
  exit 1
fi

# For now, only arm64 is supported. Though, I plan to add support for x86 builds sometime soon.
if [[ "$ARCH" == "x86_64" ]]; then
  echo "❌ Looks like your Mac is running on x86. You may opt for compiling the program yourself. Learn more: https://github.com/hitblast/cutler"
  exit 1
fi

# Find latest release tag
LATEST_TAG=$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
if [[ -z "$LATEST_TAG" ]]; then
  echo "❌ Could not determine latest cutler release."
  exit 1
fi

# Compose asset name
ASSET="cutler-$LATEST_TAG-darwin-arm64.zip"
ASSET_URL="https://github.com/$REPO/releases/download/$LATEST_TAG/$ASSET"

echo "⬇️  Downloading $ASSET_URL ..."
TMPDIR=$(mktemp -d)
cd "$TMPDIR"
curl -fsSL -O "$ASSET_URL"

echo "📦 Unzipping..."
unzip -q "$ASSET"

# Find the cutler binary inside the zip (usually in bin/)
if [[ -f "bin/cutler" ]]; then
  BIN_PATH="bin/cutler"
elif [[ -f "cutler" ]]; then
  BIN_PATH="cutler"
else
  echo "❌ Could not find cutler binary in the archive."
  exit 1
fi

# Remove quarantine attribute (macOS security)
xattr -d com.apple.quarantine "$BIN_PATH" 2>/dev/null || true

# Install to /usr/local/bin (may require sudo)
echo "🔒 Installing to $INSTALL_DIR (may require sudo)..."
sudo mkdir -p "$INSTALL_DIR"
sudo cp "$BIN_PATH" "$INSTALL_DIR/$BINARY"
sudo chmod +x "$INSTALL_DIR/$BINARY"

# Install manpage if present
if [[ -f "man/man1/cutler.1" ]]; then
  echo "📖 Installing manpage to $MAN_DIR (may require sudo)..."
  sudo mkdir -p "$MAN_DIR"
  sudo cp "man/man1/cutler.1" "$MAN_DIR/cutler.1"
  sudo chmod 644 "$MAN_DIR/cutler.1"
else
  echo "⚠️  Manpage not found in the archive. Skipping manpage installation."
fi

echo "✅ cutler installed to $INSTALL_DIR/$BINARY"
echo "✅ manpage installed to $MAN_DIR/cutler.1"

# Check if it's on PATH
if ! command -v cutler >/dev/null 2>&1; then
  echo "⚠️  $INSTALL_DIR is not on your PATH."
  echo "   Add this line to your shell profile:"
  echo "     export PATH=\"$INSTALL_DIR:\$PATH\""
fi

echo
echo "🎉 Run 'cutler --help' or 'man cutler' to get started!"
