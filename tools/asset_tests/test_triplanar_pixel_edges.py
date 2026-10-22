"""An interior barycentric point can still cross MSAA coverage in screen space."""
from pathlib import Path
import sys
import unittest
import numpy as np
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from verify_triplanar_pixels import pixel_edge_clearance,footprint_surface_consistent


class PixelEdgeTests(unittest.TestCase):
    def test_clearance_is_in_pixels_and_scales_with_resolution(self):
        triangle=np.array([[-1,-1,0],[1,-1,0],[-1,1,0]])
        self.assertAlmostEqual(pixel_edge_clearance(triangle,np.eye(4),[25,75],100,100),25)
        self.assertAlmostEqual(pixel_edge_clearance(triangle,np.eye(4),[.25,75],100,100),.25)
        self.assertAlmostEqual(pixel_edge_clearance(triangle,np.eye(4),[.5,150],200,200),.5)

    def test_behind_camera_and_degenerate_triangles_have_no_safe_interior(self):
        triangle=np.array([[-1,-1,0],[1,-1,0],[-1,1,0]])
        clip=np.eye(4);clip[3,3]=-1
        self.assertEqual(pixel_edge_clearance(triangle,clip,[25,75],100,100),0)
        triangle[1]=triangle[0]
        self.assertEqual(pixel_edge_clearance(triangle,np.eye(4),[25,75],100,100),0)

    def test_intersecting_surfaces_are_unsafe_even_away_from_triangle_edges(self):
        first=np.array([[-3,-3,.5],[3,-3,.5],[0,3,.5]])
        triangles=first[None,:,:];e1=triangles[:,1]-triangles[:,0];e2=triangles[:,2]-triangles[:,0]
        self.assertTrue(footprint_surface_consistent(triangles,e1,e2,np.eye(4),49.5,49.5,100,100,0))
        self.assertGreater(pixel_edge_clearance(first,np.eye(4),[50,50],100,100),10)
        other=first.copy();other[:,2]+=.1*other[:,0]
        triangles=np.array([first,other]);e1=triangles[:,1]-triangles[:,0];e2=triangles[:,2]-triangles[:,0]
        self.assertFalse(footprint_surface_consistent(triangles,e1,e2,np.eye(4),49.5,49.5,100,100,0))
