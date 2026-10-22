"""Independent raw prefab hierarchy/override comparison with actual scene physics.

This verifies the Editor reference, not a runtime collider conversion. It imports
neither the visual planner nor a collision authoring implementation.
"""
import argparse
import collections
import copy
import json
from pathlib import Path
import re
import numpy as np
from unity_scene import Sources, records
from unity_scene_collision_reference import inspect

KINDS = {64: 'MeshCollider', 65: 'BoxCollider', 136: 'CapsuleCollider', 154: 'TerrainCollider'}


def expand_variant(raw, outer, guid, sources):
    nested = [(i, r['fields']) for i, r in raw.items() if r['kind'] == 1001]
    if len(nested) != 1:
        raise ValueError('Only single-model collision variants are qualified')
    instance, fields = nested[0]
    model_guid = fields['m_SourcePrefab']['guid']
    model = sources.by_guid[model_guid]
    if not model.lower().endswith('.fbx') or sources.metadata(model)['ModelImporter']['meshes'].get('addColliders') != 0:
        raise ValueError('Variant inherits unqualified generated colliders')
    modification = fields['m_Modification']
    if modification['m_TransformParent']['fileID'] or any(v for k, v in modification.items() if k.startswith(('m_Removed', 'm_Added'))):
        raise ValueError('Nested variant structural changes are unqualified')
    gos = [(i, r['fields']) for i, r in raw.items() if r['kind'] == 1 and r['stripped']]
    if len(gos) != 1:
        raise ValueError('Variant must have one explicit stripped owning GameObject')
    go, owner = gos[0]
    if owner['m_PrefabInstance']['fileID'] != instance or owner['m_CorrespondingSourceObject']['guid'] != model_guid or go != ((instance ^ owner['m_CorrespondingSourceObject']['fileID']) & ((1 << 63) - 1)):
        raise ValueError('Variant owning GameObject alias differs')
    transforms = {m['target']['fileID'] for m in modification['m_Modifications'] if m['propertyPath'].startswith('m_Local')}
    if len(transforms) != 1:
        raise ValueError('Variant source transform identity is ambiguous')
    original_transform, = transforms
    transform = (instance ^ original_transform) & ((1 << 63) - 1)
    required = {f'{field}.{axis}' for field, axes in [('m_LocalPosition', 'xyz'), ('m_LocalRotation', 'xyzw'), ('m_LocalScale', 'xyz')] for axis in axes}
    supplied = {}
    for item in outer:
        if item['target'].get('guid') == guid and item['target']['fileID'] == transform and item['propertyPath'] in required:
            if item['propertyPath'] in supplied or item['objectReference']['fileID']:
                raise ValueError('Duplicate/object-valued final variant transform')
            supplied[item['propertyPath']] = scalar(item['value'])
    if set(supplied) != required:
        raise ValueError('Final collision variant must explicitly replace all ten TRS fields')
    names = [m['value'] for m in modification['m_Modifications'] if m['target']['fileID'] == owner['m_CorrespondingSourceObject']['fileID'] and m['propertyPath'] == 'm_Name']
    if len(names) != 1:
        raise ValueError('Variant source name is not explicit')
    go_fields = {'m_Name': names[0], 'm_IsActive': 1, 'm_Layer': 0}
    transform_fields = {'m_GameObject': {'fileID': go}, 'm_Father': {'fileID': 0}}
    for field, axes in [('m_LocalPosition', 'xyz'), ('m_LocalRotation', 'xyzw'), ('m_LocalScale', 'xyz')]:
        transform_fields[field] = {a: supplied[field + '.' + a] for a in axes}
    result = {go: {'kind': 1, 'fields': go_fields, 'stripped': False},
              transform: {'kind': 4, 'fields': transform_fields, 'stripped': False}}
    for identifier, row in raw.items():
        if row['kind'] in KINDS:
            if row['fields']['m_GameObject']['fileID'] != go or row['stripped']:
                raise ValueError('Variant collider ownership is unqualified')
            result[identifier] = row
        elif row['kind'] not in {1, 1001}:
            raise ValueError('Unexpected collision variant component')
    return result


def scalar(value, integer=False):
    number = int(value) if integer else float(value)
    if not np.isfinite(number) or (integer and str(number) != str(value)):
        raise ValueError('Invalid relevant source override scalar')
    return number


