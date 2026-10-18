#!/usr/bin/env bash
# NportX installer — https://github.com/nxt3r/nportx
set -euo pipefail

REPO="nxt3r/nportx"
BIN="nportx"

# Detect OS
OS="$(uname -s)"
case "$OS" in
  Linux*)  OS="unknown-linux-gnu" ;;
  Darwin*) OS="apple-darwin" ;;
  *)       echo "❌ Unsupported OS: $OS" >&2; exit 1 ;;
esac

# Detect arch
ARCH="$(uname -m)"
case "$ARCH" in
  x86_64|amd64)  ARCH="x86_64" ;;
  arm64|aarch64) ARCH="aarch64" ;;
  *)             echo "❌ Unsupported arch: $ARCH" >&2; exit 1 ;;
esac

TARGET="${ARCH}-${OS}"
echo "🔍 Detected target: $TARGET"

# Resolve latest version from GitHub API (no jq needed)
API="https://api.github.com/repos/${REPO}/releases/latest"
VERSION="$(curl -fsSL "$API" | grep -o '"tag_name": *"[^"]*"' | head -n1 | cut -d'"' -f4)"
if [ -z "${VERSION:-}" ]; then
  echo "❌ Could not determine latest version" >&2
  exit 1
fi
echo "📦 Latest version: $VERSION"

# Pick install dir: /usr/local/bin if writable, else ~/.local/bin
if [ -w /usr/local/bin ] || [ "$(id -u)" = "0" ]; then
  INSTALL_DIR="/usr/local/bin"
else
  INSTALL_DIR="$HOME/.local/bin"
  mkdir -p "$INSTALL_DIR"
fi
echo "📂 Installing to: $INSTALL_DIR"

# Download + extract
URL="https://github.com/${REPO}/releases/download/${VERSION}/${BIN}-${TARGET}.tar.gz"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

echo "⬇️  Downloading $URL"
curl -fsSL "$URL" -o "$TMP/nportx.tar.gz"
tar xzf "$TMP/nportx.tar.gz" -C "$TMP"
install -m 755 "$TMP/$BIN" "$INSTALL_DIR/$BIN"

echo "✅ Installed $BIN $VERSION to $INSTALL_DIR/$BIN"

# Nudge about PATH if needed
case ":$PATH:" in
  *":$INSTALL_DIR:"*) ;;
  *) echo "⚠️  Add to your shell profile: export PATH=\"$INSTALL_DIR:\$PATH\"" ;;
esac