"""Synthetic retained-player fixtures; no proprietary art or Blender required."""
import base64
import json
from pathlib import Path
import struct
import sys
import tempfile
import unittest

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from verify_directional_extension import ADDED, RETAINED, compare


def fixture(path, extended=False, prefix=0, change=None):
    binary = bytearray(prefix)
    views, accessors = [], []

    def view(payload):
        binary.extend(b'\0' * (-len(binary) % 4))
        views.append(dict(buffer=0, byteOffset=len(binary), byteLength=len(payload)))
        binary.extend(payload)
        return len(views)-1

    def data(values, kind, component=5126):
        values = np.array(values, dtype='<f4' if component == 5126 else '<u2')
        accessors.append(dict(bufferView=view(values.tobytes()), componentType=component,
                              count=len(values), type=kind))
        return len(accessors)-1

    positions = [[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]]
    if change == 'geometry':
        positions[0][0] += .01
    position = data(positions, 'VEC3')
    joints = data([[0, 0, 0, 0]]*3, 'VEC4', 5123)
    weights = data([[1., 0., 0., 0.]]*3, 'VEC4')
    indices = data([[0], [1], [2]], 'SCALAR', 5123)
    inverse = data([np.eye(4).flatten()], 'MAT4')
    times = data([[0.], [1.]], 'SCALAR')
    animations = []
    names = sorted(RETAINED | ADDED if extended else RETAINED)
    if change == 'missing':
        names.remove('WalkBack')
    for name in names:
        end = .11 if change == 'curve' and name == 'Walk' else .1
        output = data([[0., 0., 0.], [end, 0., 0.]], 'VEC3')
        interpolation = 'STEP' if change == 'interpolation' and name == 'Walk' else 'LINEAR'
        animations.append(dict(name=name,
                               samplers=[dict(input=times, output=output, interpolation=interpolation)],
                               channels=[dict(sampler=0, target=dict(node=0, path='translation'))]))
    png = base64.b64decode('iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVQIHWP4z8DwHwAFgAI/ScLbtAAAAABJRU5ErkJggg==')
    if change == 'palette':
        png = png[:-1] + bytes([png[-1] ^ 1])
    image = view(png)
    binary.extend(b'\0' * (-len(binary) % 4))
    document = dict(asset=dict(version='2.0'), scene=0, scenes=[dict(nodes=[0])],
                    nodes=[dict(name='Root', children=[1]), dict(name='Body', mesh=0, skin=0)],
                    meshes=[dict(primitives=[dict(attributes=dict(POSITION=position, JOINTS_0=joints,
                                                                   WEIGHTS_0=weights), indices=indices, material=0)])],
                    skins=[dict(joints=[0], inverseBindMatrices=inverse)],
                    materials=[dict(pbrMetallicRoughness=dict(baseColorTexture=dict(index=0)))],
                    textures=[dict(source=0)], images=[dict(bufferView=image, mimeType='image/png')],
                    animations=animations, buffers=[dict(byteLength=len(binary))],
                    bufferViews=views, accessors=accessors)
    if change == 'parentage':
        document['nodes'][0]['children'] = []
        document['scenes'][0]['nodes'].append(1)
    encoded = json.dumps(document).encode()
    encoded += b' ' * (-len(encoded) % 4)
    length = 12+8+len(encoded)+8+len(binary)
    path.write_bytes(struct.pack('<4sII', b'glTF', 2, length)
                     + struct.pack('<I4s', len(encoded), b'JSON')+encoded
                     + struct.pack('<I4s', len(binary), b'BIN\0')+binary)


class DirectionalExtensionTests(unittest.TestCase):
    def run_fixture(self, change=None):
        with tempfile.TemporaryDirectory() as directory:
            before, after = Path(directory)/'old.glb', Path(directory)/'new.glb'
            fixture(before)
            fixture(after, extended=True, prefix=64, change=change)
            return compare(before, after)

    def test_relocated_buffers_and_fourteen_extra_clips_preserve_known_original(self):
        result = self.run_fixture()
        self.assertEqual(result['retained_clips'], 8)
        self.assertEqual(result['added_clips'], 14)
        self.assertEqual(result['maximum_retained_world_matrix_error'], 0.)
        self.assertEqual(result['retained_pose_samples'], 24)

    def test_changed_geometry_is_refused(self):
        with self.assertRaises(AssertionError):
            self.run_fixture('geometry')

    def test_missing_direction_is_refused(self):
        with self.assertRaises(AssertionError):
            self.run_fixture('missing')

    def test_changed_retained_curve_is_refused(self):
        with self.assertRaises(AssertionError):
            self.run_fixture('curve')

    def test_changed_interpolation_is_refused_even_with_identical_key_poses(self):
        with self.assertRaises(AssertionError):
            self.run_fixture('interpolation')

    def test_changed_palette_is_refused(self):
        with self.assertRaises(AssertionError):
            self.run_fixture('palette')

    def test_changed_parentage_is_refused(self):
        with self.assertRaises(AssertionError):
            self.run_fixture('parentage')
