"""No Unity execution is simulated as qualification: exercise reference guards."""
import copy
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

import numpy as np
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import unity_editor_reference as reference
from asset_tests.test_unity_model_ids import fbx
from unity_model_ids import signed_hash


def mesh():
    return {'file_id': str(signed_hash('Type:Mesh->Cube0')), 'name': 'Cube', 'is_readable': False,
            'positions': [0, 0, 0, 2, 0, 0, 0, 3, 1],
            'bounds_center': dict(x=1, y=1.5, z=.5), 'bounds_size': dict(x=2, y=3, z=1),
            'submeshes': [{'topology': 0, 'indices': [0, 1, 2]}]}


def model():
    return {'meshes': [mesh()], 'nodes': [{'file_id': str(signed_hash('Type:Transform->//RootNode/root/Transform0')), 'parent_id': '', 'name': 'Cube',
            'mesh_file_id': mesh()['file_id'], 'local_matrix': np.eye(4).ravel().tolist(),
            'world_matrix': np.eye(4).ravel().tolist()}]}


class EditorReferenceTests(unittest.TestCase):
    def test_three_pivot_policies_are_distinguished_with_rotated_scaled_root(self):
        points = np.array([[-4, 1, 2], [-2, 1, 2], [-4, 3, 3]], dtype=float)
        frame = np.array([[0, -2, 0, 8], [2, 0, 0, -6], [0, 0, 2, 4], [0, 0, 0, 1]], dtype=float)
        node = {'vertices': points.tolist(), 'faces': [[0, 1, 2]], 'geometry_to_node': np.eye(4).tolist(),
                'node_to_world': frame.tolist(), 'geometry_to_world': frame.tolist()}
        candidates = [points, points @ frame[:3, :3].T + frame[:3, 3],
                      points + np.linalg.solve(frame[:3, :3], frame[:3, 3])]
        names = ['mesh_local', 'geometry_to_world', 'root_translation_in_local_frame']
        for name, actual in zip(names, candidates):
            result = reference.policies(node, actual * [-1, 1, 1])
            self.assertEqual([k for k, value in result.items() if value['matches']], [name])
        # Distinct rotations, scales and translations in the imported Unity
        # hierarchy must be removed using that actual frame, not just its T.
        unity = np.array([[0, 0, 3, 7], [0, 3, 0, -2], [-3, 0, 0, 1], [0, 0, 0, 1]], dtype=float)
        converted = (points @ frame[:3, :3].T + frame[:3, 3]) * [-1, 1, 1]
        inverse = np.linalg.inv(unity)
        converted = converted @ inverse[:3, :3].T + inverse[:3, 3]
        result = reference.policies(node, converted, unity)
        self.assertEqual([k for k, value in result.items() if value['matches']], ['recorded_unity_frame'])

    def test_points_reject_corrupt_indices_bounds_nonfinite_and_float_ids(self):
        reference.mesh_points(mesh())
        for fault in ['index', 'fraction', 'topology', 'bounds', 'nan', 'id', 'empty']:
            value = mesh()
            if fault == 'index': value['submeshes'][0]['indices'][0] = 3
            if fault == 'fraction': value['submeshes'][0]['indices'][0] = 0.0
            if fault == 'topology': value['submeshes'][0]['topology'] = 3
            if fault == 'bounds': value['bounds_center']['x'] = 30
            if fault == 'nan': value['positions'][0] = float('nan')
            if fault == 'id': value['file_id'] = float(value['file_id'])
            if fault == 'empty': value['submeshes'][0]['indices'] = []
            with self.subTest(fault=fault), self.assertRaises(ValueError): reference.mesh_points(value)

    def test_hierarchy_requires_consistent_complete_affine_tree_and_mesh_identity(self):
        self.assertEqual(len(reference.validate_model(model())), 1)
        for fault in ['cycle', 'missing', 'matrix', 'affine', 'mesh', 'duplicate']:
            value = model()
            if fault == 'cycle': value['nodes'][0]['parent_id'] = '-9'
            if fault == 'missing': value['nodes'][0]['parent_id'] = '-10'
            if fault == 'matrix': value['nodes'][0]['world_matrix'][3] = 20
            if fault == 'affine': value['nodes'][0]['local_matrix'][15] = 2
            if fault == 'mesh': value['nodes'][0]['mesh_file_id'] = '99'
            if fault == 'duplicate': value['meshes'].append(copy.deepcopy(value['meshes'][0]))
            with self.subTest(fault=fault), self.assertRaises(ValueError): reference.validate_model(value)

    def test_staging_and_source_bound_inspection_reject_rebound_faults(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / 'source'
            path = source / 'Assets/Cube.fbx'
            path.parent.mkdir(parents=True)
            fbx(path, single=True)
            meta = Path(str(path) + '.meta')
            meta.write_text('guid: ' + 'a' * 32 + '\nModelImporter:\n  meshes:\n    fileIdsGeneration: 2\n')
            manifest = {'archive_sha256': 'b' * 64, 'selected': {}, 'metadata': {}}
            for key, item in [('selected', path), ('metadata', meta)]:
                manifest[key]['Assets/Cube.fbx'] = {'guid': 'a' * 32, 'sha256': reference.digest(item), 'bytes': item.stat().st_size}
            (source / 'extraction.json').write_text(json.dumps(manifest))
            # A source script must never enter the scratch project.
            (source / 'Assets/Unreviewed.cs').write_text('not executed')
            output = root / 'build/reference'
            with patch.object(reference, 'BUILD', root / 'build'):
                staged = reference.stage(source, output, ['Assets/Cube.fbx'], '6000.3.2f1')
                self.assertFalse(staged['editor_executed'])
                with self.assertRaises(FileExistsError): reference.stage(source, output, ['Assets/Cube.fbx'], '6000.3.2f1')
                with self.assertRaises(ValueError): reference.stage(source, root / 'outside', ['Assets/Cube.fbx'], '6000.3.2f1')
            self.assertEqual(list(p.name for p in (output / 'Assets/Editor').iterdir()), ['TheiaMeshReference.cs'])
            self.assertFalse((output / 'Assets/Unreviewed.cs').exists())
            self.assertEqual((output / 'Assets/Cube.fbx').read_bytes(), path.read_bytes())
            self.assertEqual((output / 'Assets/Cube.fbx.meta').read_bytes(), meta.read_bytes())
            request_path = output / 'Reference/request.json'
            request = json.loads(request_path.read_text())
            report = {'schema': 1, 'unity_version': '6000.3.2f1', 'platform': 'LinuxEditor',
                      'request_sha256': reference.digest(request_path), 'collector_sha256': reference.digest(reference.COLLECTOR),
                      'models': [{**request['models'][0], **model()}]}
            fake_probe = root / 'probe'
            fake_probe.write_text('synthetic test probe; never invoked')
            probe_data = {'ufbx_version': 23001, 'source_axes': [0, 2, 4], 'nodes': [{
                'name': 'Cube', 'deformed': False, 'vertices': (np.array(mesh()['positions']).reshape(-1, 3) * [-1, 1, 1]).tolist(),
                'faces': [[0, 1, 2]], 'geometry_to_node': np.eye(4).tolist(),
                'geometry_to_world': np.eye(4).tolist(), 'node_to_world': np.eye(4).tolist()}]}
            report_path = output / 'Reference/imported.json'
            with patch.object(reference.subprocess, 'check_output', return_value=json.dumps(probe_data).encode()):
                for fault in [None, 'request', 'version', 'guid', 'source', 'id', 'transform', 'extra', 'collector', 'snapshot', 'meta']:
                    value = copy.deepcopy(report)
                    if fault == 'request': value['request_sha256'] = '0' * 64
                    if fault == 'version': value['unity_version'] = '6000.3.3f1'
                    if fault == 'guid': value['models'][0]['guid'] = 'c' * 32
                    if fault == 'source': value['models'][0]['sha256'] = '0' * 64
                    if fault == 'id':
                        value['models'][0]['meshes'][0]['file_id'] = '88'
                        value['models'][0]['nodes'][0]['mesh_file_id'] = '88'
                    if fault == 'transform': value['models'][0]['nodes'][0]['file_id'] = '99'
                    if fault == 'extra': value['models'].append(copy.deepcopy(value['models'][0]))
                    if fault == 'collector': value['collector_sha256'] = '0' * 64
                    snapshot = output / 'Reference/original-meta' / ('a' * 32 + '.meta')
                    if fault == 'snapshot': snapshot.write_text('changed')
                    if fault == 'meta': (output / 'Assets/Cube.fbx.meta').write_text('changed')
                    report_path.write_text(json.dumps(value))
                    if fault:
                        with self.subTest(fault=fault), self.assertRaises(ValueError): reference.inspect(source, output, fake_probe)
                    else:
                        result = reference.inspect(source, output, fake_probe)
                        self.assertTrue(result['models'][0]['metadata_byte_identical'])
                        self.assertTrue(result['models'][0]['meshes'][0]['policies']['mesh_local']['matches'])
                    snapshot.write_bytes(meta.read_bytes())
                    (output / 'Assets/Cube.fbx.meta').write_bytes(meta.read_bytes())

    def test_json_duplicate_keys_nonfinite_and_oversize_reject(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'request.json'
            for text, limit in [('{"a":1,"a":2}', 100), ('{"a":NaN}', 100), ('{}', 1)]:
                path.write_text(text)
                with self.assertRaises(ValueError): reference.read_json(path, limit)


if __name__ == '__main__':
    unittest.main()
