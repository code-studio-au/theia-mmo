"""Independently check final batch scene coverage, source poses and shared assets.

Reads qualified prototypes and the original source plan; imports neither batch
converter, binding selector nor scene assembler. All rendered primitives are
bound to those inputs after image deduplication.
"""
import argparse
import copy
import hashlib
import json
from pathlib import Path
import numpy as np
from merge_static_glb import read
from shared_images import verify as verify_shared,sampler
from static_geometry import accessor,hierarchy,TEXTURE_FIELDS


def material_signature(doc,binary,index,root=None):
    material=copy.deepcopy(doc['materials'][index])
    for parent,key in TEXTURE_FIELDS:
        owner=material.get(parent,{}) if parent else material
        if key not in owner:continue
        field=owner[key];texture=doc['textures'][field['index']];image=doc['images'][texture['source']]
        if 'uri' in image:
            parts=Path(image['uri']).parts
            if len(parts)!=2 or parts[0]!='shared' or root is None:raise ValueError('Unbound standalone shared texture path')
            value=(Path(root)/image['uri']).read_bytes()
        else:
            view=doc['bufferViews'][image['bufferView']];offset=view.get('byteOffset',0);value=binary[offset:offset+view['byteLength']]
        field['index']={'sha256':hashlib.sha256(value).hexdigest(),'sampler':sampler(doc,texture)}
    return material


