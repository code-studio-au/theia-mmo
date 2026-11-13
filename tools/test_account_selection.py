"""Private character handoff validation and cancellation; no browser or sockets."""
import json
import os
from pathlib import Path
import tempfile
import unittest
from types import SimpleNamespace
from account_selection import select_character

ID = '74686569-6100-0000-0000-000000000101'


class Selection(unittest.TestCase):
    def test_only_listed_character_is_selected_and_snapshot_is_private(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            (root/'selection.json').write_text(json.dumps(dict(schema=1, character_id=ID)))
            account = SimpleNamespace(characters=lambda: [dict(id=ID, name='Arden')])
            self.assertEqual(select_character(account, root, lambda: False), ID)
            self.assertEqual(json.loads((root/'characters.json').read_text()),
                             dict(schema=1, characters=[dict(id=ID, name='Arden')]))
            self.assertFalse((root/'characters.pending').exists())
            if os.name != 'nt':
                self.assertEqual((root/'characters.json').stat().st_mode & 0o777, 0o600)

    def test_invalid_unlisted_or_oversized_selection_never_returns_an_id(self):
        for raw in ['{', 'x'*1025, 'null',
                    json.dumps(dict(schema=True, character_id=ID)),
                    json.dumps(dict(schema=1, character_id='other')),
                    json.dumps(dict(schema=1, character_id=ID, create='Someone'))]:
            with self.subTest(raw=raw[:60]), tempfile.TemporaryDirectory() as folder:
                root = Path(folder)
                (root/'selection.json').write_text(raw)
                account = SimpleNamespace(characters=lambda: [dict(id=ID, name='Arden')])
                with self.assertRaisesRegex(ValueError, 'Invalid account'):
                    select_character(account, root, lambda: False)

    def test_cancellation_before_publication_does_not_query_account(self):
        def unwanted():
            self.fail('No request should be made after owner cancellation')
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            with self.assertRaises(KeyboardInterrupt):
                select_character(SimpleNamespace(characters=unwanted), root, lambda: True)
            self.assertEqual(list(root.iterdir()), [])

    def test_cancellation_while_waiting_leaves_no_selected_id(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            account = SimpleNamespace(characters=lambda: [dict(id=ID, name='Arden')])
            with self.assertRaises(KeyboardInterrupt):
                select_character(account, root, lambda: (root/'characters.json').exists())
            self.assertFalse((root/'selection.json').exists())

    def test_empty_accounts_and_missing_selection_end_without_implicit_creation(self):
        for rows in [[], [dict(id=ID, name='Arden')]]:
            with self.subTest(rows=rows), tempfile.TemporaryDirectory() as folder:
                account = SimpleNamespace(characters=lambda: rows)
                with self.assertRaises(ValueError if not rows else TimeoutError):
                    select_character(account, Path(folder), lambda: False, timeout=0)


if __name__ == '__main__':
    unittest.main()
