"""Select existing local Synty player art without converting or changing it."""
from pathlib import Path

POLYGON_CANDIDATES = (
    'build/modular-hero-v1/art-v3',
    'build/classic-directional-client-qualified-v1-20261103/art/player',
    'build/synty-character-combat-stage-v2',
    'build/synty-character-canonical-stage',
)


def default_player_assets(root=Path('.')):
    """Prefer the qualified modular hero, retaining movement/sword art as fallback."""
    for relative in POLYGON_CANDIDATES:
        directory = root / relative
        if (directory / 'character.glb').is_file():
            return directory
    return root / POLYGON_CANDIDATES[-1]
