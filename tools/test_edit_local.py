import argparse
import json
from pathlib import Path
import stat
import tempfile
import unittest
from unittest.mock import patch

import edit_local as play


class ManualEditorTests(unittest.TestCase):
    def fixture(self, root):
        binaries = root/'target/debug'
        for name in play.BINARIES:
            path = binaries/name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(b'qualified executable')
        args = argparse.Namespace(binaries=binaries, output=root/'build/manual',
                                  session_seconds=3600, access_path=1201)
        for name in ('zone', 'scenery_library', 'creature_definition', 'creature_asset',
                     'gameplay_template'):
            path = root/(name+'.json')
            path.write_text('{}')
            setattr(args, name, path)
        args.players = root/'players'
        args.players.mkdir()
        (args.players/'character.glb').write_bytes(b'art')
        return args

    def test_source_and_executables_are_isolated_without_test_input(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            args = self.fixture(root)
            output, command = play.prepare(args, root)
            (args.binaries/'theia-client').write_bytes(b'next build')
            (output/'draft.zone.json').write_text('{"edited":true}')
            self.assertEqual(args.zone.read_text(), '{}')
            self.assertEqual((output/'binaries/theia-client').read_bytes(), b'qualified executable')
            self.assertEqual(stat.S_IMODE(output.stat().st_mode), 0o700)
            self.assertEqual(command[command.index('--population-access-path')+1], '1201')
            self.assertFalse(any('qualify' in arg or 'test-route' in arg for arg in command))
            with self.assertRaises(ValueError):
                play.prepare(args, root)

    def test_invalid_input_refuses_before_output_or_services(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            args = self.fixture(root)
            for field, value in [('session_seconds', 59), ('session_seconds', 43201),
                                 ('access_path', 0), ('access_path', 2**64),
                                 ('output', root/'outside'), ('zone', root/'missing')]:
                changed = argparse.Namespace(**vars(args))
                setattr(changed, field, value)
                with self.assertRaises(ValueError):
                    play.prepare(changed, root)
                self.assertFalse(args.output.exists())
            args.zone.write_text(json.dumps({'test_players': [{'enabled': True}]}))
            with self.assertRaises(ValueError):
                play.prepare(args, root)
            self.assertFalse(args.output.exists())

    def test_owned_services_replace_inherited_credentials_and_gameplay_automation(self):
        identity = argparse.Namespace(issuer='http://127.0.0.1:1/realms/test',
                                      client_id='local', redirect='http://127.0.0.1:2/callback',
                                      users=[('synthetic', 'fixture-only')])
        cluster = argparse.Namespace(test_environment=lambda: {'THEIA_TEST_DATABASE_URL': 'owned'})
        source = dict(THEIA_POPULATION_KILL_QUALIFY='1', THEIA_WORLD_INPUT_OS_QUALIFY='old',
                      THEIA_DATABASE_URL='other', DATABASE_URL='other', PGPASSWORD='other',
                      theia_test_driver='1', DISPLAY=':7')
        env = play.environment(source, identity, cluster, Path('/tmp/editor'), 3600)
        self.assertEqual(env['THEIA_DATABASE_URL'], 'owned')
        self.assertEqual(env['DISPLAY'], ':7')
        for name in source.keys()-{'THEIA_DATABASE_URL', 'DISPLAY'}:
            self.assertNotIn(name, env)
        self.assertEqual(source['THEIA_DATABASE_URL'], 'other')

    def test_parent_exit_still_waits_for_owned_launcher_drain(self):
        editor = argparse.Namespace(pid=123, poll=lambda: 0)
        events = []
        def probe(pid, sig):
            self.assertEqual((pid, sig), (123, 0))
            events.append('probe')
            if events.count('probe') == 3:
                raise ProcessLookupError
        with patch.object(play.os, 'killpg', side_effect=probe), \
                patch.object(play.time, 'sleep', side_effect=lambda _: events.append('drain')):
            play.finish_editor(editor)
        self.assertEqual(events, ['probe', 'drain', 'probe', 'drain', 'probe'])


if __name__ == '__main__':
    unittest.main()
