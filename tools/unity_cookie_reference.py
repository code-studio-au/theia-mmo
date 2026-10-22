"""Actual imported cookie Alpha8 mip-zero reference with restored scratch settings."""
import argparse
import json
from pathlib import Path
import shutil
from unity_texture_reference import stage as stage_textures
from unity_editor_reference import digest,read_json
from unity_scene import Sources

COLLECTOR=Path(__file__).resolve().parent/'qualification/TheiaCookiePixelsReference.cs'


def stage(source,output,version):
    sources=Sources(source);output=Path(output)
    names=[sources.by_guid[g] for g in ['953b5ab26fde2ae47a7cba2ba6916045','29f2b19ba9eed9144a38658bdf71abcc']]
    request=stage_textures(sources.root,output,names,version);(output/'Assets/Editor/TheiaTextureReference.cs').unlink();shutil.copyfile(COLLECTOR,output/'Assets/Editor/TheiaCookiePixelsReference.cs');request['collector_sha256']=digest(COLLECTOR);(output/'Reference/request.json').write_text(json.dumps(request,indent=2)+'\n');return request


def inspect(source,project):
    source=Sources(source);project=Path(project).resolve(strict=True);request=read_json(project/'Reference/request.json',65536);report=read_json(project/'Reference/cookies.json',256*1024)
    if request['schema']!=1 or report['schema']!=1 or request['archive_sha256']!=source.manifest['archive_sha256'] or report['request_sha256']!=digest(project/'Reference/request.json') or report['unity_version']!=request['unity_version'] or report['platform']!='LinuxEditor' or report['orientation']!='top-left rows':raise ValueError('Cookie reference request/source/version/orientation differs')
    if request['collector_sha256']!=digest(COLLECTOR) or report['collector_sha256']!=digest(COLLECTOR) or digest(project/'Assets/Editor/TheiaCookiePixelsReference.cs')!=digest(COLLECTOR):raise ValueError('Cookie pixel collector changed')
    inputs={v['path']:v for v in request['textures']};actual={v['path']:v for v in report['textures']}
    if len(inputs)!=2 or len(actual)!=2 or set(inputs)!=set(actual):raise ValueError('Cookie reference coverage differs')
    for name,item in inputs.items():
        source.metadata(name);r=actual[name]
        if item['guid']!=source.manifest['selected'][name]['guid'] or item['sha256']!=digest(source.checked(name)) or item['meta_sha256']!=digest(source.checked(name,True)):raise ValueError('Cookie original source binding changed')
        if not name.startswith('Assets/') or not (project/name).resolve(strict=True).is_relative_to(project) or digest(project/name)!=item['sha256'] or digest(project/'Reference/original-meta'/(item['guid']+'.meta'))!=item['meta_sha256']:raise ValueError('Cookie original/staged source changed')
        if r['guid']!=item['guid'] or r['file_id']!='2800000' or r['format']!='Alpha8' or r['alpha_source']!=2 or r['width']!=2048 or r['height']!=2048 or r['mip_count']!=12:raise ValueError('Actual cookie pixel representation differs')
        if r['original_readable'] is not False or r['restored_readable'] is not False or r['settings_before']!=r['settings_after'] or len(r['settings_before'])<16 or r['platform_before']!=r['platform_after'] or digest(Path(str(project/name)+'.meta'))!=r['restored_meta_sha256']:raise ValueError('Temporary cookie settings were not restored')
        if r['alpha_file']!='cookie-'+item['guid']+'.alpha8':raise ValueError('Invalid cookie alpha reference path')
        path=project/'Reference'/r['alpha_file']
        if path.resolve(strict=True)!=path or path.stat().st_size!=2048*2048 or digest(path)!=r['alpha_sha256']:raise ValueError('Cookie alpha data binding/budget differs')
    return {'passed':True,'project':str(project),'unity_version':report['unity_version'],'request_sha256':digest(project/'Reference/request.json'),'report_sha256':digest(project/'Reference/cookies.json'),'collector_sha256':digest(COLLECTOR),'textures':report['textures'],'scope':'actual imported top-left Alpha8 mip-zero values; temporary CPU readability in ignored scratch restored with matching importer/platform settings; original package bytes/metadata unchanged, scratch metadata may reserialize; no rendered projection or compression parity claim'}


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('mode',choices=['stage','inspect']);p.add_argument('source',type=Path);p.add_argument('project',type=Path);p.add_argument('--version');a=p.parse_args();print(json.dumps(stage(a.source,a.project,a.version) if a.mode=='stage' else inspect(a.source,a.project)))
