#!/usr/bin/env bash

set -e

REPO="hitblast/cutler"
BINARY="cutler"
INSTALL_DIR="/usr/local/bin"
OS="$(uname -s)"
ARCH="$(uname -m)"

# Only macOS is supported
if [[ "$OS" != "Darwin" ]]; then
  echo "❌ cutler only supports macOS. Detected: $OS"
  exit 1
fi

# Determine ARCH_PREFIX based on architecture
if [[ "$ARCH" == "x86_64" ]]; then
  ARCH_PREFIX="x86_64"
elif [[ "$ARCH" == "arm64" ]]; then
  ARCH_PREFIX="arm64"
else
  echo "❌ Unsupported architecture: $ARCH"
  exit 1
fi

# Find latest release tag
LATEST_TAG=$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
if [[ -z "$LATEST_TAG" ]]; then
  echo "❌ Could not determine latest cutler release."
  exit 1
fi

# Compose asset name
ASSET="cutler-$LATEST_TAG-darwin-$ARCH_PREFIX.zip"
ASSET_URL="https://github.com/$REPO/releases/download/$LATEST_TAG/$ASSET"

echo "⬇️  Downloading $ASSET_URL ..."
TMPDIR=$(mktemp -d)
cd "$TMPDIR"
if ! curl -fsSL -O "$ASSET_URL"; then
  echo "❌ Failed to download $ASSET_URL"
  exit 1
fi

echo "📦 Unzipping..."
if ! unzip -q "$ASSET"; then
  echo "❌ Failed to unzip $ASSET"
  exit 1
fi

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
if ! sudo mkdir -p "$INSTALL_DIR"; then
  echo "❌ Failed to create install directory $INSTALL_DIR"
  exit 1
fi
if ! sudo cp "$BIN_PATH" "$INSTALL_DIR/$BINARY"; then
  echo "❌ Failed to copy binary to $INSTALL_DIR"
  exit 1
fi
if ! sudo chmod +x "$INSTALL_DIR/$BINARY"; then
  echo "❌ Failed to set executable permissions on $INSTALL_DIR/$BINARY"
  exit 1
fi

echo "✅ cutler installed to $INSTALL_DIR/$BINARY"

# Check if it's on PATH
if ! command -v cutler >/dev/null 2>&1; then
  echo "⚠️  $INSTALL_DIR is not on your PATH."
  echo "   Add this line to your shell profile:"
  echo "     export PATH=\"$INSTALL_DIR:\$PATH\""
fi

echo
echo "🎉 Run 'cutler --help' or 'man cutler' to get started!"
