#!/bin/sh
set -eu

REPO="odysa/portit"
BIN_NAME="portit"

usage() {
  echo "Usage: install.sh [options]"
  echo ""
  echo "Options:"
  echo "  -b, --bin-dir DIR   Install directory (default: ~/.local/bin)"
  echo "  -v, --version TAG   Install a specific version (default: latest)"
  echo "  -h, --help          Show this help"
}

get_target() {
  os="$(uname -s)"
  arch="$(uname -m)"

  case "$os" in
    Linux)  os_part="unknown-linux-gnu" ;;
    Darwin) os_part="apple-darwin" ;;
    *)      echo "Error: unsupported OS: $os" >&2; exit 1 ;;
  esac

  case "$arch" in
    x86_64|amd64)  arch_part="x86_64" ;;
    arm64|aarch64) arch_part="aarch64" ;;
    *)             echo "Error: unsupported architecture: $arch" >&2; exit 1 ;;
  esac

  echo "${arch_part}-${os_part}"
}

main() {
  bin_dir="$HOME/.local/bin"
  version=""

  while [ $# -gt 0 ]; do
    case "$1" in
      -b|--bin-dir) bin_dir="$2"; shift 2 ;;
      -v|--version) version="$2"; shift 2 ;;
      -h|--help)    usage; exit 0 ;;
      *)            echo "Unknown option: $1" >&2; usage >&2; exit 1 ;;
    esac
  done

  target="$(get_target)"

  if [ -z "$version" ]; then
    version="$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | cut -d'"' -f4)"
  fi

  if [ -z "$version" ]; then
    echo "Error: failed to determine latest version" >&2
    exit 1
  fi

  url="https://github.com/${REPO}/releases/download/${version}/${BIN_NAME}-${target}.tar.gz"

  echo "Installing ${BIN_NAME} ${version} (${target})..."

  tmpdir="$(mktemp -d)"
  trap 'rm -rf "$tmpdir"' EXIT

  curl -fsSL "$url" | tar xz -C "$tmpdir"

  mkdir -p "$bin_dir"

  if [ -w "$bin_dir" ]; then
    mv "$tmpdir/${BIN_NAME}" "$bin_dir/${BIN_NAME}"
  else
    echo "Elevated permissions required to install to ${bin_dir}"
    sudo mv "$tmpdir/${BIN_NAME}" "$bin_dir/${BIN_NAME}"
  fi

  echo "Installed ${BIN_NAME} to ${bin_dir}/${BIN_NAME}"

  # Check if install dir is in PATH
  case ":${PATH}:" in
    *":${bin_dir}:"*) ;;
    *)
      echo ""
      echo "Note: ${bin_dir} is not in your PATH. Add it with:"
      echo "  bash/zsh: export PATH=\"${bin_dir}:\$PATH\""
      echo "  fish:     set -Ua fish_user_paths ${bin_dir}"
      ;;
  esac
}

main "$@"
