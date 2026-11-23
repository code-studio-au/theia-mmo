"""Asset names shared by the source launcher and its standalone package."""
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPOSITORY = HERE.parents[1]
SPRITES = 'content/ui/sprites-v3/sprites'
ASSET_SOURCES = {
    'title': 'content/ui/frontend/title.png',
    'landscape': 'content/ui/frontend/launcher-harbour-v1.png',
    'hero': 'content/ui/frontend/account-adventurer-v1.png',
    'news': 'content/ui/loading/millhaven-sunlit-v2.png',
    'icon': f'{SPRITES}/crest/tree.png',
    'panel': f'{SPRITES}/panel/navy.png',
    'corner': f'{SPRITES}/ornament/corner-heavy-left.png',
    'small-panel': f'{SPRITES}/panel/compact-navy.png',
    'divider': f'{SPRITES}/divider/ornate.png',
    'news-icon': f'{SPRITES}/icon/book.png',
    'account-icon': f'{SPRITES}/icon/character.png',
    'settings-icon': 'content/ui/theme/standalone-v3/settings.png',
    'secondary': f'{SPRITES}/button/secondary/default.png',
    'selected': f'{SPRITES}/button/secondary/selected.png',
    'primary': f'{SPRITES}/button/primary/hover.png',
    'primary-hover': f'{SPRITES}/button/primary/selected.png',
}


def asset(name):
    packaged = HERE / 'assets'
    if packaged.is_dir():
        return packaged / f'{name}.png'
    return REPOSITORY / ASSET_SOURCES[name]
