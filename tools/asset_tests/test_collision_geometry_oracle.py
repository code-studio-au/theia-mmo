import unittest
import numpy as np
from qualify_collision_geometry import convex_surface, intersect


class CollisionGeometryOracleTests(unittest.TestCase):
    def test_closed_convex_volume_proof_rejects_open_concave_and_orphan_geometry(self):
        points = np.array([[0, 0, 0], [1, 0, 0], [0, 1, 0], [0, 0, 1]], dtype=float)
        faces = np.array([[0, 2, 1], [0, 1, 3], [0, 3, 2], [1, 2, 3]])
        self.assertTrue(convex_surface(points, faces))
        dented = np.vstack([points, [.1, .1, .1]])
        concave_faces = np.vstack([faces[:3], [[1, 2, 4], [2, 3, 4], [3, 1, 4]]])
        for vertices, triangles in [(points, faces[:-1]), (dented, faces), (dented, concave_faces)]:
            with self.subTest(triangles=triangles.tolist()), self.assertRaises(ValueError):
                convex_surface(vertices, triangles)

    def test_triangle_oracle_uses_both_sides_with_normals_opposing_the_ray(self):
        triangles = np.array([[[0, 0, 0], [1, 0, 0], [0, 0, 1]]], dtype=float)
        for sign in [-1, 1]:
            probe = {'origin': [.2, sign, .2], 'direction': [0, -sign, 0], 'maximum': 2}
            actual = intersect(triangles, probe)
            self.assertTrue(actual['hit'])
            self.assertAlmostEqual(actual['distance'], 1)
            np.testing.assert_allclose(actual['point'], [.2, 0, .2])
            np.testing.assert_allclose(actual['normal'], [0, sign, 0])
            self.assertEqual(probe, {'origin': [.2, sign, .2], 'direction': [0, -sign, 0], 'maximum': 2})
        for probe in [dict(origin=[2, 1, 2], direction=[0, -1, 0], maximum=2),
                      dict(origin=[.2, 1, .2], direction=[0, -1, 0], maximum=.99),
                      dict(origin=[.2, 1, .2], direction=[1, 0, 0], maximum=2)]:
            self.assertFalse(intersect(triangles, probe)['hit'])
