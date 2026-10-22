"""Affine source instances must retain shear, winding and shared mesh assets."""
import json
from pathlib import Path
import struct
import sys
import tempfile
import unittest
import numpy as np
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from unity_geometry import assemble,converted_matrix,factor_pose,REFLECTION,normalized_quaternion
from static_geometry import accessor,hierarchy,local_matrix
from merge_static_glb import read
from verify_unity_geometry import assembly,point_error


def prototype(path):
    corners=np.array([[0,0,0],[2,0,0],[0,1,0],[0,0,3]],dtype=np.float64)
    faces=[[0,2,1],[0,1,3],[0,3,2],[1,2,3]]
    positions=corners[np.array(faces)].reshape(-1,3);normals=[]
    for face in corners[np.array(faces)]:
        normal=np.cross(face[1]-face[0],face[2]-face[0]);normal/=np.linalg.norm(normal);normals.extend([normal]*3)
    binary=positions.astype('<f4').tobytes()+np.array(normals,dtype='<f4').tobytes()+np.arange(12,dtype='<u4').tobytes()
    document={'asset':{'version':'2.0'},'scene':0,'scenes':[{'nodes':[0]}],'nodes':[{'mesh':0}],
        'buffers':[{'byteLength':len(binary)}],
        'bufferViews':[{'buffer':0,'byteOffset':0,'byteLength':144},{'buffer':0,'byteOffset':144,'byteLength':144},{'buffer':0,'byteOffset':288,'byteLength':48}],
        'accessors':[{'bufferView':0,'componentType':5126,'count':12,'type':'VEC3','min':[0,0,0],'max':[2,1,3]},
                     {'bufferView':1,'componentType':5126,'count':12,'type':'VEC3'},
                     {'bufferView':2,'componentType':5125,'count':12,'type':'SCALAR'}],
        'meshes':[{'primitives':[{'attributes':{'POSITION':0,'NORMAL':1},'indices':2,'material':0}]}],
        'materials':[{'pbrMetallicRoughness':{'baseColorFactor':[.8,.3,.1,1],'metallicFactor':0,'roughnessFactor':.8}}]}
    text=json.dumps(document).encode();text+=b' '*(-len(text)%4)
    path.write_bytes(struct.pack('<4sII',b'glTF',2,28+len(text)+len(binary))+struct.pack('<I4s',len(text),b'JSON')+text+struct.pack('<I4s',len(binary),b'BIN\0')+binary)


