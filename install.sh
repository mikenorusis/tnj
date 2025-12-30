#!/usr/bin/env bash
set -euo pipefail

# Easy one-command installer for tnj
# Usage:
#   curl -sSfL https://raw.githubusercontent.com/mikenorusis/tnj/main/install.sh | sh
#   or (recommended, from release assets)
#   curl -L https://github.com/mikenorusis/tnj/releases/latest/download/install.sh | sh
#
# For a specific version:
#   ... | sh -s -- --version v0.1.0

REPO="mikenorusis/tnj"
BIN_NAME="tnj"

# Parse optional --version flag
VERSION="latest"
while [ $# -gt 0 ]; do
  case "$1" in
    --version)
      VERSION="$2"
      shift 2
      ;;
    *)
      echo "Unknown option: $1"
      exit 1
      ;;
  esac
done

if [ "$VERSION" = "latest" ]; then
  TAG=$(curl -s "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
else
  TAG="$VERSION"
fi

if [ -z "$TAG" ]; then
  echo "Could not find release tag. Check your internet connection or the repository releases."
  exit 1
fi

echo "Installing $BIN_NAME $TAG ..."

# Detect platform
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$OS" in
  darwin)
    case "$ARCH" in
      x86_64)  ASSET="tnj-macos-x86_64" ;;
      arm64|aarch64)   ASSET="tnj-macos-aarch64" ;;
      *) echo "Unsupported macOS architecture: $ARCH"; exit 1 ;;
    esac
    ;;
  linux)
    # Add Linux support later when you build Linux binaries
    echo "Linux binaries not yet supported by this installer. Please build from source or download manually."
    exit 1
    ;;
  mingw* | cygwin* | msys*)
    echo "Windows is not supported via this script. Please download tnj-windows-x64.exe from the release and add it to your PATH."
    exit 1
    ;;
  *)
    echo "Unsupported OS: $OS"
    exit 1
    ;;
esac

URL="https://github.com/$REPO/releases/download/$TAG/$ASSET"

TMP_DIR=$(mktemp -d)
curl -L -o "$TMP_DIR/$BIN_NAME" "$URL"
chmod +x "$TMP_DIR/$BIN_NAME"

# Install to /usr/local/bin (requires sudo on most systems)
INSTALL_PATH="/usr/local/bin/$BIN_NAME"
echo "Moving binary to $INSTALL_PATH (requires sudo)..."
sudo mv "$TMP_DIR/$BIN_NAME" "$INSTALL_PATH"

# Clean up temp dir
rm -rf "$TMP_DIR"

echo ""
echo "$BIN_NAME $TAG successfully installed!"
echo "You can now run it with: $BIN_NAME"
echo "To upgrade in the future, just re-run this installer."

