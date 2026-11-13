"""Opt-in real editor startup checks, with no renderer or display connection."""
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
EDITOR = os.environ.get('THEIA_EDITOR_BIN')


@unittest.skipUnless(EDITOR, 'requires THEIA_EDITOR_BIN')
class CreatureDefinitionCliTests(unittest.TestCase):
    def run_editor(self, *arguments):
        env = dict(os.environ)
        for key in ('DISPLAY', 'WAYLAND_DISPLAY'):
            env.pop(key, None)
        return subprocess.run([EDITOR, '--headless', *map(str, arguments)], cwd=ROOT,
                              env=env, capture_output=True, text=True, timeout=30)

    def test_standalone_and_legacy_files_validate_without_writing_the_source(self):
        for name in ('development-creature-passive.json', 'development-creature-patrol.json'):
            path = ROOT / 'content/gameplay' / name
            original = path.read_bytes()
            result = self.run_editor('--creature-definition', path)
            self.assertEqual(result.returncode, 0, result.stderr)
            context = json.loads(result.stdout)['creature_definition']
            self.assertEqual(context['id'], 'theia.dev.training-creature')
            self.assertEqual(Path(context['path']), path)
            self.assertEqual(path.read_bytes(), original)

    def test_bad_context_refuses_before_output_and_export_is_unambiguous(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            definition = root / 'invalid.json'
            definition.write_text('{"schema":3,"id":"theia.dev.invalid"}')
            output = root / 'zone.json'
            output.write_bytes(b'existing user file')
            for path in (definition, root / 'missing.json'):
                result = self.run_editor('--creature-definition', path, '--output', output)
                self.assertNotEqual(result.returncode, 0)
                self.assertEqual(output.read_bytes(), b'existing user file')
            package = root / 'package'
            result = self.run_editor('--creature-definition', definition, '--export', package,
                                     '--content-id', 'theia.test.context', '--revision', '1')
            self.assertNotEqual(result.returncode, 0)
            self.assertIn('editor context', result.stderr)
            self.assertFalse(package.exists())


if __name__ == '__main__':
    unittest.main()
