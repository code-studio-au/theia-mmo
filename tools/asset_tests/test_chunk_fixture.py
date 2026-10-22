"""Original fixture GLBs bind their matching collision and chunk owners."""
import hashlib
import json
from pathlib import Path
import sys
import tempfile
import unittest
import numpy as np
import trimesh
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from create_chunk_fixture import create
from package_world import identity


class ChunkFixtureTests(unittest.TestCase):
    def test_shape_fields_publish_complete_contributors_at_far_global_coordinates(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / 'fixture'
            create(root, 1000000000, -1000000000, static_shapes=True)
            index = json.loads((root/'world.chunks.json').read_text())
            session = json.loads((root/'world.session.json').read_text())
            self.assertEqual(index['schema'], 2)
            self.assertEqual(index['query_origin'], [128e9,0,-128e9])
            self.assertEqual(session['resident_budget'], 3)
            self.assertEqual(session['initial_chunks'], [entry['id'] for entry in index['chunks']])
            for entry in index['chunks']:
                chunk = root / f"chunk_{entry['id'][0]}_{entry['id'][1]}"
                profile = json.loads((chunk/'terrain.json').read_text())
                manifest = json.loads((chunk/'world.manifest.json').read_text())
                self.assertEqual(manifest['collision_schema'], 4)
                self.assertEqual(entry['package'], list(bytes.fromhex(identity(manifest))))
                self.assertEqual(profile['triangles'], [])
                self.assertEqual(profile['static_shapes']['heightfields'][0]['heights_column_major'], [0]*4)
                # Bounds include the complete 1 m overlap and the millimetre
                # reserve, rounded outward by the established publisher.
                self.assertEqual(entry['bounds'], [[-1.001953125,-1.001953125],[129.001953125,129.001953125]])

    def test_three_original_chunks_bind_visuals_profiles_and_far_global_roots(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / 'fixture'
            report = create(root, -7812500, 7812500)
            self.assertEqual(report['chunks'], 3)
            index = json.loads((root / 'world.chunks.json').read_text())
            self.assertEqual(index['query_origin'], [-1e9, 0., 1e9])
            session = json.loads((root / 'world.session.json').read_text())
            owners = {tuple(entry['id']) for entry in index['chunks']}
            initial = {tuple(entry) for entry in session['initial_chunks']}
            self.assertTrue(initial <= owners)
            self.assertLessEqual(len(initial), session['resident_budget'])
            for x, y, z in session['spawns']:
                owner = (int(np.floor(x / 128.)), int(np.floor(z / 128.)))
                self.assertIn(owner, initial)
                # Independent fixture support: spawn capsule's full footprint
                # stays inside a real horizontal floor, clear of the wall.
                self.assertEqual(y, 0.)
                local = [x - owner[0] * 128., z - owner[1] * 128.]
                self.assertTrue(all(0.355 < value < 127.645 for value in local))
                self.assertGreater(abs(local[0] - 64.), 0.455)

            for entry in index['chunks']:
                x, z = entry['id']
                chunk = root / f'chunk_{x}_{z}'
                manifest = json.loads((chunk / 'world.manifest.json').read_text())
                self.assertEqual(entry['package'], list(bytes.fromhex(identity(manifest))))
                profile = json.loads((chunk / 'terrain.json').read_text())
                self.assertEqual(profile['visual_sha256'], list(hashlib.sha256((chunk / 'terrain.glb').read_bytes()).digest()))
                meshes = trimesh.load_scene(chunk / 'terrain.glb', process=False).geometry.values()
                self.assertTrue(any(np.array_equal(mesh.triangles, profile['triangles']) for mesh in meshes))
            with self.assertRaisesRegex(ValueError, 'must be new'):
                create(root)

    def test_invalid_base_fails_before_creating_output(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / 'fixture'
            with self.assertRaisesRegex(ValueError, 'global bounds'):
                create(root, 2**63, 0)
            self.assertFalse(root.exists())
