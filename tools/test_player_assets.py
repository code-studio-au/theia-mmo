"""Player art preference uses supplied files; no game window or renderer."""
from pathlib import Path
import tempfile
import unittest
from player_assets import POLYGON_CANDIDATES, default_player_assets


class PlayerAssets(unittest.TestCase):
    def test_full_animation_art_wins_with_explicit_legacy_fallback(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.assertEqual(default_player_assets(root), root / POLYGON_CANDIDATES[-1])
            for relative in reversed(POLYGON_CANDIDATES):
                path = root / relative / 'character.glb'
                path.parent.mkdir(parents=True)
                path.write_bytes(b'local-fixture')
                self.assertEqual(default_player_assets(root), path.parent)


if __name__ == '__main__':
    unittest.main()
