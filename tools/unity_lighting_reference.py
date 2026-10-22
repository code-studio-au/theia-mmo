"""Stage/inspect four script-free model variants and two original cookie imports."""
import argparse
import json
from pathlib import Path
import shutil
from PIL import Image
from unity_editor_reference import BUILD, VERSION, digest, read_json, checked_id
from unity_scene import Sources, records
from unity_model_ids import identity

COLLECTOR=Path(__file__).resolve().parent/'qualification/TheiaLightingReference.cs'


def stage(source, plan, output, version):
    source=Sources(source);output=Path(output).resolve();plan=read_json(plan,64*1024*1024)
    if output.exists() or output==BUILD or not output.is_relative_to(BUILD) or source.root.is_relative_to(output):raise ValueError('Use a fresh ignored reference project')
    if not isinstance(version,str) or not VERSION.fullmatch(version) or plan['archive_sha256']!=source.manifest['archive_sha256']:raise ValueError('Source/version binding differs')
    prefabs=[];models=set()
    for name in sorted({d['source'] for d in plan['draws'] if d['source'].endswith('.prefab')}):
        rr=records(source.checked(name));nested=[v for v in rr.values() if v['kind']==1001]
        if not nested:continue
        if len(nested)!=1 or any(v['kind'] not in (1,4,23,33,64,65,1001) for v in rr.values()):raise ValueError('Reference accepts script-free single-model variants only')
        model=source.by_guid[nested[0]['fields']['m_SourcePrefab']['guid']]
        if not model.lower().endswith('.fbx'):raise ValueError('Variant model must be an actual FBX')
        identity(source.checked(model));prefabs.append(name);models.add(model)
    if len(prefabs)!=4 or len(models)!=4:raise ValueError('Select the four qualified source variants')
    textures=[source.by_guid[g] for g in ['953b5ab26fde2ae47a7cba2ba6916045','29f2b19ba9eed9144a38658bdf71abcc']]
    for directory in ['Assets/Editor','ProjectSettings','Packages','Reference/original-meta']:(output/directory).mkdir(parents=True,exist_ok=True)
    (output/'ProjectSettings/ProjectVersion.txt').write_text('m_EditorVersion: '+version+'\n');(output/'Packages/manifest.json').write_text('{"dependencies": {}}\n');shutil.copyfile(COLLECTOR,output/'Assets/Editor/TheiaLightingReference.cs')
    result={'schema':1,'unity_version':version,'collector_sha256':digest(COLLECTOR),'archive_sha256':source.manifest['archive_sha256']}
    for key,names in [('models',sorted(models)),('prefabs',prefabs),('textures',textures)]:
        result[key]=[]
        for name in names:
            source.metadata(name);original=source.checked(name);meta=source.checked(name,True);binding=source.manifest['selected'][name]
            target=output/name;target.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(original,target);shutil.copyfile(meta,str(target)+'.meta');shutil.copyfile(meta,output/'Reference/original-meta'/(binding['guid']+'.meta'))
            result[key].append({'path':name,'guid':binding['guid'],'sha256':digest(original),'meta_sha256':digest(meta)})
    (output/'Reference/request.json').write_text(json.dumps(result,indent=2)+'\n');return result