def overrides(raw, changes, guid):
    result = dict(raw)
    changed = set()
    for change in changes:
        target = change['target']
        if target.get('guid') != guid:
            raise ValueError('Source override crosses prefab identity')
        identifier = target['fileID']
        if identifier not in raw:
            continue  # Serialized unused targets have no live object to alter.
        kind = raw[identifier]['kind']
        if kind not in {1, 4, *KINDS}:
            continue  # Renderer/particle properties cannot alter these colliders.
        if identifier not in changed:
            result[identifier] = copy.deepcopy(raw[identifier])
            changed.add(identifier)
        fields = result[identifier]['fields']
        path = change['propertyPath']
        vector = re.fullmatch(r'(m_LocalPosition|m_LocalScale|m_LocalRotation|m_Center|m_Size)\.([xyzw])', path)
        if vector:
            name, axis = vector.groups()
            if axis not in fields.get(name, {}) or change['objectReference']['fileID']:
                raise ValueError('Unqualified source transform/shape override')
            fields[name][axis] = scalar(change['value'])
        elif path.startswith('m_LocalEulerAnglesHint.') and kind == 4:
            continue  # Unity's stored quaternion owns the transform.
        elif path == 'm_RootOrder' and kind == 4:
            continue  # Sibling ordering does not alter transform ancestry/TRS.
        elif kind == 1 and path == 'm_Name':
            fields[path] = change['value']
        elif path in {'m_IsActive', 'm_Layer', 'm_Enabled', 'm_IsTrigger', 'm_Convex', 'm_Direction', 'm_CookingOptions'} and path in fields:
            fields[path] = scalar(change['value'], integer=True)
        elif kind in KINDS and path in {'m_Radius', 'm_Height'} and path in fields:
            fields[path] = scalar(change['value'])
        elif kind in KINDS and path in {'m_Mesh', 'm_Material'} and path in fields:
            fields[path] = copy.deepcopy(change['objectReference'])
        else:
            raise ValueError('Unqualified relevant source override: ' + path)
    return result


