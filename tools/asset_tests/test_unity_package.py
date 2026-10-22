"""Explicit source extraction cannot overwrite inputs or follow archive paths."""
import io
from pathlib import Path
import sys
import tarfile
import struct
import tempfile
import unittest
from unittest.mock import patch
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from unity_package import asset_path, extract_selected, inspect


def archive(path, names):
    with tarfile.open(path, 'w:gz') as stream:
        for n, name in enumerate(names):
            for kind, data in [('asset', b'original model bytes'), ('pathname', name.encode())]:
                entry = tarfile.TarInfo(f'{n:032x}/{kind}');entry.size = len(data)
                stream.addfile(entry, io.BytesIO(data))


class UnityPackageTests(unittest.TestCase):
    def test_optional_metadata_is_hash_bound_complete_and_budgeted(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory);package = root / 'source.unitypackage'
            with tarfile.open(package, 'w:gz') as stream:
                for kind, data in [('pathname', b'Assets/model.fbx'), ('asset', b'fbx'), ('asset.meta', b'guid: abc\n')]:
                    entry = tarfile.TarInfo('a'*32+'/'+kind);entry.size=len(data)
                    stream.addfile(entry, io.BytesIO(data))
            before = package.read_bytes()
            result = extract_selected(package, root/'complete', ['Assets/model.fbx'], include_meta=True)
            self.assertEqual((root/'complete/Assets/model.fbx.meta').read_bytes(), b'guid: abc\n')
            self.assertEqual(result['metadata']['Assets/model.fbx']['bytes'], 10)
            self.assertEqual(package.read_bytes(), before)
            with patch('unity_package.MAX_SELECTED', 6):
                with self.assertRaisesRegex(ValueError, 'budget'):
                    extract_selected(package, root/'over-budget', ['Assets/model.fbx'], include_meta=True)
            self.assertFalse((root/'over-budget').exists())
            archive(package, ['Assets/model.fbx'])
            with self.assertRaisesRegex(ValueError, 'metadata is missing'):
                extract_selected(package, root/'missing', ['Assets/model.fbx'], include_meta=True)
            self.assertFalse((root/'missing').exists())

    def test_explicit_payloads_keep_original_paths_and_bytes_without_source_changes(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory);package = root / 'source.unitypackage'
            archive(package, ['Assets/Character.fbx\n00', 'Assets/unrequested.cs'])
            compressed = bytearray(package.read_bytes());compressed[3] |= 4
            package.write_bytes(compressed[:10] + struct.pack('<H', 4) + b'SY\0\0' + compressed[10:])
            before = package.read_bytes()
            report = extract_selected(package, root / 'output', ['Assets/Character.fbx'])
            self.assertEqual((root / 'output/Assets/Character.fbx').read_bytes(), b'original model bytes')
            self.assertEqual(len(report['selected']), 1)
            self.assertFalse((root / 'output/Assets/unrequested.cs').exists())
            self.assertEqual(package.read_bytes(), before)
            with self.assertRaises(FileExistsError):
                extract_selected(package, root / 'output', ['Assets/Character.fbx'])

    def test_traversal_duplicate_destinations_and_missing_selection_never_publish(self):
        for name in ['../escape', '/absolute', 'Assets/../escape', 'Assets/./escape', 'Assets/\\escape', 'Assets/C:/escape']:
            with self.assertRaises(ValueError):asset_path(name.encode())
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory);package = root / 'source.unitypackage'
            archive(package, ['Assets/one.fbx', 'Assets/one.fbx'])
            with self.assertRaises(ValueError):inspect(package)
            archive(package, ['Assets/one.fbx'])
            with self.assertRaises(ValueError):extract_selected(package, root / 'output', ['Assets/missing.fbx'])
            self.assertFalse((root / 'output').exists())

    def test_payload_symlink_is_rejected_without_following_its_destination(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'source.unitypackage'
            with tarfile.open(path, 'w:gz') as stream:
                entry = tarfile.TarInfo('0' * 32 + '/asset')
                entry.type = tarfile.SYMTYPE;entry.linkname = '/outside/source'
                stream.addfile(entry)
            with self.assertRaisesRegex(ValueError, 'aliases'):
                inspect(path)
