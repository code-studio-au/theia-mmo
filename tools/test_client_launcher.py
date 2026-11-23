"""Regression checks for release selection and standalone launcher preferences."""
import os
from pathlib import Path
import sys
import tempfile
import unittest

from client_install import InstallError, executable, export, target

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'apps/launcher'))
from state import newest_release, read_settings, save_settings


class ClientLauncher(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name).resolve()
        self.library = self.root / 'library'
        self.library.mkdir()
        self.staging = self.root / 'staging'
        self.staging.mkdir()
        (self.staging / executable(target())).write_bytes(b'game')

    def publish(self, release, name=None):
        return export(self.staging, self.library / (name or release), release)

    def test_library_chooses_newest_compatible_version_even_with_unrelated_folders(self):
        self.publish('0.1.0.9')
        latest = self.publish('0.1.0.10')
        broken = self.library / 'invalid'
        broken.mkdir()
        (broken / 'client-release.json').write_text('invalid JSON')
        self.assertEqual(newest_release(self.library, '0.1.0.9'), (latest, '0.1.0.10', 4))
        self.assertIsNone(newest_release(self.library, '0.1.0.10'))
        with self.assertRaisesRegex(InstallError, 'Invalid release metadata'):
            newest_release(broken, None)

    def test_a_single_release_can_be_selected_directly(self):
        single = self.publish('0.7.0')
        self.assertEqual(newest_release(single, None), (single, '0.7.0', 4))
        self.assertIsNone(newest_release(single, '0.7.0.0'))

    def test_equivalent_latest_versions_require_an_unambiguous_release(self):
        self.publish('0.8.0')
        self.publish('0.8.0.0')
        with self.assertRaisesRegex(InstallError, 'More than one package'):
            newest_release(self.library, '0.7.0')
        latest = self.publish('0.8.0.1')
        self.assertEqual(newest_release(self.library, None), (latest, '0.8.0.1', 4))

    def test_settings_are_saved_outside_the_install_and_can_change_folders(self):
        settings = self.root / 'app-data' / 'launcher.json'
        game = self.root / 'game'
        self.assertEqual(read_settings(settings), {})
        save_settings(settings, game, self.library)
        self.assertEqual(read_settings(settings), {
            'install_root': str(game), 'release_library': str(self.library),
        })
        self.assertFalse(game.exists())
        moved = game.with_name('game-moved')
        save_settings(settings, moved, None)
        self.assertEqual(read_settings(settings), {'install_root': os.fspath(moved)})
        self.assertEqual(list(settings.parent.iterdir()), [settings])
        settings.write_text('{"install_root":"relative"}')
        with self.assertRaisesRegex(InstallError, 'Invalid launcher preferences'):
            read_settings(settings)


if __name__ == '__main__':
    unittest.main()
