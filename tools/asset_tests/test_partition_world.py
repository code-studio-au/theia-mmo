"""Real package graph pruning and independently reconstructed tiled geometry."""
import copy
import hashlib
import json
from pathlib import Path
import sys
import tempfile
import unittest
import numpy as np
from PIL import Image
import trimesh

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from add_terrain_house import write_glb
from externalize_world_images import externalize
from merge_static_glb import read
from package_world import MANIFEST, package
from partition_world import partition, terrain_rows
from static_geometry import accessor, hierarchy
from verify_world_partition import verify_partition, CollisionOracle, assert_interpolated


def make_source(root):
    embedded = root / 'embedded'
    embedded.mkdir(parents=True)
    vertices = np.array([[0., 0., 0.], [0., 0., 128.], [384., 3.84, 0.], [384., 3.84, 128.]])
    faces = np.array([[0, 1, 2], [2, 1, 3]])
    terrain = trimesh.Trimesh(vertices=vertices, faces=faces, process=False)
    terrain.visual = trimesh.visual.TextureVisuals(uv=vertices[:, [0, 2]] / 16,
        material=trimesh.visual.material.PBRMaterial(baseColorTexture=Image.new('RGBA', (2, 2), (0, 255, 0, 255))))
    scene = trimesh.Scene()
    scene.add_geometry(terrain, geom_name='ground', node_name='ground_node')
    for n, x in enumerate([127., 320.]):
        mesh = trimesh.creation.box(extents=[6., 4., 8.])
        if n:
            mesh.visual = trimesh.visual.TextureVisuals(uv=np.zeros((len(mesh.vertices), 2)),
                material=trimesh.visual.material.PBRMaterial(baseColorTexture=Image.new('RGBA', (2, 2), (255, 0, 0, 255))))
        scene.add_geometry(mesh, geom_name=f'model{n}', node_name=f'model_node{n}',
                           transform=trimesh.transformations.translation_matrix([x, 2., 60.]))
    visual = trimesh.exchange.gltf.export_glb(scene)
    (embedded / 'terrain.glb').write_bytes(visual)
    document, binary = read(embedded / 'terrain.glb')
    wrapper = len(document['nodes'])
    document['nodes'].append({'name': 'world', 'translation': [0., 3., 0.], 'children': document['scenes'][0]['nodes']})
    document['scenes'][0]['nodes'] = [wrapper]
    (embedded / 'terrain.glb').unlink()
    write_glb(document, binary, embedded / 'terrain.glb')
    triangles = vertices[faces].copy()
    triangles[:, :, 1] += 3.
    profile = {'schema': 3, 'bounds': [[0., 0.], [384., 128.]], 'source_origin': [0., 0., 0.],
               'spawns': [[124., 4.24, 32.], [122., 4.22, 34.]], 'triangles': triangles.tolist(),
               'solids': [{'center': [x, 5., 60.], 'size': [6., 4., 8.]} for x in [127., 320.]],
               'visual_sha256': list(hashlib.sha256((embedded / 'terrain.glb').read_bytes()).digest())}
    (embedded / 'terrain.json').write_text(json.dumps(profile) + '\n')
    package(embedded, 'theia.test.partition-source', 1)
    source = root / 'source/town'
    externalize(embedded, source, 'theia.test.partition-source', 2)
    document, _ = read(source / 'terrain.glb', allow_external_images=True)
    terrain_id = next(n for n, mesh in enumerate(document['meshes']) if mesh.get('name') == 'ground')
    return source, terrain_id


