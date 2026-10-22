"""Only uniquely defined zero tangents may be repaired from source derivatives."""
import copy
from pathlib import Path
import sys
import unittest
import numpy as np
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from foliage_tangents import repair_tangents
from static_geometry import accessor


def fixture(kind='valid'):
    positions=np.array([[0,0,0],[1,0,0],[0,1,0]],dtype='<f4');normals=np.tile([0,0,1],(3,1)).astype('<f4');uv=np.array([[0,0],[1,0],[0,1]],dtype='<f4');faces=np.array([0,1,2],dtype='<u4');tangents=np.array([[0,0,0,1],[1,0,0,1],[1,0,0,1]],dtype='<f4')
    if kind=='mirrored':uv[:,0]*=-1
    if kind=='degenerate':uv[:]=0
    if kind=='ambiguous':faces=np.tile(faces,2)
    if kind=='normal':normals[0]=0
    if kind=='nonfinite':tangents[0,0]=float('nan')
    if kind=='handedness':tangents[0,3]=0
    binary=bytearray();views=[];accessors=[]
    for value,component,shape in [(positions,5126,'VEC3'),(normals,5126,'VEC3'),(uv,5126,'VEC2'),(faces,5125,'SCALAR'),(tangents,5126,'VEC4')]:
        views.append({'buffer':0,'byteOffset':len(binary),'byteLength':value.nbytes});binary.extend(value.tobytes());accessors.append({'bufferView':len(views)-1,'componentType':component,'type':shape,'count':len(value)})
    return {'buffers':[{'byteLength':len(binary)}],'bufferViews':views,'accessors':accessors,'meshes':[{'primitives':[{'attributes':{'POSITION':0,'NORMAL':1,'TEXCOORD_0':2,'TANGENT':4},'indices':3}]}]},bytes(binary)


class TangentRepairTests(unittest.TestCase):
    def test_sole_triangle_defines_unit_orthogonal_tangent_without_changing_source(self):
        doc,binary=fixture();before=copy.deepcopy(doc);result,packed,repairs=repair_tangents(doc,binary)
        self.assertEqual(doc,before);self.assertEqual(packed[:len(binary)],binary);self.assertEqual(len(repairs),1)
        primitive=result['meshes'][0]['primitives'][0];tangent=accessor(result,packed,primitive['attributes']['TANGENT'])
        np.testing.assert_array_equal(tangent,np.tile([1,0,0,1],(3,1)))
        for key in ['POSITION','NORMAL','TEXCOORD_0']:np.testing.assert_array_equal(accessor(result,packed,primitive['attributes'][key]),accessor(doc,binary,doc['meshes'][0]['primitives'][0]['attributes'][key]))
        again,unchanged,more=repair_tangents(result,packed);self.assertEqual(more,[]);self.assertEqual(unchanged,packed);self.assertEqual(again,result)

    def test_mirrored_uv_derivative_defines_negative_handedness(self):
        doc,binary=fixture('mirrored');result,packed,_=repair_tangents(doc,binary);p=result['meshes'][0]['primitives'][0]
        np.testing.assert_array_equal(accessor(result,packed,p['attributes']['TANGENT'])[0],[-1,0,0,-1])

    def test_ambiguous_degenerate_nonfinite_and_invalid_basis_reject(self):
        for kind in ['ambiguous','degenerate','normal','nonfinite','handedness']:
            doc,binary=fixture(kind)
            with self.subTest(kind=kind),self.assertRaises(ValueError):repair_tangents(doc,binary)
