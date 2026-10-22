"""Check emitted static source placements against independent mathutils matrices.

Requires the isolated Blender Python environment. No planner implementation is
imported. This checks emitted placements; it does not prove absence of omissions,
Unity Editor equivalence, FBX axes, geometry or effects.
"""
import argparse
import json
import math
from pathlib import Path
from unity_scene import Sources, records


def verify(source, plan_path):
    import bpy  # Registers the bundled mathutils module in the isolated wheel.
    from mathutils import Matrix, Vector, Quaternion
    sources=Sources(source);plan=json.loads(Path(plan_path).read_text())
    if plan['archive_sha256']!=sources.manifest['archive_sha256']:
        raise ValueError('Instance plan archive differs from source')
    scene=records(sources.checked(plan['scene']));sources.metadata(plan['scene'])
    docs={};edits={};prefab_matrices={};scene_matrices={};prefab_enabled={};scene_enabled={0:True};visiting=set()
    def data(instance):
        fields=scene[instance]['fields'];ref=fields['m_SourcePrefab'];path=sources.by_guid[ref['guid']]
        if path not in docs:docs[path]=records(sources.checked(path));sources.metadata(path)
        if instance not in edits:
            edits[instance]={(m['target']['fileID'],m['propertyPath']):m
                for m in fields['m_Modification']['m_Modifications']
                if m['target'].get('guid')==ref['guid'] and m['target']['fileID'] in docs[path]}
        return path,docs[path]
    def field(instance,identifier,path,original):
        if instance is None:return original
        data(instance);mod=edits[instance].get((identifier,path))
        return mod['value'] if mod else original
    def trs(instance,identifier,fields):
        def vector(key,axes):
            return [float(field(instance,identifier,key+'.'+a,fields[key][a])) for a in axes]
        position=vector('m_LocalPosition','xyz');scale=vector('m_LocalScale','xyz')
        x,y,z,w=vector('m_LocalRotation','xyzw')
        return Matrix.LocRotScale(Vector(position),Quaternion((w,x,y,z)).normalized(),Vector(scale))
    def prefab_matrix(instance,identifier):
        key=(instance,identifier)
        if key in prefab_matrices:return prefab_matrices[key]
        if key in visiting or len(visiting)>128:raise ValueError('Reference hierarchy cycle/depth')
        visiting.add(key)
        _,rr=data(instance);f=rr[identifier]['fields'];parent=f['m_Father']['fileID']
        matrix=trs(instance,identifier,f)
        if parent:matrix=prefab_matrix(instance,parent)@matrix
        owner=f['m_GameObject']['fileID']
        prefab_enabled[key]=bool(int(field(instance,owner,'m_IsActive',rr[owner]['fields']['m_IsActive']))) and (prefab_enabled[(instance,parent)] if parent else True)
        prefab_matrices[key]=matrix;visiting.remove(key)
        return matrix
    def scene_matrix(identifier):
        if not identifier:return Matrix.Identity(4)
        if identifier in scene_matrices:return scene_matrices[identifier]
        key=('scene',identifier)
        if key in visiting or len(visiting)>128:raise ValueError('Reference scene cycle/depth')
        visiting.add(key);v=scene[identifier];f=v['fields']
        if v['stripped']:
            instance=f['m_PrefabInstance']['fileID'];data(instance)
            if f['m_CorrespondingSourceObject']['guid']!=scene[instance]['fields']['m_SourcePrefab']['guid']:
                raise ValueError('Stripped reference source mismatch')
            parent=scene[instance]['fields']['m_Modification']['m_TransformParent']['fileID']
            source_id=f['m_CorrespondingSourceObject']['fileID']
            matrix=scene_matrix(parent)@prefab_matrix(instance,source_id)
            scene_enabled[identifier]=scene_enabled[parent] and prefab_enabled[(instance,source_id)]
        else:
            parent=f['m_Father']['fileID']
            matrix=scene_matrix(parent)@trs(None,identifier,f)
            scene_enabled[identifier]=scene_enabled[parent] and bool(scene[f['m_GameObject']['fileID']]['fields']['m_IsActive'])
        scene_matrices[identifier]=matrix;visiting.remove(key)
        return matrix
    maximum=0.;keys=set()
    for draw in plan['draws']:
        instance=draw['instance'];identifier=draw['renderer'];key=(instance,identifier)
        if key in keys:raise ValueError('Duplicate emitted renderer instance')
        keys.add(key)
        path,rr=data(instance) if instance is not None else (plan['scene'],scene)
        if path!=draw['source']:raise ValueError('Draw source differs from bound instance')
        renderer=rr[identifier]['fields'];go=renderer['m_GameObject']['fileID'];game=rr[go]['fields']
        component_ids=[r['component']['fileID'] for r in game['m_Component']]
        transform,=[i for i in component_ids if rr[i]['kind']==4]
        mesh,=[rr[i]['fields']['m_Mesh'] for i in component_ids if rr[i]['kind']==33]
        if transform!=draw['transform'] or mesh!=draw['mesh']:raise ValueError('Draw mesh/transform binding differs from source')
        if not int(field(instance,identifier,'m_Enabled',renderer['m_Enabled'])):
            raise ValueError('Disabled renderer emitted')
        ancestor=transform
        while ancestor:
            f=rr[ancestor]['fields'];owner=f['m_GameObject']['fileID']
            if not int(field(instance,owner,'m_IsActive',rr[owner]['fields']['m_IsActive'])):
                raise ValueError('Inactive prefab hierarchy emitted')
            ancestor=f['m_Father']['fileID']
        materials=list(renderer['m_Materials'])
        if instance is not None:
            for index in range(len(materials)):
                mod=edits[instance].get((identifier,f'm_Materials.Array.data[{index}]'))
                if mod:materials[index]=mod['objectReference']
        if materials!=draw['materials']:raise ValueError('Draw material slots differ from source')
        for v in rr.values():
            if v['kind']!=205:continue
            levels=v['fields']['m_LODs']
            membership=[{r['renderer']['fileID'] for r in level['renderers']} for level in levels]
            if any(identifier in ids for ids in membership) and identifier not in membership[0]:
                raise ValueError('Non-highest-detail LOD renderer emitted')
        if instance is None:expected=scene_matrix(transform)
        else:
            parent=scene[instance]['fields']['m_Modification']['m_TransformParent']['fileID']
            expected=scene_matrix(parent)@prefab_matrix(instance,transform)
        if (instance is None and not scene_enabled[transform]) or (instance is not None and not scene_enabled[parent]):
            raise ValueError('Inactive scene parent emitted')
        actual=draw['matrix']
        if len(actual)!=4 or any(len(row)!=4 or any(not math.isfinite(v) for v in row) for row in actual):
            raise ValueError('Invalid emitted matrix')
        difference=max(abs(float(expected[r][c])-actual[r][c]) for r in range(4) for c in range(4))
        maximum=max(maximum,difference)
        # mathutils uses f32; the planner retains f64 source math.
        if difference>.0001:raise ValueError(f'Source transform differs from reference: {difference}; {key}')
    return {'passed':True,'draws_checked':len(keys),'max_absolute_matrix_difference':maximum,
            'matrix_tolerance':.0001,'reference':'independent mathutils LocRotScale/parent multiplication',
            'scope':'emitted source placements/LOD/active/material bindings; not omission proof, Unity Editor, FBX axes or geometry'}


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('source',type=Path);parser.add_argument('plan',type=Path)
    parser.add_argument('--report',required=True,type=Path)
    args=parser.parse_args()
    if args.report.exists():raise FileExistsError('Reference report must be new')
    result=verify(args.source,args.plan)
    args.report.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result))
