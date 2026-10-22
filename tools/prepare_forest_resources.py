"""Share qualified inspector PNG variants without changing source scene semantics.

Package/runtime extension admission remains unchanged. This output is an opt-in
inspector resource stage with the existing known static material extensions.
"""
import argparse
import copy
import hashlib
import json
from pathlib import Path
import re
import shutil
from add_terrain_house import write_glb
from merge_static_glb import read
from externalize_world_images import extract_document
from shared_images import MAX_IMAGE_BYTES, collect, usage_fields
from verify_triplanar_catalog import verify as verify_source


def sha(path):return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def prepare(source, output):
    source=Path(source).resolve(strict=True);output=Path(output).resolve();build=Path(__file__).resolve().parents[1]/'build'
    if output.exists() or output==build or not output.is_relative_to(build) or output.is_relative_to(source) or source.is_relative_to(output):raise ValueError('Use a fresh ignored resource stage outside source')
    checked=verify_source(source);asset=source/'preview.glb';doc,binary=read(asset,allow_external_images=True);config=json.loads((source/'triplanar.json').read_text())
    if config['geometry_sha256']!=sha(asset):raise ValueError('Source catalog binding differs')
    roles={}
    for field,colour in usage_fields(doc):roles.setdefault(field['index'],set()).add(colour)
    if set(roles)!=set(range(len(doc.get('textures',[])))) or any(len(v)!=1 for v in roles.values()):raise ValueError('Qualify all source texture roles without linear/sRGB ambiguity before sharing')
    packed=bytearray(binary)
    # Normalize the already-bound external PNGs in memory for the existing byte-
    # preserving extraction helper. No intermediate source/package is rewritten.
    for image in doc.get('images',[]):
        if 'uri' not in image:continue
        uri=image['uri']
        if 'bufferView' in image or not re.fullmatch(r'shared/[A-Za-z0-9_.-]+\.png',uri):raise ValueError('Unqualified catalog image path')
        path=(source/uri).resolve(strict=True)
        if not path.is_relative_to(source) or not 1<=path.stat().st_size<=MAX_IMAGE_BYTES:raise ValueError('Source image escapes stage or exceeds budget')
        data=path.read_bytes();packed.extend(b'\0'*(-len(packed)%4));image['bufferView']=len(doc['bufferViews']);doc['bufferViews'].append({'buffer':0,'byteOffset':len(packed),'byteLength':len(data)});packed.extend(data);image.pop('uri');image['mimeType']='image/png'
    doc['buffers'][0]['byteLength']=len(packed);packed.extend(b'\0'*(-len(packed)%4))
    output.mkdir(parents=True);(output/'shared').mkdir()
    try:
        binary,descriptors=extract_document(doc,packed,output/'shared')
        canonical_doc=copy.deepcopy(doc)
        for image in doc['images']:
            if not image['uri'].startswith('../shared/'):raise ValueError('Unexpected extracted image URI')
            image['uri']=image['uri'][3:]
        write_glb(doc,binary,output/'preview.glb')
        if collect(canonical_doc,output,allow_material_extensions=True)!=descriptors:raise ValueError('Extracted image variants differ')
        changed=copy.deepcopy(config);changed['geometry_sha256']=sha(output/'preview.glb');(output/'triplanar.json').write_text(json.dumps(changed,indent=2)+'\n')
        custom={t['file']:t['sha256'] for p in config['profiles'] for t in p['textures'].values()}
        for name,digest in custom.items():
            if not re.fullmatch(r'triplanar-(?:color|normal)-[0-9a-f]{64}\.png',name) or sha(source/name)!=digest:raise ValueError('Custom profile image binding differs')
            shutil.copyfile(source/name,output/name)
        result={'schema':1,'profile':'forest-shared-images-inspector-v1','source':str(source),'source_asset_sha256':sha(asset),'source_catalog_receipt_sha256':sha(source/'catalog-assembly.json'),
            'source_metadata_sha256':sha(source/'triplanar.json'),'asset_sha256':changed['geometry_sha256'],'metadata_sha256':sha(output/'triplanar.json'),
            'source_images':checked['images'],'shared_images':descriptors,'custom_images':custom,
            'scope':'immutable PNG byte/color-space/sampler sharing for known static inspector materials; no runtime package admission, mips, rebase or capacity claim'}
        (output/'resources.json').write_text(json.dumps(result,indent=2)+'\n');return result
    except BaseException:
        shutil.rmtree(output);raise


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('source',type=Path);p.add_argument('output',type=Path);a=p.parse_args();r=prepare(a.source,a.output);print(json.dumps({'passed':True,'asset_sha256':r['asset_sha256'],'source_images':r['source_images'],'shared_images':len(r['shared_images'])}))
