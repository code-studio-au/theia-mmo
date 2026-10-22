"""Independently verify the scene's one-mesh model variants and emitted poses.

No variant compiler or source planner is imported. Completeness is checked for
this model-variant category only. Geometry/importer axes remain unqualified.
"""
import argparse
import hashlib
import json
import math
from pathlib import Path
from unity_scene import Sources, records
from unity_model_ids import identity, bound_candidates


def verify(source, plan_path):
    import bpy  # Registers the isolated wheel's mathutils module.
    from mathutils import Matrix, Vector, Quaternion
    sources=Sources(source);plan=json.loads(Path(plan_path).read_text())
    if plan['archive_sha256']!=sources.manifest['archive_sha256']:raise ValueError('Plan source archive differs')
    scene=records(sources.checked(plan['scene']));sources.metadata(plan['scene'])
    definitions={};documents={};world={0:Matrix.Identity(4)};active={0:True};visiting=set()
    prefab_world={};prefab_active={}
    def matrix(fields):
        p=[float(fields['m_LocalPosition'][k]) for k in 'xyz']
        q=[float(fields['m_LocalRotation'][k]) for k in 'xyzw']
        scale=[float(fields['m_LocalScale'][k]) for k in 'xyz']
        if not all(math.isfinite(v) for v in p+q+scale) or abs(sum(v*v for v in q)-1.)>.0021:
            raise ValueError('Invalid authored model pose')
        return Matrix.LocRotScale(Vector(p),Quaternion((q[3],*q[:3])).normalized(),Vector(scale))
    def parent_matrix(identifier):
        if identifier in world:return world[identifier]
        if identifier in visiting or len(visiting)>=128:raise ValueError('Reference scene-parent cycle/depth')
        visiting.add(identifier)
        record=scene[identifier];f=record['fields']
        if record['kind']!=4:raise ValueError('Invalid scene parent kind')
        if record['stripped']:
            instance=f['m_PrefabInstance']['fileID'];source_ref=f['m_CorrespondingSourceObject']
            if source_ref['guid']!=scene[instance]['fields']['m_SourcePrefab']['guid']:raise ValueError('Stripped parent source differs')
            parent=scene[instance]['fields']['m_Modification']['m_TransformParent']['fileID']
            value=parent_matrix(parent)@prefab_matrix(instance,source_ref['fileID'])
            active[identifier]=active[parent] and prefab_active[(instance,source_ref['fileID'])]
        else:
            parent=f['m_Father']['fileID'];value=parent_matrix(parent)@matrix(f)
            go=scene[f['m_GameObject']['fileID']]['fields']
            active[identifier]=active[parent] and bool(go['m_IsActive'])
        world[identifier]=value;visiting.remove(identifier);return value
    def definition(path):
        if path in definitions:return definitions[path]
        rr=records(sources.checked(path));sources.metadata(path);documents[path]=rr
        nested=[(i,v['fields']) for i,v in rr.items() if v['kind']==1001]
        if not nested:definitions[path]=None;return None
        if len(nested)!=1:raise ValueError('Nested variants outside reference category')
        nested_id,fields=nested[0];ref=fields['m_SourcePrefab'];model_path=sources.by_guid.get(ref['guid'])
        if not model_path or not model_path.endswith('.fbx'):raise ValueError('Non-model nested source needs separate reference')
        model=identity(sources.checked(model_path));importer=sources.metadata(model_path)['ModelImporter']
        if len(model['models'])!=1 or len(model['meshes'])!=1 or importer['importAnimation'] or importer['meshes']['importVisibility']:
            raise ValueError('Unqualified model profile in reference')
        candidates,_=bound_candidates(model,importer)
        by_kind={}
        for kind in ('GameObject','Transform','MeshRenderer','Mesh'):
            values=[i for i,v in candidates.items() if v['kind']==kind]
            if len(values)!=1:raise ValueError('Ambiguous model object kind')
            by_kind[kind]=values[0]
        aliases={k:(identifier^nested_id)&((1<<63)-1) for k,identifier in by_kind.items() if k!='Mesh'}
        materials=[m['objectReference'] for m in fields['m_Modification']['m_Modifications']
            if m['target'].get('fileID')==by_kind['MeshRenderer'] and m['target'].get('guid')==ref['guid']
            and m['propertyPath']=='m_Materials.Array.data[0]']
        if len(materials)!=1:raise ValueError('Reference model requires one explicit source material')
        result={'aliases':aliases,'mesh':{'fileID':by_kind['Mesh'],'guid':ref['guid'],'type':3},
                'material':materials[0],'path':path};definitions[path]=result;return result
    def prefab_matrix(instance,source_id):
        key=(instance,source_id)
        if key in prefab_world:return prefab_world[key]
        if key in visiting or len(visiting)>=128:raise ValueError('Reference prefab parent cycle/depth')
        visiting.add(key)
        f=scene[instance]['fields'];guid=f['m_SourcePrefab']['guid'];path=sources.by_guid[guid]
        data=definition(path);rr=documents[path]
        mods=[m for m in f['m_Modification']['m_Modifications'] if m['target'].get('guid')==guid]
        if data is not None:
            if source_id!=data['aliases']['Transform']:raise ValueError('Unknown inherited model transform')
            values={m['propertyPath']:m['value'] for m in mods if m['target']['fileID']==source_id}
            fields={k:{a:float(values[k+'.'+a]) for a in axes} for k,axes in
                [('m_LocalPosition','xyz'),('m_LocalRotation','xyzw'),('m_LocalScale','xyz')]}
            value=matrix(fields);owner=data['aliases']['GameObject'];own_enabled=True;parent_enabled=True
        else:
            record=rr[source_id];fields=record['fields']
            if record['kind']!=4 or record['stripped']:raise ValueError('Unqualified nested parent source')
            fields={**fields,**{k:dict(fields[k]) for k in ['m_LocalPosition','m_LocalRotation','m_LocalScale']}}
            for m in mods:
                if m['target']['fileID']!=source_id:continue
                parts=m['propertyPath'].split('.')
                if len(parts)==2 and parts[0] in ('m_LocalPosition','m_LocalRotation','m_LocalScale'):
                    fields[parts[0]][parts[1]]=float(m['value'])
            parent=fields['m_Father']['fileID'];value=matrix(fields)
            if parent:value=prefab_matrix(instance,parent)@value
            owner=fields['m_GameObject']['fileID']
            own_enabled=bool(rr[owner]['fields']['m_IsActive']);parent_enabled=prefab_active[(instance,parent)] if parent else True
        for m in mods:
            if m['target']['fileID']==owner and m['propertyPath']=='m_IsActive':own_enabled=bool(int(m['value']))
        prefab_world[key]=value;prefab_active[key]=own_enabled and parent_enabled;visiting.remove(key);return value
    expected={}
    for instance,record in scene.items():
        if record['kind']!=1001:continue
        f=record['fields'];ref=f['m_SourcePrefab'];path=sources.by_guid.get(ref['guid'])
        if not path or not path.endswith('.prefab'):continue
        data=definition(path)
        if data is None:continue
        t=data['aliases']['Transform'];g=data['aliases']['GameObject'];r=data['aliases']['MeshRenderer']
        values={};enabled=True;renderer_enabled=True;material=data['material']
        for m in f['m_Modification']['m_Modifications']:
            target=m['target']
            if target.get('guid')!=ref['guid']:raise ValueError('Foreign model override source')
            if target['fileID']==t:
                if m['propertyPath'] in values:raise ValueError('Duplicate model override in reference')
                values[m['propertyPath']]=m['value']
            if target['fileID']==g and m['propertyPath']=='m_IsActive':enabled=bool(int(m['value']))
            if target['fileID']==r and m['propertyPath']=='m_Enabled':renderer_enabled=bool(int(m['value']))
            if target['fileID']==r and m['propertyPath']=='m_Materials.Array.data[0]':material=m['objectReference']
        fields={key:{axis:float(values[key+'.'+axis]) for axis in axes} for key,axes in
            [('m_LocalPosition','xyz'),('m_LocalRotation','xyzw'),('m_LocalScale','xyz')]}
        parent=f['m_Modification']['m_TransformParent']['fileID'];pose=parent_matrix(parent)@matrix(fields)
        if enabled and renderer_enabled and active[parent]:
            expected[(instance,r)]={'matrix':pose,'data':data,'material':material}
    observed={}
    paths={p for p,v in definitions.items() if v is not None}
    for draw in plan['draws']:
        if draw['source'] not in paths:continue
        key=(draw['instance'],draw['renderer'])
        if key in observed:raise ValueError('Duplicate model-variant draw')
        observed[key]=draw
    if set(observed)!=set(expected):raise ValueError('Missing or extra model-variant placements')
    maximum=0.
    for key,reference in expected.items():
        draw=observed[key];data=reference['data']
        if draw['source']!=data['path'] or draw['transform']!=data['aliases']['Transform'] or draw['mesh']!=data['mesh'] or draw['materials']!=[reference['material']]:
            raise ValueError('Model source mesh/material/transform identity differs')
        actual=draw['matrix']
        if len(actual)!=4 or any(len(row)!=4 or any(not math.isfinite(v) for v in row) for row in actual):raise ValueError('Invalid model matrix')
        error=max(abs(actual[i][j]-reference['matrix'][i][j]) for i in range(4) for j in range(4))
        maximum=max(maximum,error)
        if error>.0001:raise ValueError('Model pose differs from independent reference')
    return {'passed':True,'model_variants':len(paths),'model_placements':len(expected),
            'max_absolute_matrix_difference':maximum,'matrix_tolerance':.0001,
            'scope':'all category placements, source identities and explicit source-coordinate TRS; no FBX geometry/axes, collision or full scene equivalence'}


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('source',type=Path);parser.add_argument('plan',type=Path);parser.add_argument('--report',required=True,type=Path)
    args=parser.parse_args()
    if args.report.exists():raise FileExistsError('Use a new reference report')
    result=verify(args.source,args.plan);args.report.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result))
