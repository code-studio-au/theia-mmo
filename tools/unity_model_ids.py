"""Bounded FBX identity inspection and checked modern Unity ModelImporter IDs.

Derivation reference (Unity engineer's explanation, not a stable Unity API):
https://discussions.unity.com/t/fbx-submesh-fileids/803882
No geometry decompression or source execution. Generated candidates must match
actual serialized references before use. Older metadata schemes are rejected.
"""
import argparse
from collections import Counter
import json
from pathlib import Path
import struct

import xxhash
from unity_scene import Sources, records, references


MAX_BYTES = 128 * 1024 * 1024


def signed_hash(value):
    number = xxhash.xxh64_intdigest(value.encode('utf-8'))
    return number - (1 << 64) if number >= 1 << 63 else number


def identity(path):
    path=Path(path)
    if not 27 <= path.stat().st_size <= MAX_BYTES:raise ValueError('FBX outside byte budget')
    data=path.read_bytes()
    if data.startswith(b'version https://git-lfs.github.com/spec/v1\n'):
        raise ValueError('Source is a Git LFS pointer, not model payload')
    if data[:23]!=b'Kaydara FBX Binary  \x00\x1a\x00':raise ValueError('Expected binary FBX')
    version=struct.unpack_from('<I',data,23)[0]
    if version not in (7100,7200,7300,7400,7500,7600,7700):raise ValueError('Unsupported FBX version')
    fmt='<QQQB' if version>=7500 else '<IIIB';header=struct.calcsize(fmt)
    models,geometry,connections,documents={}, {}, [], set()
    object_types={}
    total=0

    def values(start,end,count):
        if not 0<=count<=64:raise ValueError('FBX scalar identity property budget')
        result=[];pos=start
        for _ in range(count):
            if pos>=end:raise ValueError('Truncated FBX property')
            kind=chr(data[pos]);pos+=1
            if kind in 'YCFDIL':
                code={'Y':'h','C':'?','F':'f','D':'d','I':'i','L':'q'}[kind];size=struct.calcsize('<'+code)
                if pos+size>end:raise ValueError('Truncated FBX scalar')
                value=struct.unpack_from('<'+code,data,pos)[0];pos+=size
            elif kind in 'SR':
                if pos+4>end:raise ValueError('Truncated FBX string length')
                size=struct.unpack_from('<I',data,pos)[0];pos+=4
                if size>65536 or pos+size>end:raise ValueError('FBX identity string outside budget')
                value=data[pos:pos+size].decode('utf-8') if kind=='S' else None;pos+=size
            else:raise ValueError('Expected scalar FBX identity properties')
            result.append(value)
        if pos!=end:raise ValueError('FBX identity property extent mismatch')
        return result

    def walk(start,end,depth=0,parent=''):
        nonlocal total
        pos=start
        while pos+header<=end:
            stop,count,size,length=struct.unpack_from(fmt,data,pos)
            if stop==0:
                if any(data[pos:pos+header]):raise ValueError('Malformed FBX null record')
                return
            total+=1
            prop=pos+header+length
            if total>262144 or depth>16 or not prop<=prop+size<=stop<=end or stop<=pos:
                raise ValueError('FBX node exceeds bounds/count/depth')
            name=data[pos+header:prop].decode('ascii')
            if parent=='Objects' and count and size>=9 and data[prop]==ord('L'):
                object_id=struct.unpack_from('<q',data,prop+1)[0]
                if object_id in object_types:raise ValueError('Duplicate FBX object ID')
                object_types[object_id]=name
            if parent=='Objects' and name in ('Model','Geometry'):
                vv=values(prop,prop+size,count)
                if len(vv)!=3 or type(vv[0]) is not int or not isinstance(vv[1],str) or not isinstance(vv[2],str):raise ValueError('Malformed FBX identity')
                dest=models if name=='Model' else geometry
                if vv[0] in dest:raise ValueError('Duplicate FBX object identity')
                dest[vv[0]]={'name':vv[1].split('\0')[0],'type':vv[2]}
            elif parent=='Connections' and name=='C':
                vv=values(prop,prop+size,count)
                if len(vv)<3:raise ValueError('Malformed FBX connection')
                if vv[0]=='OO':connections.append((vv[1],vv[2]))
            elif parent=='Documents' and name=='Document':
                vv=values(prop,prop+size,count);documents.add(vv[0])
            # Only recurse where identities live. Vertex arrays/compression and
            # unrelated FBX subtrees are deliberately outside this reader.
            if depth==0 and name in ('Objects','Connections','Documents'):
                walk(prop+size,stop,depth+1,name)
            pos=stop
    walk(27,len(data))
    if not models or set(models)&set(geometry):raise ValueError('Missing/overlapping FBX model identities')
    parents={};attachments={};declared={}
    for child,parent in connections:
        if child in models:
            if object_types.get(parent) in ('CollectionExclusive', 'Collection'):
                continue
            declared.setdefault(child,set()).add(parent)
        elif child in geometry and parent in models:
            attachments.setdefault(parent,[]).append(child)
    for model in models:
        targets=declared.get(model,set())
        unknown=targets-(set(models)|documents|{0})
        if not targets or unknown:
            raise ValueError('Unresolved FBX model parent: '+str({p:object_types.get(p) for p in sorted(unknown)}))
        # Document and Collection membership are separate from model parentage.
        hierarchy_parents=targets&set(models)
        if len(hierarchy_parents)>1:raise ValueError('Multiple FBX model parents')
        parents[model]=next(iter(hierarchy_parents)) if hierarchy_parents else 0
    roots=[k for k in models if parents[k] not in models]
    paths={};visiting=set()
    def model_path(key):
        if key in paths:return paths[key]
        if key in visiting or len(visiting)>128:raise ValueError('Cyclic/excessive FBX hierarchy')
        visiting.add(key);parent=parents[key]
        if parent not in models:
            value='//RootNode/root' if len(roots)==1 else '//RootNode/root/'+models[key]['name']
        else:value=model_path(parent)+'/'+models[key]['name']
        visiting.remove(key);paths[key]=value;return value
    for model in models:model_path(model)
    if len(set(paths.values()))!=len(paths):raise ValueError('Ambiguous duplicate FBX hierarchy paths')
    meshes={}
    for model,ids in attachments.items():
        if len(ids)!=1:raise ValueError('Multiple geometry attachments need explicit import contract')
        geom=geometry[ids[0]]
        if geom['type']!='Mesh':raise ValueError('Non-mesh FBX geometry')
        # Serialized references in this source use the connected Model name,
        # including FBXs whose Geometry is named merely Mesh.001 or Mesh.005.
        # This candidate remains unusable unless its ID matches the reference.
        mesh_name=models[model]['name']
        if mesh_name in meshes:raise ValueError('Ambiguous duplicate FBX mesh names')
        meshes[mesh_name]={'model':model,'geometry':ids[0]}
    candidates={}
    def add(kind,name,binding):
        value=signed_hash(f'Type:{kind}->{name}0')
        entry={'kind':kind,'name':name,**binding}
        if value in candidates and candidates[value]!=entry:raise ValueError('Unity candidate ID collision')
        candidates[value]=entry
    for model,path in paths.items():
        binding={'model':model}
        add('GameObject',path,binding);add('Transform',path+'/Transform',binding)
        if model in attachments:
            for kind in ('MeshRenderer','SkinnedMeshRenderer','MeshFilter'):
                add(kind,path+'/'+kind,binding)
        if parents[model] not in models:add('Animator',path+'/Animator',binding)
    if len(roots)>1:
        add('GameObject','//RootNode/root',{'model':None})
        add('Transform','//RootNode/root/Transform',{'model':None})
        add('Animator','//RootNode/root/Animator',{'model':None})
    for name,binding in meshes.items():add('Mesh',name,binding)
    return {'fbx_version':version,'models':models,'model_paths':paths,'meshes':meshes,'candidates':candidates}


