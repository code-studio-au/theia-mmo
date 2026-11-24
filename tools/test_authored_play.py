"""Authored-world launcher selection; no window, OS input or server is started."""
import contextlib
import inspect
import json
from types import SimpleNamespace
import io
from pathlib import Path
import unittest
from unittest.mock import patch
import play


class AuthoredPlay(unittest.TestCase):
    def test_chat_area_cli_retains_explicit_account_profile(self):
        signature = inspect.signature(play.play)
        observed = {}
        with patch.object(play, 'play', lambda *a, **k: observed.update(signature.bind(*a, **k).arguments)):
            play.main(['--authored-world', 'valley', '--classic', '--account-login', '--chat-area', '7'])
        self.assertEqual(observed['chat_area'], 7)
        self.assertTrue(observed['account_login'])

    def test_chat_profile_refuses_invalid_area_or_anonymous_launch_before_children(self):
        with patch.object(play.subprocess, 'Popen') as child:
            for extra in [dict(chat_area=0, classic=True, account_login=True),
                          dict(chat_area=7, classic=False, account_login=True),
                          dict(chat_area=7, classic=True, account_login=False),
                          dict(chat_area=True, classic=True, account_login=True)]:
                with self.assertRaisesRegex(ValueError, 'Chat requires'):
                    play.play(None, Path('players'), 1, Path('bin'), **extra)
            child.assert_not_called()

    def test_route_cli_reaches_launcher_as_client_only_selection(self):
        signature = inspect.signature(play.play)
        observed = {}
        def launch(*args, **kwargs):
            observed.update(signature.bind(*args, **kwargs).arguments)
        with patch.object(play, 'play', launch):
            play.main(['--authored-world', 'valley', '--classic', '--test-route', 'route.json'])
        self.assertEqual(observed['test_route'], Path('route.json').resolve())

    def test_route_requires_single_authored_classic_client_before_any_child(self):
        with patch.object(play.subprocess, 'Popen') as child:
            for options in [dict(authored_world=None, classic=True, clients=1),
                            dict(authored_world=Path('world'), classic=False, clients=1),
                            dict(authored_world=Path('world'), classic=True, clients=2)]:
                with self.subTest(options=options), self.assertRaisesRegex(ValueError, 'Test route requires'):
                    play.play(None, Path('players'), binaries=Path('bin'), prediction=True,
                              test_route=Path('route.json'), **options)
            child.assert_not_called()

    def test_route_is_preflighted_before_server_or_account_login(self):
        with patch.object(Path, 'is_file', return_value=True), \
                patch.object(play.subprocess, 'run', return_value=SimpleNamespace(
                    returncode=1, stdout='', stderr='Test route preparation: wrong floor')) as preflight, \
                patch.object(play.subprocess, 'Popen') as child:
            with self.assertRaisesRegex(ValueError, 'wrong floor'):
                play.play(None, Path('players'), 1, Path('bin'), prediction=True,
                          authored_world=Path('world'), classic=True, test_route=Path('route.json'))
            command = preflight.call_args.args[0]
            self.assertIn('--configuration-report', command)
            self.assertEqual(command[command.index('--test-route') + 1], str(Path('route.json').resolve()))
            child.assert_not_called()

    def test_authored_selection_never_falls_back_to_saved_terrain(self):
        observed = {}
        def launch(*args, **kwargs):
            observed.update(inspect.signature(play_function).bind(*args, **kwargs).arguments)
        play_function = play.play
        with patch.object(play, 'play', launch):
            play.main(['--authored-world', 'build/starter-zone-world-v1', '--classic'])
        self.assertEqual(observed['authored_world'], Path('build/starter-zone-world-v1').resolve())
        self.assertIsNone(observed['saved_terrain'])
        self.assertIsNone(observed['world'])
        self.assertTrue(observed['prediction'])
        self.assertTrue(observed['classic'])
        self.assertIsNone(observed['dev_spawn'])

    def test_mixed_world_and_bundle_without_navigation_refuse_before_launch(self):
        with patch.object(play, 'play') as launch, contextlib.redirect_stderr(io.StringIO()):
            for extra in (['--saved-terrain', 'saved'], ['--world', 'legacy'],
                          ['--gameplay-manifest', 'bundle.json']):
                with self.subTest(extra=extra), self.assertRaises(SystemExit):
                    play.main(['--authored-world', 'authored', *extra])
            launch.assert_not_called()

    def test_authored_bundle_cannot_override_its_bound_contact(self):
        with patch.object(play, 'play') as launch, contextlib.redirect_stderr(io.StringIO()):
            with self.assertRaises(SystemExit):
                play.main(['--authored-world', 'authored', '--gameplay-manifest', 'bundle.json',
                           '--navigation', 'nav', '--guide-position', '8004,8000'])
            launch.assert_not_called()

    def test_unsupported_travel_or_missing_package_never_starts_children(self):
        with patch.object(play.subprocess, 'Popen') as child:
            for options in ({'prediction': False}, {'prediction': True, 'dev_tour': True},
                            {'prediction': True, 'saved_terrain': Path('saved')},
                            {'prediction': True, 'dev_spawn': '8000,8000'},
                            {'prediction': True, 'dev_spawn': '8192,8192'}):
                with self.subTest(options=options), self.assertRaises(ValueError):
                    play.play(None, Path('players'), 1, Path('binaries'),
                              authored_world=Path('missing-authored-package'), **options)
            with self.assertRaises((ValueError, FileNotFoundError)):
                play.play(None, Path('players'), 1, Path('binaries'), prediction=True,
                          authored_world=Path('missing-authored-package'))
            child.assert_not_called()

    def test_authored_server_command_does_not_override_saved_spawn(self):
        import os
        import tempfile
        class CapturedLaunch(Exception):
            pass
        calls=[]
        def capture(arguments, **kwargs):
            calls.append(arguments)
            raise CapturedLaunch()
        previous=Path.cwd()
        with tempfile.TemporaryDirectory() as folder:
            try:
                os.chdir(folder)
                with patch.object(Path,'is_file',return_value=True), patch.object(play.subprocess,'Popen',capture), patch.object(play.subprocess, 'run', return_value=SimpleNamespace(returncode=0, stdout=json.dumps(dict(renderer_started=False, network_started=False, saved_state_admitted=False)), stderr='')):
                    for extra in ({}, dict(creature_profile=Path('wolf'), navigation=Path('nav'),
                                           combat_catalog=Path('abilities')),
                                  dict(dialogue_catalog=Path('dialogue'), guide_position='8194,0.1,8192'),
                                  dict(dialogue_catalog=Path('dialogue'), guide_position='authored')):
                        with self.assertRaises(CapturedLaunch):
                            play.play(None,Path('players'),1,Path('binaries'),prediction=True,
                                      authored_world=Path('authored'), **extra)
            finally:
                os.chdir(previous)
        self.assertEqual(len(calls),4)
        for call in calls:
            self.assertIn('--authored-world',call)
            self.assertNotIn('--dev-spawn',call)
            self.assertNotIn('--saved-terrain',call)
        self.assertIn('--creature-profile',calls[1])
        self.assertIn('--navigation',calls[1])
        self.assertEqual(calls[2][calls[2].index('--guide-position') + 1], '8194,0.1,8192')
        self.assertEqual(calls[3][calls[3].index('--guide-position') + 1], 'authored')

    def test_invalid_guide_height_or_coordinate_refuses_before_starting_children(self):
        with patch.object(play.subprocess, 'Popen') as child:
            for position in ['8192,NaN,8192', '8192,inf,8192', '1,2,3,4', '-1,0,8192', '8192,0,16000']:
                with self.subTest(position=position), self.assertRaisesRegex(ValueError, 'Guide position'):
                    play.play(None, Path('players'), 1, Path('binaries'), prediction=True,
                              authored_world=Path('valley'), dialogue_catalog=Path('dialogue'),
                              guide_position=position)
            child.assert_not_called()

    def test_authored_account_walking_uses_normal_admission_and_never_local_tickets(self):
        import os
        import tempfile
        class CapturedLaunch(Exception): pass
        calls = []
        def capture(arguments, **kwargs):
            calls.append(arguments)
            raise CapturedLaunch()
        previous = Path.cwd()
        with tempfile.TemporaryDirectory() as folder:
            try:
                os.chdir(folder)
                with patch.dict(os.environ, dict(THEIA_DATABASE_URL='synthetic-not-connected', THEIA_OIDC_ISSUER='http://127.0.0.1:1', THEIA_OIDC_CLIENT_ID='fixture')), patch.object(Path, 'is_file', return_value=True), patch.object(play.subprocess, 'Popen', capture):
                    with self.assertRaises(CapturedLaunch):
                        play.play(None, Path('players'), 1, Path('binaries'), prediction=True,
                                  classic=True, authored_world=Path('authored'), account_login=True)
            finally:
                os.chdir(previous)
        self.assertEqual(len(calls), 1)
        self.assertIn('--account-login', calls[0])
        self.assertIn('--authored-world', calls[0])
        self.assertNotIn('--saved-terrain', calls[0])
        self.assertNotIn('--renet-tickets', calls[0])
        self.assertNotIn('--dev-spawn', calls[0])

    def test_authored_patrol_preflight_failure_prevents_server_and_login_start(self):
        with patch.object(Path, 'is_file', return_value=True), patch.object(play.subprocess, 'run', return_value=SimpleNamespace(returncode=1, stdout='', stderr='Patrol route is absent')), patch.object(play.subprocess, 'Popen') as children:
            with self.assertRaisesRegex(ValueError, 'Patrol route is absent'):
                play.play(None, Path('players'), 1, Path('binaries'), prediction=True,
                          authored_world=Path('authored'), creature_profile=Path('wolf'),
                          navigation=Path('navigation'), combat_catalog=Path('abilities'))
            children.assert_not_called()

    def test_authored_creature_flags_reach_the_real_launcher_without_saved_terrain(self):
        observed = {}
        signature = inspect.signature(play.play)
        def launch(*args, **kwargs):
            observed.update(signature.bind(*args, **kwargs).arguments)
        with patch.object(play, 'play', launch):
            play.main(['--authored-world', 'valley', '--classic', '--creatures',
                       '--creature-profile', 'content/gameplay/starter-valley-wolf-patrol.json',
                       '--navigation', 'build/starter-valley-navigation-v1'])
        self.assertIsNone(observed['saved_terrain'])
        self.assertEqual(observed['authored_world'], Path('valley').resolve())
        self.assertEqual(observed['creature_profile'].name, 'starter-valley-wolf-patrol.json')
        self.assertIsNotNone(observed['combat_catalog'])
        self.assertIsNotNone(observed['character_stats'])
        self.assertFalse(observed['account_login'])

    def test_authored_patrol_requires_its_navigation_and_preserves_account_gate(self):
        options = dict(prediction=True, classic=True, authored_world=Path('valley'),
                       combat_catalog=Path('abilities'), creature_profile=Path('wolf'))
        with patch.object(play.subprocess, 'Popen') as child:
            with self.assertRaisesRegex(ValueError, 'selected together'):
                play.play(None, Path('players'), 1, Path('binaries'), **options)
            with self.assertRaisesRegex(ValueError, 'Missing baked navigation'):
                play.play(None, Path('players'), 1, Path('binaries'), navigation=Path('absent'), **options)
            with self.assertRaisesRegex(ValueError, 'Account login requires'):
                play.play(None, Path('players'), 1, Path('binaries'), navigation=Path('absent'), account_login=True, **options)
            child.assert_not_called()


if __name__ == '__main__':
    unittest.main()
