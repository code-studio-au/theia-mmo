"""Regression checks for release selection and standalone launcher preferences."""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

from client_install import InstallError, executable, export, target

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'apps/launcher'))
from launcher_update import install_launcher_update, newest_launcher_release
from news_feed import MAX_FEED_BYTES, NEWS_URL, NewsArticle, parse_news
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
        save_settings(settings, game)
        settings.write_text(json.dumps({'install_root': str(game), 'release_library': str(self.library)}))
        self.assertEqual(read_settings(settings), {
            'install_root': str(game),
        })
        self.assertFalse(game.exists())
        moved = game.with_name('game-moved')
        save_settings(settings, moved)
        self.assertEqual(read_settings(settings), {'install_root': os.fspath(moved)})
        self.assertEqual(list(settings.parent.iterdir()), [settings])
        settings.write_text('{"install_root":"relative"}')
        with self.assertRaisesRegex(InstallError, 'Invalid launcher preferences'):
            read_settings(settings)


class LauncherNews(unittest.TestCase):
    def test_uses_plain_summaries_and_unique_article_links_on_the_official_site(self):
        data = f'''<rss><channel>
          <item><title>Latest update</title><description>New &amp; improved</description>
                <link>{NEWS_URL}latest-update/</link></item>
          <item><title>Duplicate</title><link>{NEWS_URL}latest-update/</link></item>
          <item><title>External</title><link>https://another.example/news/more/</link></item>
          <item><title>Unsafe</title><link>{NEWS_URL}../download/</link></item>
          <item><title>Older update</title><link>{NEWS_URL}older-update/</link></item>
        </channel></rss>'''.encode()
        self.assertEqual(parse_news(data), [
            NewsArticle('Latest update', 'New & improved', NEWS_URL + 'latest-update/'),
            NewsArticle('Older update', '', NEWS_URL + 'older-update/'),
        ])

    def test_rejects_oversized_and_invalid_xml(self):
        for data in (b'x' * (MAX_FEED_BYTES + 1), b'<!DOCTYPE rss><rss/>', b'<broken>'):
            with self.subTest(length=len(data)), self.assertRaises(ValueError):
                parse_news(data)


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


@unittest.skipUnless(Path('/usr/bin/dpkg-deb').is_file(), 'Debian packaging tools required')
class LauncherRelease(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.library = Path(temporary.name)
        self.releases = self.library / 'launcher'
        self.releases.mkdir()
        self.control = self.library / 'staging' / 'DEBIAN' / 'control'
        self.control.parent.mkdir(parents=True)

    def publish(self, release, package='theia-launcher', architecture='amd64', filename=None):
        self.control.write_text(
            f'Package: {package}\nVersion: {release}\nArchitecture: {architecture}\n'
            'Maintainer: Tests\nDescription: Launcher test package\n', encoding='utf-8',
        )
        path = self.releases / f'theia-launcher_{filename or release}_amd64.deb'
        subprocess.run(['/usr/bin/dpkg-deb', '--build', str(self.control.parent.parent), str(path)],
                       check=True, capture_output=True)
        return path

    def test_accepts_only_newer_launcher_packages_with_matching_metadata(self):
        newest = self.publish('0.1.10')
        self.publish('0.1.9')
        self.publish('0.1.11', architecture='arm64')
        self.publish('0.1.12', package='other-package')
        self.publish('0.1.13', filename='0.1.14')
        (self.releases / 'theia-launcher_0.1.15_amd64.deb').symlink_to(newest)
        self.assertEqual(newest_launcher_release(self.library, '0.1.3'), (newest, '0.1.10'))
        self.assertIsNone(newest_launcher_release(self.library, '0.1.10'))
        self.assertIsNone(newest_release(self.library, '0.1.0.4'))

    def test_revalidates_staged_version_before_asking_for_administrator_approval(self):
        mismatched = self.publish('0.1.3', filename='0.1.4')
        with self.assertRaisesRegex(InstallError, 'package changed'):
            install_launcher_update((mismatched, '0.1.4'))

    def test_requires_installed_version_confirmation_and_handles_cancel(self):
        package = self.publish('0.1.4')
        actual_run = subprocess.run

        def run(command, **options):
            if command[0] == '/usr/bin/pkexec':
                self.assertEqual(command[1:6], [
                    '--disable-internal-agent', '/usr/bin/apt-get', '--assume-yes', '--no-remove', 'install',
                ])
                self.assertEqual(Path(command[-1]).read_bytes(), package.read_bytes())
                return subprocess.CompletedProcess(command, result, '', '')
            if command[0] == '/usr/bin/dpkg-query':
                return subprocess.CompletedProcess(command, 0, f'install ok installed\t{installed}'.encode(), b'')
            return actual_run(command, **options)

        with patch('launcher_update.subprocess.run', side_effect=run), patch('launcher_update.SYSTEM_LAUNCHER', package):
            result, installed = 126, '0.1.3'
            with self.assertRaisesRegex(InstallError, 'cancelled'):
                install_launcher_update((package, '0.1.4'))
            result = 0
            with self.assertRaisesRegex(InstallError, 'did not confirm'):
                install_launcher_update((package, '0.1.4'))
            installed = '0.1.4'
            self.assertEqual(install_launcher_update((package, '0.1.4')), installed)


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
