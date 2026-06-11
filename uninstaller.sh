#!/usr/bin/env bash
set -euo pipefail
DEST="${HOME}/.config/fresh/themes"
FILES="berlin.json bogota.json helsinki.json lahabana.json london.json madrid.json miami.json oslo.json paris.json praha.json tokio.json x.json"
for f in ${FILES}; do
  if [ -f "${DEST}/${f}" ]; then
    rm -f "${DEST}/${f}"
  fi
done
if [ -d "${DEST}" ] && [ -z "$(ls -A "${DEST}")" ]; then
  rmdir "${DEST}"
fi
echo "Themes removed from ${DEST}"
