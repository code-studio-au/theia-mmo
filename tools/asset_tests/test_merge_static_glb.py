import json
from pathlib import Path
import struct
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from merge_static_glb import merge, read


def fixture(path, binary, extra=None):
    document = {'asset': {'version': '2.0'}, 'buffers': [{'byteLength': len(binary)}],
                'bufferViews': [{'buffer': 0, 'byteOffset': 0, 'byteLength': len(binary)}],
                'accessors': [{'bufferView': 0, 'componentType': 5126, 'type': 'VEC3', 'count': 1}],
                'images': [{'bufferView': 0, 'mimeType': 'image/png'}],
                'textures': [{'source': 0, 'sampler': 0}], 'samplers': [{}],
                'materials': [{'pbrMetallicRoughness': {'baseColorTexture': {'index': 0}}}],
                'meshes': [{'primitives': [{'attributes': {'POSITION': 0}, 'material': 0}]}],
                'nodes': [{'children': [1, 2]}, {'mesh': 0}, {'mesh': 0}],
                'scenes': [{'nodes': [0]}], 'scene': 0}
    document.update(extra or {})
    text = json.dumps(document).encode()
    text += b' ' * (-len(text) % 4)
    path.write_bytes(struct.pack('<4sII', b'glTF', 2, 28 + len(text) + len(binary))
                     + struct.pack('<I4s', len(text), b'JSON') + text
                     + struct.pack('<I4s', len(binary), b'BIN\0') + binary)


class MergeTests(unittest.TestCase):
    def test_rebases_references_and_keeps_two_instances_sharing_one_mesh(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            a, b, output = [root / n for n in ['a.glb', 'b.glb', 'out.glb']]
            fixture(a, bytes(12))
            fixture(b, b'123456789abc')
            merge(a, b, output)
            doc, binary = read(output)
            self.assertEqual(binary, bytes(12) + b'123456789abc')
            self.assertEqual(doc['scenes'][0]['nodes'], [0, 3])
            self.assertEqual(doc['nodes'][3]['children'], [4, 5])
            self.assertEqual(doc['nodes'][4]['mesh'], doc['nodes'][5]['mesh'])
            self.assertEqual(doc['meshes'][1]['primitives'][0]['attributes']['POSITION'], 1)
            self.assertEqual(doc['materials'][1]['pbrMetallicRoughness']['baseColorTexture']['index'], 1)
            self.assertEqual(doc['textures'][1], {'source': 1, 'sampler': 1})
            self.assertEqual(doc['images'][1]['bufferView'], 1)
            self.assertEqual(doc['bufferViews'][1]['byteOffset'], 12)
            previous = output.read_bytes()
            with self.assertRaises(FileExistsError):
                merge(a, b, output)
            self.assertEqual(output.read_bytes(), previous)

    def test_nonstatic_and_external_inputs_are_rejected_on_either_side(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            a, b, output = [root / n for n in ['a.glb', 'b.glb', 'out.glb']]
            for invalid in [{'animations': [{}]}, {'images': [{'uri': 'loose.png'}]}, {'extensionsUsed': ['EXT_test']}]:
                fixture(a, bytes(12), invalid)
                fixture(b, bytes(12))
                for first, second in [(a, b), (b, a)]:
                    with self.assertRaises(ValueError):
                        merge(first, second, output)
                    self.assertFalse(output.exists())
