"""Exercise real patch preparation and verify visual/collision triangle identity."""
import hashlib
import json
from pathlib import Path
import sys
import tempfile
import unittest

import numpy as np
from PIL import Image
import trimesh

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from prepare_terrain_world import prepare


class TerrainWorldTests(unittest.TestCase):
    def fixture(self, root):
        source = root / 'terrain.obj'
        source.write_text('mtllib terrain.mtl\nv 100 20 200\nv 100 21 210\nv 110 22 200\n'
                          'v 200 20 200\nv 200 21 210\nv 210 22 200\n'
                          'vt 0 0\nvt 0 1\nvt 1 0\nvn 0 1 0\n'
                          'usemtl grass\nf 1/1/1 2/2/1 3/3/1\nf 4/1/1 5/2/1 6/3/1\n')
        (root / 'terrain.mtl').write_text('newmtl grass\nmap_Kd grass.png\n')
        Image.new('RGB', (2, 2), (32, 64, 48)).save(root / 'grass.png')
        return source

    def test_rendered_positions_equal_collision_and_keep_texture_uv_normals(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = self.fixture(root)
            original = source.read_bytes()
            report = prepare(source, root / 'out', [100, 20, 200])
            profile_bytes = (root / 'out/terrain.json').read_bytes()
            profile = json.loads(profile_bytes)
            glb = root / 'out/terrain.glb'
            self.assertEqual(bytes(profile['visual_sha256']).hex(), hashlib.sha256(glb.read_bytes()).hexdigest())
            self.assertEqual(report['world_sha256'], hashlib.sha256(profile_bytes).hexdigest())
            self.assertEqual(report['triangles'], 1)  # remote face excluded
            mesh, = trimesh.load_scene(glb, process=False).geometry.values()
            np.testing.assert_array_equal(mesh.triangles, profile['triangles'])
            np.testing.assert_array_equal(mesh.triangles[0], [[0, 0, 0], [0, 1, 10], [10, 2, 0]])
            self.assertIsNotNone(mesh.visual.uv)
            np.testing.assert_array_equal(mesh.vertex_normals, [[0, 1, 0]] * 3)
            np.testing.assert_array_equal(np.asarray(mesh.visual.material.baseColorTexture.convert('RGB')),
                                          np.full((2, 2, 3), [32, 64, 48], dtype=np.uint8))
            self.assertEqual(source.read_bytes(), original)
            with self.assertRaisesRegex(ValueError, 'must be new'):
                prepare(source, root / 'out', [100, 20, 200])

    def test_missing_texture_empty_patch_and_invalid_origin_fail_before_writing(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = self.fixture(root)
            for center in [[1000, 0, 1000], [float('nan'), 0, 0], [0, 0]]:
                with self.assertRaises(ValueError): prepare(source, root / 'out', center)
                self.assertFalse((root / 'out').exists())
            (root / 'grass.png').unlink()
            with self.assertRaisesRegex(ValueError, 'Missing terrain texture'):
                prepare(source, root / 'out', [100, 20, 200])
            self.assertFalse((root / 'out').exists())
