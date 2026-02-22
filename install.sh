#!/bin/sh
set -e

REPO="odysa/portit"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"

get_target() {
  os="$(uname -s)"
  arch="$(uname -m)"

  case "$os" in
    Linux)  os_part="unknown-linux-gnu" ;;
    Darwin) os_part="apple-darwin" ;;
    *)      echo "Unsupported OS: $os" >&2; exit 1 ;;
  esac

  case "$arch" in
    x86_64|amd64)  arch_part="x86_64" ;;
    arm64|aarch64) arch_part="aarch64" ;;
    *)             echo "Unsupported architecture: $arch" >&2; exit 1 ;;
  esac

  echo "${arch_part}-${os_part}"
}

main() {
  target="$(get_target)"

  if [ -n "$1" ]; then
    version="$1"
  else
    version="$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | cut -d'"' -f4)"
  fi

  if [ -z "$version" ]; then
    echo "Failed to determine latest version" >&2
    exit 1
  fi

  url="https://github.com/${REPO}/releases/download/${version}/portit-${target}.tar.gz"

  echo "Installing portit ${version} (${target})..."

  tmpdir="$(mktemp -d)"
  trap 'rm -rf "$tmpdir"' EXIT

  curl -fsSL "$url" | tar xz -C "$tmpdir"

  if [ -w "$INSTALL_DIR" ]; then
    mv "$tmpdir/portit" "$INSTALL_DIR/portit"
  else
    sudo mv "$tmpdir/portit" "$INSTALL_DIR/portit"
  fi

  echo "Installed portit to ${INSTALL_DIR}/portit"
}

main "$@"
