import csv
import math
from pathlib import Path
import sys
import tempfile
import unittest

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from assemble_map import placement_plan
from map_placement import adt_transform, doodad_transform


def row(**changes):
    result = dict(Type='m2', ModelId='1', ModelFile='object.obj', PositionX='16600',
                  PositionY='10', PositionZ='14600', RotationX='0', RotationY='0',
                  RotationZ='0', RotationW='1', ScaleFactor='1', DoodadSetNames='Set_A')
    result.update(changes)
    return result


class PlacementTests(unittest.TestCase):
    def test_world_translation_and_yaw_preserve_up_and_source_units(self):
        transform = adt_transform(row())
        np.testing.assert_allclose(transform[:3, 3], [466 + 2/3, 10, 2466 + 2/3])
        np.testing.assert_allclose(transform[:3, :3] @ [1, 0, 0], [0, 0, -1], atol=1e-12)
        np.testing.assert_allclose(transform[:3, :3] @ [0, 1, 0], [0, 1, 0], atol=1e-12)
        rotation = adt_transform(row(Type='wmo', RotationY='90', ScaleFactor='2'))[:3, :3]
        np.testing.assert_allclose(rotation @ [1, 0, 0], [-2, 0, 0], atol=1e-12)
        self.assertAlmostEqual(np.linalg.det(rotation), 8)

    def test_nonzero_pitch_and_roll_map_to_the_expected_axes(self):
        pitch = adt_transform(row(RotationX='90'))[:3, :3]
        roll = adt_transform(row(RotationZ='90'))[:3, :3]
        np.testing.assert_allclose(pitch @ [0, 1, 0], [0, 0, -1], atol=1e-12)
        np.testing.assert_allclose(roll @ [0, 1, 0], [1, 0, 0], atol=1e-12)

    def test_nested_wmo_quaternion_uses_wxyz_and_building_local_coordinates(self):
        nested = row(PositionX='1', PositionY='2', PositionZ='3', RotationW=str(math.sqrt(.5)), RotationZ=str(math.sqrt(.5)))
        relative = doodad_transform(nested)
        np.testing.assert_allclose(relative[:3, 3], [1, 3, -2], atol=1e-12)
        np.testing.assert_allclose(relative[:3, :3] @ [1, 0, 0], [0, 0, -1], atol=1e-12)
        world = adt_transform(row(Type='wmo')) @ relative
        np.testing.assert_allclose(world[:3, 3], [464 + 2/3, 13, 2465 + 2/3])

    def test_invalid_coordinates_rotations_scales_and_types_fail(self):
        for invalid in [row(PositionX='nan'), row(RotationY='inf'), row(ScaleFactor='-1'), row(Type='gobj')]:
            with self.assertRaises(ValueError):
                adt_transform(invalid)
        with self.assertRaises(ValueError):
            doodad_transform(row(RotationW='0'))

    def test_only_the_selected_and_global_wmo_doodad_sets_are_instanced(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            body = row(Type='wmo')
            path = root / 'tile.csv'
            with path.open('w', newline='') as file:
                writer = csv.DictWriter(file, body.keys(), delimiter=';')
                writer.writeheader()
                writer.writerow(body)
            sidecar = root / 'object_ModelPlacementInformation.csv'
            with sidecar.open('w', newline='') as file:
                sample = dict(row(), DoodadSet='Set_A')
                writer = csv.DictWriter(file, sample.keys(), delimiter=';')
                writer.writeheader()
                for selected in ['Set_A', 'Set_$DefaultGlobal', 'Set_B']:
                    writer.writerow(dict(sample, DoodadSet=selected))
            plan, _, count, filtered = placement_plan(path)
            self.assertEqual((len(plan), count, filtered), (3, 1, 1))
            self.assertEqual(plan[1][1], plan[0][0])
