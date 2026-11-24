"""Recovery and launch checks for local client releases."""
import json
import os
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

from client_install import (
    ACTIVE, MANIFEST, InstallError, active, executable, export, install,
    install_lock, launch, rollback, target,
)


class ClientInstaller(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.folder = Path(temporary.name).resolve()
        self.store = self.folder / 'installation'

    def package(self, release, payload=b'client bytes'):
        source = self.folder / f'source-{release}'
        source.mkdir()
        (source / executable(target())).write_bytes(payload)
        (source / 'data').mkdir()
        (source / 'data' / 'built-ins.bin').write_bytes(b'embedded content')
        return export(source, self.folder / f'package-{release}', release)

    def test_update_switches_verified_versions_and_can_roll_back(self):
        first = self.package('0.1.0.1', b'first')
        second = self.package('0.1.0.2', b'second')
        self.assertEqual(install(first, self.store), '0.1.0.1')
        preferences = self.store / 'interface.json'
        preferences.write_text('{"ui_scale": 1.25}')
        progress = []
        self.assertEqual(install(second, self.store, lambda a, b: progress.append((a, b))), '0.1.0.2')
        self.assertEqual(progress[0][0], 0)
        self.assertEqual(progress[-1][0], progress[-1][1])
        self.assertTrue(all(a[0] <= b[0] <= b[1] for a, b in zip(progress, progress[1:])))
        self.assertEqual(active(self.store)['previous'], '0.1.0.1')
        self.assertEqual(rollback(self.store), '0.1.0.1')
        self.assertEqual(active(self.store)['previous'], '0.1.0.2')
        self.assertEqual(preferences.read_text(), '{"ui_scale": 1.25}')

    def test_failed_hash_and_cancel_keep_the_current_pointer_and_private_settings(self):
        first = self.package('0.1.0.1')
        install(first, self.store)
        pointer = (self.store / ACTIVE).read_bytes()
        second = self.package('0.1.0.2')
        file = second / 'payload' / 'data' / 'built-ins.bin'
        file.write_bytes(b'tampered content')  # Same length, different hash.
        with self.assertRaisesRegex(InstallError, 'did not match'):
            install(second, self.store)
        self.assertEqual((self.store / ACTIVE).read_bytes(), pointer)
        self.assertEqual(list((self.store / 'versions').iterdir()), [self.store / 'versions' / '0.1.0.1'])
        third = self.package('0.1.0.3', b'cancel this')
        progress = []
        with self.assertRaisesRegex(InstallError, 'cancelled'):
            install(third, self.store, lambda done, _: progress.append(done),
                    lambda: any(progress))
        self.assertEqual((self.store / ACTIVE).read_bytes(), pointer)
        self.assertFalse((self.store / 'versions' / '0.1.0.3').exists())

    def test_wrong_platform_and_escaping_or_colliding_paths_are_refused(self):
        bundle = self.package('0.1.0')
        path = bundle / MANIFEST
        manifest = json.loads(path.read_bytes())
        for bad in ('../outside', 'C:/program', 'a//b', 'a/../b', 'aux.log', '/absolute'):
            entry = dict(manifest['files'][0], path=bad)
            path.write_text(json.dumps(dict(manifest, files=[entry, *manifest['files']])))
            with self.subTest(path=bad), self.assertRaises(InstallError):
                install(bundle, self.store)
        entry = dict(manifest['files'][0], path=manifest['files'][0]['path'].upper())
        path.write_text(json.dumps(dict(manifest, files=[entry, *manifest['files']])))
        with self.assertRaisesRegex(InstallError, 'duplicate'):
            install(bundle, self.store)
        path.write_text(json.dumps(dict(manifest, target='another-target')))
        with self.assertRaisesRegex(InstallError, 'another platform'):
            install(bundle, self.store)
        self.assertFalse(self.store.exists())

    @unittest.skipIf(sys.platform == 'win32', 'Symbolic links require extra Windows privileges')
    def test_symlink_in_payload_cannot_disclose_an_external_file(self):
        bundle = self.package('0.1.0')
        file = bundle / 'payload' / 'data' / 'built-ins.bin'
        outside = self.folder / 'outside.bin'
        outside.write_bytes(file.read_bytes())
        file.unlink()
        file.symlink_to(outside)
        with self.assertRaisesRegex(InstallError, 'regular files'):
            install(bundle, self.store)
        self.assertIsNone(active(self.store)['current'])
        self.assertEqual(outside.read_bytes(), b'embedded content')

    def test_running_update_excludes_a_second_writer(self):
        bundle = self.package('0.1.0')
        with install_lock(self.store):
            with self.assertRaisesRegex(InstallError, 'Another client update'):
                install(bundle, self.store)
        self.assertEqual(install(bundle, self.store), '0.1.0')

    def test_desktop_offer_cannot_activate_a_changed_or_older_release(self):
        first = self.package('0.1.0.1')
        latest = self.package('0.1.0.2')
        install(latest, self.store, offered_version='0.1.0.2')
        with self.assertRaisesRegex(InstallError, 'selected release changed'):
            install(first, self.store, offered_version='0.1.0.3')
        with self.assertRaisesRegex(InstallError, 'installed version changed'):
            install(first, self.store, offered_version='0.1.0.1')
        self.assertEqual(active(self.store)['current'], '0.1.0.2')
        self.assertFalse((self.store / 'versions' / '0.1.0.1').exists())

    def test_reinstallation_refuses_changed_content_in_an_existing_version(self):
        bundle = self.package('0.1.0', b'original-version')
        install(bundle, self.store)
        source = self.folder / 'replacement'
        source.mkdir()
        (source / executable(target())).write_bytes(b'changed-version')
        replacement = export(source, self.folder / 'replacement-package', '0.1.0')
        with self.assertRaisesRegex(InstallError, 'different release manifest'):
            install(replacement, self.store)
        dest = self.store / 'versions' / '0.1.0' / 'payload' / executable(target())
        self.assertEqual(dest.read_bytes(), b'original-version')

    def test_launch_checks_current_files_and_uses_the_installed_program(self):
        bundle = self.package('0.1.0')
        install(bundle, self.store)
        entry = self.store / 'versions' / '0.1.0' / 'payload' / executable(target())
        argv = ['--window-size', '1280x720']
        with patch('client_install.subprocess.Popen') as spawn:
            launch(self.store, argv)
            spawn.assert_called_once_with([str(entry), *argv], cwd=entry.parent)
            spawn.reset_mock()
            entry.write_bytes(b'corrupted')
            with self.assertRaises(InstallError):
                launch(self.store, argv)
            spawn.assert_not_called()

    def test_unlisted_installed_payload_file_prevents_launch(self):
        bundle = self.package('0.1.0')
        install(bundle, self.store)
        payload_dir = self.store / 'versions' / '0.1.0' / 'payload'
        (payload_dir / 'unlisted.dll').write_bytes(b'unlisted')
        with patch('client_install.subprocess.Popen') as spawn:
            with self.assertRaisesRegex(InstallError, 'Unexpected installed payload'):
                launch(self.store, [])
            spawn.assert_not_called()

    @unittest.skipUnless(sys.platform == 'linux', 'Native executable fixture is a Linux script')
    def test_installed_entry_is_executable_and_receives_real_arguments(self):
        bundle = self.package('0.1.0', b'#!/bin/sh\nprintf "%s\\n" "$1" > "$2"\n')
        install(bundle, self.store)
        started = self.folder / 'started.txt'
        process = launch(self.store, ['verified-client', str(started)])
        self.assertEqual(process.wait(timeout=10), 0)
        entry = self.store / 'versions' / '0.1.0' / 'payload'
        self.assertEqual(started.read_text(), 'verified-client\n')
        self.assertTrue(os.access(entry / executable(target()), os.X_OK))


if __name__ == '__main__':
    unittest.main()
