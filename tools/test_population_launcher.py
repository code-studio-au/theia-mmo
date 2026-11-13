"""Population launcher argument/preflight checks; never starts a game window."""
from contextlib import redirect_stderr, redirect_stdout
import inspect
import io
import json
import os
from pathlib import Path
import subprocess
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch
import play

PLAY_SIGNATURE = inspect.signature(play.play)


class PopulationLauncher(unittest.TestCase):
    @unittest.skipUnless(all(os.environ.get(key) for key in (
        'THEIA_AUTHORED_WORLD', 'THEIA_POPULATION_CONFIG', 'THEIA_CLIENT_BIN')),
        'requires actual authored population and built client')
    def test_actual_client_preflight_completes_before_any_game_process(self):
        class EndBeforeLaunch(Exception): pass
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            args = self.fixture(root)
            args.update(authored_world=Path(os.environ['THEIA_AUTHORED_WORLD']),
                        population_config=Path(os.environ['THEIA_POPULATION_CONFIG']),
                        binaries=Path(os.environ['THEIA_CLIENT_BIN']).parent,
                        combat_catalog=Path('content/gameplay/development-abilities.json').resolve())
            commands = []
            def launch(command, **_):
                commands.append(command)
                raise EndBeforeLaunch()
            # Keep real subprocess.run's internals intact; stop only game launch.
            runner = SimpleNamespace(run=subprocess.run, Popen=launch,
                                     STDOUT=subprocess.STDOUT,
                                     TimeoutExpired=subprocess.TimeoutExpired)
            previous = Path.cwd()
            try:
                os.chdir(root)
                with patch.object(play, 'subprocess', runner), self.assertRaises(EndBeforeLaunch):
                    play.play(**args)
            finally:
                os.chdir(previous)
            self.assertEqual(len(commands), 1)
            self.assertTrue(Path(commands[0][0]).name.startswith('theia-server'))

    def test_cli_selects_population_without_implying_patrol_or_resources(self):
        with patch.object(play, 'play') as launch:
            play.main(['--authored-world', 'world', '--population-config', 'population.json'])
        args = PLAY_SIGNATURE.bind(*launch.call_args.args).arguments
        self.assertEqual(args['population_config'], Path('population.json').resolve())
        self.assertTrue(args['classic'])
        self.assertTrue(args['prediction'])
        self.assertIsNotNone(args['combat_catalog'])
        self.assertIsNone(args['creature_profile'])
        self.assertIsNone(args['navigation'])
        self.assertIsNone(args['character_stats'])

    def test_explicit_character_selection_is_forwarded_without_implying_account_login(self):
        selected = '12345678-1234-1234-1234-123456789abc'
        with patch.object(play, 'play') as launch:
            play.main(['--account-login', '--character-id', selected])
        args = PLAY_SIGNATURE.bind(*launch.call_args.args).arguments
        self.assertEqual(args['character_id'], selected)
        self.assertTrue(args['account_login'])

    def test_invalid_or_unauthenticated_character_selection_refuses_before_io(self):
        with patch.object(play.subprocess, 'Popen') as launch:
            for selected, login in [('bad', True), ('00000000-0000-0000-0000-000000000000', True),
                                    ('12345678-1234-1234-1234-123456789abc', False)]:
                with self.subTest(selected=selected, login=login), self.assertRaises(ValueError):
                    play.play(None, Path('missing'), 1, Path('missing'), account_login=login,
                              character_id=selected)
        launch.assert_not_called()

    def test_stopped_owner_returns_before_files_or_processes_and_account_prompt_is_refused(self):
        with patch.object(play.subprocess, 'Popen') as launch:
            play.play(None, Path('missing'), 1, Path('missing'), stop_requested=lambda: True)
            with self.assertRaisesRegex(ValueError, 'explicit character'):
                play.play(None, Path('missing'), 1, Path('missing'), account_login=True,
                          stop_requested=lambda: False)
        launch.assert_not_called()

    def test_picker_refuses_missing_ownership_or_conflicting_explicit_character_before_io(self):
        for options in [dict(), dict(account_login=True),
                        dict(account_login=True, stop_requested=lambda: False),
                        dict(account_login=True, stop_requested=lambda: False, session_directory='unused',
                             character_id='12345678-1234-1234-1234-123456789abc')]:
            with self.subTest(options=options), patch.object(play.subprocess, 'Popen') as launch:
                with self.assertRaisesRegex(ValueError, 'Character picker'):
                    play.play(None, Path('missing'), 1, Path('missing'), character_picker=True, **options)
                launch.assert_not_called()

    def test_cli_population_player_life_selects_resources_without_a_patrol(self):
        with patch.object(play, 'play') as launch:
            play.main(['--authored-world', 'world', '--population-config', 'population.json',
                       '--creature-retaliation', 'retaliation.json',
                       '--player-respawn', 'respawn.json', '--public-avatar-life'])
        args = PLAY_SIGNATURE.bind(*launch.call_args.args).arguments
        self.assertIsNone(args['creature_profile'])
        self.assertIsNone(args['navigation'])
        self.assertIsNotNone(args['character_stats'])
        self.assertTrue(args['public_avatar_life'])
        self.assertEqual(args['player_respawn'], Path('respawn.json').resolve())

    def test_cli_population_grants_keep_the_population_without_a_patrol_override(self):
        with patch.object(play, 'play') as launch:
            play.main(['--authored-world', 'world', '--population-config', 'population.json',
                       '--account-login', '--progression', '--rewards', '--creature-credit', 'credit.json'])
        args = PLAY_SIGNATURE.bind(*launch.call_args.args).arguments
        self.assertIsNone(args['creature_profile'])
        self.assertIsNone(args['navigation'])
        self.assertIsNotNone(args['progression_catalog'])
        self.assertIsNotNone(args['reward_catalog'])

    def test_population_grants_refuse_before_launch_without_explicit_credit(self):
        with tempfile.TemporaryDirectory() as directory:
            args = self.fixture(Path(directory))
            args['progression_catalog'] = Path('xp.json')
            with patch.object(play.subprocess, 'Popen') as child:
                with self.assertRaisesRegex(ValueError, 'solo creature credit'):
                    play.play(**args)
            child.assert_not_called()

    def test_cli_keeps_explicit_creature_art_as_a_client_presentation_override(self):
        with patch.object(play, 'play') as launch:
            play.main(['--authored-world', 'world', '--population-config', 'population.json',
                       '--creature-asset', 'wolf.glb'])
        args = PLAY_SIGNATURE.bind(*launch.call_args.args).arguments
        self.assertEqual(args['creature_asset'], Path('wolf.glb').resolve())
        self.assertIsNone(args['creature_profile'])
        self.assertIsNone(args['navigation'])

    def test_singleton_bundle_conflict_refuses_before_loading_or_launching(self):
        with patch.object(play, 'play') as launch, redirect_stderr(io.StringIO()):
            with self.assertRaises(SystemExit):
                play.main(['--population-config', 'population.json', '--gameplay-manifest', 'missing.json'])
        launch.assert_not_called()

    def fixture(self, root):
        for name in ['world/authored.world.json', 'world/world.manifest.json',
                     'world/terrain.json', 'world/terrain.glb', 'players/character.glb',
                     'bin/theia-client', 'bin/theia-server', 'population.json', 'abilities.json']:
            if os.name == 'nt' and name.startswith('bin/'):
                name += '.exe'
            path = root/name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text('{}')
        return dict(world=None, players=root/'players', clients=2, binaries=root/'bin',
                    authored_world=root/'world', prediction=True, classic=True,
                    combat_catalog=root/'abilities.json', population_config=root/'population.json')

    def test_failed_headless_preflight_never_starts_children_or_session(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            args = self.fixture(root)
            for response in [SimpleNamespace(returncode=1, stderr='Invalid placement', stdout=''),
                             SimpleNamespace(returncode=0, stderr='', stdout='{}'),
                             SimpleNamespace(returncode=0, stderr='', stdout='not JSON')]:
                with patch.object(play.subprocess, 'run', return_value=response) as preflight, \
                        patch.object(play.subprocess, 'Popen') as child:
                    with self.assertRaises(ValueError):
                        play.play(**args)
                child.assert_not_called()
                command = preflight.call_args.args[0]
                self.assertIn('--configuration-report', command)
                self.assertEqual(command[command.index('--population-config')+1], str(root/'population.json'))

    def test_existing_explicit_session_is_never_overwritten(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            args = self.fixture(root)
            session = root/'existing'
            session.mkdir()
            for name in ['server.log', 'server-cert.der']:
                (session/name).write_text('existing owned data')
            report = dict(renderer_started=False, network_started=False, saved_state_admitted=False)
            with patch.object(play.subprocess, 'run', return_value=SimpleNamespace(
                    returncode=0, stderr='', stdout=json.dumps(report))), \
                    patch.object(play.subprocess, 'Popen') as child:
                with self.assertRaises(FileExistsError):
                    play.play(**args, session_directory=session)
            child.assert_not_called()
            for name in ['server.log', 'server-cert.der']:
                self.assertEqual((session/name).read_text(), 'existing owned data')

    def test_same_population_is_forwarded_after_preflight_and_owned_children_clean_up(self):
        for isolated in (False, True):
            with self.subTest(isolated=isolated):
                self._check_forwarded_population_cleanup(isolated)

    def _check_forwarded_population_cleanup(self, isolated):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            args = self.fixture(root)
            session = root/('owned-session' if isolated else 'build/development-session')
            if isolated:
                args['session_directory'] = session
            args['creature_asset'] = root/'wolf.glb'
            args['creature_asset'].write_bytes(b'presentation path fixture')
            report = dict(renderer_started=False, network_started=False, saved_state_admitted=False)
            commands, stopped = [], []
            class Process:
                def __init__(self, server): self.server, self.code = server, None if server else 0
                def poll(self): return self.code
                def send_signal(self, _): stopped.append('server'); self.code = 0
                def terminate(self): stopped.append('server'); self.code = 0
                def wait(self, **_): return self.code
            def launch(command, **kwargs):
                commands.append(command)
                server = len(commands) == 1
                if server:
                    kwargs['stdout'].write('listening=127.0.0.1:12345 local_tickets=fixture\n')
                    kwargs['stdout'].flush()
                return Process(server)
            previous = Path.cwd()
            try:
                os.chdir(root)
                with patch.object(play.subprocess, 'run', return_value=SimpleNamespace(
                        returncode=0, stderr='', stdout=json.dumps(report))), \
                        patch.object(play.subprocess, 'Popen', side_effect=launch), redirect_stdout(io.StringIO()):
                    play.play(**args)
            finally:
                os.chdir(previous)
            self.assertEqual(len(commands), 3)
            for command in commands:
                self.assertEqual(command.count('--population-config'), 1)
                self.assertEqual(command[command.index('--population-config')+1], str(root/'population.json'))
                self.assertNotIn('--creature-profile', command)
            self.assertNotIn('--creature-asset', commands[0])
            for command in commands[1:]:
                self.assertEqual(command[command.index('--creature-asset')+1], str(root/'wolf.glb'))
            self.assertEqual(stopped, ['server'])
            self.assertEqual(list(session.glob('renet-*')), [])
            if isolated:
                self.assertFalse((root/'build/development-session').exists())
                if os.name != 'nt':
                    self.assertEqual(session.stat().st_mode & 0o777, 0o700)


if __name__ == '__main__':
    unittest.main()
