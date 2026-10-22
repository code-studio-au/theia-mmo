"""Resolve bounded static Unity prefab hierarchy/overrides to source instance plans.

Matrices remain in Unity source coordinates. This does not export geometry,
convert FBX coordinates, implement effects, or establish scene equivalence.
"""
import argparse
from collections import Counter
import copy
import json
import math
from pathlib import Path
import re

import numpy as np
from unity_scene import Sources, records
from unity_prefab_ids import compile_single_model_variant

MAX_DEPTH = 128
MAX_DRAWS = 262144


def local_matrix(fields):
    p = [float(fields['m_LocalPosition'][k]) for k in 'xyz']
    q = [float(fields['m_LocalRotation'][k]) for k in 'xyzw']
    scale = [float(fields['m_LocalScale'][k]) for k in 'xyz']
    if not all(math.isfinite(v) and abs(v) <= 1e12 for v in p+q+scale):
        raise ValueError('Invalid source TRS scalar')
    norm = math.sqrt(sum(v*v for v in q))
    if abs(norm-1.) > .001:
        raise ValueError('Invalid source rotation quaternion')
    x,y,z,w = [v/norm for v in q]
    result = np.eye(4, dtype=np.float64)
    result[:3,:3] = np.array([[1-2*(y*y+z*z),2*(x*y-z*w),2*(x*z+y*w)],
                              [2*(x*y+z*w),1-2*(x*x+z*z),2*(y*z-x*w)],
                              [2*(x*z-y*w),2*(y*z+x*w),1-2*(x*x+y*y)]]) @ np.diag(scale)
    result[:3,3] = p
    return result


def apply_overrides(rr, modifications, guid):
    """Only explicitly supported source properties can alter a static plan."""
    result = dict(rr)
    changed = set();unused=[]
    for modification in modifications:
        target = modification['target'];identifier = target['fileID']
        if target.get('guid') != guid:
            raise ValueError('Override target is not a bound source object')
        if identifier not in rr:
            # Only called after ruling out nested/stripped/model inheritance.
            # Unity retains invalid-target overrides as unused serialized data.
            unused.append({'target':dict(target),'propertyPath':modification['propertyPath']})
            continue
        if identifier not in changed:
            result[identifier] = copy.deepcopy(rr[identifier]);changed.add(identifier)
        record = result[identifier];f = record['fields'];path = modification['propertyPath']
        kind = record['kind'];value = modification['value'];reference = modification['objectReference']
        vector = re.fullmatch(r'(m_LocalPosition|m_LocalScale|m_LocalRotation|m_LocalEulerAnglesHint)\.([xyzw])',path)
        material = re.fullmatch(r'm_Materials\.Array\.data\[(\d+)\]',path)
        if vector and kind == 4:
            field,axis = vector.groups()
            if axis not in f.get(field,{}):raise ValueError('Invalid transform override axis')
            if reference.get('fileID'):raise ValueError('Unexpected transform object override')
            number = float(value)
            if not math.isfinite(number):raise ValueError('Nonfinite transform override')
            f[field][axis] = number
        elif path in ('m_Name','m_Layer','m_IsActive') and kind == 1:
            if reference.get('fileID'):raise ValueError('Unexpected GameObject object override')
            if path == 'm_Name':f[path] = str(value)
            else:
                number = int(value)
                if str(number) != str(value) or (path=='m_IsActive' and number not in (0,1)):
                    raise ValueError('Invalid GameObject flag override')
                f[path] = number
        elif path in ('m_Enabled','m_CastShadows') and kind in (23,64,65,205):
            if path not in f or reference.get('fileID'):raise ValueError('Invalid component flag override')
            number = int(value)
            if str(number) != str(value) or number not in (0,1):raise ValueError('Invalid component flag')
            f[path] = number
        elif material and kind == 23:
            index = int(material[1])
            if index >= len(f['m_Materials']) or not reference.get('fileID') or not reference.get('guid'):
                raise ValueError('Invalid renderer material override')
            f['m_Materials'][index] = dict(reference)
        elif path == 'm_RootOrder' and kind == 4:
            # Source sibling ordering has no effect on static world transforms.
            if reference.get('fileID') or int(value) < 0:raise ValueError('Invalid root order')
        else:
            raise ValueError(f'Unsupported override {record["name"]}.{path}')
    return result,unused


