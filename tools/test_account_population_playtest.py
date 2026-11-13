"""Frozen account population launch setup; never starts services or windows."""
import argparse
import hashlib
import json
from pathlib import Path
import tempfile
import unittest
import account_population_playtest as launcher


class AccountPopulation(unittest.TestCase):
    def fixture(self, root):
        for name in ['package/population.json', 'package/world/authored.world.json',
                     'package/creature.glb', 'players/character.glb']:
            path = root/name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(b'fixture')
        for _, name in launcher.PROFILES:
            path = root/'content/gameplay'/name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(name.encode())
        return argparse.Namespace(package=root/'package', players=root/'players', binaries=root/'bin',
                                  output=root/'session', window_size='1280x720', character_id=None)

    def test_profiles_are_frozen_and_exact_paths_drive_the_owned_account_launcher(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            args = self.fixture(root)
            command = launcher.prepare(args, root)
            report = json.loads((args.output/'launch.json').read_text())
            self.assertEqual(len(report['profiles']), 16)
            self.assertIn('--character-picker', command)
            self.assertIn('--stop-on-stdin-eof', command)
            self.assertIn('--account-login', command)
            self.assertNotIn('--creature-profile', command)
            self.assertNotIn('--test-controller', command)
            self.assertEqual(command[command.index('--session-directory')+1], str(args.output/'runtime'))
            for record in report['profiles']:
                path = args.output/record['file']
                self.assertEqual(hashlib.sha256(path.read_bytes()).hexdigest(), record['sha256'])
                self.assertEqual(command.count(record['option']), 1)
                self.assertEqual(command[command.index(record['option'])+1], str(path))
                (root/'content/gameplay'/path.name).write_bytes(b'changed source')
                self.assertNotEqual(path.read_bytes(), b'changed source')

    def test_existing_output_is_not_replaced_and_explicit_character_excludes_picker(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            args = self.fixture(root)
            args.character_id = '74686569-6100-0000-0000-000000000101'
            command = launcher.prepare(args, root)
            self.assertNotIn('--character-picker', command)
            self.assertEqual(command[command.index('--character-id')+1], args.character_id)
            snapshot = (args.output/'launch.json').read_bytes()
            with self.assertRaises(FileExistsError):
                launcher.prepare(args, root)
            self.assertEqual((args.output/'launch.json').read_bytes(), snapshot)

    def test_missing_or_oversized_dependencies_refuse_before_creating_output(self):
        for oversized in (False, True):
            with self.subTest(oversized=oversized), tempfile.TemporaryDirectory() as folder:
                root = Path(folder)
                args = self.fixture(root)
                if oversized:
                    (root/'content/gameplay'/launcher.PROFILES[0][1]).write_bytes(b'x'*(1024*1024+1))
                else:
                    (args.package/'population.json').unlink()
                with self.assertRaises(ValueError):
                    launcher.prepare(args, root)
                self.assertFalse(args.output.exists())


if __name__ == '__main__':
    unittest.main()
