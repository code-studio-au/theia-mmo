"""Regression checks for release selection and standalone launcher preferences."""
import os
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

from client_install import InstallError, executable, export, target

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'apps/launcher'))
from state import DEFAULT_LOGIN_BIND, game_launch_args, login_address, login_ready, newest_release, read_settings
from state import save_settings


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


class GameLaunchMode(unittest.TestCase):
    @patch.dict(os.environ, {}, clear=True)
    def test_normal_play_and_account_sessions_default_to_fullscreen(self):
        fullscreen = ['--fullscreen', '--window-size', '1280x720']
        self.assertEqual(game_launch_args([]), fullscreen)
        session = ['--game-front-end', '/tmp/account-session']
        self.assertEqual(game_launch_args(session), [*session, *fullscreen])

    @patch.dict(os.environ, {'THEIA_WINDOW_SIZE': '1600x900'})
    def test_preserves_configured_window_size_when_toggling_fullscreen(self):
        self.assertEqual(game_launch_args([]), ['--fullscreen'])

    def test_explicit_window_modes_config_reports_and_utilities_are_forwarded(self):
        for options in (
            ['--window-size', '1600x900'],
            ['--test-window', '1/2'],
            ['--window-size', '1600x900', '--fullscreen'],
            ['--configuration-report'],
            ['--preview-fantasy-ui', 'assets'],
            ['--inspect-asset', 'example.glb'],
            ['--chunk-smoke'],
        ):
            with self.subTest(options=options):
                self.assertEqual(game_launch_args(options), options)


class LoginReadiness(unittest.TestCase):
    def test_address_matches_development_listener_contract(self):
        self.assertEqual(login_address(DEFAULT_LOGIN_BIND), ('127.0.0.1', 5050))
        self.assertEqual(login_address('[::1]:8244'), ('::1', 8244))
        for bad in ('localhost:5050', '127.0.0.1:0', '192.0.2.1:5050', '127.0.0.1:5050/ready',
                    'user@127.0.0.1:5050', '127.0.0.1:70000'):
            with self.subTest(bad=bad), self.assertRaises(ValueError):
                login_address(bad)

    @patch('state.http.client.HTTPConnection')
    def test_ready_requires_the_account_service_response(self, connect):
        address = ('127.0.0.1', 5050)
        connection = connect.return_value
        response = connection.getresponse.return_value
        response.status = 200
        response.read.return_value = b'theia-account-ready\n'
        self.assertTrue(login_ready(address))
        connect.assert_called_with(*address, timeout=2)
        connection.request.assert_called_with('GET', '/ready', headers={'Connection': 'close'})
        connection.close.assert_called()

        response.read.return_value = b'arbitrary-service'
        self.assertFalse(login_ready(address))
        response.read.return_value = b'theia-account-ready\n'
        response.status = 503
        self.assertFalse(login_ready(address))
        connection.request.side_effect = ConnectionRefusedError
        self.assertFalse(login_ready(address))


if __name__ == '__main__':
    unittest.main()