def bound_candidates(model, importer):
    table=importer.get('internalIDToNameTable')
    if not table:
        if importer['meshes']['fileIdsGeneration']!=2:
            raise ValueError('Model metadata requires a separate ID mapping contract')
        return model['candidates'], 'modern-source-derived-candidates'
    if len(table)>65536:raise ValueError('Explicit model ID table exceeds budget')
    kinds={1:'GameObject',4:'Transform',23:'MeshRenderer',33:'MeshFilter',43:'Mesh',95:'Animator',137:'SkinnedMeshRenderer'}
    roots=[identifier for identifier,path in model['model_paths'].items() if path=='//RootNode/root']
    result={}
    for item in table:
        if not isinstance(item.get('first'),dict) or len(item['first'])!=1:
            raise ValueError('Malformed explicit model ID entry')
        cls,identifier=next(iter(item['first'].items()));name=item['second'];kind=kinds.get(cls)
        if kind is None or type(identifier) is not int or not -(1<<63)<=identifier<(1<<63) or identifier in result:
            raise ValueError('Unsupported/duplicate explicit model ID')
        if kind=='Mesh':
            if name not in model['meshes']:raise ValueError('Explicit model mesh name does not match source')
            binding=model['meshes'][name]
        elif name=='//RootNode' and len(roots)==1:
            binding={'model':roots[0]}
        else:raise ValueError('Explicit model hierarchy path needs separate mapping')
        result[identifier]={'kind':kind,'name':name,**binding}
    return result,'explicit-metadata-name-table'


