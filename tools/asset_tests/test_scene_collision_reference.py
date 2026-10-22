import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
import unity_scene_collision_reference as reference


def source(root, executable=False):
    root.mkdir()
    (root / 'Assets').mkdir()
    names = ['Assets/Scene.unity'] + (['Assets/Unsafe.cs'] if executable else [])
    manifest = {'archive_sha256': 'b' * 64, 'selected': {}, 'metadata': {}}
    for i, name in enumerate(names):
        path = root / name
        path.write_text('source data only\n')
        guid = 'a' * 31 + format(i + 1, 'x')
        metadata = Path(str(path) + '.meta')
        metadata.write_text('guid: ' + guid + '\nDefaultImporter: {}\n')
        for key, item in [('selected', path), ('metadata', metadata)]:
            manifest[key][name] = {'guid': guid, 'sha256': reference.digest(item),
                                   'bytes': item.stat().st_size}
    (root / 'extraction.json').write_text(json.dumps(manifest))
    return root


class SceneCollisionReferenceTests(unittest.TestCase):
    def test_data_staging_copies_bound_bytes_and_only_the_reviewed_collector(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            original = source(root / 'source')
            (original / 'Assets/Unselected.cs').write_text('must never be staged')
            output = root / 'build/reference'
            with patch.object(reference, 'BUILD', root / 'build'):
                result = reference.stage(original, output, 'Assets/Scene.unity', '6000.6.6f1')
            self.assertEqual(len(result['inputs']), 1)
            self.assertEqual((output / 'Assets/Scene.unity').read_bytes(), (original / 'Assets/Scene.unity').read_bytes())
            self.assertFalse((output / 'Assets/Unselected.cs').exists())
            self.assertEqual([p.name for p in (output / 'Assets/Editor').iterdir()], ['TheiaSceneCollisionReference.cs'])
            self.assertEqual(json.loads((output / 'Packages/manifest.json').read_text()), {'dependencies': {}})

    def test_selected_executable_is_rejected_before_project_creation(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            original = source(root / 'source', executable=True)
            output = root / 'build/reference'
            with patch.object(reference, 'BUILD', root / 'build'), self.assertRaises(ValueError):
                reference.stage(original, output, 'Assets/Scene.unity', '6000.6.6f1')
            self.assertFalse(output.exists())

    def test_existing_projects_outside_outputs_and_ambiguous_versions_reject(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            original = source(root / 'source')
            existing = root / 'build/existing'
            existing.mkdir(parents=True)
            for output, version in [(existing, '6000.6.6f1'), (root / 'outside', '6000.6.6f1'),
                                    (root / 'build/fresh', '6000')]:
                with self.subTest(output=output, version=version), patch.object(reference, 'BUILD', root / 'build'), self.assertRaises(ValueError):
                    reference.stage(original, output, 'Assets/Scene.unity', version)
