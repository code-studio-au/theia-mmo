import hashlib
import json
from pathlib import Path
import struct
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from stage_editor_scenery import stage


class EditorSceneryTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.batch = self.root / 'batch'
        self.prototype = self.batch / 'prototypes' / ('a' * 64)
        self.prototype.mkdir(parents=True)
        binary = struct.pack('<9f', 0, 0, 0, 1, 0, 0, 0, 0, 1)
        document = dict(asset={'version': '2.0'}, buffers=[{'byteLength': len(binary)}],
                        bufferViews=[{'buffer': 0, 'byteLength': len(binary)}],
                        accessors=[{'bufferView': 0, 'componentType': 5126, 'type': 'VEC3', 'count': 3}],
                        meshes=[{'primitives': [{'attributes': {'POSITION': 0}}]}],
                        nodes=[{'mesh': 0}], scenes=[{'nodes': [0]}], scene=0)
        text = json.dumps(document).encode()
        text += b' ' * (-len(text) % 4)
        self.original = (struct.pack('<4sII', b'glTF', 2, 28 + len(text) + len(binary))
                         + struct.pack('<I4s', len(text), b'JSON') + text
                         + struct.pack('<I4s', len(binary), b'BIN\0') + binary)
        (self.prototype / 'preview.glb').write_bytes(self.original)
        sha = hashlib.sha256(self.original).hexdigest()
        self.record = dict(key='a' * 64, source='Assets/Prop.prefab', profile='standard-opaque', prototype_sha256=sha)
        (self.batch / 'batch.json').write_text(json.dumps({'converted': [self.record]}))
        proof = dict(passed=True, asset_sha256=sha, triangles=1,
                     bounds_y_up_metres=dict(min=[0, 0, 0], max=[1, 0, 1]))
        (self.prototype / 'independent-verification.json').write_text(json.dumps(proof))
        (self.prototype / 'conversion.json').write_text(json.dumps({'archive_sha256': 'b' * 64}))
        self.selection = self.root / 'selection.json'
        self.selection.write_text(json.dumps(dict(schema=1, id='theia.test.library', assets=[
            dict(id='theia.prop', name='Prop', category='Props', prefab='Prop', collision='triangle_mesh')
        ])))
        self.output = self.root / 'library'

    def test_exact_assets_and_bounds_are_staged_without_replacement(self):
        result = stage(self.batch, self.selection, self.output)
        self.assertEqual(result['assets'], 1)
        catalog = json.loads((self.output / 'catalog.json').read_bytes())
        asset = catalog['assets'][0]
        self.assertEqual(asset['bounds'], [[0, 0, 0], [1, 0, 1]])
        self.assertEqual((self.output / asset['file']).read_bytes(), self.original)
        self.assertEqual((self.prototype / 'preview.glb').read_bytes(), self.original)
        collision = json.loads((self.output / asset['collision']['file']).read_bytes())
        self.assertEqual(collision['vertices'], [[0,0,0],[1,0,0],[0,0,1]])
        self.assertEqual(collision['indices'], [[0,1,2]])
        with self.assertRaises(ValueError):
            stage(self.batch, self.selection, self.output)
        self.assertEqual((self.output / asset['file']).read_bytes(), self.original)

    def test_changed_asset_or_ambiguous_binding_refuses_before_publication(self):
        (self.prototype / 'preview.glb').write_bytes(self.original + b'changed')
        with self.assertRaisesRegex(ValueError, 'verification'):
            stage(self.batch, self.selection, self.output)
        self.assertFalse(self.output.exists())
        (self.prototype / 'preview.glb').write_bytes(self.original)
        (self.batch / 'batch.json').write_text(json.dumps({'converted': [self.record, self.record]}))
        with self.assertRaisesRegex(ValueError, 'one qualified'):
            stage(self.batch, self.selection, self.output)
        self.assertFalse(self.output.exists())

    def test_wrong_bounds_and_unsafe_ids_refuse(self):
        proof = self.prototype / 'independent-verification.json'
        data = json.loads(proof.read_text())
        data['bounds_y_up_metres']['max'][0] = 2
        proof.write_text(json.dumps(data))
        with self.assertRaisesRegex(ValueError, 'bounds differ'):
            stage(self.batch, self.selection, self.output)
        selected = json.loads(self.selection.read_text())
        selected['assets'][0]['id'] = '../asset'
        self.selection.write_text(json.dumps(selected))
        with self.assertRaisesRegex(ValueError, 'scenery ID'):
            stage(self.batch, self.selection, self.output)
        self.assertFalse(self.output.exists())

    def test_collision_policy_is_explicit_and_decoration_has_no_mesh(self):
        selected = json.loads(self.selection.read_text())
        del selected['assets'][0]['collision']
        self.selection.write_text(json.dumps(selected))
        with self.assertRaisesRegex(ValueError, 'explicit'):
            stage(self.batch, self.selection, self.output)
        self.assertFalse(self.output.exists())
        selected['assets'][0]['collision'] = 'decorative'
        self.selection.write_text(json.dumps(selected))
        stage(self.batch, self.selection, self.output)
        catalog = json.loads((self.output / 'catalog.json').read_text())
        self.assertEqual(catalog['assets'][0]['collision'], {'kind': 'decorative'})
        self.assertEqual(list(self.output.glob('*.collision.json')), [])
