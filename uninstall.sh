#!/usr/bin/env bash

set -e

BINARY="cutler"
INSTALL_DIR="/usr/local/bin"
MANPAGE_DIR="/usr/local/share/man/man1"

echo "🔒 Uninstalling $BINARY binary from $INSTALL_DIR..."
if [ -f "$INSTALL_DIR/$BINARY" ]; then
  sudo rm "$INSTALL_DIR/$BINARY"
  echo "✅ Removed $INSTALL_DIR/$BINARY"
else
  echo "⚠️  $INSTALL_DIR/$BINARY not found"
fi

echo "📖 Uninstalling manpage from $MANPAGE_DIR..."

if [ -f "$MANPAGE_DIR/$BINARY.1.gz" ]; then
  sudo rm "$MANPAGE_DIR/$BINARY.1.gz"
  echo "✅ Removed $MANPAGE_DIR/$BINARY.1.gz"
else
  echo "⚠️ Compressed manpage for $BINARY not found in $MANPAGE_DIR"
fi

if [ -f "$MANPAGE_DIR/$BINARY.1" ]; then
  sudo rm "$MANPAGE_DIR/$BINARY.1"
  echo "✅ Removed $MANPAGE_DIR/$BINARY.1"
else
  echo "⚠️ Manpage for $BINARY not found in $MANPAGE_DIR"
fi

echo "🎉 Uninstallation complete."
echo "Note that the .cutler_snapshot file in the home directory will not be removed.\nYou may remove it yourself if you wish."
