"""Assemble qualified foliage bindings and deduplicate standard shared PNGs.

Produces a standalone visual inspection scene. Canonical shared-image descriptors
are retained; standalone URIs stay inside the inspector's asset root. This is not
an authoritative world package or a default-map change.
"""
import argparse
import hashlib
import json
from pathlib import Path
from add_terrain_house import write_glb
from externalize_world_images import extract_document
from merge_static_glb import read,merge_documents
from shared_images import verify
from unity_geometry import normalized_quaternion


def assemble_batch(batch,base,output):
    batch=Path(batch).resolve(strict=True);base=Path(base).resolve(strict=True);output=Path(output).resolve()
    if output.exists() or output.is_relative_to(batch):raise FileExistsError('Use a new standalone scene output outside the batch')
    report=json.loads((batch/'batch.json').read_text());plan_path=Path(report['source_plan']);plan_bytes=plan_path.read_bytes()
    if hashlib.sha256(plan_bytes).hexdigest()!=report['plan_sha256']:raise ValueError('Source plan changed')
    if report.get('converted_draws',-1)+report.get('unresolved_draws',-1)!=report['planned_draws']:raise ValueError('Batch has not finished explicit classification')
    document,binary=read(base);packed=bytearray(binary);base_hash=hashlib.sha256(base.read_bytes()).hexdigest();inputs={str(base):base_hash}
    triangles=0;mesh_instances=0
    from static_geometry import hierarchy,accessor
    def inventory(doc,data):
        _,world=hierarchy(doc);count=faces=0
        for node in world:
            if 'mesh' not in doc['nodes'][node]:continue
            for p in doc['meshes'][doc['nodes'][node]['mesh']]['primitives']:
                count+=1;faces+=len(accessor(doc,data,p['indices']).ravel())//3
        return count,faces
    mesh_instances,triangles=inventory(document,binary)
    for record in report['converted']:
        path=batch/'prototypes'/record['key']/'placements.glb';value=path.read_bytes()
        if hashlib.sha256(value).hexdigest()!=record['placements_sha256']:raise ValueError('Placed prototype changed after qualification')
        doc,data=read(path);document=merge_documents(document,doc,len(packed));packed.extend(data)
        count,faces=inventory(doc,data);mesh_instances+=count;triangles+=faces;inputs[str(path)]=record['placements_sha256']
    quaternion_repairs=[]
    for index,node in enumerate(document['nodes']):
        rotation=node.get('rotation',[])
        if any(abs(value)>1 for value in rotation):
            node['rotation']=normalized_quaternion(rotation);quaternion_repairs.append(index)
    document['buffers'][0]['byteLength']=len(packed);output.mkdir(parents=True);shared=output/'shared';shared.mkdir()
    geometry,images=extract_document(document,packed,shared);verify(document,output,images)
    # The inspector's root is the GLB parent. Keep all image paths inside it;
    # package authoring can restore ../shared paths under its own asset root.
    for image in document['images']:
        if not image['uri'].startswith('../shared/'):raise ValueError('Unexpected extracted image URI')
        image['uri']=image['uri'][3:]
    write_glb(document,geometry,output/'forest.glb')
    result={'schema':1,'unit_quaternion_rounding_repairs':quaternion_repairs,'source_plan_sha256':report['plan_sha256'],'batch_report_sha256':hashlib.sha256((batch/'batch.json').read_bytes()).hexdigest(),
            'base':str(base),'base_sha256':base_hash,'inputs':inputs,'converted_draws':report['converted_draws'],'unresolved_draws':report['unresolved_draws'],
            'triangles':triangles,'mesh_instances':mesh_instances,'source_meshes':len(document['meshes']),'materials':len(document['materials']),'images':len(document['images']),
            'shared_images':images,'asset_sha256':hashlib.sha256((output/'forest.glb').read_bytes()).hexdigest(),
            'scope':'standalone qualified static subset plus selected base; explicit unconverted records; no object collision/full scene adoption'}
    (output/'assembly.json').write_text(json.dumps(result,indent=2)+'\n');return result


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('batch',type=Path);parser.add_argument('base',type=Path);parser.add_argument('output',type=Path);args=parser.parse_args()
    result=assemble_batch(args.batch,args.base,args.output);print(json.dumps({k:v for k,v in result.items() if k not in ('inputs','shared_images')}))
