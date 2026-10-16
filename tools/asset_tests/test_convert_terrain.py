"""Exercise the real offline OBJ/glTF converter with tiny disposable assets."""
import json
from pathlib import Path
import struct
import sys
import tempfile
import unittest

from PIL import Image

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from convert_terrain import convert


class TerrainConversionTests(unittest.TestCase):
    def fixture(self, folder):
        source = folder / 'terrain.obj'
        source.write_text('mtllib terrain.mtl\nv 0 0 0\nv 0 0 1\nv 1 0 0\n'
                          'vt 0 0\nvt 0 1\nvt 1 0\nvn 0 1 0\n'
                          'usemtl ground\nf 1/1/1 2/2/1 3/3/1\n')
        (folder / 'terrain.mtl').write_text('newmtl ground\nmap_Kd ground.png\n')
        Image.new('RGB', (2, 2), (30, 90, 40)).save(folder / 'ground.png')
        return source

    def test_real_glb_keeps_geometry_texture_and_uv_and_refuses_overwrite(self):
        with tempfile.TemporaryDirectory() as temporary:
            folder = Path(temporary)
            source = self.fixture(folder)
            output = folder / 'terrain.glb'
            report = convert(source, output)
            self.assertEqual(report['triangles'], 1)
            self.assertEqual(report['bounds'], [[0, 0, 0], [1, 0, 1]])
            data = output.read_bytes()
            self.assertEqual(struct.unpack_from('<4sII', data), (b'glTF', 2, len(data)))
            size = struct.unpack_from('<I', data, 12)[0]
            doc = json.loads(data[20:20 + size])
            primitive = doc['meshes'][0]['primitives'][0]
            self.assertIn('TEXCOORD_0', primitive['attributes'])
            self.assertIn('NORMAL', primitive['attributes'])
            self.assertEqual(len(doc['images']), 1)
            self.assertEqual(doc['materials'][0]['pbrMetallicRoughness']['baseColorFactor'], [1, 1, 1, 1])
            with self.assertRaises(ValueError):
                convert(source, output)
            self.assertEqual(output.read_bytes(), data)

    def test_missing_texture_fails_without_a_default_material_or_output(self):
        with tempfile.TemporaryDirectory() as temporary:
            folder = Path(temporary)
            source = self.fixture(folder)
            (folder / 'ground.png').unlink()
            output = folder / 'terrain.glb'
            with self.assertRaisesRegex(ValueError, 'Missing texture'):
                convert(source, output)
            self.assertFalse(output.exists())

    def test_explicit_diffuse_colour_is_preserved(self):
        with tempfile.TemporaryDirectory() as temporary:
            folder = Path(temporary)
            source = self.fixture(folder)
            with (folder / 'terrain.mtl').open('a') as file:
                file.write('Kd 0.5 0.25 1\n')
            output = folder / 'terrain.glb'
            convert(source, output)
            data = output.read_bytes()
            size = struct.unpack_from('<I', data, 12)[0]
            colour = json.loads(data[20:20 + size])['materials'][0]['pbrMetallicRoughness']['baseColorFactor']
            for actual, expected in zip(colour, [0.5, 0.25, 1, 1]):
                self.assertAlmostEqual(actual, expected, delta=0.005)


if __name__ == '__main__':
    unittest.main()
