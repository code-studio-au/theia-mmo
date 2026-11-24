"""Independent sample/order and fail-closed import checks on asymmetric terrain."""
import hashlib
import json
from pathlib import Path
import struct
import sys
import tempfile
import unittest

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from import_heightfield import publish, verify


class ImportHeightfieldTests(unittest.TestCase):
    def test_rectangular_offset_source_preserves_each_sample_and_edge(self):
        for spacing in (1, 2, 4):
            with self.subTest(spacing=spacing), tempfile.TemporaryDirectory() as tmp:
                root = Path(tmp)
                cells = 128 // spacing
                z, x = np.mgrid[:cells+1, :2*cells+1]
                heights = np.float32(300 + x*.03125 + z*.125 + (x % 7)*.5)
                source, output = root/'source.npy', root/'terrain'
                np.save(source, heights)
                before = source.read_bytes()
                result = publish(source, output, 'theia.test.heightfield', spacing=spacing, origin=(32, 5))
                self.assertEqual(result['chunks_xz'], [2, 1])
                self.assertEqual(source.read_bytes(), before)
                edges = []
                for cx in (32, 33):
                    data = (output/f'tile_{cx}_5.height').read_bytes()
                    self.assertEqual(struct.unpack('<qq', data[8:24]), (cx, 5))
                    actual = np.frombuffer(data[24:], dtype='<f4').reshape(cells+1, cells+1)
                    for iz in range(cells+1):
                        for ix in range(cells+1):
                            self.assertEqual(actual[iz, ix], heights[iz, (cx-32)*cells+ix])
                    edges.append(actual)
                np.testing.assert_array_equal(edges[0][:, -1], edges[1][:, 0])
                with self.assertRaises(FileExistsError):
                    publish(source, output, 'theia.test.heightfield')
                # A rehashed sample change must fail the independent source oracle.
                tile = output/'tile_32_5.height'
                data = bytearray(tile.read_bytes())
                struct.pack_into('<f', data, 24+4*(cells+2), 500)
                tile.write_bytes(data)
                path = output/'terrain.manifest.json'
                manifest = json.loads(path.read_text())
                manifest['tiles'][0] = list(hashlib.sha256(data).digest())
                path.write_text(json.dumps(manifest))
                with self.assertRaisesRegex(ValueError, 'source sample'):
                    verify(source, output, spacing, (32, 5))

    def test_invalid_sources_do_not_publish(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            for heights in (np.zeros((128, 129), dtype='f4'), np.zeros((129, 129), dtype='f8'),
                            np.full((129, 129), np.nan, dtype='f4'), np.full((129, 129), 1025, dtype='f4')):
                source = root/'bad.npy'
                np.save(source, heights)
                with self.assertRaises(ValueError):
                    publish(source, root/'out', 'theia.test.heightfield')
                self.assertFalse((root/'out').exists())
            np.save(source, np.zeros((129, 129), dtype='f4'))
            for origin in ((125, 0), (-1, 0)):
                with self.assertRaises(ValueError):
                    publish(source, root/'out', 'theia.test.heightfield', origin=origin)
                self.assertFalse((root/'out').exists())
