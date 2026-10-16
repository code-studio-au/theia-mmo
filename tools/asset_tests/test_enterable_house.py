"""Check the authored measured doorway, ramp, wall layout and source guard."""
from pathlib import Path
import sys
import tempfile
import unittest

import numpy as np
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from prepare_enterable_house import layout, prepare, SCALE


class EnterableHouseTests(unittest.TestCase):
    def test_door_fits_pawn_ramp_is_walkable_and_wall_gap_stays_open(self):
        bounds=[[-9.562015,-.14303547,-9.259302],[9.714225,17.085135,9.171431]]
        data=layout(bounds,1.3,lambda x,z:1.2)
        self.assertGreater(data['door_clearance'],1.8+.15)
        ramp=np.asarray(data['ramp'])
        normal=np.cross(ramp[:,1]-ramp[:,0],ramp[:,2]-ramp[:,0])
        self.assertTrue(np.all(normal[:,1]>0))
        slope=np.linalg.norm(normal[:,[0,2]],axis=1)/normal[:,1]
        self.assertTrue(np.all(slope<=np.tan(np.radians(40))))
        x=data['entry_x'];z=data['landing'][1][2][2]
        for box in data['solids']:
            self.assertFalse(abs(x-box['center'][0])<box['size'][0]/2+.35 and
                             abs(z-box['center'][2])<box['size'][2]/2+.35)
        self.assertAlmostEqual(data['landing'][0][0][1]-data['entry_floor_y'],.061*SCALE)
        self.assertLess(data['landing'][0][0][1]-data['entry_floor_y'],data['step_height'])
        roof=data['camera_solids'][0]
        self.assertGreater(roof['center'][1]-roof['size'][1]/2-data['entry_floor_y'],1.8+.15)

    def test_unqualified_model_and_existing_output_are_rejected(self):
        with tempfile.TemporaryDirectory() as temp:
            root=Path(temp);terrain=root/'terrain';terrain.mkdir()
            (terrain/'terrain.json').write_text('{}');(terrain/'terrain.glb').write_bytes(b'not glb')
            house=root/'house.glb';house.write_bytes(b'unqualified house')
            with self.assertRaisesRegex(ValueError,'Unqualified farmhouse'):
                prepare(terrain,house,root/'out')
            self.assertFalse((root/'out').exists())
            (root/'out').mkdir()
            with self.assertRaisesRegex(ValueError,'must be new'):
                prepare(terrain,house,root/'out')
