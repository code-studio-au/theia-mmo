import os
from pathlib import Path
import tempfile
import sys
import unittest

from edit_zone import build_environment, editor_arguments


class EditorLaunchTests(unittest.TestCase):
    def test_new_draft_sets_only_the_later_save_path(self):
        with tempfile.TemporaryDirectory() as root:
            draft = Path(root) / 'new.json'
            self.assertEqual(editor_arguments(['--headless'], draft), ['--headless', '--save-path', str(draft)])
            self.assertFalse(draft.exists())

    def test_existing_draft_is_reopened_without_output_or_byte_changes(self):
        with tempfile.TemporaryDirectory() as root:
            draft = Path(root) / 'draft.json'
            draft.write_bytes(b'user-authored bytes')
            self.assertEqual(editor_arguments([], draft), ['--zone', str(draft)])
            self.assertEqual(draft.read_bytes(), b'user-authored bytes')

    def test_explicit_authoring_arguments_are_preserved(self):
        for flag in ['--zone', '--commands', '--output', '--save-path', '--export']:
            arguments = [flag, 'caller-relative.file', '--headless']
            self.assertEqual(editor_arguments(arguments, Path('unused.json')), arguments)

    def test_build_environment_preserves_caller_settings_and_does_not_mutate_them(self):
        with tempfile.TemporaryDirectory() as root:
            source = {'PKG_CONFIG_PATH': '/caller/pkg', 'LIBRARY_PATH': '/caller/lib', 'CUSTOM': 'kept'}
            original = dict(source)
            sdk = Path(root) / 'build/native-sdk/usr/lib/x86_64-linux-gnu'
            (sdk / 'pkgconfig').mkdir(parents=True)
            result = build_environment(source, Path(root))
            self.assertEqual(source, original)
            self.assertEqual(result['CUSTOM'], 'kept')
            if os.name == 'posix' and sys.platform.startswith('linux'):
                self.assertEqual(result['PKG_CONFIG_PATH'], str(sdk/'pkgconfig')+os.pathsep+'/caller/pkg')
                self.assertEqual(result['LIBRARY_PATH'], str(sdk)+os.pathsep+'/caller/lib')


if __name__ == '__main__':
    unittest.main()
