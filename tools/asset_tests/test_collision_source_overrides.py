import copy
import unittest
from types import SimpleNamespace
from verify_unity_scene_collision import expand_variant, overrides


def change(identifier, path, value, guid='a' * 32, reference=None):
    return {'target': {'fileID': identifier, 'guid': guid}, 'propertyPath': path,
            'value': value, 'objectReference': reference or {'fileID': 0}}


class ColliderOverrideTests(unittest.TestCase):
    def test_physics_and_ancestry_overrides_preserve_the_original(self):
        raw = {1: {'kind': 1, 'fields': {'m_IsActive': 1, 'm_Layer': 0}},
               2: {'kind': 4, 'fields': {'m_LocalPosition': dict(x=0, y=0, z=0)}},
               3: {'kind': 136, 'fields': {'m_Enabled': 1, 'm_Radius': .2}},
               4: {'kind': 23, 'fields': {}}}
        before = copy.deepcopy(raw)
        actual = overrides(raw, [change(1, 'm_IsActive', '0'),
                                 change(2, 'm_LocalPosition.x', '12.5'),
                                 change(2, 'm_RootOrder', '4'),
                                 change(3, 'm_Enabled', '0'),
                                 change(3, 'm_Radius', '.4'),
                                 change(4, 'unrelated_visual_property', '1'),
                                 change(99, 'unused_target', '1')], 'a' * 32)
        self.assertEqual(actual[1]['fields']['m_IsActive'], 0)
        self.assertEqual(actual[2]['fields']['m_LocalPosition']['x'], 12.5)
        self.assertEqual(actual[3]['fields'], {'m_Enabled': 0, 'm_Radius': .4})
        self.assertEqual(raw, before)

    def test_unknown_relevant_cross_prefab_and_invalid_scalars_reject(self):
        raw = {3: {'kind': 136, 'fields': {'m_Enabled': 1, 'm_Radius': .2,
                                         'm_Center': dict(x=0, y=0, z=0)}}}
        for item in [change(3, 'unknown_physics', '1'),
                     change(3, 'm_Enabled', '1', guid='b' * 32),
                     change(3, 'm_Enabled', '1.5'),
                     change(3, 'm_Radius', 'nan'),
                     change(3, 'm_Center.x', '1', reference={'fileID': 123})]:
            with self.subTest(item=item), self.assertRaises(ValueError):
                overrides(raw, [item], 'a' * 32)

    def variant(self):
        guid = 'b' * 32
        nested = 100
        go, transform = nested ^ 11, nested ^ 12
        raw = {
            nested: {'kind': 1001, 'stripped': False, 'fields': {
                'm_SourcePrefab': {'guid': guid}, 'm_Modification': {
                    'm_TransformParent': {'fileID': 0}, 'm_Modifications': [
                        change(12, 'm_LocalPosition.x', '0', guid),
                        change(11, 'm_Name', 'SourceBranch', guid)], 'm_RemovedComponents': []}}},
            go: {'kind': 1, 'stripped': True, 'fields': {
                'm_PrefabInstance': {'fileID': nested},
                'm_CorrespondingSourceObject': {'fileID': 11, 'guid': guid}}},
            42: {'kind': 64, 'stripped': False, 'fields': {'m_GameObject': {'fileID': go}}}}
        outer = []
        for field, axes in [('m_LocalPosition', 'xyz'), ('m_LocalRotation', 'xyzw'), ('m_LocalScale', 'xyz')]:
            for axis in axes:
                value = 1 if field == 'm_LocalScale' or (field == 'm_LocalRotation' and axis == 'w') else 0
                outer.append(change(transform, field + '.' + axis, str(value)))
        sources = SimpleNamespace(by_guid={guid: 'Assets/Branch.fbx'},
                                  metadata=lambda _: {'ModelImporter': {'meshes': {'addColliders': 0}}})
        return raw, outer, sources, go, transform

    def test_single_model_variant_keeps_added_collider_and_exact_final_trs(self):
        raw, outer, sources, go, transform = self.variant()
        before = copy.deepcopy(raw)
        result = expand_variant(raw, outer, 'a' * 32, sources)
        self.assertEqual(set(result), {go, transform, 42})
        self.assertEqual(result[go]['fields']['m_Name'], 'SourceBranch')
        self.assertEqual(result[transform]['fields']['m_LocalScale'], dict(x=1, y=1, z=1))
        self.assertEqual(result[42], raw[42])
        self.assertEqual(raw, before)

    def test_variant_missing_final_trs_duplicate_fields_and_wrong_owner_reject(self):
        for fault in ['missing', 'duplicate', 'owner', 'generated_colliders']:
            raw, outer, sources, _, _ = self.variant()
            if fault == 'missing': outer.pop()
            if fault == 'duplicate': outer.append(copy.deepcopy(outer[0]))
            if fault == 'owner': raw[42]['fields']['m_GameObject']['fileID'] = 999
            if fault == 'generated_colliders': sources.metadata = lambda _: {'ModelImporter': {'meshes': {'addColliders': 1}}}
            with self.subTest(fault=fault), self.assertRaises(ValueError):
                expand_variant(raw, outer, 'a' * 32, sources)