def audit(source, dependency_report):
    sources=Sources(source);dependency=json.loads(Path(dependency_report).read_text())
    if dependency['archive_sha256']!=sources.manifest['archive_sha256']:raise ValueError('Dependency audit source differs')
    refs={}
    for name in dependency['documents']:
        for rr in records(sources.checked(name)).values():
            for ref,context in references(rr['fields'],rr['name']):
                target=sources.by_guid.get(ref['guid'])
                if target and target.lower().endswith('.fbx') and ref['fileID'] not in (0,100100000):
                    refs.setdefault(target,{}).setdefault(ref['fileID'],set()).add(context)
    results={};missing=[];matched=Counter();errors={}
    for name,wanted in sorted(refs.items()):
        meta=sources.metadata(name)['ModelImporter']
        try:
            model=identity(sources.checked(name))
            candidates,strategy=bound_candidates(model,meta)
        except ValueError as error:
            errors[name]=str(error)
            missing.extend({'model':name,'fileID':i,'contexts':sorted(c),'error':str(error)} for i,c in wanted.items())
            continue
        matches={}
        for identifier,contexts in wanted.items():
            candidate=candidates.get(identifier)
            if candidate:
                matches[str(identifier)]={**candidate,'contexts':sorted(contexts)};matched[candidate['kind']]+=1
            else:missing.append({'model':name,'fileID':identifier,'contexts':sorted(contexts)})
        results[name]={'models':len(model['models']),'meshes':len(model['meshes']),'strategy':strategy,'matches':matches}
    return {'schema':1,'scope':'source-derived ID candidates checked against unique serialized FBX references; not Unity Editor equivalence or scene instantiation',
            'archive_sha256':sources.manifest['archive_sha256'],'models':results,'matched_by_kind':dict(matched),
            'matched_references':sum(matched.values()),'unmatched':missing,'model_errors':errors,
            'derivation_reference':'https://discussions.unity.com/t/fbx-submesh-fileids/803882'}


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('source',type=Path)
    parser.add_argument('--dependencies',required=True,type=Path);parser.add_argument('--report',required=True,type=Path)
    args=parser.parse_args();result=audit(args.source,args.dependencies)
    with args.report.open('x') as output:json.dump(result,output,indent=2);output.write('\n')
    print(json.dumps({k:v for k,v in result.items() if k!='models'},indent=2))
