"""Independently check shared inspector images, complete streams and material meaning."""
import argparse
import copy
import hashlib
import json
from pathlib import Path
import re
import numpy as np
from merge_static_glb import read
from static_geometry import accessor, hierarchy
from shared_images import collect, sampler, filename
from verify_foliage_batch_scene import material_signature
from verify_triplanar_catalog import verify as verify_source


def sha(path):return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def source_image(doc, binary, image, root):
    if 'uri' not in image:
        v=doc['bufferViews'][image['bufferView']];start=v.get('byteOffset',0);return binary[start:start+v['byteLength']]
    uri=image['uri']
    if not re.fullmatch(r'shared/[A-Za-z0-9_.-]+\.png',uri):raise ValueError('Unqualified source image URI')
    path=(root/uri).resolve(strict=True)
    if not path.is_relative_to(root.resolve()):raise ValueError('Source image escapes root')
    return path.read_bytes()


def verify(stage):
    stage=Path(stage).resolve(strict=True);receipt=json.loads((stage/'resources.json').read_text());source=Path(receipt['source']).resolve(strict=True)
    if receipt['schema']!=1 or receipt['profile']!='forest-shared-images-inspector-v1':raise ValueError('Unknown resource stage profile')
    if sha(source/'preview.glb')!=receipt['source_asset_sha256'] or sha(source/'catalog-assembly.json')!=receipt['source_catalog_receipt_sha256'] or sha(source/'triplanar.json')!=receipt['source_metadata_sha256']:
        raise ValueError('Resource predecessor binding differs')
    before_proof=verify_source(source)
    if sha(stage/'preview.glb')!=receipt['asset_sha256'] or sha(stage/'triplanar.json')!=receipt['metadata_sha256']:raise ValueError('Resource output binding differs')
    before,old_binary=read(source/'preview.glb',allow_external_images=True);after,binary=read(stage/'preview.glb',allow_external_images=True);hierarchy(after)
    canonical_doc=copy.deepcopy(after)
    for image in canonical_doc['images']:
        if not isinstance(image.get('uri'),str) or not re.fullmatch(r'shared/[0-9a-f]{64}-[sl]-[0-9-]+\.png',image['uri']):raise ValueError('Noncanonical inspector shared image URI')
        image['uri']='../'+image['uri']
    descriptors=collect(canonical_doc,stage,allow_material_extensions=True)
    if descriptors!=receipt['shared_images'] or receipt['source_images']!=len(before['images']):raise ValueError('Resource image declarations differ')
    uris=[image.get('uri') for image in after['images']]
    if len(uris)!=len(set(uris)) or set(uris)!={'shared/'+filename(d) for d in descriptors} or any('bufferView' in i for i in after['images']):raise ValueError('Resource image deduplication differs')
    expected_files={filename(d) for d in descriptors}
    if {p.name for p in (stage/'shared').iterdir()}!=expected_files:raise ValueError('Unexpected shared resource files')
    roles={}
    for material in before['materials']:
        for owner,key,colour in [(material.get('pbrMetallicRoughness',{}),'baseColorTexture','srgb'),(material,'emissiveTexture','srgb'),(material.get('pbrMetallicRoughness',{}),'metallicRoughnessTexture','linear'),(material,'normalTexture','linear'),(material,'occlusionTexture','linear')]:
            if key in owner:roles.setdefault(owner[key]['index'],set()).add(colour)
    if set(roles)!=set(range(len(before['textures']))) or any(len(v)!=1 for v in roles.values()):raise ValueError('Source texture role ambiguity or omission')
    expected_variants=set()
    image_cache={}
    for index,texture in enumerate(before['textures']):
        n=texture['source']
        if n not in image_cache:image_cache[n]=source_image(before,old_binary,before['images'][n],source)
        data=image_cache[n];colour,=roles[index];expected_variants.add((hashlib.sha256(data).hexdigest(),colour,tuple(sampler(before,texture))))
    actual_variants={(bytes(v['sha256']).hex(),v['color_space'],tuple(v['sampler'])) for v in descriptors}
    if actual_variants!=expected_variants:raise ValueError('Shared source PNG/colour-space/sampler variants differ')
    excluded={'buffers','bufferViews','accessors','images','textures','materials'}
    if {k:v for k,v in before.items() if k not in excluded}!={k:v for k,v in after.items() if k not in excluded}:raise ValueError('Resource scene/hierarchy/primitive/profile metadata changed')
    if len(before['accessors'])!=len(after['accessors']):raise ValueError('Resource accessor coverage differs')
    for i,(a,b) in enumerate(zip(before['accessors'],after['accessors'])):
        if {k:v for k,v in a.items() if k!='bufferView'}!={k:v for k,v in b.items() if k!='bufferView'} or not np.array_equal(accessor(before,old_binary,i),accessor(after,binary,i)):
            raise ValueError('Resource vertex/index/normal/UV/tangent stream changed')
    if len(before['textures'])!=len(after['textures']) or any({k:v for k,v in a.items() if k!='source'}!={k:v for k,v in b.items() if k!='source'} for a,b in zip(before['textures'],after['textures'])):raise ValueError('Resource texture/sampler coverage differs')
    if len(before['materials'])!=len(after['materials']):raise ValueError('Resource material coverage differs')
    for i in range(len(before['materials'])):
        if material_signature(before,old_binary,i,root=source)!=material_signature(after,binary,i,root=stage):raise ValueError('Resource material/image/role semantics changed')
    expected=json.loads((source/'triplanar.json').read_text());expected['geometry_sha256']=receipt['asset_sha256']
    if json.loads((stage/'triplanar.json').read_text())!=expected:raise ValueError('Custom material metadata changed beyond geometry binding')
    custom={t['file']:t['sha256'] for p in expected['profiles'] for t in p['textures'].values()}
    if custom!=receipt['custom_images']:raise ValueError('Custom image inventory changed')
    for name,digest in custom.items():
        if not re.fullmatch(r'triplanar-(?:color|normal)-[0-9a-f]{64}\.png',name) or sha(stage/name)!=digest or (stage/name).read_bytes()!=(source/name).read_bytes():raise ValueError('Custom image bytes changed')
    return {'passed':True,'asset_sha256':receipt['asset_sha256'],'source_asset_sha256':receipt['source_asset_sha256'],
        'source_images':len(before['images']),'shared_images':len(descriptors),'source_textures':len(before['textures']),'custom_images':len(custom),
        'mesh_instances':before_proof['mesh_instances'],'triangles':before_proof['triangles'],'authored_materials':len(after['materials']),'source_meshes':len(after['meshes']),
        'expected_bounds':before_proof['expected_bounds'],'accessors_checked':len(before['accessors']),
        'scope':'complete source scene/mesh/stream/material/PNG/colour-space/sampler/custom-profile equivalence; inspector only, no package admission, mips, rebase or capacity claim'}


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('stage',type=Path);p.add_argument('--report',type=Path,required=True);a=p.parse_args()
    if a.report.exists():raise FileExistsError('Use a fresh verification report')
    result=verify(a.stage);a.report.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result))
