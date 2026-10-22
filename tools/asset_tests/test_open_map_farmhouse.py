import math
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from open_map_farmhouse import prepare, subdivided_boxes


class MapFarmhouseTests(unittest.TestCase):
    def test_subdivision_preserves_clearance_through_the_rotated_doorway(self):
        angle = math.radians(25)
        matrix = np.eye(4)
        matrix[:3,:3] = [[math.cos(angle),0,-math.sin(angle)],[0,1,0],[math.sin(angle),0,math.cos(angle)]]
        matrix[:3,3] = [100,30,200]
        walls = []
        for low, high in [([-6.5,0,6],[-2.78,9.7,7.1]), ([-.11,0,6],[6.8,9.7,7.1])]:
            walls.extend(subdivided_boxes(low,high,matrix))
        self.assertGreater(len(walls), 2)
        for z in np.linspace(5,9,80):
            position = matrix[:3,:3] @ [-1.445,4.2,z] + matrix[:3,3]
            for box in walls:
                self.assertTrue(abs(position[0]-box['center'][0]) >= box['size'][0]/2+.35 or
                                abs(position[2]-box['center'][2]) >= box['size'][2]/2+.35)
        # Every point on either authored wall must still be covered; subdivision
        # must not trade the original wall for an open gap between boxes.
        for x in np.linspace(-6.5,-2.78,80):
            position = matrix[:3,:3] @ [x,4.2,6.5] + matrix[:3,3]
            self.assertTrue(any(all(abs(position[i]-box['center'][i]) <= box['size'][i]/2+1e-8 for i in range(3)) for box in walls))

    def test_source_hash_and_existing_output_are_rejected_before_writing(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            world = root/'world';world.mkdir()
            for name in ['terrain.json','terrain.glb','preparation.json']:
                (world/name).write_bytes(b'fixture')
            model = root/'house.obj';model.write_text('unqualified')
            csv = root/'placements.csv';csv.write_text('fixture')
            output = root/'out'
            with patch('open_map_farmhouse.placement_plan', return_value=([('adt_65749577','world',model,np.eye(4))],set(),1,0)):
                with self.assertRaisesRegex(ValueError,'source hash'):
                    prepare(world,csv,output)
            self.assertFalse(output.exists())
            output.mkdir()
            with self.assertRaisesRegex(ValueError,'must be new'):
                prepare(world,csv,output)
