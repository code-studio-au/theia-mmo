"""Direct source normals must survive a topology editor's normal re-encoding."""
from pathlib import Path
import sys
import unittest
import numpy as np
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from asset_tests.test_unity_reframe import fixture
from unity_corner_streams import write
from static_geometry import accessor


def source():
    return {'vertices_z_up_metres':[[0,0,0],[2,0,0],[0,-1,3]],'triangles':[[0,1,2]],'triangle_slots':[0],
            'triangle_uv_blender':[[[.1,.2],[.8,.3],[.2,.9]]],
            'triangle_uv1_blender':[[[.25,.5],[.5,.75],[.75,.25]]],
            'triangle_normals_z_up':[[[.3123,.2345,.7654]]*3]}


class CornerTests(unittest.TestCase):
    def test_original_corner_normals_uvs_geometry_and_immutable_material_prefix_survive(self):
        doc,binary=fixture();doc['materials'][0]['extras']={'theia_source_slot':0}
        doc['meshes'][0]['primitives'][0]['attributes'].pop('TANGENT')
        result,packed,report=write(doc,binary,source());p=result['meshes'][0]['primitives'][0]
        self.assertEqual(packed[:len(binary)],binary);self.assertEqual(result['materials'],doc['materials']);self.assertEqual(result['nodes'],doc['nodes'])
        np.testing.assert_array_equal(accessor(result,packed,p['attributes']['POSITION']),[[0,0,0],[2,0,0],[0,3,1]])
        direction=np.array([.3123,.7654,-.2345]);direction/=np.linalg.norm(direction)
        np.testing.assert_allclose(accessor(result,packed,p['attributes']['NORMAL']),[direction]*3,atol=1e-7)
        np.testing.assert_allclose(accessor(result,packed,p['attributes']['TEXCOORD_0']),[[.1,.8],[.8,.7],[.2,.1]],atol=1e-7)
        np.testing.assert_allclose(accessor(result,packed,p['attributes']['TEXCOORD_1']),[[.25,.5],[.5,.25],[.75,.75]],atol=1e-7)
        np.testing.assert_array_equal(accessor(result,packed,p['indices']).ravel(),[0,1,2]);self.assertEqual(report['vertices_by_slot'],{'0':3})

    def test_missing_slot_zero_normals_lost_secondary_uv_and_unknown_attributes_reject(self):
        for fault in ['slot','bool','normal','uv1','attribute']:
            doc,binary=fixture();doc['materials'][0]['extras']={'theia_source_slot':0};ref=source()
            doc['meshes'][0]['primitives'][0]['attributes'].pop('TANGENT')
            if fault=='slot':ref['triangle_slots']=[1]
            if fault=='bool':ref['triangle_slots']=[False]
            if fault=='normal':ref['triangle_normals_z_up']=[[[0,0,0]]*3]
            if fault=='uv1':ref.pop('triangle_uv1_blender');doc['meshes'][0]['primitives'][0]['attributes']['TEXCOORD_1']=2
            if fault=='attribute':doc['meshes'][0]['primitives'][0]['attributes']['COLOR_0']=1
            with self.subTest(fault=fault),self.assertRaises(ValueError):write(doc,binary,ref)
