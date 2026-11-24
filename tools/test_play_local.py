"""Private playtest isolation and service teardown; never opens a game window."""
import argparse
from pathlib import Path
import signal
import stat
import tempfile
import unittest
from unittest.mock import patch
import play_local as play


class LocalPlay(unittest.TestCase):
    def fixture(self, root):
        source = root/'prepared'
        for name in ['theia-client', 'theia-server', 'validate_gameplay',
                     'art/player/character.glb', 'art/wolf/wolf.glb']:
            path = source/name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(b'original prepared data')
        for name in ['build/graybox-terrain-v1/terrain.manifest.json',
                     'build/navigation-terrain-v1/navigation.manifest.json',
                     'content/test.json']:
            path = root/name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text('{}')
        hud = root/'hud'
        hud.mkdir()
        return argparse.Namespace(binaries=source, client=None, server=None,
                                  output=root/'build/playtest', manifest=root/'content/test.json',
                                  fantasy_ui=hud, session_seconds=3600, character='Arden',
                                  ui_scale=1.25, ui_preferences=None, window_size='1920x1200')

    def test_frozen_build_and_content_survive_source_edits_and_use_isolated_cwd(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            args = self.fixture(root)
            output, command = play.freeze(args, root)
            (args.binaries/'theia-client').write_bytes(b'new development build')
            args.manifest.write_text('{"changed":true}')
            self.assertEqual((output/'binaries/theia-client').read_bytes(), b'original prepared data')
            self.assertEqual((output/'content/test.json').read_text(), '{}')
            self.assertEqual(stat.S_IMODE(output.stat().st_mode), 0o700)
            self.assertIn(str(output/'binaries/art/wolf/wolf.glb'), command)
            self.assertIn(str(root/'build/graybox-terrain-v1'), command)
            self.assertNotIn('--world-input-os-qualify', command)
            with self.assertRaises(ValueError):
                play.freeze(args, root)

    def test_saved_preferences_can_survive_disposable_accounts_without_an_implicit_scale_override(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            args = self.fixture(root)
            args.ui_scale = None
            args.ui_preferences = root/'build/local-ui.json'
            _, command = play.freeze(args, root)
            self.assertNotIn('--ui-scale', command)
            self.assertEqual(command[command.index('--ui-preferences')+1], str(args.ui_preferences.resolve()))
            self.assertFalse(args.ui_preferences.exists())

    def test_invalid_inputs_fail_before_creating_private_services_or_output(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            args = self.fixture(root)
            for field, value in [('session_seconds', 59), ('session_seconds', 43201),
                                 ('character', '../invalid'), ('ui_scale', float('nan')),
                                 ('window_size', '1920x0'), ('output', root/'outside-build')]:
                changed = argparse.Namespace(**vars(args))
                setattr(changed, field, value)
                with self.assertRaises(ValueError):
                    play.validate(changed, root)
                self.assertFalse(args.output.exists())

    def test_interrupt_stops_game_before_database_and_provider(self):
        events = []
        handlers = {}
        class Identity:
            issuer = 'http://127.0.0.1:123/realms/test'
            client_id = 'owned'
            redirect = 'http://127.0.0.1:124/callback'
            users = [('synthetic', 'private-fixture-password')]
            def __init__(self, *_args, **kwargs):
                assert kwargs == dict(token_seconds=3600, interactive=True)
            def __enter__(self): return self
            def __exit__(self, *_): events.append('provider stopped')
        class Cluster:
            def __init__(self, *_, **kwargs):
                assert kwargs == dict(isolate_process=True)
            def __enter__(self): return self
            def __exit__(self, *_): events.append('database stopped')
            def test_environment(self): return {'THEIA_TEST_DATABASE_URL': 'private-fixture-url'}
        class Input:
            def write(self, _): pass
            def flush(self): pass
            def close(self): pass
        class Launcher:
            pid = 123
            stdin = Input()
            returncode = None
            def poll(self): return self.returncode
            def wait(self, **_): self.returncode = 0
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory)
            launcher = Launcher()
            args = argparse.Namespace(session_seconds=3600, character='Arden')
            def launch(_command, **kwargs):
                self.assertEqual(kwargs['cwd'], output)
                self.assertEqual(kwargs['env']['THEIA_ACCOUNT_SESSION_SECONDS'], '3600')
                return launcher
            with patch.object(play, 'freeze', return_value=(output, ['owned command'])), \
                    patch.object(play, 'Identity', Identity), patch.object(play, 'Cluster', Cluster), \
                    patch.object(play.signal, 'signal', side_effect=lambda sig, fn: handlers.update({sig: fn})), \
                    patch.object(play.subprocess, 'Popen', side_effect=launch), \
                    patch.object(play.os, 'killpg', side_effect=lambda *_: events.append('game stopped')), \
                    patch.object(play.time, 'sleep', side_effect=lambda _: handlers[signal.SIGINT]()):
                play.run(args)
            self.assertEqual(events, ['game stopped', 'database stopped', 'provider stopped'])


if __name__ == '__main__':
    unittest.main()