def static_prefab(rr, modifications=(), guid=None, lod_level=0):
    if type(lod_level) is not int or lod_level<0:raise ValueError("Invalid selected LOD level")
    if len(rr) > 131072 or any(v['stripped'] or v['kind']==1001 for v in rr.values()):
        raise ValueError('Nested/model prefabs require instantiated model bindings')
    rr,unused = apply_overrides(rr, modifications, guid)
    gos = {i:v['fields'] for i,v in rr.items() if v['kind']==1}
    transforms = {i:v['fields'] for i,v in rr.items() if v['kind']==4}
    by_go = {}
    for i,f in transforms.items():
        go = f['m_GameObject']['fileID']
        if go not in gos or go in by_go:raise ValueError('Missing/duplicate GameObject transform')
        by_go[go] = i
    if set(by_go) != set(gos):raise ValueError('GameObject missing transform')
    roots = [i for i,f in transforms.items() if not f['m_Father']['fileID']]
    if len(roots) != 1:raise ValueError('Prefab must have one root transform')
    for i,f in transforms.items():
        children = [r['fileID'] for r in f['m_Children']]
        expected = {k for k,v in transforms.items() if v['m_Father']['fileID']==i}
        if len(children)!=len(set(children)) or set(children)!=expected:
            raise ValueError('Inconsistent source parent/child links')
        parent = f['m_Father']['fileID']
        if parent and parent not in transforms:raise ValueError('Unresolved source transform parent')
    matrices = {};active = {};visiting = set()
    def visit(i):
        if i in matrices:return
        if i in visiting or len(visiting)>=MAX_DEPTH:raise ValueError('Cyclic/excessive source hierarchy')
        visiting.add(i);f = transforms[i];parent = f['m_Father']['fileID']
        if parent:visit(parent)
        matrix = (matrices[parent] if parent else np.eye(4)) @ local_matrix(f)
        if not np.isfinite(matrix).all() or np.abs(matrix).max()>1e12:raise ValueError('Source world matrix exceeds bounds')
        flag = gos[f['m_GameObject']['fileID']]['m_IsActive']
        if flag not in (0,1):raise ValueError('Invalid source active flag')
        matrices[i] = matrix;active[i] = bool(flag) and (active[parent] if parent else True)
        visiting.remove(i)
    for i in transforms:visit(i)
    for identifier,v in rr.items():
        owner=v['fields'].get('m_GameObject',{}).get('fileID')
        if owner:
            if owner not in gos or identifier not in {c['component']['fileID'] for c in gos[owner]['m_Component']}:
                raise ValueError('Component absent from owning GameObject')
    filters = {}
    renderers = {}
    for i,v in rr.items():
        if v['kind'] not in (23,33):continue
        f = v['fields'];go = f['m_GameObject']['fileID']
        if go not in by_go:raise ValueError('Orphan render component')
        if v['kind']==33:
            if go in filters:raise ValueError('Duplicate source mesh filter')
            filters[go] = f
        else:renderers[i] = f
    excluded = set();members = set();groups = 0
    for v in rr.values():
        if v['kind']!=205:continue
        f = v['fields'];group_go = f['m_GameObject']['fileID']
        if group_go not in by_go or f.get('m_Enabled')!=1 or not f['m_LODs']:
            raise ValueError('Unsupported disabled/empty LOD group')
        if lod_level>=len(f['m_LODs']):raise ValueError('Selected LOD level is absent')
        groups += 1;seen = set();selected = set()
        for level,lod in enumerate(f['m_LODs']):
            for ref in lod['renderers']:
                i = ref['renderer']['fileID']
                if i not in renderers:raise ValueError('Unresolved LOD renderer')
                transform = by_go[renderers[i]['m_GameObject']['fileID']]
                parent = transform
                while parent and parent != by_go[group_go]:parent=transforms[parent]['m_Father']['fileID']
                if not parent:raise ValueError('LOD renderer outside group hierarchy')
                seen.add(i)
                if level==lod_level:selected.add(i)
        if members & seen:raise ValueError('Renderer belongs to multiple LOD groups')
        members |= seen;excluded |= seen-selected
    draws = []
    for i,f in renderers.items():
        go = f['m_GameObject']['fileID'];transform=by_go[go]
        if f['m_Enabled'] not in (0,1):raise ValueError('Invalid source renderer enabled flag')
        if not f['m_Enabled'] or not active[transform] or i in excluded:continue
        if go not in filters:raise ValueError('Renderer missing source mesh filter')
        mesh = filters[go]['m_Mesh'];materials=f['m_Materials']
        if not mesh.get('fileID') or not mesh.get('guid') or not materials or any(not m.get('fileID') or not m.get('guid') for m in materials):
            raise ValueError('Unresolved source mesh/material reference')
        draws.append({'renderer':i,'transform':transform,'name':gos[go]['m_Name'],
                      'mesh':dict(mesh),'materials':copy.deepcopy(materials),
                      'matrix':matrices[transform].tolist(),'cast_shadows':f.get('m_CastShadows',1)})
    ignored = Counter(str(v['kind']) for v in rr.values() if v['kind'] not in (1,4,23,33,64,65,205))
    return {'root':roots[0], 'matrices':matrices, 'active':active,
            'draws':draws,'lod_groups':groups,'excluded_lod_renderers':len(excluded),
            'unimplemented_components':dict(ignored),'unused_target_overrides':unused}


