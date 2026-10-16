import copy
from pathlib import Path
import sys
import tempfile
import unittest

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from add_map_liquids import UNIT_SIZE, add_liquids, build_scene, surface


def instance(**changes):
    value = {'chunkIndex': 0, 'instanceIndex': 0, 'width': 2, 'height': 2,
             'xOffset': 1, 'yOffset': 2, 'terrainChunkPosition': [100., 200., 900.],
             'worldPosition': [0., 9999., 0.], 'minHeightLevel': 7., 'maxHeightLevel': 7.,
             'vertexData': {'height': [7.] * 9}, 'bitmap': [], 'liquidType': 5, 'liquidObject': 18472}
    value.update(changes)
    return value


class LiquidTests(unittest.TestCase):
    def test_chunk_offsets_absolute_heights_and_upward_winding(self):
        mesh = surface(instance())
        self.assertEqual(len(mesh.faces), 8)
        np.testing.assert_allclose(mesh.vertices[0], [200 - UNIT_SIZE, 7, 100 - 2 * UNIT_SIZE])
        np.testing.assert_allclose(mesh.bounds[:, 1], [7, 7])
        self.assertTrue((mesh.face_normals[:, 1] > .999).all())
        # Chunk Z=900 and misleading worldPosition Y=9999 are never added to height.
        self.assertEqual(mesh.bounds[1, 1], 7)

    def test_bitmap_bits_cross_byte_boundaries_in_row_major_order(self):
        mesh = surface(instance(width=3, height=3, vertexData={'height': [7.] * 16}, bitmap=[128, 1]))
        self.assertEqual(len(mesh.faces), 4)
        self.assertEqual(len(mesh.vertices), 6)
        np.testing.assert_allclose(mesh.bounds[:, 2], [100 - 5 * UNIT_SIZE, 100 - 4 * UNIT_SIZE])

    def test_sloped_samples_generate_unit_normals_from_the_actual_surface(self):
        heights = [7 + col * UNIT_SIZE for row in range(3) for col in range(3)]
        mesh = surface(instance(vertexData={'height': heights}))
        expected = np.tile([2**-.5, 2**-.5, 0.], (len(mesh.vertices), 1))
        np.testing.assert_allclose(mesh.vertex_normals, expected, atol=1e-12)

    def test_masked_zero_samples_do_not_change_geometry_bounds(self):
        heights = [0.] * 9
        for index in [0, 1, 3, 4]:
            heights[index] = 7.
        mesh = surface(instance(vertexData={'height': heights}, bitmap=[1]))
        self.assertEqual((len(mesh.vertices), len(mesh.faces)), (4, 2))
        np.testing.assert_allclose(mesh.bounds[:, 1], [7, 7])
        self.assertIsNone(surface(instance(bitmap=[0])))

    def test_missing_heights_require_a_declared_flat_surface(self):
        mesh = surface(instance(vertexData={}, minHeightLevel=5., maxHeightLevel=5.))
        np.testing.assert_allclose(mesh.bounds[:, 1], [5, 5])
        for invalid in [instance(vertexData={}, maxHeightLevel=8.),
                        instance(vertexData={'height': [7.]}),
                        instance(vertexData={'height': [float('nan')] * 9})]:
            with self.assertRaises(ValueError):
                surface(invalid)

    def test_invalid_masks_rectangles_positions_and_chunk_identity_fail(self):
        for invalid in [instance(width=8), instance(bitmap=[256]), instance(width=-1),
                        instance(terrainChunkPosition=[0, float('inf'), 0]),
                        instance(width=7, height=6, xOffset=0, yOffset=0, bitmap=[1])]:
            with self.assertRaises(ValueError):
                surface(invalid)
        chunks = [None] * 256
        chunks[0] = {'instances': [instance(chunkIndex=1)]}
        with self.assertRaises(ValueError):
            build_scene({'liquidChunks': chunks})

    def test_empty_document_fails_and_real_surface_has_standard_blend_material(self):
        with self.assertRaises(ValueError):
            build_scene({'liquidChunks': [None] * 256})
        chunks = [None] * 256
        chunks[0] = {'instances': [instance()]}
        original = copy.deepcopy(chunks)
        scene, records, empty = build_scene({'liquidChunks': chunks})
        self.assertEqual(chunks, original)
        self.assertEqual((len(records), empty, records[0]['quads']), (1, 0, 4))
        material = next(iter(scene.geometry.values())).visual.material
        self.assertEqual(material.alphaMode, 'BLEND')
        self.assertTrue(material.doubleSided)

    def test_existing_outputs_and_original_files_are_never_overwritten(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source, map_asset, output = [root / n for n in ['source.json', 'map.glb', 'out.glb']]
            source.write_bytes(b'original json')
            map_asset.write_bytes(b'original map')
            output.write_bytes(b'keep')
            for existing in (source, map_asset, output):
                with self.assertRaises(ValueError):
                    add_liquids(source, map_asset, existing)
            self.assertEqual(source.read_bytes(), b'original json')
            self.assertEqual(map_asset.read_bytes(), b'original map')
            self.assertEqual(output.read_bytes(), b'keep')
