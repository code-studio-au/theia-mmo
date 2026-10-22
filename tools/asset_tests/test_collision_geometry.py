"""Independent qualifier geometry, including capsule caps/edges and floors."""
import math
from pathlib import Path
import sys
import unittest
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from collision_geometry import mesh_capsule_clearance, segment_triangle_sq

class GeometryTests(unittest.TestCase):
    def test_flat_sloped_stacked_and_penetrating_capsules(self):
        floor=[[[-10,0,-10],[10,0,10],[10,0,-10]],[[-10,0,-10],[-10,0,10],[10,0,10]]]
        self.assertAlmostEqual(mesh_capsule_clearance(floor,{'xz':[0,0],'y':.908}),.008)
        self.assertLess(mesh_capsule_clearance(floor,{'xz':[0,0],'y':.89}),0)
        upper=[[[x,y+3,z] for x,y,z in triangle] for triangle in floor]
        self.assertAlmostEqual(mesh_capsule_clearance(floor+upper,{'xz':[0,0],'y':.908}),.008)
        slope=[[[x,x*.3,z] for x,y,z in triangle] for triangle in floor]
        root=.9+(.35+.008)*math.sqrt(1.09)-.35
        self.assertAlmostEqual(mesh_capsule_clearance(slope,{'xz':[0,0],'y':root}),.008)

    def test_segment_triangle_face_edge_corner_and_crossing(self):
        a,b,c=(0,0,0),(2,0,0),(0,0,2)
        self.assertAlmostEqual(segment_triangle_sq((.5,1,.5),(.5,2,.5),a,b,c),1)
        self.assertAlmostEqual(segment_triangle_sq((1,1,-1),(1,2,-1),a,b,c),2)
        self.assertAlmostEqual(segment_triangle_sq((-1,1,-1),(-1,2,-1),a,b,c),3)
        self.assertEqual(segment_triangle_sq((.5,-1,.5),(.5,1,.5),a,b,c),0)