class GeometryTests(unittest.TestCase):
    def test_quaternion_rounding_is_unit_and_bounded_without_accepting_invalid_pose(self):
        self.assertEqual(normalized_quaternion([0,0,0,1.0000000000000002]),[0,0,0,1])
        self.assertEqual(normalized_quaternion([0,-1.0000000000000002,0,0]),[0,-1,0,0])
        for invalid in [[0,0,0,0],[0,0,0,1.001],[0,0,float('nan'),1],[1,0,0]]:
            with self.assertRaises(ValueError):normalized_quaternion(invalid)

    def test_factor_keeps_nonuniform_rotated_parent_shear_as_positive_trs_nodes(self):
        matrix=np.array([[1,.4,0,4],[0,2,.3,2],[0,0,1,-3],[0,0,0,1.]])
        first,second,mirrored=factor_pose(matrix)
        self.assertFalse(mirrored);self.assertNotIn('matrix',first);self.assertNotIn('matrix',second)
        np.testing.assert_allclose(local_matrix(first)@local_matrix(second),REFLECTION@matrix@REFLECTION,atol=1e-12)
        self.assertTrue(all(v>0 for v in first['scale']))
        self.assertEqual(converted_matrix(matrix)[0,3],-4)

    def test_shared_parity_mesh_preserves_points_normals_and_front_face_winding(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);source=root/'source.glb';prototype(source)
            matrices=[[[1,.4,0,4],[0,2,.3,0],[0,0,1,-2],[0,0,0,1]],
                      [[-1,0,0,-3],[0,1.5,0,1],[0,0,2,5],[0,0,0,1]]]
            result=assemble(source,[{'id':i,'matrix':m} for i,m in enumerate(matrices)],root/'result.glb')
            self.assertEqual((result['instances'],result['mesh_variants'],result['materials']),(2,2,1))
            json.dumps(result)
            original,original_binary=read(source);document,binary=read(root/'result.glb');_,world=hierarchy(document)
            base=accessor(original,original_binary,0);base_normals=accessor(original,original_binary,1)
            for i,matrix in enumerate(matrices):
                node=2*i+1;frame=world[node];self.assertGreater(np.linalg.det(frame[:3,:3]),0)
                primitive=document['meshes'][document['nodes'][node]['mesh']]['primitives'][0]
                points=accessor(document,binary,primitive['attributes']['POSITION']);points=points@frame[:3,:3].T+frame[:3,3]
                expected=converted_matrix(matrix);wanted=base@expected[:3,:3].T+expected[:3,3]
                np.testing.assert_allclose(points,wanted,atol=1e-12)
                normals=accessor(document,binary,primitive['attributes']['NORMAL'])@np.linalg.inv(frame[:3,:3])
                normals/=np.linalg.norm(normals,axis=1)[:,None]
                target=base_normals@np.linalg.inv(expected[:3,:3]);target/=np.linalg.norm(target,axis=1)[:,None]
                np.testing.assert_allclose(normals,target,atol=1e-7)
                indices=accessor(document,binary,primitive['indices']).ravel().reshape(-1,3)
                for ids in indices:
                    tri=points[ids];cross=np.cross(tri[1]-tri[0],tri[2]-tri[0]);cross/=np.linalg.norm(cross)
                    self.assertGreater(np.dot(cross,normals[ids[0]]),.99999)
            self.assertEqual(binary[:len(original_binary)],original_binary)

    def test_same_parity_instances_reuse_one_mesh(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);source=root/'source.glb';prototype(source)
            report=assemble(source,[{'id':i,'matrix':np.eye(4).tolist()} for i in range(3)],root/'result.glb')
            self.assertEqual(report['mesh_variants'],1)
            document,_=read(root/'result.glb');self.assertEqual([n['mesh'] for n in document['nodes'] if 'mesh' in n],[0,0,0])

    def test_invalid_and_collapsed_frames_or_duplicate_ids_reject(self):
        for matrix in [np.zeros((4,4)),np.eye(3),np.diag([1e-12,1,1,1]),np.full((4,4),float('nan'))]:
            with self.assertRaises(ValueError):factor_pose(matrix)
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);source=root/'source.glb';prototype(source);instance={'id':1,'matrix':np.eye(4).tolist()}
            with self.assertRaises(ValueError):assemble(source,[instance,instance],root/'result.glb')
            self.assertFalse((root/'result.glb').exists())

    def test_independent_oracle_rejects_pose_change_and_omitted_instance(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);source=root/'source.glb';prototype(source)
            instances=[{'id':i,'matrix':np.eye(4).tolist()} for i in range(2)]
            assemble(source,instances,root/'result.glb')
            self.assertTrue(assembly(source,instances,root/'result.glb')['passed'])
            with self.assertRaisesRegex(ValueError,'Missing/extra'):assembly(source,instances[:1],root/'result.glb')
            instances[0]['matrix'][0][3]=.01
            with self.assertRaisesRegex(ValueError,'positions differ'):assembly(source,instances,root/'result.glb')

    def test_independent_point_matching_is_bidirectional_and_tolerant(self):
        points=np.array([[0,0,0],[2,1,3]],dtype=float)
        self.assertLess(point_error(points,points+.000001),.0001)
        with self.assertRaises(ValueError):point_error(points,points[:1])
        with self.assertRaises(ValueError):point_error(points[:1],points)