def verify(source, project):
    import bpy  # Registers the independently used mathutils implementation.
    from mathutils import Matrix, Quaternion, Vector
    checked = inspect(source, project)
    sources = Sources(source)
    project = Path(project)
    request = json.loads((project / 'Reference/request.json').read_text())
    report = json.loads((project / 'Reference/imported.json').read_text())
    scene_name = request['scene']
    scene = records(sources.checked(scene_name))
    documents = {}
    prepared = {}
    scene_frames = {0: (Matrix.Identity(4), True)}

    def local(fields):
        p, q, s = (fields[k] for k in ['m_LocalPosition', 'm_LocalRotation', 'm_LocalScale'])
        return Matrix.LocRotScale(Vector(tuple(p[a] for a in 'xyz')),
                                  Quaternion(tuple(q[a] for a in 'wxyz')).normalized(),
                                  Vector(tuple(s[a] for a in 'xyz')))

    def prepare(instance):
        if instance in prepared:
            return prepared[instance]
        fields = scene[instance]['fields']
        guid = fields['m_SourcePrefab']['guid']
        name = sources.by_guid[guid]
        if name not in documents:
            sources.metadata(name)
            documents[name] = records(sources.checked(name))
        raw = documents[name]
        modification = fields['m_Modification']
        if any(v for k, v in modification.items() if k.startswith(('m_Removed', 'm_Added'))):
            structural = {k: v for k, v in modification.items() if k.startswith(('m_Removed', 'm_Added')) and v}
            raise ValueError(f'Collision-bearing/parent prefab needs separate nested hierarchy qualification: {instance} {name} {structural}')
        if any(r['kind'] == 1001 or r['stripped'] for r in raw.values()):
            raw = expand_variant(raw, modification['m_Modifications'], guid, sources)
        rr = overrides(raw, modification['m_Modifications'], guid)
        by_go = {r['fields']['m_GameObject']['fileID']: i for i, r in rr.items() if r['kind'] == 4}
        frames = {0: (Matrix.Identity(4), True)}

        def frame(identifier, pending=()):
            if identifier in frames:
                return frames[identifier]
            if identifier in pending or len(pending) > 128:
                raise ValueError('Prefab collider hierarchy cycle/depth')
            f = rr[identifier]['fields']
            parent, active = frame(f['m_Father']['fileID'], pending + (identifier,))
            flag = rr[f['m_GameObject']['fileID']]['fields']['m_IsActive']
            if flag not in [0, 1]:
                raise ValueError('Invalid source active flag')
            frames[identifier] = parent @ local(f), active and bool(flag)
            return frames[identifier]
        result = name, guid, rr, by_go, frame
        prepared[instance] = result
        return result

    def scene_frame(identifier, pending=()):
        if identifier in scene_frames:
            return scene_frames[identifier]
        if identifier in pending or len(pending) > 128:
            raise ValueError('Scene collider hierarchy cycle/depth')
        row = scene[identifier]
        fields = row['fields']
        if row['stripped']:
            instance = fields['m_PrefabInstance']['fileID']
            _, guid, _, _, frame = prepare(instance)
            ref = fields['m_CorrespondingSourceObject']
            if ref['guid'] != guid:
                raise ValueError('Stripped collider parent identity differs')
            outer = scene[instance]['fields']['m_Modification']['m_TransformParent']['fileID']
            parent, enabled = scene_frame(outer, pending + (identifier,))
            matrix, active = frame(ref['fileID'])
            result = parent @ matrix, enabled and active
        else:
            parent, enabled = scene_frame(fields['m_Father']['fileID'], pending + (identifier,))
            result = parent @ local(fields), enabled and bool(scene[fields['m_GameObject']['fileID']]['fields']['m_IsActive'])
        scene_frames[identifier] = result
        return result

    expected = {}
    for instance, row in scene.items():
        if row['kind'] != 1001:
            continue
        name = sources.by_guid[row['fields']['m_SourcePrefab']['guid']]
        if name not in documents:
            sources.metadata(name)
            documents[name] = records(sources.checked(name))
        if not any(r['kind'] in KINDS for r in documents[name].values()):
            # No inherited model colliders are silently admitted. The existing
            # visual variant contract supplies importer addCollider=0 below.
            for nested in documents[name].values():
                if nested['kind'] == 1001:
                    path = sources.by_guid[nested['fields']['m_SourcePrefab']['guid']]
                    if not path.lower().endswith('.fbx') or sources.metadata(path)['ModelImporter']['meshes'].get('addColliders') != 0:
                        raise ValueError('Unqualified inherited model collision')
            continue
        name, guid, rr, by_go, frame = prepare(instance)
        parent, enabled = scene_frame(row['fields']['m_Modification']['m_TransformParent']['fileID'])
        for identifier, record in rr.items():
            if record['kind'] not in KINDS:
                continue
            fields = record['fields']
            transform, active = frame(by_go[fields['m_GameObject']['fileID']])
            expected[(instance, identifier)] = record['kind'], fields, parent @ transform, enabled and active, rr[fields['m_GameObject']['fileID']]['fields'], guid
    by_go = {r['fields']['m_GameObject']['fileID']: i for i, r in scene.items() if r['kind'] == 4 and not r['stripped']}
    for identifier, row in scene.items():
        if row['kind'] in KINDS and not row['stripped']:
            fields = row['fields']
            frame, active = scene_frame(by_go[fields['m_GameObject']['fileID']])
            expected[(0, identifier)] = row['kind'], fields, frame, active, scene[fields['m_GameObject']['fileID']]['fields'], None
    observed = {}
    maximum = 0.
    active_counts = collections.Counter()
    for shape in report['colliders']:
        identifier = int(shape['object_id'])
        if identifier >= 1 << 63:
            identifier -= 1 << 64
        key = int(shape['prefab_id']), identifier
        if key in observed or key not in expected:
            raise ValueError('Unexpected/duplicate actual source collider identity')
        observed[key] = shape
        kind, fields, frame, active, go, guid = expected[key]
        if shape['kind'] != KINDS[kind] or shape['active'] != active or shape['enabled'] != bool(fields['m_Enabled']) or shape['trigger'] != bool(fields.get('m_IsTrigger', 0)) or shape['layer'] != go['m_Layer'] or shape['name'] != go['m_Name']:
            raise ValueError('Actual collider flags/ownership/layer/name differ from source')
        if guid and shape['source'] != {'guid': guid, 'file_id': str(identifier)}:
            raise ValueError('Actual source collider persistent identity differs')
        difference = float(abs(np.asarray(frame, dtype=float) - np.array(shape['world_matrix']).reshape(4, 4)).max())
        maximum = max(maximum, difference)
        if difference > .0001:
            raise ValueError('Actual collider world matrix differs from raw hierarchy')
        if active and fields['m_Enabled']:
            active_counts[shape['kind']] += 1
        if kind == 64:
            mesh = fields['m_Mesh']
            if shape['mesh'] != {'guid': mesh['guid'], 'file_id': str(mesh['fileID'])} or shape['convex'] != bool(fields['m_Convex']) or shape['cooking_options'] != fields.get('m_CookingOptions', 30):
                raise ValueError('Actual source mesh collider binding/cooking differs')
        elif kind in {65, 136}:
            for name in ['center'] + (['size'] if kind == 65 else []):
                if not np.allclose([shape[name][a] for a in 'xyz'], [fields['m_' + name.title()][a] for a in 'xyz'], atol=.000001, rtol=0):
                    raise ValueError('Actual collider primitive dimensions differ')
            if kind == 136 and (abs(shape['radius'] - fields['m_Radius']) > .000001 or abs(shape['height'] - fields['m_Height']) > .000001 or shape['capsule_direction'] != fields['m_Direction']):
                raise ValueError('Actual source capsule dimensions differ')
        elif kind == 154:
            ref = fields['m_TerrainData']
            if shape['terrain'] != {'guid': ref['guid'], 'file_id': str(ref['fileID'])} or shape['terrain_resolution'] != 513:
                raise ValueError('Actual source TerrainCollider differs')
    if set(observed) != set(expected) or dict(active_counts) != checked['active_enabled_colliders']:
        raise ValueError('Actual/raw source collider coverage differs')
    return dict(checked, independently_verified_colliders=len(expected),
                max_source_matrix_difference_metres=maximum,
                scope='Complete actual/raw source collider identities, active states, overrides, world matrices and primitive/mesh bindings; native probe data retained; no Avian/cooking/runtime admission parity')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('source', type=Path)
    parser.add_argument('project', type=Path)
    parser.add_argument('--report', type=Path, required=True)
    args = parser.parse_args()
    if args.report.exists():
        raise FileExistsError('Use a fresh report')
    result = verify(args.source, args.project)
    args.report.write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps(result))
