import os
from pathlib import Path
import tempfile
import sys
import unittest

from edit_zone import build_environment, editor_arguments, example_arguments, scenery_arguments


class EditorLaunchTests(unittest.TestCase):
    def test_local_scenery_is_optional_and_explicit_catalogs_win(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            arguments = ['--zone', 'my.zone.json']
            self.assertEqual(scenery_arguments(arguments, root), arguments)
            catalog = root / 'build/starter-forest-scenery-v2/catalog.json'
            catalog.parent.mkdir(parents=True)
            catalog.write_bytes(b'caller-owned catalog bytes')
            self.assertEqual(scenery_arguments(arguments, root), [*arguments, '--scenery-library', str(catalog)])
            self.assertEqual(scenery_arguments(arguments, root, enabled=False), arguments)
            explicit = [*arguments, '--scenery-library', 'custom.json']
            self.assertEqual(scenery_arguments(explicit, root), explicit)
            self.assertEqual(catalog.read_bytes(), b'caller-owned catalog bytes')
            self.assertEqual(arguments, ['--zone', 'my.zone.json'])

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

    def test_example_seed_selects_a_separate_nonwriting_destination(self):
        with tempfile.TemporaryDirectory() as root:
            root = Path(root)
            draft = root / 'new-example.json'
            arguments = example_arguments(['--headless'], draft, root)
            self.assertEqual(arguments, ['--headless', '--zone', str(root/'content/zones/wayfarer-hollow.base.json'),
                                         '--commands', str(root/'content/zones/wayfarer-hollow.commands.jsonl'), '--save-path', str(draft)])
            self.assertFalse(draft.exists())

    def test_example_reopens_user_changes_without_reapplying_the_seed(self):
        with tempfile.TemporaryDirectory() as root:
            draft = Path(root) / 'example.json'
            draft.write_bytes(b'user authored example')
            self.assertEqual(example_arguments([], draft), ['--zone', str(draft)])
            self.assertEqual(draft.read_bytes(), b'user authored example')

    def test_example_refuses_ambiguous_explicit_authoring_flags(self):
        for flag in ['--zone', '--commands', '--output', '--save-path', '--export']:
            with self.assertRaises(ValueError):
                example_arguments([flag, 'path'], Path('unused.json'))

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
