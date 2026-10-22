import copy
import hashlib
import json
from pathlib import Path
import tempfile
import unittest
from static_shapes import validate
from package_world import package
from index_chunk_contributors import collision_bounds


def shapes():
    return {'meshes': [{'vertices': [[0,0,0],[1,0,0],[0,1,1]], 'indices': [[0,2,1]]}],
            'objects': [{'kind':'mesh','mesh':0,'affine':[1,.2,0,-1, 0,1,0,0, 0,0,-1,2, 0,0,0,1],'convex':False},
                        {'kind':'box','center':[3,1,3],'size':[1,2,2],'rotation':[0,.38268343,0,.9238795]},
                        {'kind':'capsule','center':[-3,1.5,4],'radius':.3,'half_segment':[.4,.8,0]}],
            'heightfields':[{'rows':3,'columns':5,'heights_column_major':list(range(15)), 'center':[0,0,0],'scale':[8,1,6]}]}


class StaticShapeTests(unittest.TestCase):
    def test_shared_affine_primitives_and_column_major_field_have_complete_bounds(self):
        data = shapes()
        before = copy.deepcopy(data)
        proof = validate(data)
        self.assertEqual((proof['pool_triangles'], proof['instantiated_triangles'], proof['height_samples'], proof['bodies']), (1,1,15,4))
        self.assertTrue(proof['world_geometry'])
        self.assertEqual(data, before)
        box = collision_bounds({'schema':4, 'static_shapes':data})
        self.assertLess(box[0][0], -4)
        self.assertGreater(box[1][0], 4)
        self.assertGreater(box[1][1], 4.3)

    def test_unknown_fields_types_references_and_unsupported_geometry_reject(self):
        for fault in ['unknown','dynamic','index','reference','unused','singular','projective','frame','rotation','radius','queries','boolean_query','height_count','height_scale','convex_volume']:
            data = shapes()
            if fault == 'unknown': data['objects'][1]['mass'] = 1
            if fault == 'dynamic': data['objects'][1]['kind'] = 'dynamic'
            if fault == 'index': data['meshes'][0]['indices'][0][0] = 99
            if fault == 'reference': data['objects'][0]['mesh'] = 99
            if fault == 'unused': data['objects'].pop(0)
            if fault == 'singular': data['objects'][0]['affine'][0] = 0
            if fault == 'projective': data['objects'][0]['affine'][12] = 1
            if fault == 'frame': data['objects'][0]['affine'][3] = 1024
            if fault == 'rotation': data['objects'][1]['rotation'] = [0,0,0,0]
            if fault == 'radius': data['objects'][2]['radius'] = -1
            if fault == 'queries': data['objects'][1]['queries'] = 8
            if fault == 'boolean_query': data['objects'][1]['queries'] = True
            if fault == 'height_count': data['heightfields'][0]['heights_column_major'].pop()
            if fault == 'height_scale': data['heightfields'][0]['scale'][0] = 0
            if fault == 'convex_volume': data['objects'][0]['convex'] = True
            with self.subTest(fault=fault), self.assertRaises(ValueError): validate(data)

    def test_instancing_and_samples_cannot_amplify_small_inputs_past_budgets(self):
        data = shapes()
        data['heightfields'] = []
        data['meshes'][0]['indices'] *= 65
        data['objects'] = [data['objects'][0]] * 4096
        with self.assertRaisesRegex(ValueError, 'Instantiated'): validate(data)
        data = {'heightfields':[{'rows':513,'columns':514,'heights_column_major':[0]*(513*514),'center':[0,0,0],'scale':[16,1,16]}]}
        with self.assertRaisesRegex(ValueError, 'sample budget'): validate(data)

    def test_schema4_package_publisher_binds_shapes_and_rejects_bad_rebound_input(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            visual = b'Original qualification binding payload; no GPU claim'
            (root/'terrain.glb').write_bytes(visual)
            profile = {'schema':4,'bounds':[[-8,-8],[8,8]],'source_origin':[0,0,0],
                       'visual_sha256':list(hashlib.sha256(visual).digest()),'triangles':[],
                       'spawns':[],'static_shapes':shapes()}
            (root/'terrain.json').write_text(json.dumps(profile))
            package(root, 'theia.test.shapes', 1)
            self.assertEqual(json.loads((root/'world.manifest.json').read_text())['collision_schema'], 4)
            (root/'world.manifest.json').unlink()
            profile['static_shapes']['objects'][0]['mesh'] = 99
            (root/'terrain.json').write_text(json.dumps(profile))
            with self.assertRaises(ValueError): package(root, 'theia.test.shapes', 2)
            self.assertFalse((root/'world.manifest.json').exists())
