"""Checked inherited object aliases for FBX-backed Unity prefab variants.

The XOR/mask rule is documented by a Unity engineer at:
https://discussions.unity.com/t/how-is-fileid-generated-for-m_correspondingsourceobject/754235
Identity resolution is separate from ModelImporter geometry/transform semantics.
"""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
from unity_scene import Sources, records
from unity_model_ids import identity, bound_candidates

MASK63=(1<<63)-1


def inherited_id(instance, source):
    if type(instance) is not int or not -(1<<63)<=instance<1<<63 or instance==0 or type(source) is not int or not -(1<<63)<=source<1<<63 or source==0:
        raise ValueError('Inherited fileID inputs outside signed 64-bit contract')
    result=(instance^source)&MASK63
    if not result:raise ValueError('Inherited fileID collides with null')
    return result


def alias_table(instance, candidates):
    result={}
    for source,binding in candidates.items():
        if binding['kind'] not in {'GameObject','Transform','MeshRenderer','MeshFilter','SkinnedMeshRenderer','Animator'}:continue
        alias=inherited_id(instance,source)
        if alias in result:raise ValueError('Inherited fileID collision')
        result[alias]={'source_fileID':source,**binding}
    return result


def variant_bindings(sources, name):
    rr=records(sources.checked(name));sources.metadata(name)
    nested=[(i,v['fields']) for i,v in rr.items() if v['kind']==1001]
    if not name.endswith('.prefab') or len(nested)!=1:
        raise ValueError('Select a single FBX-backed variant')
    instance,fields=nested[0];reference=fields['m_SourcePrefab']
    model_path=sources.by_guid.get(reference['guid'])
    if not model_path or not model_path.lower().endswith('.fbx'):
        raise ValueError('Variant source is not a bound FBX')
    modification=fields['m_Modification']
    if modification['m_TransformParent']['fileID'] or any(v for k,v in modification.items() if k.startswith(('m_Removed','m_Added'))):
        raise ValueError('Nested/structural variant changes need separate instantiation')
    model=identity(sources.checked(model_path));importer=sources.metadata(model_path)['ModelImporter']
    candidates,scheme=bound_candidates(model,importer);aliases=alias_table(instance,candidates)
    stripped=[];components=[];used=set()
    kinds={1:'GameObject',4:'Transform',23:'MeshRenderer',33:'MeshFilter',43:'Mesh',95:'Animator',137:'SkinnedMeshRenderer'}
    for identifier,record in rr.items():
        f=record['fields']
        if record['stripped']:
            source=f['m_CorrespondingSourceObject'];binding=aliases.get(identifier)
            if (f['m_PrefabInstance']['fileID']!=instance or source.get('guid')!=reference['guid']
                    or not binding or binding['source_fileID']!=source['fileID'] or binding['kind']!=kinds.get(record['kind'])):
                raise ValueError('Stripped object does not match inherited source identity')
            stripped.append({'fileID':identifier,'source_fileID':source['fileID'],'kind':binding['kind']});used.add(identifier)
        elif record['kind']!=1001:
            # Added local components can name a stripped owner. Preserve their
            # identities; applying collision/render behavior remains separate.
            owner=f.get('m_GameObject',{}).get('fileID');binding=aliases.get(owner)
            if not binding or binding['kind']!='GameObject':raise ValueError('Added variant component has unresolved owner')
            if identifier in aliases:raise ValueError('Added component collides with inherited identity')
            components.append({'fileID':identifier,'kind':record['kind'],'owner':owner});used.add(owner)
    changes=[]
    for m in modification.get('m_Modifications',[]):
        target=m['target'];binding=candidates.get(target['fileID'])
        if target.get('guid')!=reference['guid'] or binding is None:
            raise ValueError('Variant modification has unbound model target')
        alias=inherited_id(instance,target['fileID']);used.add(alias)
        changes.append({'source_fileID':target['fileID'],'fileID':alias,'kind':binding['kind'],
                        'propertyPath':m['propertyPath'],'value':m['value'],'objectReference':m['objectReference']})
    return {'source':name,'variant_instance':instance,'model':model_path,'model_guid':reference['guid'],
            'model_sha256':hashlib.sha256(sources.checked(model_path).read_bytes()).hexdigest(),
            'model_id_scheme':scheme,'aliases':aliases,'stripped':stripped,
            'added_components':components,'modifications':changes,'bound_aliases_used':sorted(used),
            'geometry_transform_imported':False}



