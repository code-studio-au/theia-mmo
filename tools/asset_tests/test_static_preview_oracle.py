"""Independent static preview oracle must catch reversed faces and changed UVs."""
from pathlib import Path
import sys
import unittest
import numpy as np
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from verify_unity_static import signatures


class StaticPreviewOracleTests(unittest.TestCase):
    def test_cyclic_order_matches_but_winding_uv_and_duplicate_faces_do_not(self):
        tri=np.array([[0,0,0,0,0],[1,0,0,1,0],[0,1,0,0,1]],float)
        original=signatures([tri])
        self.assertEqual(original,signatures([np.roll(tri,1,axis=0)]))
        self.assertNotEqual(original,signatures([tri[::-1]]))
        changed=tri.copy();changed[1,3]=.5
        self.assertNotEqual(original,signatures([changed]))
        self.assertNotEqual(original,signatures([tri,tri]))
