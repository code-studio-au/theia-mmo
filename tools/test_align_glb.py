"""Binary-preservation and failure checks for the offline GLB repair."""
import json
import struct
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

from align_glb import realign


def pack(doc, binary):
    doc = json.dumps(doc).encode()
    doc += b' ' * (-len(doc) % 4)
    binary += b'\0' * (-len(binary) % 4)
    return (struct.pack('<4sII', b'glTF', 2, 28 + len(doc) + len(binary))
            + struct.pack('<I4s', len(doc), b'JSON') + doc
            + struct.pack('<I4s', len(binary), b'BIN\0') + binary)


def unpack(data):
    size = struct.unpack_from('<I', data, 12)[0]
    return json.loads(data[20:20 + size]), data[28 + size:]


class RepairTests(unittest.TestCase):
    def fixture(self):
        return {'asset': {'version': '2.0'}, 'buffers': [{'byteLength': 10}],
                'bufferViews': [{'buffer': 0, 'byteOffset': 0, 'byteLength': 2},
                                {'buffer': 0, 'byteOffset': 2, 'byteLength': 8}],
                'accessors': [{'bufferView': 1, 'componentType': 5126,
                               'count': 2, 'type': 'SCALAR', 'min': [-9], 'max': [9]}],
                'animations': [{'name': 'empty', 'channels': [], 'samplers': []},
                               {'name': 'kept', 'channels': [{}], 'samplers': [{}]}]}

    def test_alignment_preserves_every_view_payload_and_unrelated_json(self):
        doc = self.fixture()
        binary = b'ab' + struct.pack('<ff', 1, 2)
        output, data = unpack(realign(pack(doc, binary)))
        for before, after in zip(doc['bufferViews'], output['bufferViews']):
            self.assertEqual(after['byteOffset'] % 4, 0)
            n = before['byteLength']
            self.assertEqual(binary[before['byteOffset']:before['byteOffset'] + n],
                             data[after['byteOffset']:after['byteOffset'] + n])
        self.assertEqual(doc['accessors'], output['accessors'])
        self.assertEqual(doc['animations'], output['animations'])

    def test_optional_metadata_repair_changes_bounds_and_drops_only_empty_clips(self):
        output, data = unpack(realign(pack(self.fixture(), b'ab' + struct.pack('<ff', 1, 2)), True))
        self.assertEqual(output['accessors'][0]['min'], [1])
        self.assertEqual(output['accessors'][0]['max'], [2])
        self.assertEqual([a['name'] for a in output['animations']], ['kept'])
        self.assertEqual(struct.unpack_from('<ff', data, 4), (1, 2))

    def test_rejects_truncated_external_extended_and_out_of_range_inputs(self):
        with self.assertRaises(ValueError):
            realign(b'glTF')
        for key, value in [('extensionsUsed', ['EXT_meshopt_compression']),
                           ('buffers', [{'byteLength': 10, 'uri': 'other.bin'}]),
                           ('bufferViews', [{'byteOffset': 9, 'byteLength': 8}])]:
            doc = self.fixture()
            doc[key] = value
            with self.assertRaises(ValueError):
                realign(pack(doc, bytes(10)))

    def test_existing_output_and_source_are_never_overwritten(self):
        script = Path(__file__).with_name('align_glb.py')
        with tempfile.TemporaryDirectory() as folder:
            source, output = Path(folder) / 'source.glb', Path(folder) / 'out.glb'
            source.write_bytes(pack(self.fixture(), b'ab' + struct.pack('<ff', 1, 2)))
            original = source.read_bytes()
            output.write_bytes(b'keep')
            for destination in [source, output]:
                result = subprocess.run([sys.executable, str(script), str(source), str(destination)], capture_output=True)
                self.assertNotEqual(result.returncode, 0)
            self.assertEqual(source.read_bytes(), original)
            self.assertEqual(output.read_bytes(), b'keep')


if __name__ == '__main__':
    unittest.main()
