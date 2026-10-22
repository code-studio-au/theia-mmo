"""Compare corrected glTF material triangles with actual Editor submesh slots."""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import numpy as np
from merge_static_glb import read
from static_geometry import accessor,hierarchy


def signatures(triangles):
    result=Counter()
    for triangle in triangles:
        rows=[tuple(v) for v in np.asarray(triangle,dtype=np.float32)]
        if not np.isfinite(triangle).all():raise ValueError('Nonfinite source slot triangle')
        result[min(tuple(rows[n:]+rows[:n]) for n in range(3))]+=1
    return result


def compare(doc,binary,mesh,slots):
    _,world=hierarchy(doc)
    if len(doc['nodes'])!=1 or len(doc['meshes'])!=1 or not np.array_equal(world[0],np.eye(4)):raise ValueError('Select one corrected mesh-local prototype')
    if len(mesh['submeshes'])!=slots:raise ValueError('Editor/source material submesh count differs')
    points=np.asarray(mesh['positions'],dtype=np.float32).reshape(-1,3).copy();points[:,0]*=-1
    expected={}
    for slot,submesh in enumerate(mesh['submeshes']):
        if submesh['topology']!=0 or len(submesh['indices'])%3:raise ValueError('Select triangulated Editor submeshes')
        indices=np.asarray(submesh['indices'])
        if indices.dtype.kind not in 'iu' or indices.min()<0 or indices.max()>=len(points):raise ValueError('Editor slot indices outside source mesh')
        # Calibrated against actual Editor versus qualified single-slot geometry.
        expected[slot]=signatures(points[indices.reshape(-1,3)[:,[0,2,1]]])
    actual={i:Counter() for i in range(slots)}
    for primitive in doc['meshes'][0]['primitives']:
        slot=doc['materials'][primitive['material']].get('extras',{}).get('theia_source_slot')
        if type(slot) is not int or slot not in actual:raise ValueError('Unknown/malformed source material slot')
        positions=accessor(doc,binary,primitive['attributes']['POSITION']);indices=accessor(doc,binary,primitive['indices']).reshape(-1,3)
        actual[slot].update(signatures(positions[indices]))
    if actual!=expected:
        differences={str(i):{'expected':sum(expected[i].values()),'actual':sum(actual[i].values()),'missing':sum((expected[i]-actual[i]).values()),'extra':sum((actual[i]-expected[i]).values())} for i in actual}
        raise ValueError('Actual Editor submesh material triangles differ: '+json.dumps(differences))
    return {str(i):sum(v.values()) for i,v in actual.items()}


def verify(stage):
    stage=Path(stage);conversion=json.loads((stage/'conversion.json').read_text());receipt=json.loads((stage/'unity-frame.json').read_text());project=Path(receipt['project'])
    sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
    if conversion['unity_frame_reference']!=receipt or sha(stage/'preview.glb')!=receipt['asset_sha256'] or sha(project/'Reference/request.json')!=receipt['request_sha256'] or sha(project/'Reference/imported.json')!=receipt['editor_report_sha256']:raise ValueError('Editor slot/frame/geometry binding changed')
    report=json.loads((project/'Reference/imported.json').read_text());model,=[v for v in report['models'] if v['path']==conversion['model']];mesh,=[v for v in model['meshes'] if v['file_id']==str(conversion['mesh_fileID'])]
    doc,binary=read(stage/'preview.glb');counts=compare(doc,binary,mesh,len(conversion['material_refs']))
    return {'passed':True,'asset_sha256':receipt['asset_sha256'],'editor_report_sha256':receipt['editor_report_sha256'],'triangles_by_slot':counts,
            'scope':'complete actual Editor per-submesh triangle multisets, exact f32 positions and winding; source material identities/UV/normal/PBR are separately qualified'}


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('stage',type=Path);p.add_argument('--report',type=Path,required=True);a=p.parse_args()
    if a.report.exists():raise FileExistsError('Use a fresh report')
    r=verify(a.stage);a.report.write_text(json.dumps(r,indent=2)+'\n');print(json.dumps(r))