def scene_plan(sources, scene):
    rr = records(sources.checked(scene));sources.metadata(scene)
    if not scene.endswith('.unity'):raise ValueError('Select an explicit Unity scene')
    instances = {i:v['fields'] for i,v in rr.items() if v['kind']==1001}
    transforms = {i:v for i,v in rr.items() if v['kind']==4}
    gos = {i:v['fields'] for i,v in rr.items() if v['kind']==1}
    documents = {};prepared = {};errors = {};world = {};active = {};visiting = set()
    def prepare(i):
        if i in errors:raise ValueError(errors[i])
        if i in prepared:return prepared[i]
        f = instances[i];ref=f['m_SourcePrefab'];path=sources.by_guid.get(ref['guid'])
        if path is None or not path.endswith('.prefab'):raise ValueError('Instance source is not a bound prefab')
        changes = f['m_Modification']
        if any(value for key,value in changes.items() if key.startswith(('m_Removed','m_Added'))):
            raise ValueError('Structural prefab overrides require explicit instantiation')
        if path not in documents:
            documents[path] = records(sources.checked(path));sources.metadata(path)
        document=documents[path]
        if any(v['kind']==1001 for v in document.values()):
            document=compile_single_model_variant(sources,path,changes.get('m_Modifications',[]),ref['guid'])
        value = static_prefab(document,changes.get('m_Modifications',[]),ref['guid'])
        value['source'] = path;prepared[i] = value
        return value
    def parent_world(i, depth=0):
        if not i:return np.eye(4),True
        if i in world:return world[i],active[i]
        if i in visiting or depth>=MAX_DEPTH:raise ValueError('Cyclic/excessive scene hierarchy')
        if i not in transforms:raise ValueError('Unresolved scene transform')
        visiting.add(i)
        try:
            v=transforms[i];f=v['fields']
            if v['stripped']:
                instance=f['m_PrefabInstance']['fileID'];value=prepare(instance)
                source=f['m_CorrespondingSourceObject'];expected=instances[instance]['m_SourcePrefab']['guid']
                if source.get('guid')!=expected or source['fileID'] not in value['matrices']:
                    raise ValueError('Stripped transform lacks bound source object')
                parent,enabled=parent_world(instances[instance]['m_Modification']['m_TransformParent']['fileID'],depth+1)
                matrix=parent@value['matrices'][source['fileID']];enabled &= value['active'][source['fileID']]
            else:
                parent,enabled=parent_world(f['m_Father']['fileID'],depth+1)
                matrix=parent@local_matrix(f)
                go=gos.get(f['m_GameObject']['fileID'])
                if go is None:raise ValueError('Scene transform missing GameObject')
                if go['m_IsActive'] not in (0,1):raise ValueError('Invalid scene active flag')
                enabled &= bool(go['m_IsActive'])
            if not np.isfinite(matrix).all() or np.abs(matrix).max()>1e12:raise ValueError('Scene matrix exceeds bounds')
            world[i]=matrix;active[i]=enabled
            return matrix,enabled
        finally:visiting.remove(i)
    draws=[];counts=Counter();unimplemented=Counter();unused=[]
    for i,f in instances.items():
        first_draw=len(draws)
        try:
            value=prepare(i)
            parent,enabled=parent_world(f['m_Modification']['m_TransformParent']['fileID'])
            for draw in value['draws']:
                if not enabled:continue
                matrix=parent@np.array(draw['matrix'])
                if not np.isfinite(matrix).all() or np.abs(matrix).max()>1e12:raise ValueError('Draw matrix exceeds bounds')
                draws.append(dict(draw,instance=i,source=value['source'],matrix=matrix.tolist()))
                if len(draws)>MAX_DRAWS:raise ValueError('Expanded draw budget exceeded')
            counts['resolved_instances']+=1;counts['lod_groups']+=value['lod_groups']
            counts['excluded_lod_renderers']+=value['excluded_lod_renderers']
            unimplemented.update(value['unimplemented_components'])
            unused.extend(dict(item,instance=i,source=value['source']) for item in value['unused_target_overrides'])
        except (ValueError,KeyError,TypeError) as error:
            del draws[first_draw:]
            errors[i]=str(error);counts['unresolved_instances']+=1
    # Explicit inline static scene meshes remain separate from prefab placements.
    by_go={v['fields']['m_GameObject']['fileID']:i for i,v in transforms.items() if not v['stripped']}
    filters={v['fields']['m_GameObject']['fileID']:v['fields'] for v in rr.values() if v['kind']==33 and not v['stripped']}
    inline_errors=[]
    for i,v in rr.items():
        if v['kind']!=23 or v['stripped']:continue
        f=v['fields'];go=f['m_GameObject']['fileID']
        try:
            matrix,enabled=parent_world(by_go[go])
            if enabled and f['m_Enabled']:
                draws.append({'renderer':i,'transform':by_go[go],'name':gos[go]['m_Name'],
                              'mesh':filters[go]['m_Mesh'],'materials':f['m_Materials'],
                              'matrix':matrix.tolist(),'instance':None,'source':scene,
                              'cast_shadows':f.get('m_CastShadows',1)})
        except (ValueError,KeyError,TypeError) as error:inline_errors.append({'renderer':i,'error':str(error)})
    return {'schema':1,'scope':'static source instance plan in Unity coordinates; no geometry export/FBX axis conversion',
            'archive_sha256':sources.manifest['archive_sha256'],'scene':scene,
            'lod_policy':'highest-detail first serialized LOD; no runtime crossfade',
            'instances':len(instances),'counts':dict(counts),'draws':draws,
            'unresolved':[{'instance':i,'error':error} for i,error in errors.items()],
            'inline_errors':inline_errors,'unused_target_overrides':unused,'unimplemented_prefab_components':dict(unimplemented),
            'unimplemented_scene_components':dict(Counter(str(v['kind']) for v in rr.values() if v['kind'] not in (1,4,23,33,64,65,1001))),
            'complete_scene_export':False}


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('source',type=Path);parser.add_argument('--scene',required=True)
    parser.add_argument('--output',required=True,type=Path)
    args=parser.parse_args()
    if args.output.exists():raise FileExistsError('Plan output must be new')
    result=scene_plan(Sources(args.source),args.scene)
    args.output.parent.mkdir(parents=True,exist_ok=True)
    args.output.write_text(json.dumps(result,separators=(',',':'))+'\n')
    print(json.dumps({k:v for k,v in result.items() if k not in ('draws','unresolved','unused_target_overrides')}))
