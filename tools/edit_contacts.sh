#!/usr/bin/env bash
# Open the qualified two-contact valley in a fresh disposable account session.
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.."
exec bash tools/edit_valley.sh \
  --zone build/multi-contact-editor-v1/source.zone.json \
  --gameplay-template content/development-gameplay-valley-contacts.json \
  "$@"
