#!/bin/sh
set -eu

REPO="LuMiSxh/typos"
INSTALL_DIR="${TYPOS_INSTALL_DIR:-$HOME/.local/bin}"

main() {
    os="$(detect_os)"
    arch="$(detect_arch)"
    target="${arch}-${os}"

    latest="$(fetch_latest_tag)"
    if [ -z "$latest" ]; then
        err "could not determine latest release"
    fi

    url="https://github.com/${REPO}/releases/download/${latest}/typos-${target}.tar.gz"
    say "downloading typos ${latest} for ${target}..."

    tmpdir="$(mktemp -d)"
    trap 'rm -rf "$tmpdir"' EXIT

    if command -v curl >/dev/null 2>&1; then
        curl -fsSL "$url" -o "$tmpdir/typos.tar.gz"
    elif command -v wget >/dev/null 2>&1; then
        wget -qO "$tmpdir/typos.tar.gz" "$url"
    else
        err "need curl or wget"
    fi

    tar xzf "$tmpdir/typos.tar.gz" -C "$tmpdir"
    mkdir -p "$INSTALL_DIR"
    mv "$tmpdir/typos" "$INSTALL_DIR/typos"
    chmod +x "$INSTALL_DIR/typos"
    say "installed typos to ${INSTALL_DIR}/typos"

    if ! echo "$PATH" | tr ':' '\n' | grep -qx "$INSTALL_DIR"; then
        say ""
        say "add to your PATH:"
        say "  export PATH=\"${INSTALL_DIR}:\$PATH\""
    fi
}

detect_os() {
    case "$(uname -s)" in
        Linux*)  echo "unknown-linux-gnu" ;;
        Darwin*) echo "apple-darwin" ;;
        *)       err "unsupported OS — use install.ps1 on Windows" ;;
    esac
}

detect_arch() {
    case "$(uname -m)" in
        x86_64|amd64)  echo "x86_64" ;;
        arm64|aarch64) echo "aarch64" ;;
        *)             err "unsupported architecture: $(uname -m)" ;;
    esac
}

fetch_latest_tag() {
    if command -v curl >/dev/null 2>&1; then
        curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
            | grep '"tag_name"' | head -1 | sed 's/.*"tag_name": *"\([^"]*\)".*/\1/'
    else
        wget -qO- "https://api.github.com/repos/${REPO}/releases/latest" \
            | grep '"tag_name"' | head -1 | sed 's/.*"tag_name": *"\([^"]*\)".*/\1/'
    fi
}

say() { printf '%s\n' "$1"; }
err() { say "error: $1" >&2; exit 1; }

main
