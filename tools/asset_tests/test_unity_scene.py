"""Serialization audit rejects ambiguous input and preserves Unity source IDs."""
import hashlib
import json
from pathlib import Path
import sys
import tempfile
import unittest
import yaml
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from unity_scene import records, load_yaml, audit, Sources


class UnitySceneTests(unittest.TestCase):
    def test_lfs_pointer_reports_missing_payload_and_rejects_malformed_record(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);path=root/'Assets/mesh.fbx';path.parent.mkdir()
            for tail in ['size 12345\n','size invalid\n']:
                path.write_text('version https://git-lfs.github.com/spec/v1\noid sha256:'+'c'*64+'\n'+tail)
                binding={'guid':'a'*32,'bytes':path.stat().st_size,'sha256':hashlib.sha256(path.read_bytes()).hexdigest()}
                (root/'extraction.json').write_text(json.dumps({'selected':{'Assets/mesh.fbx':binding}}))
                sources=Sources(root)
                if 'invalid' in tail:
                    with self.assertRaisesRegex(ValueError,'Malformed'):sources.lfs_pointers()
                else:
                    pointers=sources.lfs_pointers();self.assertEqual(len(pointers),1)
                    self.assertEqual(pointers[0]['oid'],'c'*64)
                    self.assertEqual(pointers[0]['missing_bytes'],12345)

    def test_signed_ids_stripped_records_and_utf8_names(self):
        with tempfile.TemporaryDirectory() as directory:
            p = Path(directory) / 'scene.unity'
            p.write_text('%YAML 1.1\n%TAG !u! tag:unity3d.com,2011:\n--- !u!4 &-7 stripped\nTransform:\n  m_Father: {fileID: 9}\n--- !u!1 &9\nGameObject:\n  m_Name: Elvén Hall\n')
            rr = records(p)
            self.assertTrue(rr[-7]['stripped'])
            self.assertEqual(rr[-7]['fields']['m_Father']['fileID'], 9)
            self.assertEqual(rr[9]['fields']['m_Name'], 'Elvén Hall')

    def test_duplicates_aliases_tags_and_unserialized_prefix_reject(self):
        for text in ['x: 1\nx: 2', 'x: &a [*a]', 'x: !!python/object:__main__.Thing {}']:
            with self.assertRaises((ValueError, yaml.YAMLError)):
                load_yaml(text)
        with tempfile.TemporaryDirectory() as directory:
            p = Path(directory) / 'scene.unity'
            for text in ['--- !u!4 &1\nTransform: {}\n--- !u!4 &1\nTransform: {}',
                         'ignored: true\n--- !u!4 &1\nTransform: {}', 'plain: true']:
                p.write_text(text)
                with self.assertRaises(ValueError):records(p)

    def test_recursive_dependency_audit_reports_model_prefab_gap_and_missing_assets(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            ids = {'Assets/demo.unity':'a'*32, 'Assets/building.prefab':'b'*32, 'Assets/model.fbx':'c'*32}
            text = {
                'Assets/demo.unity': '--- !u!1001 &1\nPrefabInstance:\n  m_SourcePrefab: {fileID: 100100000, guid: '+ 'b'*32 +'}\n  m_Modification:\n    m_Modifications: []\n    m_RemovedComponents: []\n',
                'Assets/building.prefab': '--- !u!1001 &-2\nPrefabInstance:\n  m_SourcePrefab: {fileID: 100100000, guid: '+ 'c'*32 +'}\n  m_Modification:\n    m_Modifications:\n    - propertyPath: m_Name\n      target: {fileID: -9, guid: '+ 'c'*32 +'}\n      value: Gate\n    m_RemovedComponents: [{fileID: 4, guid: '+ 'd'*32 +'}]\n',
                'Assets/model.fbx': 'binary data placeholder, never interpreted as YAML'}
            manifest = {'archive_sha256':'e'*64,'selected':{},'metadata':{}}
            for name, content in text.items():
                path = root/name;path.parent.mkdir(parents=True,exist_ok=True);path.write_text(content)
                meta = Path(str(path)+'.meta');meta.write_text('guid: '+ids[name]+'\nModelImporter:\n  internalIDToNameTable: []\n')
                for kind, p in [('selected',path),('metadata',meta)]:
                    manifest[kind][name] = {'guid':ids[name],'bytes':p.stat().st_size,'sha256':hashlib.sha256(p.read_bytes()).hexdigest()}
            (root/'extraction.json').write_text(json.dumps(manifest))
            report = audit(root, 'Assets/demo.unity')
            self.assertEqual(len(report['prefab_edges']),2)
            self.assertFalse(report['model_prefab_edges'][0]['has_internal_name_table'])
            self.assertIn('d'*32,report['missing_external_guids'])
            self.assertEqual(report['modifications'],{'m_Name':1})
            self.assertFalse(report['complete_scene_export'])
            (root/'Assets/model.fbx').write_text('tampered')
            with self.assertRaisesRegex(ValueError,'mismatch'):
                Sources(root).checked('Assets/model.fbx')
