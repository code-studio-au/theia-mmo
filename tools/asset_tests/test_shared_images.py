"""Immutable image extraction, canonical variant binding and real GLB buffers."""
import hashlib
import json
from pathlib import Path
import shutil
import struct
import sys
import tempfile
import unittest

import numpy as np
from PIL import Image
import trimesh

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from externalize_world_images import externalize
from merge_static_glb import read
from package_world import MANIFEST, identity, package
from shared_images import collect, filename, verify


def encode(document, binary):
    text = json.dumps(document).encode()
    text += b' ' * (-len(text) % 4)
    return (struct.pack('<4sII', b'glTF', 2, 28 + len(text) + len(binary))
            + struct.pack('<I4s', len(text), b'JSON') + text
            + struct.pack('<I4s', len(binary), b'BIN\0') + binary)


def fixture(root, dual=False, ambiguous=False):
    root.mkdir()
    vertices = np.array([[0., 0., 0.], [0., 0., 128.], [128., 0., 0.], [128., 0., 128.]])
    faces = np.array([[0, 1, 2], [2, 1, 3]])
    image = Image.new('RGBA', (2, 2), (0, 255, 0, 255))
    material = trimesh.visual.material.PBRMaterial(baseColorTexture=image, normalTexture=image if dual else None)
    visual = trimesh.visual.TextureVisuals(uv=np.array([[0., 0.], [0., 1.], [1., 0.], [1., 1.]]), material=material)
    mesh = trimesh.Trimesh(vertices=vertices, faces=faces, visual=visual, process=False)
    glb = trimesh.exchange.gltf.export_glb(trimesh.Scene(mesh))
    (root / 'terrain.glb').write_bytes(glb)
    if ambiguous:
        document, binary = read(root / 'terrain.glb')
        material = document['materials'][0]
        material['normalTexture']['index'] = material['pbrMetallicRoughness']['baseColorTexture']['index']
        glb = encode(document, binary)
        (root / 'terrain.glb').write_bytes(glb)
    profile = {'schema': 3, 'bounds': [[0., 0.], [128., 128.]], 'source_origin': [0., 0., 0.],
               'visual_sha256': list(hashlib.sha256(glb).digest()), 'spawns': [], 'triangles': vertices[faces].tolist()}
    (root / 'terrain.json').write_text(json.dumps(profile) + '\n')
    package(root, 'theia.test.embedded', 1)


