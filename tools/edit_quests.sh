#!/usr/bin/env bash
# Latest local quest slice: destination-only escort and a separate collection path.
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.."
quest_source=build/quest-travel-source.zone.json
if [[ ! -f "$quest_source" ]]; then
  python3 tools/prepare_quest_travel.py \
    --source build/starter-economy-editor-v2/source.zone.json --output "$quest_source"
fi
exec bash tools/edit_valley.sh \
  --zone "$quest_source" \
  --creature-definition content/gameplay/development-creature-deflect-pursuit.json \
  --gameplay-template content/development-gameplay-starter-economy.json \
  "$@"
