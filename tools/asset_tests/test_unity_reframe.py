"""A root correction must not change shading streams or topology."""
import copy
from pathlib import Path
import sys
import unittest
import numpy as np
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from reframe_unity_static import frame, transform_document
from verify_unity_reframe import compare


def fixture():
    arrays = [np.array([[0, 0, 0], [2, 0, 0], [0, 3, 1]], '<f4'),
              np.array([[0, 1, 0]] * 3, '<f4'), np.array([[0, 0], [1, 0], [0, 1]], '<f4'),
              np.array([[1, 0, 0, -1]] * 3, '<f4'), np.array([[0], [1], [2]], '<u4')]
    doc = {'asset': {'version': '2.0'}, 'scene': 0, 'scenes': [{'nodes': [0]}], 'nodes': [{'mesh': 0}],
           'meshes': [{'primitives': [{'attributes': {'POSITION': 0, 'NORMAL': 1, 'TEXCOORD_0': 2, 'TANGENT': 3}, 'indices': 4, 'material': 0}]}],
           'materials': [{'name': 'source material'}], 'bufferViews': [], 'accessors': [], 'buffers': [{'byteLength': 0}]}
    binary = b''
    for array, kind in zip(arrays, ['VEC3', 'VEC3', 'VEC2', 'VEC4', 'SCALAR']):
        doc['bufferViews'].append({'buffer': 0, 'byteOffset': len(binary), 'byteLength': array.nbytes})
        doc['accessors'].append({'bufferView': len(doc['bufferViews']) - 1, 'componentType': 5125 if kind == 'SCALAR' else 5126, 'type': kind, 'count': len(array)})
        binary += array.tobytes()
    doc['buffers'][0]['byteLength'] = len(binary)
    return doc, binary


class UnityReframeTests(unittest.TestCase):
    def test_translation_and_uniform_scale_preserve_all_other_bytes(self):
        doc, binary = fixture()
        transform = np.diag([1.25, 1.25, 1.25, 1.])
        transform[:3, 3] = [41.85, -.006, -6.21]
        transformed, packed = transform_document(doc, binary, transform)
        self.assertLess(compare(doc, binary, transformed, packed, 1.25, transform[:3, 3]), .00001)
        self.assertEqual(binary, packed[:len(binary)])
        self.assertEqual(doc['nodes'], transformed['nodes'])
        self.assertEqual(doc['meshes'][0]['primitives'][0]['attributes']['POSITION'], 0)

    def test_material_winding_uv_normal_tangent_and_pose_faults_reject(self):
        doc, binary = fixture()
        transform = np.eye(4); transform[:3, 3] = [1, 2, 3]
        result, packed = transform_document(doc, binary, transform)
        for fault in ['material', 'normal', 'uv', 'tangent', 'indices', 'node', 'vertex', 'receipt']:
            bad = copy.deepcopy(result); data = bytearray(packed)
            primitive = bad['meshes'][0]['primitives'][0]
            if fault == 'material': bad['materials'][0]['name'] = 'different'
            if fault in ('normal', 'uv', 'tangent'):
                primitive['attributes'][{'normal': 'NORMAL', 'uv': 'TEXCOORD_0', 'tangent': 'TANGENT'}[fault]] = 0
            if fault == 'indices': primitive['indices'] = 0
            if fault == 'node': bad['nodes'][0]['translation'] = [2, 0, 0]
            if fault == 'vertex': data[0] ^= 1
            translation = [0, 0, 0] if fault == 'receipt' else [1, 2, 3]
            with self.subTest(fault=fault), self.assertRaises(ValueError): compare(doc, binary, bad, bytes(data), 1, translation)

    def test_frame_rejects_nonuniform_rotated_mirrored_and_unbounded_changes(self):
        for mode in ['nonuniform', 'rotated', 'mirrored', 'nan', 'translation']:
            value = np.eye(4)
            if mode == 'nonuniform': value[1, 1] = 2
            if mode == 'rotated': value[0, 1] = .01
            if mode == 'mirrored': value[0, 0] = -1
            if mode == 'nan': value[0, 0] = float('nan')
            if mode == 'translation': value[0, 3] = 5000
            with self.subTest(mode=mode), self.assertRaises(ValueError): frame({'canonical_mesh_to_unity_local': (np.diag([-1, 1, 1, 1]) @ value).tolist()})

    def test_prototype_with_transform_or_out_of_bounds_result_rejects(self):
        doc, binary = fixture()
        bad = copy.deepcopy(doc); bad['nodes'][0]['translation'] = [1, 0, 0]
        with self.assertRaises(ValueError): transform_document(bad, binary, np.eye(4))
        transform = np.eye(4); transform[0, 3] = 5000
        with self.assertRaises(ValueError): transform_document(doc, binary, transform)
