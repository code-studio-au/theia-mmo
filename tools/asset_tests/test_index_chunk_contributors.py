"""Contributors come from real bound package geometry, including GLB transforms."""
import hashlib
import json
from pathlib import Path
import struct
import shutil
import sys
import tempfile
import unittest

import numpy as np
import trimesh

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from create_chunk_fixture import create
from index_chunk_contributors import author
from merge_static_glb import read
from package_world import MANIFEST, package


def republish(root, chunk, visual, profile):
    index = json.loads((root / 'world.chunks.json').read_bytes())
    profile['visual_sha256'] = list(hashlib.sha256(visual).digest())
    (chunk / 'terrain.glb').write_bytes(visual)
    (chunk / 'terrain.json').write_text(json.dumps(profile) + '\n')
    (chunk / MANIFEST).unlink()
    identity = package(chunk, 'theia.test.contributors', 2)['identity']
    for entry in index['chunks']:
        if chunk.name == f"chunk_{entry['id'][0]}_{entry['id'][1]}":
            entry['package'] = list(bytes.fromhex(identity))
    (root / 'world.chunks.json').write_text(json.dumps(index) + '\n')


def encode(document, binary):
    text = json.dumps(document).encode()
    text += b' ' * (-len(text) % 4)
    return (struct.pack('<4sII', b'glTF', 2, 28 + len(text) + len(binary))
            + struct.pack('<I4s', len(text), b'JSON') + text
            + struct.pack('<I4s', len(binary), b'BIN\0') + binary)


class ContributorIndexTests(unittest.TestCase):
    def test_rotated_sparse_mesh_binds_transformed_local_aabb_not_only_its_vertices(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory)/'fixture';create(root)
            chunk=root/'chunk_0_0';scene=trimesh.load_scene(chunk/'terrain.glb',process=False)
            triangle=trimesh.Trimesh(vertices=[[0.,0.,0.],[8.,0.,0.],[0.,0.,4.]],faces=[[0,1,2]],process=False)
            transform=trimesh.transformations.rotation_matrix(np.pi/4,[0.,1.,0.])
            transform[:3,3]=[190.,2.,140.];scene.add_geometry(triangle,transform=transform)
            visual=trimesh.exchange.gltf.export_glb(scene)
            profile=json.loads((chunk/'terrain.json').read_text());republish(root,chunk,visual,profile)
            output=root/'next.json';author(root,output)
            bounds=next(e['bounds'] for e in json.loads(output.read_text())['chunks'] if e['id']==[0,0])
            # Corner (8,0,4) is not a source vertex, but belongs to the local
            # mesh AABB used by Bevy. It reaches x=198.485..., beyond every
            # transformed source vertex (maximum x=195.657...).
            self.assertEqual(bounds[1][0],198.4853515625)

    def test_derives_actual_transformed_visual_and_collision_bounds_without_mutating_packages(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / 'fixture'
            create(root, -7812500, 7812500)
            chunk = root / 'chunk_-7812500_7812500'
            scene = trimesh.load_scene(chunk / 'terrain.glb', process=False)
            box = trimesh.creation.box(extents=[4., 2., 4.])
            transform = np.eye(4)
            transform[:3, 3] = [190., 2., 140.]
            scene.add_geometry(box, transform=transform)
            visual = trimesh.exchange.gltf.export_glb(scene)
            profile = json.loads((chunk / 'terrain.json').read_bytes())
            profile['camera_solids'] = [{'center': [-10., 4., 20.], 'size': [4., 2., 2.]}]
            republish(root, chunk, visual, profile)
            # Give accessors false extrema: derive from positions, not metadata.
            document, binary = read(chunk / 'terrain.glb')
            for accessor in document['accessors']:
                if accessor['type'] == 'VEC3':
                    accessor['min'] = [0., 0., 0.]
                    accessor['max'] = [0., 0., 0.]
            republish(root, chunk, encode(document, binary), profile)
            before = {p.relative_to(root): p.read_bytes() for p in root.rglob('*') if p.is_file()}
            output = root / 'world.chunks.next.json'
            report = author(root, output)
            self.assertEqual(report['chunks'], 3)
            self.assertEqual(report['maximum_cell_contributors'], 3)
            result = json.loads(output.read_bytes())
            self.assertEqual(result['schema'], 2)
            middle = result['chunks'][1]
            self.assertEqual(middle['bounds'], [[-12., -1.], [192., 142.]])
            self.assertEqual(result['query_origin'], [-1e9, 0., 1e9])
            for path, data in before.items():
                self.assertEqual((root / path).read_bytes(), data)
            with self.assertRaises(FileExistsError):
                author(root, output)

    def test_rejects_artifact_or_identity_mismatch_without_publishing(self):
        for corruption in ['artifact', 'identity']:
            with self.subTest(corruption=corruption), tempfile.TemporaryDirectory() as directory:
                root = Path(directory) / 'fixture'
                create(root)
                if corruption == 'artifact':
                    (root / 'chunk_0_0/terrain.glb').write_bytes(b'changed')
                else:
                    index = json.loads((root / 'world.chunks.json').read_bytes())
                    index['chunks'][1]['package'] = [0] * 32
                    (root / 'world.chunks.json').write_text(json.dumps(index))
                output = root / 'next.json'
                with self.assertRaisesRegex(ValueError, 'disagree'):
                    author(root, output)
                self.assertFalse(output.exists())

    def test_rejects_animated_or_out_of_envelope_visuals_despite_valid_bindings(self):
        for invalid in ['animation', 'bounds']:
            with self.subTest(invalid=invalid), tempfile.TemporaryDirectory() as directory:
                root = Path(directory) / 'fixture'
                create(root)
                chunk = root / 'chunk_0_0'
                document, binary = read(chunk / 'terrain.glb')
                if invalid == 'animation':
                    document['animations'] = [{'channels': [], 'samplers': []}]
                else:
                    document['nodes'][0].pop('matrix', None)
                    document['nodes'][0]['translation'] = [2000., 0., 0.]
                profile = json.loads((chunk / 'terrain.json').read_bytes())
                republish(root, chunk, encode(document, binary), profile)
                output = root / 'next.json'
                with self.assertRaisesRegex(ValueError, 'static|envelope'):
                    author(root, output)
                self.assertFalse(output.exists())

    def test_duplicate_or_out_of_frame_owners_are_not_authored(self):
        for invalid in ['duplicate', 'frame']:
            with self.subTest(invalid=invalid), tempfile.TemporaryDirectory() as directory:
                root = Path(directory) / 'fixture'
                create(root)
                index = json.loads((root / 'world.chunks.json').read_bytes())
                if invalid == 'duplicate':
                    index['chunks'][1] = index['chunks'][0]
                else:
                    index['query_origin'] = [10000., 0., 0.]
                (root / 'world.chunks.json').write_text(json.dumps(index))
                output = root / 'next.json'
                with self.assertRaisesRegex(ValueError, 'duplicate|query frame'):
                    author(root, output)
                self.assertFalse(output.exists())

    def test_real_overlapping_packages_cannot_exceed_contributor_budget(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / 'fixture'
            create(root)
            index = json.loads((root / 'world.chunks.json').read_bytes())
            for x in [-1, 0]:
                shutil.copytree(root / 'chunk_0_0', root / f'chunk_{x}_1')
                index['chunks'].append({'id': [x, 1], 'package': index['chunks'][1]['package']})
            (root / 'world.chunks.json').write_text(json.dumps(index))
            output = root / 'next.json'
            with self.assertRaisesRegex(ValueError, 'contributors exceed'):
                author(root, output, resident_limit=4)
            self.assertFalse(output.exists())
