"""Independent graph pruning and triangle interpolation/seam invariants."""
import copy
from pathlib import Path
import sys
import tempfile
import unittest
import numpy as np
import trimesh
from PIL import Image

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from merge_static_glb import read
from static_geometry import accessor, clip_triangle, hierarchy, prune


class StaticGeometryTests(unittest.TestCase):
    def test_prunes_unused_assets_preserving_shared_mesh_ancestry_and_binary_payloads(self):
        scene = trimesh.Scene()
        for n, color in enumerate([(0, 255, 0, 255), (255, 0, 0, 255)]):
            mesh = trimesh.creation.box()
            mesh.visual = trimesh.visual.TextureVisuals(uv=np.zeros((len(mesh.vertices), 2)),
                material=trimesh.visual.material.PBRMaterial(baseColorTexture=Image.new('RGBA', (2, 2), color)))
            scene.add_geometry(mesh, node_name=f'part{n}', geom_name=f'mesh{n}', transform=trimesh.transformations.translation_matrix([n * 200., 2., 3.]))
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'source.glb'
            path.write_bytes(trimesh.exchange.gltf.export_glb(scene))
            document, binary = read(path)
        # The graph pruner handles shared external image dependencies; actual
        # payload extraction/binding is tested by the image publisher suite.
        for n, image in enumerate(document['images']):
            image.pop('bufferView')
            image['uri'] = f'../shared/test-{n}.png'
        selected = next(n for n, node in enumerate(document['nodes']) if node.get('name') == 'part0')
        before = copy.deepcopy(document)
        reduced, packed = prune(document, binary, {selected})
        self.assertEqual(document, before)
        self.assertEqual(len(reduced['meshes']), 1)
        self.assertEqual(len(reduced['materials']), 1)
        self.assertEqual(len(reduced['images']), 1)
        _, old_world = hierarchy(document)
        _, new_world = hierarchy(reduced)
        new_node = next(n for n, node in enumerate(reduced['nodes']) if node.get('name') == 'part0')
        np.testing.assert_array_equal(new_world[new_node], old_world[selected])
        original = document['meshes'][document['nodes'][selected]['mesh']]['primitives'][0]
        current = reduced['meshes'][0]['primitives'][0]
        for key, old in original['attributes'].items():
            np.testing.assert_array_equal(accessor(document, binary, old), accessor(reduced, packed, current['attributes'][key]))
        np.testing.assert_array_equal(accessor(document, binary, original['indices']), accessor(reduced, packed, current['indices']))

    def test_clipping_conserves_area_plane_winding_and_linear_uv_across_negative_seams(self):
        points = np.array([[-20., -2., 20.], [200., 20., 20.], [-20., -2., 180.]])
        rows = np.concatenate([points, points[:, [0, 2]] / 16], axis=1)
        source_area = np.linalg.norm(np.cross(points[1] - points[0], points[2] - points[0])) / 2
        result = []
        for x in [-1, 0, 1]:
            for z in [0, 1]:
                result.extend(clip_triangle(rows, [x * 128., z * 128.], [(x + 1) * 128., (z + 1) * 128.]))
        area = 0
        original_normal = np.cross(points[1] - points[0], points[2] - points[0])
        for triangle in result:
            normal = np.cross(triangle[1, :3] - triangle[0, :3], triangle[2, :3] - triangle[0, :3])
            self.assertGreater(np.dot(normal, original_normal), 0)
            area += np.linalg.norm(normal) / 2
            np.testing.assert_allclose(triangle[:, 1], triangle[:, 0] * .1, atol=1e-12)
            np.testing.assert_allclose(triangle[:, 3:], triangle[:, [0, 2]] / 16, atol=1e-12)
        self.assertAlmostEqual(area, source_area, places=9)
        self.assertTrue(any(np.any(t[:, 0] == 128.) for t in result))

    def test_hierarchy_rejects_cycles_duplicate_parentage_and_mixed_transforms(self):
        for nodes in [[{'children': [0]}], [{'children': [1, 1]}, {}],
                      [{'matrix': np.eye(4).T.reshape(-1).tolist(), 'translation': [1., 0., 0.]}]]:
            with self.assertRaises(ValueError):
                hierarchy({'nodes': nodes, 'scenes': [{'nodes': [0]}]})
        for unsupported in [{'animations': [{'channels': []}]}, {'extensionsUsed': ['KHR_materials_transmission']},
                            {'nodes': [{'extensions': {'EXT_mesh_gpu_instancing': {}}}]}]:
            document = {'nodes': [{}], 'scenes': [{'nodes': [0]}]}
            document.update(unsupported)
            with self.assertRaises(ValueError):
                hierarchy(document)

    def test_clipping_degenerate_boundary_fragments_does_not_duplicate_ground(self):
        triangle = np.array([[0., 0., 0.], [128., 0., 0.], [0., 0., 128.]])
        self.assertEqual(len(clip_triangle(triangle, [0., 0.], [128., 128.])), 1)
        self.assertEqual(clip_triangle(triangle, [128., 0.], [256., 128.]), [])
