"""Calibrate static FBX mesh-local axes/units against serialized Unity LOD bounds.

Uses a separately compiled, pinned ufbx probe. Does not execute Unity, export a
scene, or prove every ModelImporter setting. Mismatches remain explicit.
"""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import numpy as np
from unity_scene import Sources,records
from unity_model_ids import identity,bound_candidates
from unity_prefabs import static_prefab


def box_corners(points):
    points=np.asarray(points,dtype=np.float64)
    if points.ndim!=2 or points.shape[1]!=3 or not len(points) or not np.isfinite(points).all():
        raise ValueError('Invalid bounded mesh points')
    lo=points.min(0);hi=points.max(0)
    return np.array([[hi[a] if bits&(1<<a) else lo[a] for a in range(3)] for bits in range(8)])


def audit(source,dependencies,probe):
    sources=Sources(source);dependency=json.loads(Path(dependencies).read_text());probe=Path(probe).resolve(strict=True)
    if dependency['archive_sha256']!=sources.manifest['archive_sha256']:raise ValueError('Dependency archive differs')
    cache={};result=[];errors=[]
    def geometry(reference):
        path=sources.by_guid[reference['guid']]
        if path not in cache:
            model=sources.checked(path);importer=sources.metadata(path)['ModelImporter'];settings=importer['meshes']
            if settings['globalScale']!=1 or settings['useFileUnits']!=1 or settings['useFileScale']!=1 or settings['bakeAxisConversion']!=0:
                raise ValueError('Unqualified importer scale/axis policy')
            data=json.loads(subprocess.check_output([str(probe),str(model),'right'],timeout=30))
            if data['source_axes']!=[0,2,4] or data['ufbx_version']!=23001:
                raise ValueError('Unqualified source axes/probe version')
            ids,_=bound_candidates(identity(model),importer);cache[path]=(data,ids)
        data,ids=cache[path];binding=ids.get(reference['fileID'])
        if not binding or binding['kind']!='Mesh':raise ValueError('Unbound source mesh fileID')
        node,=[node for node in data['nodes'] if node['name']==binding['name']]
        if node['deformed']:raise ValueError('Deformed mesh needs separate basis qualification')
        points=np.array(node['vertices']);matrix=np.array(node['geometry_to_node'])
        return points@matrix[:3,:3].T+matrix[:3,3]
    for name in dependency['documents']:
        if not name.endswith('.prefab'):continue
        rr=records(sources.checked(name));sources.metadata(name)
        groups=[(i,v['fields']) for i,v in rr.items() if v['kind']==205]
        if not groups:continue
        try:
            static=static_prefab(rr)
            transforms={v['fields']['m_GameObject']['fileID']:i for i,v in rr.items() if v['kind']==4}
            filters={v['fields']['m_GameObject']['fileID']:v['fields'] for v in rr.values() if v['kind']==33}
            for identifier,group in groups:
                inverse=np.linalg.inv(static['matrices'][transforms[group['m_GameObject']['fileID']]])
                points={key:[] for key in ['mirror-x','mirror-z','unchanged']}
                for lod in group['m_LODs']:
                    for ref in lod['renderers']:
                        renderer=rr[ref['renderer']['fileID']]['fields'];go=renderer['m_GameObject']['fileID']
                        original=geometry(filters[go]['m_Mesh']);frame=inverse@static['matrices'][transforms[go]]
                        for mode,sign in [('mirror-x',[-1,1,1]),('mirror-z',[1,1,-1]),('unchanged',[1,1,1])]:
                            corners=box_corners(original*np.array(sign));points[mode].extend(corners@frame[:3,:3].T+frame[:3,3])
                expected=np.array([group['m_LocalReferencePoint'][a] for a in 'xyz']);size=group['m_Size'];comparisons={}
                for mode,values in points.items():
                    values=np.array(values);center=(values.min(0)+values.max(0))*.5;actual_size=np.ptp(values,axis=0).max()
                    comparisons[mode]={'center':center.tolist(),'size':float(actual_size),
                        'center_error':float(np.abs(center-expected).max()),'size_error':float(abs(size-actual_size))}
                result.append({'source':name,'group':identifier,'source_center':expected.tolist(),'source_size':size,'comparisons':comparisons})
        except (ValueError,KeyError,subprocess.CalledProcessError) as error:errors.append({'source':name,'error':str(error)})
    accepted=[v for v in result if v['comparisons']['mirror-x']['center_error']<=.0001 and v['comparisons']['mirror-x']['size_error']<=.0001]
    return {'schema':1,'scope':'static mesh-local FBX/Unity serialized LOD bound calibration; not Unity Editor import or full scene equivalence',
            'archive_sha256':sources.manifest['archive_sha256'],'probe_sha256':hashlib.sha256(probe.read_bytes()).hexdigest(),
            'groups':result,'errors':errors,'tolerance_metres':.0001,'mirror_x_matches':len(accepted),
            'groups_checked':len(result),'models_read':len(cache),
            'discriminate_unchanged':sum(v['comparisons']['unchanged']['center_error']>.0001 or v['comparisons']['unchanged']['size_error']>.0001 for v in accepted),
            'discriminate_mirror_z':sum(v['comparisons']['mirror-z']['center_error']>.0001 or v['comparisons']['mirror-z']['size_error']>.0001 for v in accepted)}


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('source',type=Path);parser.add_argument('--dependencies',required=True,type=Path)
    parser.add_argument('--probe',required=True,type=Path);parser.add_argument('--report',required=True,type=Path)
    args=parser.parse_args()
    if args.report.exists():raise FileExistsError('Use a new report')
    result=audit(args.source,args.dependencies,args.probe);args.report.write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps({k:v for k,v in result.items() if k!='groups'}))
