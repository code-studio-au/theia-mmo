import copy
import math
from pathlib import Path
import sys
import unittest

import numpy as np

sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from open_map_blacksmith import rebase_profile
from open_map_farmhouse import subdivided_boxes


class BlacksmithTests(unittest.TestCase):
    def test_rebasing_retains_the_first_buildings_support_and_camera_geometry(self):
        original={'source_origin':[100,20,200], 'bounds':[[-50,-50],[50,50]],
            'triangles':[[[0,0,0],[0,1,5],[5,2,0]]],
            'walk_surfaces':[[[10,3,10],[10,3,12],[12,3,10]]],
            'solids':[{'center':[10,5,10],'size':[2,6,2]}],
            'camera_solids':[{'center':[10,8,10],'size':[8,1,8]}]}
        before=copy.deepcopy(original)
        result,delta=rebase_profile(original,[110,20,220])
        np.testing.assert_array_equal(delta,[10,0,20])
        np.testing.assert_allclose(np.asarray(result['walk_surfaces'])+delta, original['walk_surfaces'])
        np.testing.assert_allclose(np.asarray(result['triangles'])+delta, original['triangles'])
        np.testing.assert_allclose(np.asarray(result['solids'][0]['center'])+delta,[10,5,10])
        np.testing.assert_allclose(np.asarray(result['camera_solids'][0]['center'])+delta,[10,8,10])
        self.assertEqual(result['solids'][0]['size'],[2,6,2])
        self.assertEqual(result['bounds'],[[-60,-70],[40,30]])
        self.assertEqual(original,before)

    def test_rotated_east_door_keeps_a_clear_pawn_route_and_its_walls_remain_covered(self):
        angle=math.radians(130)
        matrix=np.eye(4);matrix[:3,:3]=[[math.cos(angle),0,-math.sin(angle)],[0,1,0],[math.sin(angle),0,math.cos(angle)]]
        matrix[:3,3]=[300,35,2230]
        boxes=[]
        for low,high in [([10.1,0,-6],[11.2,8.3,-2.2]),([10.1,0,2.1],[11.2,8.3,6])]:
            boxes.extend(subdivided_boxes(low,high,matrix))
        for x in np.linspace(9,13,80):
            p=matrix[:3,:3]@[x,2.7,0]+matrix[:3,3]
            self.assertTrue(all(abs(p[0]-b['center'][0])>=b['size'][0]/2+.35 or
                                abs(p[2]-b['center'][2])>=b['size'][2]/2+.35 for b in boxes))
        for z in np.linspace(-6,-2.2,80):
            p=matrix[:3,:3]@[10.6,2.7,z]+matrix[:3,3]
            self.assertTrue(any(all(abs(p[i]-b['center'][i])<=b['size'][i]/2+1e-8 for i in range(3)) for b in boxes))