def inspect(source, project):
    source=Sources(source);project=Path(project).resolve(strict=True);request=read_json(project/'Reference/request.json',65536);report=read_json(project/'Reference/lighting.json',4*1024*1024)
    if request['schema']!=1 or report['schema']!=1 or request['archive_sha256']!=source.manifest['archive_sha256'] or report['request_sha256']!=digest(project/'Reference/request.json') or report['unity_version']!=request['unity_version'] or report['platform']!='LinuxEditor':raise ValueError('Lighting reference request/version/source differs')
    if report['collector_sha256']!=digest(COLLECTOR) or request['collector_sha256']!=digest(COLLECTOR) or digest(project/'Assets/Editor/TheiaLightingReference.cs')!=digest(COLLECTOR):raise ValueError('Lighting reference collector changed')
    for key,count in [('models',4),('prefabs',4),('textures',2)]:
        inputs=request[key]
        if len(inputs)!=count or len({v['path'] for v in inputs})!=count:raise ValueError('Lighting reference source coverage differs')
        for item in inputs:
            name=item['path'];source.metadata(name)
            if not name.startswith('Assets/') or not (project/name).resolve(strict=True).is_relative_to(project):raise ValueError('Lighting input escapes project')
            if item['guid']!=source.manifest['selected'][name]['guid'] or digest(source.checked(name))!=item['sha256'] or digest(source.checked(name,True))!=item['meta_sha256'] or digest(project/name)!=item['sha256'] or digest(Path(str(project/name)+'.meta'))!=item['meta_sha256'] or digest(project/'Reference/original-meta'/(item['guid']+'.meta'))!=item['meta_sha256']:raise ValueError('Lighting input or metadata changed')
    prefabs={p['path']:p for p in report['prefabs']};textures={t['path']:t for t in report['textures']}
    if len(prefabs)!=4 or set(prefabs)!={p['path'] for p in request['prefabs']} or len(textures)!=2 or set(textures)!={p['path'] for p in request['textures']}:raise ValueError('Lighting reference report coverage differs')
    model_guids={m['guid'] for m in request['models']}
    for item in request['prefabs']:
        value=prefabs[item['path']]
        if value['guid']!=item['guid'] or not 1<=len(value['renderers'])<=256:raise ValueError('Variant layer reference differs')
        rr=records(source.checked(item['path']));nested=[(i,v['fields']) for i,v in rr.items() if v['kind']==1001]
        if len(nested)!=1:raise ValueError('Reference variant structure differs')
        instance,fields=nested[0];model_guid=fields['m_SourcePrefab']['guid']
        seen=set()
        for renderer in value['renderers']:
            if type(renderer['layer']) is not int or not 0<=renderer['layer']<=31:raise ValueError('Invalid referenced layer')
            for key in ['renderer','game_object','original_renderer','original_game_object']:checked_id(renderer[key]['file_id'])
            identity=(renderer['renderer']['guid'],renderer['renderer']['file_id'])
            if identity in seen or renderer['original_renderer']['guid'] not in model_guids or renderer['original_game_object']['guid'] not in model_guids:raise ValueError('Unbound or duplicate original layer identity')
            if renderer['renderer']['guid']!=item['guid'] or renderer['game_object']['guid']!=item['guid'] or renderer['original_renderer']['guid']!=model_guid or renderer['original_game_object']['guid']!=model_guid:raise ValueError('Variant persistent namespace differs')
            for key,original in [('renderer','original_renderer'),('game_object','original_game_object')]:
                expected=(checked_id(renderer[original]['file_id']) ^ instance) & ((1<<63)-1)
                if checked_id(renderer[key]['file_id'])!=expected:raise ValueError('Actual inherited variant identifier differs')
            seen.add(identity)
    for item in request['textures']:
        t=textures[item['path']]
        if t['guid']!=item['guid'] or t['file_id']!='2800000' or t['importer_type']!=4 or t['alpha_source']!=2 or t['format']!='Alpha8' or source.metadata(item['path'])['TextureImporter']['cookieLightType'] not in (0,1):raise ValueError('Effective source cookie type/alpha differs')
        with Image.open(source.checked(item['path'])) as image:
            if list(image.size)!=[t['width'],t['height']]:raise ValueError('Editor resized source cookie')
        if t['filter']!=1 or t['wrap_u'] not in (0,1) or t['wrap_v']!=t['wrap_u'] or t['aniso']!=1 or t['mip_bias']!=0 or t['mip_count']!=12 or t['srgb'] is not True or t['mipmaps'] is not True:raise ValueError('Effective cookie sampler/mips differ')
    return {'passed':True,'unity_version':report['unity_version'],'project':str(project),'collector_sha256':digest(COLLECTOR),'request_sha256':digest(project/'Reference/request.json'),'report_sha256':digest(project/'Reference/lighting.json'),'prefabs':report['prefabs'],'textures':report['textures'],'scope':'actual script-free variant renderer layers/identities and effective cookie importer state with unchanged source bytes; no full scene or rendered cookie parity'}


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('mode',choices=['stage','inspect']);p.add_argument('source',type=Path);p.add_argument('project',type=Path);p.add_argument('--plan',type=Path);p.add_argument('--version');a=p.parse_args();print(json.dumps(stage(a.source,a.plan,a.project,a.version) if a.mode=='stage' else inspect(a.source,a.project)))
