#!/bin/sh

set -eu

REPOSITORY="Akram012388/codex-akram"
TARGET="aarch64-apple-darwin"
ASSET="codex-akram-package-${TARGET}.tar.gz"
CHECKSUM_ASSET="codex-akram-package_SHA256SUMS"
REQUESTED_RELEASE="${CODEX_AKRAM_RELEASE:-latest}"
BIN_DIR="${CODEX_AKRAM_BIN_DIR:-$HOME/.local/bin}"
AKRAM_HOME="${CODEX_AKRAM_HOME:-$HOME/.codex-akram}"
RELEASES_DIR="$AKRAM_HOME/packages/standalone/releases"
CURRENT_LINK="$AKRAM_HOME/packages/standalone/current"

fail() {
  printf 'codex-akram installer: %s\n' "$1" >&2
  exit 1
}

fetch() {
  url="$1"
  output="$2"
  if command -v curl >/dev/null 2>&1; then
    curl -fsSL "$url" -o "$output"
  elif command -v wget >/dev/null 2>&1; then
    wget -q "$url" -O "$output"
  else
    fail "curl or wget is required"
  fi
}

validate_version() {
  printf '%s\n' "$1" | grep -Eq '^[0-9]+\.[0-9]+\.[0-9]+-ak\.[0-9]+\.[0-9]+$' \
    || fail "invalid fork release version: $1"
}

sha256_file() {
  file="$1"
  if command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$file" | awk '{print $1}'
  elif command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$file" | awk '{print $1}'
  elif command -v openssl >/dev/null 2>&1; then
    openssl dgst -sha256 "$file" | awk '{print $NF}'
  else
    fail "shasum, sha256sum, or openssl is required"
  fi
}

[ "$(uname -s)" = "Darwin" ] || fail "the initial release supports macOS only"
[ "$(uname -m)" = "arm64" ] || fail "the initial release supports Apple Silicon only"

mkdir -p "$RELEASES_DIR" "$BIN_DIR"
work_dir="$(mktemp -d "${TMPDIR:-/tmp}/codex-akram-install.XXXXXX")"
stage_dir=""
cleanup() {
  if [ -n "$stage_dir" ] && [ -d "$stage_dir" ]; then
    rm -rf "$stage_dir"
  fi
  rm -rf "$work_dir"
}
trap cleanup EXIT HUP INT TERM

if [ "$REQUESTED_RELEASE" = "latest" ]; then
  metadata="$work_dir/release.json"
  fetch "https://api.github.com/repos/${REPOSITORY}/releases/latest" "$metadata"
  tag="$(sed -n 's/.*"tag_name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' "$metadata" | head -n 1)"
  [ -n "$tag" ] || fail "could not resolve the latest fork release"
else
  case "$REQUESTED_RELEASE" in
    v*) tag="$REQUESTED_RELEASE" ;;
    *) tag="v$REQUESTED_RELEASE" ;;
  esac
fi

version="${tag#v}"
validate_version "$version"
[ "$tag" = "v$version" ] || fail "release tags must use the v prefix"

release_url="https://github.com/${REPOSITORY}/releases/download/${tag}"
archive="$work_dir/$ASSET"
checksums="$work_dir/$CHECKSUM_ASSET"
fetch "$release_url/$ASSET" "$archive"
fetch "$release_url/$CHECKSUM_ASSET" "$checksums"

expected="$(awk -v asset="$ASSET" '$2 == asset || $2 == "*" asset { print $1; exit }' "$checksums")"
[ -n "$expected" ] || fail "checksum manifest does not contain $ASSET"
actual="$(sha256_file "$archive")"
[ "$actual" = "$expected" ] || fail "archive checksum mismatch"

extract_dir="$work_dir/package"
mkdir -p "$extract_dir"
tar -xzf "$archive" -C "$extract_dir"
[ -x "$extract_dir/bin/codex-akram" ] || fail "package is missing bin/codex-akram"
[ -x "$extract_dir/bin/codex-code-mode-host" ] || fail "package is missing bin/codex-code-mode-host"
[ -x "$extract_dir/codex-path/rg" ] || fail "package is missing codex-path/rg"

reported_version="$("$extract_dir/bin/codex-akram" --version | sed -n 's/.* \([0-9][0-9A-Za-z.+-]*\)$/\1/p' | head -n 1)"
[ "$reported_version" = "$version" ] \
  || fail "package reports version $reported_version, expected $version"

release_name="$version-$TARGET"
release_dir="$RELEASES_DIR/$release_name"
if [ ! -d "$release_dir" ]; then
  stage_dir="$(mktemp -d "$RELEASES_DIR/.staging.XXXXXX")"
  cp -R "$extract_dir/." "$stage_dir/"
  mv "$stage_dir" "$release_dir"
  stage_dir=""
fi

current_tmp="$AKRAM_HOME/packages/standalone/.current.$$"
ln -s "releases/$release_name" "$current_tmp"
mv -f "$current_tmp" "$CURRENT_LINK"

bin_tmp="$BIN_DIR/.codex-akram.$$"
ln -s "$CURRENT_LINK/bin/codex-akram" "$bin_tmp"
mv -f "$bin_tmp" "$BIN_DIR/codex-akram"

"$BIN_DIR/codex-akram" --version >/dev/null
printf 'codex-akram %s installed at %s\n' "$version" "$BIN_DIR/codex-akram"
