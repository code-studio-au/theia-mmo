"""Known skeletal motion checks for the independent exported-clip oracle."""
from pathlib import Path
import sys
import unittest
import numpy as np
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from verify_synty_character import sample, world, deformed


class SkinnedOracleTests(unittest.TestCase):
    def test_quaternion_shortest_arc_and_step_at_exact_key_time(self):
        times = np.array([0., 1., 2.])
        self.assertEqual(sample(times, np.array([[2.], [4.], [8.]]), 1., step=True)[0], 4.)
        values = np.array([[0., 0., 0., 1.], [0., 0., -2**-.5, -2**-.5]])
        q = sample(times[:2], values, .5, rotation=True)
        np.testing.assert_allclose(q, [0., 0., np.sin(np.pi/8), np.cos(np.pi/8)], atol=1e-8)

    def test_two_level_hierarchy_and_inverse_bind_produce_known_skinned_position(self):
        doc = {'scene': 0, 'scenes': [{'nodes': [0]}],
               'nodes': [{'translation': [1., 0., 0.], 'children': [1]}, {'translation': [0., 1., 0.]}],
               'skins': [{'joints': [1]}]}
        bind = world(doc)
        curves = [(0, 'rotation', np.array([0., 1.]), np.array([[0., 0., 0., 1.], [0., 0., 2**-.5, 2**-.5]]), False)]
        current = world(doc, curves, 1.)
        data = (np.array([[2., 1., 0., 1.]]), np.array([[0]]), np.array([[1.]]), np.array([np.linalg.inv(bind[1])]))
        np.testing.assert_allclose(deformed(doc, data, current), [[0., 1., 0.]], atol=1e-8)
