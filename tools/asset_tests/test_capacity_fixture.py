"""Native full-capacity fixtures bind real original geometry and shared images."""
import json
from pathlib import Path
import sys
import tempfile
import unittest
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from create_capacity_fixture import create
from measure_chunk_budgets import contributors
from index_chunk_contributors import package_bounds


class CapacityFixtureTests(unittest.TestCase):
    def test_initial_thirty_two_roots_share_one_immutable_image_variant(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary) / 'world'
            result = create(root)
            self.assertEqual((result['packages'], result['initial']), (32, 32))
            self.assertEqual(len(list((root / 'shared').iterdir())), 1)
            index = json.loads((root / 'world.chunks.json').read_bytes())
            self.assertTrue(all(len(values) == 1 for values in contributors(index['chunks']).values()))
            with self.assertRaises(ValueError):
                create(root)

    def test_six_new_providers_are_required_at_the_loading_edge(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary) / 'world'
            create(root, 'publication', -7_812_500)
            index = json.loads((root / 'world.chunks.json').read_bytes())
            required = contributors(index['chunks'])
            self.assertEqual(len(required[(-7_812_500, -7_812_500)]), 1)
            self.assertEqual(len(required[(-7_812_499, -7_812_500)]), 6)
            for entry in index['chunks']:
                x, z = entry['id']
                self.assertEqual(package_bounds(root / f'chunk_{x}_{z}', entry['package']), entry['bounds'])
            self.assertEqual(len(json.loads((root / 'world.session.json').read_bytes())['initial_chunks']), 1)
