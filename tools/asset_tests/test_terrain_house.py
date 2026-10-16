"""Qualify authored house placement, sloped foundation and binary preservation."""
import hashlib
import json
from pathlib import Path
import sys
import tempfile
import unittest

import numpy as np
import trimesh
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from add_terrain_house import add_house, clipped_heights
from merge_static_glb import read


class TerrainHouseTests(unittest.TestCase):
    def fixture(self, root):
        terrain = root / 'terrain'
        terrain.mkdir()
        vertices = np.array([[-24, -2.4, -24], [24, 2.4, -24], [24, 2.4, 24], [-24, -2.4, 24]])
        mesh = trimesh.Trimesh(vertices=vertices, faces=[[0, 2, 1], [0, 3, 2]],
                               vertex_normals=[[0, 1, 0]] * 4, process=False)
        glb = trimesh.exchange.gltf.export_glb(trimesh.Scene(mesh), include_normals=True)
        (terrain / 'terrain.glb').write_bytes(glb)
        profile = {'schema': 1, 'source_origin': [0, 0, 0], 'visual_sha256': list(hashlib.sha256(glb).digest()),
                   'triangles': mesh.triangles.tolist()}
        (terrain / 'terrain.json').write_text(json.dumps(profile))
        house = trimesh.creation.box(extents=[4, 6, 4])
        house.vertex_normals = np.asarray(house.vertices) / np.linalg.norm(house.vertices, axis=1)[:, None]
        house.apply_translation([1, 2, 3])
        source = root / 'house.glb'
        source.write_bytes(trimesh.exchange.gltf.export_glb(trimesh.Scene(house), include_normals=True))
        return terrain, source

    def test_house_fits_envelope_foundation_matches_clipped_slope_and_buffers_survive(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            terrain, house = self.fixture(root)
            original = {p: p.read_bytes() for p in [terrain / 'terrain.glb', terrain / 'terrain.json', house]}
            report = add_house(terrain, house, root / 'out')
            self.assertTrue(report['terrain_binary_preserved'] and report['house_binary_preserved'])
            low, high = report['foundation_y']
            self.assertAlmostEqual(low, -1.33, places=6)
            self.assertAlmostEqual(high, -.37, places=6)
            source = trimesh.load_scene(house, process=False)
            corners = trimesh.bounds.corners(source.bounds)
            placed = trimesh.transform_points(corners, report['house_transform'])
            solid = report['solid']
            lower = np.array(solid['center']) - np.array(solid['size']) / 2
            upper = np.array(solid['center']) + np.array(solid['size']) / 2
            self.assertTrue(np.all(placed >= lower - 1e-6) and np.all(placed <= upper + 1e-6))
            self.assertAlmostEqual(placed[:, 1].min(), high, places=6)
            _, final_bin = read(root / 'out/terrain.glb')
            _, terrain_bin = read(terrain / 'terrain.glb')
            _, house_bin = read(house)
            self.assertEqual(final_bin[:len(terrain_bin)], terrain_bin)
            self.assertEqual(final_bin[len(terrain_bin):len(terrain_bin) + len(house_bin)], house_bin)
            profile = json.loads((root / 'out/terrain.json').read_text())
            self.assertEqual(bytes(profile['visual_sha256']).hex(), report['visual_sha256'])
            self.assertEqual(profile['triangles'], json.loads((terrain / 'terrain.json').read_text())['triangles'])
            for path, data in original.items(): self.assertEqual(path.read_bytes(), data)
            with self.assertRaisesRegex(ValueError, 'must be new'): add_house(terrain, house, root / 'out')

    def test_mismatched_visual_and_already_authored_profile_fail_before_writing(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            terrain, house = self.fixture(root)
            path = terrain / 'terrain.json'
            profile = json.loads(path.read_text())
            profile['visual_sha256'] = [0] * 32
            path.write_text(json.dumps(profile))
            with self.assertRaisesRegex(ValueError, 'hash mismatch'): add_house(terrain, house, root / 'out')
            profile['solids'] = [{}]
            path.write_text(json.dumps(profile))
            with self.assertRaisesRegex(ValueError, 'terrain-only'): add_house(terrain, house, root / 'out')
            self.assertFalse((root / 'out').exists())
            with self.assertRaisesRegex(ValueError, 'No terrain'): clipped_heights([], -12, -4)
