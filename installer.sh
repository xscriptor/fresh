#!/usr/bin/env bash
set -euo pipefail
DEST="${HOME}/.config/fresh/themes"
mkdir -p "${DEST}"
TMP_DIR="$(mktemp -d)"
URL="https://codeload.github.com/xscriptor/fresh/tar.gz/refs/heads/main"
if command -v curl >/dev/null 2>&1; then
  curl -fsSL "${URL}" | tar -xz -C "${TMP_DIR}"
elif command -v wget >/dev/null 2>&1; then
  wget -qO- "${URL}" | tar -xz -C "${TMP_DIR}"
else
  echo "Error: curl or wget is required" >&2
  exit 1
fi
THEMES_DIR="$(find "${TMP_DIR}" -type d -name themes | head -n 1)"
if [ -z "${THEMES_DIR}" ]; then
  echo "Error: themes directory not found in archive" >&2
  exit 1
fi
cp -f "${THEMES_DIR}"/*.json "${DEST}/"
echo "Themes installed to ${DEST}"
