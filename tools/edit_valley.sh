#!/usr/bin/env bash
# Open the locally prepared valley with disposable account services.
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.."

exec python3 tools/edit_local.py \
  --zone build/starter-valley-population-editor-native-v2/source.zone.json \
  --scenery-library build/starter-valley-scenery-v2/catalog.json \
  --players build/classic-directional-client-qualified-v1-20261103/art/player \
  --creature-definition content/gameplay/development-creature-pursuit.json \
  --creature-asset build/starter-valley-population-editor-native-v2/package/creature.glb \
  --gameplay-template content/development-gameplay-classic-quest-loot.json \
  --fantasy-ui '/home/steven/Downloads/Synty Models/HUDs' \
  --roam-radius 4 \
  --access-path 1201 \
  --output "build/manual-valley-$(date +%Y%m%d-%H%M%S-%N)" \
  "$@"
