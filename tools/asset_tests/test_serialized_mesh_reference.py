import copy
import unittest
import numpy as np
from unity_serialized_mesh_reference import decode


def fixture():
    vertices = np.array([[0, 0, 0], [1, 0, 0], [0, 1, 0]], dtype='<f4')
    indices = np.array([0, 1, 2], dtype='<u2')
    return {'m_VertexData': {'m_VertexCount': 3, 'm_DataSize': vertices.nbytes,
                            'm_Channels': [{'stream': 0, 'offset': 0, 'format': 0, 'dimension': 3}],
                            '_typelessdata': vertices.tobytes().hex()},
            'm_IndexFormat': 0, 'm_IndexBuffer': indices.tobytes().hex(),
            'm_StreamData': {'offset': 0, 'size': 0, 'path': ''},
            'm_MeshCompression': 0, 'm_BindPose': [],
            'm_SubMeshes': [{'firstByte': 0, 'indexCount': 3, 'baseVertex': 0, 'topology': 0}]}


class SerializedMeshTests(unittest.TestCase):
    def test_exact_little_endian_positions_and_base_vertex_indices(self):
        source = fixture()
        before = copy.deepcopy(source)
        vertices, indices = decode(source)
        np.testing.assert_array_equal(vertices, [[0, 0, 0], [1, 0, 0], [0, 1, 0]])
        np.testing.assert_array_equal(indices[0], [0, 1, 2])
        self.assertEqual(source, before)

    def test_rejects_unsupported_streams_formats_and_invalid_triangle_slices(self):
        for fault in ['count', 'bytes', 'declared_size', 'channel', 'extra_channel',
                      'external', 'compression', 'skin', 'nonfinite', 'index_format',
                      'index_range', 'offset', 'length', 'topology', 'base_vertex']:
            source = fixture()
            if fault == 'count': source['m_VertexData']['m_VertexCount'] = 2000001
            if fault == 'bytes': source['m_VertexData']['_typelessdata'] += '00'
            if fault == 'declared_size': source['m_VertexData']['m_DataSize'] += 4
            if fault == 'channel': source['m_VertexData']['m_Channels'][0]['stream'] = 1
            if fault == 'extra_channel': source['m_VertexData']['m_Channels'].append({'dimension': 3})
            if fault == 'external': source['m_StreamData']['offset'] = 1
            if fault == 'compression': source['m_MeshCompression'] = 1
            if fault == 'skin': source['m_BindPose'] = [1]
            if fault == 'nonfinite': source['m_VertexData']['_typelessdata'] = np.full((3, 3), np.nan, dtype='<f4').tobytes().hex()
            if fault == 'index_format': source['m_IndexFormat'] = 1
            if fault == 'index_range': source['m_IndexBuffer'] = np.array([0, 1, 3], dtype='<u2').tobytes().hex()
            if fault == 'offset': source['m_SubMeshes'][0]['firstByte'] = 1
            if fault == 'length': source['m_SubMeshes'][0]['indexCount'] = 4
            if fault == 'topology': source['m_SubMeshes'][0]['topology'] = 1
            if fault == 'base_vertex': source['m_SubMeshes'][0]['baseVertex'] = -1
            with self.subTest(fault=fault), self.assertRaises(ValueError):
                decode(source)
