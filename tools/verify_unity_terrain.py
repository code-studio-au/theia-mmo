"""Independent source TerrainData/sample/patch/UV/colour-preview verification.

Does not import the terrain reader, baker or converter. Source patch extrema and
raw serialized height bytes provide additional independent normalization checks.
"""
import argparse
import hashlib
import io
import json
import math
from pathlib import Path
import struct
import numpy as np
from PIL import Image
from unity_scene import Sources,records
from merge_static_glb import read
from static_geometry import accessor,hierarchy


def sample(pixels,x,y,repeat=False):
    height,width=pixels.shape[:2]
    if not repeat:x=max(0,min(width-1,x));y=max(0,min(height-1,y))
    col=math.floor(x);row=math.floor(y);fx=x-col;fy=y-row
    result=np.zeros(pixels.shape[2],dtype=float)
    for dx,dy,weight in [(0,0,(1-fx)*(1-fy)),(1,0,fx*(1-fy)),(0,1,(1-fx)*fy),(1,1,fx*fy)]:
        ix=(col+dx)%width if repeat else min(col+dx,width-1)
        iy=(row+dy)%height if repeat else min(row+dy,height-1)
        result+=pixels[iy,ix]*weight
    return result


def verify(root,stage):
    import UnityPy
    sources=Sources(root);stage=Path(stage);report=json.loads((stage/'conversion.json').read_text())
    if report['archive_sha256']!=sources.manifest['archive_sha256']:raise ValueError('Terrain archive binding differs')
    for name,digest in report['source_bindings'].items():
        meta=name.endswith('.meta');base=name[:-5] if meta else name
        if hashlib.sha256(sources.checked(base,meta).read_bytes()).hexdigest()!=digest:raise ValueError('Terrain source binding differs')
    for name,digest in report['outputs'].items():
        if Path(name).name!=name or hashlib.sha256((stage/name).read_bytes()).hexdigest()!=digest:raise ValueError('Terrain output binding differs')
    rr=records(sources.checked(report['scene']));terrain=rr[report['terrain_id']]
    if terrain['kind']!=218 or terrain['fields']['m_TerrainData']['fileID']!=report['terrain_data_id']:raise ValueError('Terrain scene object binding differs')
    source_name=sources.by_guid[terrain['fields']['m_TerrainData']['guid']]
    if report['terrain']!=source_name:raise ValueError('Terrain source GUID binding differs')
    env=UnityPy.load(sources.checked(source_name).read_bytes());obj=next(o for o in env.objects if o.path_id==report['terrain_data_id'])
    data=obj.read_typetree();height=data['m_Heightmap'];n=height['m_Resolution'];source=np.array(height['m_Heights'],dtype=np.uint16).reshape(n,n)
    packed=struct.pack('<I',n*n)+source.astype('<u2').tobytes();raw=obj.get_raw_data()
    if raw.count(packed)!=1:raise ValueError('Decoded height samples lack unique serialized raw-byte binding')
    if (stage/'heights.u16').read_bytes()!=packed[4:]:raise ValueError('Terrain raw height samples changed')
    extrema=[]
    for step in [2**level for level in range(4,int(math.log2(n-1))+1)]:
        for row in range(0,n-1,step):
            for col in range(0,n-1,step):
                patch=source[row:row+step+1,col:col+step+1]
                extrema.extend([int(patch.min())/32766.,int(patch.max())/32766.])
    patch_error=float(np.max(np.abs(np.array(extrema)-height['m_MinMaxPatchHeights'])))
    if patch_error>3e-8:raise ValueError('Independent source terrain patch extrema disagree')
    game_object=terrain['fields']['m_GameObject']['fileID'];transform,=[v['fields'] for v in rr.values() if v['kind']==4 and not v['stripped'] and v['fields']['m_GameObject']['fileID']==game_object]
    if transform['m_Father']['fileID'] or transform['m_LocalRotation']!={'x':0,'y':0,'z':0,'w':1} or transform['m_LocalScale']!={'x':1,'y':1,'z':1}:
        raise ValueError('Independent source transform exceeds root terrain profile')
    origin=[transform['m_LocalPosition'][axis] for axis in 'xyz'];scale=[height['m_Scale'][axis] for axis in 'xyz']
    z,x=np.mgrid[:n,:n];wanted=np.empty((n,n,3));wanted[:,:,0]=-origin[0]-x*scale[0];wanted[:,:,1]=origin[1]+source.astype(float)*(scale[1]/32766.);wanted[:,:,2]=origin[2]+z*scale[2]
    doc,binary=read(stage/'terrain.glb');_,world=hierarchy(doc)
    if len(doc['nodes'])!=1 or not np.array_equal(world[0],np.eye(4)):raise ValueError('Terrain must contain one identity mesh')
    primitive,=doc['meshes'][doc['nodes'][0]['mesh']]['primitives'];positions=accessor(doc,binary,primitive['attributes']['POSITION']);normals=accessor(doc,binary,primitive['attributes']['NORMAL']);uv=accessor(doc,binary,primitive['attributes']['TEXCOORD_0']);faces=accessor(doc,binary,primitive['indices']).reshape(-1,3)
    error=float(np.max(np.abs(positions-wanted.reshape(-1,3))))
    if positions.shape!=(n*n,3) or error>4e-6:raise ValueError('Terrain positions differ from all source height samples')
    if not np.isfinite(normals).all() or not (normals[:,1]>0).all() or np.max(np.abs(np.linalg.norm(normals,axis=1)-1))>1e-6:raise ValueError('Invalid geometric terrain normals')
    wanted_uv=np.column_stack([x.ravel()/(n-1),1-z.ravel()/(n-1)])
    if uv.shape!=wanted_uv.shape or np.max(np.abs(uv-wanted_uv))>1e-7:raise ValueError('Terrain UV orientation differs')
    top=(z[:-1,:-1]*n+x[:-1,:-1]).ravel();expected=np.column_stack([top,top+1,top+n+1,top,top+n+1,top+n]).reshape(-1,3)
    if not np.array_equal(expected,faces):raise ValueError('Terrain grid coverage/diagonal/winding differs')
    triangles=positions[faces]
    if not (np.cross(triangles[:,1]-triangles[:,0],triangles[:,2]-triangles[:,0])[:,1]>0).all():raise ValueError('Terrain triangle faces downward')
    alpha=[]
    for ref in data['m_SplatDatabase']['m_AlphaTextures']:
        texture=next(o for o in env.objects if o.path_id==ref['m_PathID']).read_typetree();width=texture['m_Width']
        alpha.append(np.frombuffer(texture['image data'][:width*width*4],dtype=np.uint8).reshape(width,width,4))
    controls=np.concatenate(alpha,axis=2)[:,:,:len(data['m_SplatDatabase']['m_TerrainLayers'])]
    if (stage/'weights.u8').read_bytes()!=controls.tobytes():raise ValueError('Terrain raw layer weights changed')
    layers=[]
    for i,ref in enumerate(data['m_SplatDatabase']['m_TerrainLayers']):
        raw_guid=obj.assets_file.externals[ref['m_FileID']-1].guid.hex();guid=''.join(raw_guid[k:k+2][::-1] for k in range(0,32,2));name=sources.by_guid[guid]
        layer=records(sources.checked(name))[ref['m_PathID']]['fields'];source_texture=sources.by_guid[layer['m_DiffuseTexture']['guid']]
        binding=report['layers'][i]
        if (binding['source'],binding['fileID'],binding['textures']['m_DiffuseTexture'])!=(name,ref['m_PathID'],source_texture):raise ValueError('Terrain layer GUID/fileID/texture binding differs')
        with Image.open(sources.checked(source_texture)) as image:pixels=np.array(image.convert('RGB'),dtype=float)[::-1]/255
        pixels=np.where(pixels<=.04045,pixels/12.92,((pixels+.055)/1.055)**2.4);layers.append((pixels,layer))
    image=doc['images'][0];view=doc['bufferViews'][image['bufferView']];offset=view.get('byteOffset',0);png=binary[offset:offset+view['byteLength']]
    if png!=(stage/'base-color.png').read_bytes():raise ValueError('Embedded base-colour image differs')
    with Image.open(io.BytesIO(png)) as image:baked=np.array(image.convert('RGB'));size=image.width
    rng=np.random.default_rng(20261020);samples={tuple(v) for v in rng.integers(0,size,size=(512,2))}
    samples.update((int(y),int(x)) for y in np.linspace(0,size-1,13) for x in np.linspace(0,size-1,13))
    for layer in range(controls.shape[2]):
        row,col=np.unravel_index(controls[:,:,layer].argmax(),controls.shape[:2]);samples.add((min(size-1,int((1-row/(width-1))*size)),min(size-1,int(col/(width-1)*size))))
    colour_error=0
    for row,col in sorted(samples):
        u=(col+.5)/size;v=1-(row+.5)/size;weights=sample(controls.astype(float)/255,u*(width-1),v*(width-1));value=np.zeros(3)
        for i,(pixels,layer) in enumerate(layers):
            tile=layer['m_TileSize'];offset=layer['m_TileOffset'];px=(u*(n-1)*scale[0]+offset['x'])/tile['x']*pixels.shape[1]-.5;py=(v*(n-1)*scale[2]+offset['y'])/tile['y']*pixels.shape[0]-.5
            value+=sample(pixels,px,py,True)*weights[i]
        encoded=np.array([c*12.92 if c<=.0031308 else 1.055*c**(1/2.4)-.055 for c in value]);expected=np.rint(np.clip(encoded,0,1)*255).astype(int)
        colour_error=max(colour_error,int(np.abs(expected-baked[row,col].astype(int)).max()))
    if colour_error>1:raise ValueError('Baked terrain colour differs from independent sampled source blend')
    return {'passed':True,'source_samples':n*n,'triangles':len(faces),'patches_checked':len(extrema)//2,
            'patch_extrema_max_error':patch_error,'max_position_error_metres':error,'sampled_colour_pixels':len(samples),'max_colour_channel_error':colour_error,
            'expected_bounds':[wanted.reshape(-1,3).min(0).tolist(),wanted.reshape(-1,3).max(0).tolist()],
            'asset_sha256':hashlib.sha256((stage/'terrain.glb').read_bytes()).hexdigest(),
            'scope':'all source heights/raw bytes/patches, fixed-grid UV/winding and sampled baked colour; no Unity tessellation/normal-map/collision/full scene equivalence'}


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('source',type=Path);parser.add_argument('stage',type=Path);parser.add_argument('--report',required=True,type=Path);args=parser.parse_args()
    if args.report.exists():raise FileExistsError('Use a new report')
    result=verify(args.source,args.stage);args.report.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result))