class WorldPartitionTests(unittest.TestCase):
    def test_independent_oracles_reject_equal_area_surface_winding_and_uv_corruption(self):
        source_faces = np.array([[[0., 3., 0.], [0., 3., 128.], [384., 6.84, 0.]]])
        oracle = CollisionOracle(source_faces)
        oracle.check(source_faces)
        for changed in [source_faces + [0., .01, 0.], source_faces[:, [0, 2, 1]]]:
            with self.assertRaises(AssertionError):
                oracle.check(changed)
        uv = np.array([[[0., 0.], [0., 8.], [24., 0.]]])
        point = source_faces.mean(axis=1)
        assert_interpolated(point, uv.mean(axis=1), source_faces, uv)
        with self.assertRaises(AssertionError):
            assert_interpolated(point, uv.mean(axis=1) + .01, source_faces, uv)
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source, terrain = make_source(root)
            output = root / 'chunks'
            partition(source, output, 'theia.test.oracle', 1, [terrain])
            result = verify_partition(source, output, [terrain])
            self.assertEqual(result['model_instances_exact'], 2)
            self.assertEqual(result['collision_boxes_exact'], 2)
            profile_path = output / 'chunk_0_0/terrain.json'
            profile = json.loads(profile_path.read_bytes())
            profile['triangles'] = (np.array(profile['triangles']) + [0., .01, 0.]).tolist()
            profile_path.write_text(json.dumps(profile))
            with self.assertRaises(AssertionError):
                verify_partition(source, output, [terrain])
            self.assertFalse((output / 'partition-verification.json').exists())

    def test_reflected_and_singular_terrain_are_explicitly_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            source, terrain = make_source(Path(temporary))
            document, binary = read(source / 'terrain.glb', allow_external_images=True)
            primitive = document['meshes'][terrain]['primitives'][0]
            for scale in [-1., 0.]:
                matrix = np.eye(4)
                matrix[0, 0] = scale
                with self.assertRaisesRegex(ValueError, 'orientation-preserving'):
                    terrain_rows(document, binary, primitive, matrix)

    def test_real_cells_keep_models_once_and_prune_unused_texture_dependencies(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source, terrain = make_source(root)
            original_files = {p.name: p.read_bytes() for p in source.iterdir()}
            output = root / 'chunks'
            report = partition(source, output, 'theia.test.partition', 1, [terrain])
            self.assertEqual(len(report['chunks']), 3)
            self.assertEqual(report['model_instances'], 2)
            self.assertTrue(report['runtime_metadata_emitted'])
            source_doc, source_data = read(source / 'terrain.glb', allow_external_images=True)
            _, original_transforms = hierarchy(source_doc)
            observed = set()
            for cell in report['chunks']:
                x, z = cell['id']
                directory = output / f'chunk_{x}_{z}'
                document, data = read(directory / 'terrain.glb', allow_external_images=True)
                _, transforms = hierarchy(document)
                for mapping in cell['model_nodes']:
                    self.assertNotIn(mapping['source'], observed)
                    observed.add(mapping['source'])
                    current = transforms[mapping['output']].copy()
                    current[:3, 3] += [x * 128., 0., z * 128.]
                    np.testing.assert_allclose(current, original_transforms[mapping['source']], atol=1e-12)
                    old_mesh = source_doc['meshes'][source_doc['nodes'][mapping['source']]['mesh']]
                    new_mesh = document['meshes'][document['nodes'][mapping['output']]['mesh']]
                    for old, new in zip(old_mesh['primitives'], new_mesh['primitives']):
                        for key, index in old['attributes'].items():
                            np.testing.assert_array_equal(accessor(source_doc, source_data, index), accessor(document, data, new['attributes'][key]))
                if x < 2:
                    self.assertEqual(len(document['textures']), 1)
                else:
                    self.assertEqual(len(document['textures']), 2)
            for name, data in original_files.items():
                self.assertEqual((source / name).read_bytes(), data)
            with self.assertRaises(FileExistsError):
                partition(source, output, 'theia.test.overwrite', 9, [terrain])

    def test_signed_distant_origin_preserves_terrain_area_plane_and_barycentric_uvs(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source, terrain = make_source(root)
            origin = (-1e9, 1e9)
            output = root / 'chunks'
            report = partition(source, output, 'theia.test.distant', 1, [terrain], origin)
            original, original_data = read(source / 'terrain.glb', allow_external_images=True)
            _, transforms = hierarchy(original)
            node = next(n for n, value in enumerate(original['nodes']) if value.get('mesh') == terrain)
            primitive = original['meshes'][terrain]['primitives'][0]
            vertices = accessor(original, original_data, primitive['attributes']['POSITION']).astype(float)
            vertices = vertices @ transforms[node][:3, :3].T + transforms[node][:3, 3]
            uv = accessor(original, original_data, primitive['attributes']['TEXCOORD_0'])
            coefficients = np.linalg.lstsq(np.column_stack([vertices[:, [0, 2]], np.ones(len(vertices))]), uv, rcond=None)[0]
            indices = accessor(original, original_data, primitive['indices']).reshape((-1, 3))
            source_area = sum(np.linalg.norm(np.cross(t[1] - t[0], t[2] - t[0])) / 2 for t in vertices[indices])
            area, collision_area = 0., 0.
            for item in report['chunks']:
                x, z = item['id']
                offset = np.array([x * 128. - origin[0], 0., z * 128. - origin[1]])
                directory = output / f'chunk_{x}_{z}'
                document, data = read(directory / 'terrain.glb', allow_external_images=True)
                for value in document['nodes']:
                    if not value.get('name', '').startswith('clipped_'):
                        continue
                    primitive = document['meshes'][value['mesh']]['primitives'][0]
                    positions = accessor(document, data, primitive['attributes']['POSITION']).astype(float)
                    self.assertTrue(np.all(positions[:, [0, 2]] >= 0) and np.all(positions[:, [0, 2]] <= 128))
                    world = positions + offset
                    current_uv = accessor(document, data, primitive['attributes']['TEXCOORD_0'])
                    expected_uv = np.column_stack([world[:, [0, 2]], np.ones(len(world))]) @ coefficients
                    np.testing.assert_allclose(current_uv, expected_uv, atol=2e-6)
                    np.testing.assert_allclose(world[:, 1], 3. + world[:, 0] * .01, atol=2e-6)
                    faces = accessor(document, data, primitive['indices']).reshape((-1, 3))
                    for triangle in world[faces]:
                        normal = np.cross(triangle[1] - triangle[0], triangle[2] - triangle[0])
                        self.assertGreater(normal[1], 0)
                        area += np.linalg.norm(normal) / 2
                profile = json.loads((directory / 'terrain.json').read_bytes())
                for triangle in np.array(profile['triangles']):
                    collision_area += np.linalg.norm(np.cross(triangle[1] - triangle[0], triangle[2] - triangle[0])) / 2
            self.assertAlmostEqual(area, source_area, delta=.001)
            self.assertAlmostEqual(collision_area, source_area, delta=.001)

    def test_invalid_selection_and_corrupt_source_never_publish_output(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source, terrain = make_source(root)
            for selection in [[], [-1], [99999]]:
                output = root / 'invalid'
                with self.assertRaises(ValueError):
                    partition(source, output, 'theia.test.invalid', 1, selection)
                self.assertFalse(output.exists())
            (source / 'terrain.glb').write_bytes(b'changed')
            with self.assertRaisesRegex(ValueError, 'binding mismatch'):
                partition(source, root / 'corrupt', 'theia.test.corrupt', 1, [terrain])
            self.assertFalse((root / 'corrupt').exists())