class SharedImageTests(unittest.TestCase):
    def test_new_package_identity_matches_rust_golden_and_canonicalizes_order(self):
        path = Path(__file__).resolve().parents[2] / 'crates/network/tests/fixtures/content-manifest-shared.json'
        document = json.loads(path.read_bytes())
        expected = '9f940fe1bbc96c62789c98e2d947c8b59f6e8169f52c1bfdf4b763efcd106092'
        self.assertEqual(identity(document), expected)
        document['shared_images'].reverse()
        self.assertEqual(identity(document), expected)

    def test_extracts_one_compatible_shared_file_for_two_packages_and_preserves_geometry(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / 'source'
            fixture(source)
            before = {p.name: p.read_bytes() for p in source.iterdir()}
            output = root / 'output'
            reports = [externalize(source, output / f'chunk_{n}_0', f'theia.test.shared{n}', 2) for n in [0, 1]]
            self.assertTrue(all(r['shared_images'] == 1 for r in reports))
            self.assertEqual(len(list((output / 'shared').iterdir())), 1)
            original, binary = read(source / 'terrain.glb')
            document, packed = read(output / 'chunk_0_0/terrain.glb', allow_external_images=True)
            self.assertEqual(document['nodes'], original['nodes'])
            self.assertEqual(document['materials'], original['materials'])
            self.assertEqual(document['meshes'], original['meshes'])
            for old, new in zip(original['accessors'], document['accessors']):
                self.assertEqual({k: v for k, v in old.items() if k != 'bufferView'},
                                 {k: v for k, v in new.items() if k != 'bufferView'})
                a, b = original['bufferViews'][old['bufferView']], document['bufferViews'][new['bufferView']]
                self.assertEqual(binary[a.get('byteOffset', 0):a.get('byteOffset', 0) + a['byteLength']],
                                 packed[b.get('byteOffset', 0):b.get('byteOffset', 0) + b['byteLength']])
            self.assertTrue(all('uri' in image and 'bufferView' not in image for image in document['images']))
            self.assertLess(len(packed), len(binary))
            for name, data in before.items():
                self.assertEqual((source / name).read_bytes(), data)
            manifest = json.loads((output / 'chunk_0_0' / MANIFEST).read_bytes())
            verify(document, output, manifest['shared_images'])
            with self.assertRaises(FileExistsError):
                externalize(source, output / 'chunk_0_0', 'theia.test.overwrite', 9)

    def test_splits_ambiguous_color_contexts_and_keeps_distinct_image_keys(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / 'source'
            fixture(source, dual=True, ambiguous=True)
            target = root / 'output/chunk_0_0'
            report = externalize(source, target, 'theia.test.variants', 2)
            self.assertEqual(report['shared_images'], 2)
            document, _ = read(target / 'terrain.glb', allow_external_images=True)
            resources = collect(document, target.parent)
            self.assertEqual({r['color_space'] for r in resources}, {'srgb', 'linear'})
            self.assertEqual(resources[0]['sha256'], resources[1]['sha256'])
            self.assertNotEqual(filename(resources[0]), filename(resources[1]))
            base = document['materials'][0]['pbrMetallicRoughness']['baseColorTexture']['index']
            normal = document['materials'][0]['normalTexture']['index']
            self.assertNotEqual(base, normal)
            self.assertNotEqual(document['textures'][base]['source'], document['textures'][normal]['source'])

    def test_corrupt_shared_content_or_unsafe_uri_cannot_publish_another_package(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / 'source'
            fixture(source)
            first = root / 'output/chunk_0_0'
            externalize(source, first, 'theia.test.first', 2)
            image, = (first.parent / 'shared').iterdir()
            image.write_bytes(b'corrupt')
            second = first.parent / 'chunk_1_0'
            with self.assertRaisesRegex(ValueError, 'unsafe or corrupt'):
                externalize(source, second, 'theia.test.second', 2)
            self.assertFalse(second.exists())
            document, _ = read(first / 'terrain.glb', allow_external_images=True)
            document['images'][0]['uri'] = '../../private.png'
            with self.assertRaisesRegex(ValueError, 'Noncanonical'):
                collect(document, first.parent)

    def test_changed_representation_requires_a_new_revision_and_keeps_source_immutable(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / 'source'
            fixture(source)
            before = {p.name: p.read_bytes() for p in source.iterdir()}
            with self.assertRaisesRegex(ValueError, 'newer immutable revision'):
                externalize(source, root / 'output', 'theia.test.embedded', 1)
            self.assertFalse((root / 'output').exists())
            for name, data in before.items():
                self.assertEqual((source / name).read_bytes(), data)

    def test_qualified_material_extensions_preserve_strict_legacy_opt_out(self):
        from shared_images import validate_references
        document={'extensionsUsed':['KHR_materials_unlit','KHR_materials_emissive_strength'],
            'materials':[{'extensions':{'KHR_materials_unlit':{}}},{'extensions':{'KHR_materials_emissive_strength':{'emissiveStrength':2.6}}}]}
        with self.assertRaisesRegex(ValueError,'must be unextended'):validate_references(document,[],allow_material_extensions=False)
        validate_references(document,[])
        document['materials'][1]['extensions']['KHR_materials_emissive_strength']['emissiveStrength']=float('nan')
        with self.assertRaises(ValueError):validate_references(document,[],allow_material_extensions=True)
