import copy
import math
import unittest
from schema4_clearance import profile_capsule_clearance
from collision_geometry import segment_triangle_sq


class ShapeClearanceTests(unittest.TestCase):
    def test_heightfield_clearance_and_penetration(self):
        profile = {'static_shapes':{'heightfields':[{'rows':3,'columns':5,
                   'heights_column_major':[0]*15,'center':[0,0,0],'scale':[8,1,8]}]}}
        for y,want in [(.9,0),(1.1,.2),(.8,-.1)]:
            self.assertAlmostEqual(profile_capsule_clearance(profile,{'xz':[.1,.2],'y':y}),want)
        camera = copy.deepcopy(profile)
        camera['static_shapes']['heightfields'][0]['queries'] = 4
        self.assertTrue(math.isinf(profile_capsule_clearance(camera,{'xz':[0,0],'y':.9})))

    def test_inside_oriented_box_and_analytic_capsule(self):
        box = {'kind':'box','center':[0,.9,0],'size':[2,2,2],
               'rotation':[0,.382683432365,0,.923879532511]}
        profile = {'static_shapes':{'objects':[box]}}
        self.assertLess(profile_capsule_clearance(profile,{'xz':[0,0],'y':.9}),0)
        capsule = {'kind':'capsule','center':[0,.9,0],'radius':.4,'half_segment':[.4,.8,0]}
        profile['static_shapes']['objects'] = [capsule]
        self.assertAlmostEqual(profile_capsule_clearance(profile,{'xz':[0,0],'y':.9}),-.75)
        self.assertGreater(profile_capsule_clearance(profile,{'xz':[2,0],'y':.9}),.5)

    def test_zero_area_source_face_uses_edges_without_division_by_zero(self):
        self.assertAlmostEqual(segment_triangle_sq([0,1,0],[0,2,0],[0,0,0],[0,0,0],[1,0,0]),1)
        self.assertAlmostEqual(segment_triangle_sq([0,1,0],[0,2,0],[0,0,0],[1,0,0],[2,0,0]),1)

    def test_affine_convex_instance_detects_actor_inside_closed_volume(self):
        vertices = [[0,0,0],[1,0,0],[0,1,0],[0,0,1]]
        faces = [[0,2,1],[0,1,3],[0,3,2],[1,2,3]]
        profile = {'static_shapes':{'meshes':[{'vertices':vertices,'indices':faces}],
                   'objects':[{'kind':'mesh','mesh':0,'convex':True,
                   'affine':[-5,.2,0,2, 0,5,0,0, 0,0,5,0, 0,0,0,1]}]}}
        self.assertLess(profile_capsule_clearance(profile,{'xz':[1,.5],'y':.9}),0)
        with self.assertRaisesRegex(ValueError,'closed volume'):
            profile['static_shapes']['meshes'][0]['indices'].pop()
            profile_capsule_clearance(profile,{'xz':[1,.5],'y':.9})
