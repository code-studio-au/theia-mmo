"""Cosmetic input bounds and the real private creation handoff."""
import json
from pathlib import Path
import tempfile
from types import SimpleNamespace
import unittest
from character_appearance import validate, FIELDS, COUNTS
from account_selection import select_character

APPEARANCE = dict(schema=1, body=1, face=2, hair=4, facial_hair=0, skin=2, hair_colour=3, outfit=1)


class Appearance(unittest.TestCase):
    def test_rejects_each_invalid_boundary_boolean_and_combination(self):
        self.assertEqual(validate(APPEARANCE), APPEARANCE)
        for key, count in zip(FIELDS, COUNTS):
            for invalid in (-1, count, True, 0.5):
                with self.subTest(key=key, invalid=invalid), self.assertRaises(ValueError):
                    validate(dict(APPEARANCE, **{key: invalid}))
        for changes in ({'schema': 2}, {'unknown': 0}, {'facial_hair': 1}):
            with self.assertRaises(ValueError):
                validate(dict(APPEARANCE, **changes))

    def test_handoff_passes_exact_validated_appearance_and_never_accepts_path(self):
        for appearance in (APPEARANCE, dict(APPEARANCE, mesh='/tmp/arbitrary.glb')):
            with tempfile.TemporaryDirectory() as folder:
                root = Path(folder)
                (root/'selection.json').write_text(json.dumps(dict(schema=1, create_name='Arden', appearance=appearance)))
                calls = []
                def create(name, value):
                    calls.append((name, value))
                    return {'id': 'synthetic'}
                account = SimpleNamespace(characters=lambda: [], create_character=create)
                if appearance == APPEARANCE:
                    self.assertEqual(select_character(account, root, lambda: False), 'synthetic')
                    self.assertEqual(calls, [('Arden', APPEARANCE)])
                else:
                    with self.assertRaises(ValueError):
                        select_character(account, root, lambda: False)
                    self.assertEqual(calls, [])

if __name__ == '__main__':
    unittest.main()