def compile_single_model_variant(sources, name, outer_modifications, outer_guid):
    """Materialize the reviewed one-mesh static source-plan profile.

    All final root TRS components must be supplied by the outer instance, so
    imported FBX root pose/default scale is never invented here. Geometry import
    and axis conversion remain separate from this Unity-coordinate source plan.
    """
    description=variant_bindings(sources,name)
    model_path=description['model'];model=identity(sources.checked(model_path))
    importer=sources.metadata(model_path)['ModelImporter'];settings=importer['meshes']
    if (len(model['models'])!=1 or len(model['meshes'])!=1
            or next(iter(model['models'].values()))['type']!='Mesh'
            or settings.get('importVisibility')!=0 or importer.get('importAnimation')!=0
            or settings.get('importBlendShapes')!=0 or importer['materials'].get('materialImportMode')!=0):
        raise ValueError('Model variant is outside reviewed static single-mesh profile')
    aliases=description['aliases']
    def component(kind):
        found=[(i,v) for i,v in aliases.items() if v['kind']==kind]
        if len(found)!=1:raise ValueError('Ambiguous model component identity')
        return found[0][0]
    go=component('GameObject');transform=component('Transform')
    renderer=component('MeshRenderer');mesh_filter=component('MeshFilter')
    required={f'{key}.{axis}' for key,axes in [('m_LocalPosition','xyz'),('m_LocalRotation','xyzw'),('m_LocalScale','xyz')] for axis in axes}
    supplied={}
    for m in outer_modifications:
        if m['target'].get('guid')==outer_guid and m['target']['fileID']==transform and m['propertyPath'] in required:
            if m['propertyPath'] in supplied or m['objectReference'].get('fileID'):
                raise ValueError('Duplicate/object-valued model variant TRS override')
            supplied[m['propertyPath']]=m['value']
    if set(supplied)!=required:raise ValueError('Model variant needs complete outer root TRS; imported defaults are unqualified')
    material=None;root_name=next(iter(model['models'].values()))['name']
    for m in description['modifications']:
        kind=m['kind'];path=m['propertyPath']
        if kind=='MeshRenderer' and path=='m_Materials.Array.data[0]' and m['fileID']==renderer:
            if material is not None:raise ValueError('Duplicate variant material override')
            material=m['objectReference']
            if not material.get('fileID') or material.get('guid') not in sources.by_guid:
                raise ValueError('Model variant has unresolved material')
        elif kind=='GameObject' and path=='m_Name' and m['fileID']==go:root_name=str(m['value'])
        elif kind=='Transform' and m['fileID']==transform and (path=='m_RootOrder' or path.startswith(('m_LocalPosition.','m_LocalRotation.','m_LocalEulerAnglesHint.','m_LocalScale.'))):
            # The final scene explicitly replaces every TRS component.
            continue
        else:raise ValueError('Unsupported live model-variant source override')
    if material is None:raise ValueError('Model variant requires explicit renderer material')
    candidates,_=bound_candidates(model,importer)
    mesh_ids=[i for i,v in candidates.items() if v['kind']=='Mesh']
    if len(mesh_ids)!=1:raise ValueError('Ambiguous source mesh identity')
    def record(kind,label,fields):return {'kind':kind,'name':label,'fields':fields,'stripped':False}
    fields={'m_GameObject':{'fileID':go},'m_Father':{'fileID':0},'m_Children':[],
            'm_LocalEulerAnglesHint':dict(x=0,y=0,z=0)}
    for key,axes in [('m_LocalPosition','xyz'),('m_LocalRotation','xyzw'),('m_LocalScale','xyz')]:
        fields[key]={axis:float(supplied[key+'.'+axis]) for axis in axes}
    result={go:record(1,'GameObject',{'m_Name':root_name,'m_IsActive':1,'m_Layer':0,
                'm_Component':[{'component':{'fileID':i}} for i in [transform,renderer,mesh_filter]]}),
            transform:record(4,'Transform',fields),
            renderer:record(23,'MeshRenderer',{'m_GameObject':{'fileID':go},'m_Enabled':1,'m_CastShadows':1,'m_Materials':[dict(material)]}),
            mesh_filter:record(33,'MeshFilter',{'m_GameObject':{'fileID':go},'m_Mesh':{'fileID':mesh_ids[0],'guid':description['model_guid'],'type':3}})}
    raw=records(sources.checked(name))
    for component in description['added_components']:
        if component['kind'] not in (64,65,136):raise ValueError('Unsupported added model-variant component')
        identifier=component['fileID']
        if identifier in result:raise ValueError('Added component identity collision')
        result[identifier]=raw[identifier]
        result[go]['fields']['m_Component'].append({'component':{'fileID':identifier}})
    return result


def audit(source, dependencies):
    sources=Sources(source);dependency=json.loads(Path(dependencies).read_text())
    if dependency['archive_sha256']!=sources.manifest['archive_sha256']:
        raise ValueError('Dependency report archive does not match source')
    variants={};errors={}
    for edge in dependency['model_prefab_edges']:
        name=edge['document']
        if not name.endswith('.prefab') or name in variants or name in errors:continue
        try:variants[name]=variant_bindings(sources,name)
        except (ValueError,KeyError) as error:errors[name]=str(error)
    # Verify actual outer scene overrides against compiled variant aliases.
    scene_name=dependency['scene'];scene=records(sources.checked(scene_name));sources.metadata(scene_name)
    instances=[];unmatched=[];kinds=Counter();paths=Counter()
    for identifier,record in scene.items():
        if record['kind']!=1001:continue
        fields=record['fields'];ref=fields['m_SourcePrefab'];name=sources.by_guid.get(ref['guid'])
        if name not in variants:continue
        variant=variants[name];count=0
        for m in fields['m_Modification'].get('m_Modifications',[]):
            target=m['target'];binding=variant['aliases'].get(target['fileID'])
            if target.get('guid')!=ref['guid'] or not binding:
                unmatched.append({'instance':identifier,'source':name,'target':target,'propertyPath':m['propertyPath']});continue
            count+=1;kinds[binding['kind']]+=1;paths[m['propertyPath']]+=1
        instances.append({'instance':identifier,'source':name,'matched_modifications':count})
    return {'schema':1,'scope':'source/inherited identity binding only; no model TRS, geometry, collider or scene instantiation',
            'archive_sha256':sources.manifest['archive_sha256'],'variants':list(variants.values()),
            'variant_errors':errors,'scene':scene_name,'instances':instances,
            'matched_scene_modifications':sum(v['matched_modifications'] for v in instances),
            'matched_by_kind':dict(kinds),'matched_property_paths':dict(paths),'unmatched':unmatched,
            'complete_scene_export':False}


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('source',type=Path);parser.add_argument('--dependencies',type=Path,required=True)
    parser.add_argument('--report',type=Path,required=True)
    args=parser.parse_args()
    if args.report.exists():raise FileExistsError('Use a new report file')
    result=audit(args.source,args.dependencies);args.report.write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps({k:v for k,v in result.items() if k not in ('variants','instances')}))
