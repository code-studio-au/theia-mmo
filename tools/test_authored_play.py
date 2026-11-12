"""Authored-world launcher selection; no window, OS input or server is started."""
import contextlib
import inspect
import io
from pathlib import Path
import unittest
from unittest.mock import patch
import play


class AuthoredPlay(unittest.TestCase):
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

    def test_mixed_world_and_unqualified_bundle_refuse_before_launch(self):
        with patch.object(play, 'play') as launch, contextlib.redirect_stderr(io.StringIO()):
            for extra in (['--saved-terrain', 'saved'], ['--world', 'legacy'],
                          ['--gameplay-manifest', 'bundle.json']):
                with self.subTest(extra=extra), self.assertRaises(SystemExit):
                    play.main(['--authored-world', 'authored', *extra])
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
                with patch.object(Path,'is_file',return_value=True), patch.object(play.subprocess,'Popen',capture):
                    with self.assertRaises(CapturedLaunch):
                        play.play(None,Path('players'),1,Path('binaries'),prediction=True,
                                  authored_world=Path('authored'))
            finally:
                os.chdir(previous)
        self.assertEqual(len(calls),1)
        self.assertIn('--authored-world',calls[0])
        self.assertNotIn('--dev-spawn',calls[0])


if __name__ == '__main__':
    unittest.main()
