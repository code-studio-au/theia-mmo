import hashlib
import io
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
import numpy as np
from PIL import Image
from convert_unity_terrain import write_glb
import prepare_heightfield_reference as reference


def source(root, fault=None):
    stage = root / 'source'
    stage.mkdir()
    z, x = np.mgrid[0:3, 0:5]
    points = np.stack([x, .2 * x + z * z + .11 * x * z, z], -1).reshape(-1, 3).astype('<f4')
    faces = []
    for row in range(2):
        for col in range(4):
            a = row * 5 + col
            faces.extend([[a, a + 5, a + 1], [a + 5, a + 6, a + 1]])
    if fault == 'diagonal': faces[:2] = [[0, 5, 6], [0, 6, 1]]
    if fault == 'hole': faces.pop()
    if fault == 'winding': faces[0] = list(reversed(faces[0]))
    if fault == 'duplicate_xz': points[1, [0, 2]] = points[0, [0, 2]]
    if fault == 'irregular': points[points[:, 0] == 1, 0] = 1.125
    image = io.BytesIO()
    Image.new('RGB', (2, 2), (80, 140, 50)).save(image, format='PNG')
    write_glb(stage / 'terrain.glb', points, np.tile([0, 1, 0], (15, 1)).astype('<f4'), np.zeros((15, 2), dtype='<f4'), np.array(faces, dtype='<u4'), image.getvalue())
    (stage / 'conversion.json').write_text(json.dumps({'outputs': {'terrain.glb': hashlib.sha256((stage / 'terrain.glb').read_bytes()).hexdigest()}}))
    return stage, points, faces


class HeightfieldReferenceTests(unittest.TestCase):
    def test_all_rectangular_samples_and_oriented_triangle_centroids_are_retained(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            stage, points, faces = source(root)
            before = (stage / 'terrain.glb').read_bytes()
            output = root / 'build/field.json'
            with patch.object(reference, 'BUILD', root / 'build'):
                result = reference.prepare(stage, output)
                with self.assertRaises(ValueError): reference.prepare(stage, output)
            actual = json.loads(output.read_text())
            self.assertEqual((actual['rows'], actual['columns']), (3, 5))
            np.testing.assert_array_equal(np.array(actual['heights_column_major']).reshape(5, 3).T, points[:, 1].reshape(3, 5))
            np.testing.assert_array_equal(actual['vertices'], points)
            np.testing.assert_allclose(actual['triangle_centers'], points[np.array(faces)].astype(float).mean(1), atol=0, rtol=0)
            self.assertEqual((result['vertices'], result['triangles']), (15, 16))
            self.assertEqual((stage / 'terrain.glb').read_bytes(), before)

    def test_rebound_holes_wrong_diagonals_winding_and_irregular_grids_reject(self):
        for fault in ['diagonal', 'hole', 'winding', 'duplicate_xz', 'irregular']:
            with self.subTest(fault=fault), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                stage, _, _ = source(root, fault)
                output = root / 'build/field.json'
                with patch.object(reference, 'BUILD', root / 'build'), self.assertRaises(ValueError):
                    reference.prepare(stage, output)
                self.assertFalse(output.exists())