def verify(batch,scene):
    batch=Path(batch);scene=Path(scene);report=json.loads((batch/'batch.json').read_text());metadata=json.loads((scene/'assembly.json').read_text())
    plan_bytes=Path(report['source_plan']).read_bytes();plan=json.loads(plan_bytes)
    if hashlib.sha256(plan_bytes).hexdigest()!=report['plan_sha256'] or metadata['source_plan_sha256']!=report['plan_sha256']:raise ValueError('Source plan differs')
    if hashlib.sha256((batch/'batch.json').read_bytes()).hexdigest()!=metadata['batch_report_sha256'] or hashlib.sha256((scene/'forest.glb').read_bytes()).hexdigest()!=metadata['asset_sha256']:raise ValueError('Scene/batch binding differs')
    groups={}
    for draw in plan['draws']:
        key=hashlib.sha256(json.dumps([draw['mesh'],draw['materials']],sort_keys=True,separators=(',',':')).encode()).hexdigest();groups.setdefault(key,[]).append(draw)
    records=report['converted']+report['unresolved']
    if len({r['key'] for r in records})!=len(records) or {r['key'] for r in records}!=set(groups):raise ValueError('Batch classification omitted/duplicated a source binding')
    if any(r['draws']!=len(groups[r['key']]) for r in records):raise ValueError('Batch source category count differs')
    doc,binary=read(scene/'forest.glb',allow_external_images=True)
    canonical=dict(doc,images=[dict(image,uri='../'+image['uri']) for image in doc['images']]);verify_shared(canonical,scene,metadata['shared_images'])
    parents,world=hierarchy(doc);roots={};owned={};all_meshes=set()
    for root in doc['scenes'][0]['nodes']:
        identifier=doc['nodes'][root].get('extras',{}).get('source_instance')
        if identifier is not None:
            if identifier in roots:raise ValueError('Duplicate source renderer root')
            roots[identifier]=root
    for node,frame in world.items():
        if 'mesh' not in doc['nodes'][node]:continue
        root=node
        while parents[root] is not None:root=parents[root]
        owned.setdefault(root,[]).append(node);all_meshes.add(node)
    material_cache={};mesh_cache={}
    def primitives(node):
        index=doc['nodes'][node]['mesh']
        if index not in mesh_cache:
            value=[]
            for p in doc['meshes'][index]['primitives']:
                material=p['material']
                if material not in material_cache:material_cache[material]=material_signature(doc,binary,material,scene)
                value.append((p,{k:accessor(doc,binary,i) for k,i in p['attributes'].items()},accessor(doc,binary,p['indices']).reshape(-1,3),material_cache[material]))
            mesh_cache[index]=value
        return mesh_cache[index]
    base_path=Path(metadata['base']);base_bytes=base_path.read_bytes()
    if hashlib.sha256(base_bytes).hexdigest()!=metadata['base_sha256']:raise ValueError('Qualified base scene changed')
    base,base_binary=read(base_path);_,base_world=hierarchy(base);checked_nodes=set();base_triangles=0
    for node,frame in base_world.items():
        if 'mesh' not in base['nodes'][node]:continue
        if node not in world or not np.array_equal(frame,world[node]):raise ValueError('Base source frame changed')
        expected=base['meshes'][base['nodes'][node]['mesh']]['primitives'];actual=primitives(node)
        if len(actual)!=len(expected):raise ValueError('Base primitive set changed')
        for source,(p,attributes,indices,material) in zip(expected,actual):
            if set(attributes)!=set(source['attributes']) or any(not np.array_equal(value,accessor(base,base_binary,source['attributes'][key])) for key,value in attributes.items()) or not np.array_equal(indices,accessor(base,base_binary,source['indices']).reshape(-1,3)):
                raise ValueError('Base source geometry changed')
            if material!=material_signature(base,base_binary,source['material']):raise ValueError('Base source material/image changed')
            base_triangles+=len(indices)
        checked_nodes.add(node)
    expected_ids=set();maximum=0.;foliage_triangles=0;mesh_instances=sum(len(primitives(node)) for node in checked_nodes)
    all_points=[]
    for node in checked_nodes:
        frame=world[node]
        for _,attributes,_,_ in primitives(node):
            points=attributes['POSITION'].astype(float)@frame[:3,:3].T+frame[:3,3];all_points.extend([points.min(0),points.max(0)])
    for record in report['converted']:
        directory=batch/'prototypes'/record['key'];prototype=directory/'preview.glb'
        if hashlib.sha256(prototype.read_bytes()).hexdigest()!=record['prototype_sha256']:raise ValueError('Qualified prototype changed')
        reference=json.loads((directory/'independent-verification.json').read_text());geometry=json.loads((directory/'canonical-reference.json').read_text())
        if not reference['passed'] or reference['asset_sha256']!=record['prototype_sha256'] or not geometry['passed']:raise ValueError('Prototype qualification missing')
        original,source_binary=read(prototype);original_primitives=[]
        for source in original['meshes'][0]['primitives']:
            original_primitives.append((source,{k:accessor(original,source_binary,i) for k,i in source['attributes'].items()},accessor(original,source_binary,source['indices']).reshape(-1,3),material_signature(original,source_binary,source['material'])))
        for draw in groups[record['key']]:
            identifier=str(draw['instance'])+':'+str(draw['renderer']);expected_ids.add(identifier)
            if identifier not in roots or len(owned.get(roots[identifier],[]))!=1:raise ValueError('Missing source renderer geometry')
            node,=owned[roots[identifier]];frame=world[node];actual=primitives(node)
            if np.linalg.det(frame[:3,:3])<=0 or len(actual)!=len(original_primitives):raise ValueError('Invalid source geometry frame/primitive set')
            target=np.array(draw['matrix'],dtype=float);target[0,:]*=-1;target[:,0]*=-1;negative=np.linalg.det(target[:3,:3])<0
            for (source,wanted,source_indices,source_material),(p,attributes,indices,material) in zip(original_primitives,actual):
                if p.get('mode',4)!=4 or set(attributes)!=set(wanted):raise ValueError('Final source primitive attributes differ')
                expected=wanted['POSITION'].astype(float)@target[:3,:3].T+target[:3,3];observed=attributes['POSITION'].astype(float)@frame[:3,:3].T+frame[:3,3]
                error=float(np.max(np.abs(expected-observed)));maximum=max(maximum,error)
                if error>1e-6:raise ValueError('Final source positions differ')
                all_points.extend([expected.min(0),expected.max(0)])
                for key,value in wanted.items():
                    if key=='POSITION':continue
                    if key=='NORMAL':
                        a=value@np.linalg.inv(target[:3,:3]);b=attributes[key]@np.linalg.inv(frame[:3,:3]);a/=np.linalg.norm(a,axis=1)[:,None];b/=np.linalg.norm(b,axis=1)[:,None]
                        if not np.allclose(a,b,atol=1e-6,rtol=0):raise ValueError('Final source normals differ')
                    elif key=='TANGENT':
                        a=value[:,:3]@target[:3,:3].T;b=attributes[key][:,:3]@frame[:3,:3].T;a/=np.linalg.norm(a,axis=1)[:,None];b/=np.linalg.norm(b,axis=1)[:,None]
                        if not np.allclose(a,b,atol=1e-6,rtol=0) or not np.array_equal(attributes[key][:,3],value[:,3]*(-1 if negative else 1)):raise ValueError('Final source tangents/handedness differ')
                    elif not np.array_equal(value,attributes[key]):raise ValueError('Final source UV/attribute changed')
                if not np.array_equal(indices,source_indices[:,[0,2,1]] if negative else source_indices) or material!=source_material:raise ValueError('Final source winding/material/image changed')
                foliage_triangles+=len(indices);mesh_instances+=1
            checked_nodes.add(node)
    actual_ids={key for key in roots if isinstance(key,str) and ':' in key}
    if actual_ids!=expected_ids or checked_nodes!=all_meshes:raise ValueError('Final scene has missing/extra source renderers')
    points=np.array(all_points)
    if len(expected_ids)!=report['converted_draws'] or foliage_triangles+base_triangles!=metadata['triangles'] or mesh_instances!=metadata['mesh_instances']:raise ValueError('Final scene inventory differs')
    return {'passed':True,'source_renderers':len(expected_ids),'unresolved_source_draws':report['unresolved_draws'],'mesh_instances':mesh_instances,'triangles':foliage_triangles+base_triangles,
            'max_source_position_error_metres':maximum,'expected_bounds':[points.min(0).tolist(),points.max(0).tolist()],
            'asset_sha256':metadata['asset_sha256'],'scope':'complete converted binding/source-pose/geometry/tangent/material/shared-image coverage; explicit omissions; no full forest/collision adoption'}


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('batch',type=Path);parser.add_argument('scene',type=Path);parser.add_argument('--report',required=True,type=Path);args=parser.parse_args()
    if args.report.exists():raise FileExistsError('Use a new report')
    result=verify(args.batch,args.scene);args.report.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result))
