"""Cross-language package identity and offline artifact binding."""
import hashlib
import json
from pathlib import Path
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from package_world import package, identity, MAX_VISUAL_BYTES, MANIFEST


class PackageTests(unittest.TestCase):
    def fixture(self, root):
        visual = b'Binding fixture, not a rendered GLB'
        (root / 'terrain.glb').write_bytes(visual)
        (root / 'terrain.json').write_text(json.dumps({'schema': 2, 'visual_sha256': list(hashlib.sha256(visual).digest())}))

    def test_shared_rust_golden_vector(self):
        fixture = Path(__file__).resolve().parents[2] / 'crates/network/tests/fixtures/content-manifest.json'
        self.assertEqual(identity(json.loads(fixture.read_text())),
                         '779362a91793754893713aa35b709e773164332295f7ccb3fc49d7f5756b2d26')

    def test_package_preserves_inputs_is_path_independent_and_never_overwrites(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            results = []
            for name in ['first', 'other']:
                world = root / name
                world.mkdir()
                self.fixture(world)
                original = {p.name: p.read_bytes() for p in world.iterdir()}
                results.append(package(world, 'theia.test.flat', 7))
                manifest = json.loads((world / MANIFEST).read_text())
                self.assertEqual(manifest['collision_sha256'], list(hashlib.sha256(original['terrain.json']).digest()))
                for name, data in original.items():
                    self.assertEqual((world / name).read_bytes(), data)
                with self.assertRaises(FileExistsError):
                    package(world, 'theia.test.flat', 8)
            self.assertEqual(results[0]['identity'], results[1]['identity'])

    def test_invalid_identity_revision_and_visual_binding_write_nothing(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            self.fixture(root)
            for name, revision in [('../world', 1), ('World', 1), ('a' * 97, 1), ('theia.test', 0), ('theia.test', True), ('theia.test', 2**32)]:
                with self.assertRaises(ValueError):
                    package(root, name, revision)
                self.assertFalse((root / MANIFEST).exists())
            (root / 'terrain.glb').write_bytes(b'wrong')
            with self.assertRaisesRegex(ValueError, 'does not match'):
                package(root, 'theia.test.flat', 1)
            self.assertFalse((root / MANIFEST).exists())

    def test_missing_and_oversized_files_fail_before_publication(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            self.fixture(root)
            (root / 'terrain.glb').unlink()
            with self.assertRaises(FileNotFoundError):
                package(root, 'theia.test.flat', 1)
            with (root / 'terrain.glb').open('wb') as file:
                file.truncate(MAX_VISUAL_BYTES + 1)
            with self.assertRaisesRegex(ValueError, 'byte budget'):
                package(root, 'theia.test.flat', 1)
            self.assertFalse((root / MANIFEST).exists())
