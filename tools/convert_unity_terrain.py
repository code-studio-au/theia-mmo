"""Convert source terrain geometry and an explicit baked base-colour preview.

Preserves all height samples; regular grid diagonals and geometric normals are
our static representation. Baked colour is an approximation, not source shader,
normal-map, lighting, collision or full scene equivalence.
"""
import argparse
import hashlib
import io
import json
from pathlib import Path
import struct
import numpy as np
from PIL import Image
from unity_terrain import read_terrain,HEIGHT_MAX


def linear(value):
    return np.where(value<=.04045,value/12.92,((value+.055)/1.055)**2.4)


def srgb(value):
    return np.where(value<=.0031308,value*12.92,1.055*np.maximum(value,0)**(1/2.4)-.055)


def bilinear(array,x,y,repeat=False):
    height,width=array.shape[:2]
    if not repeat:x=np.clip(x,0,width-1);y=np.clip(y,0,height-1)
    ix=np.floor(x).astype(np.int64);iy=np.floor(y).astype(np.int64);fx=(x-ix)[...,None];fy=(y-iy)[...,None]
    if repeat:x0=ix%width;x1=(ix+1)%width;y0=iy%height;y1=(iy+1)%height
    else:x0=ix;x1=np.minimum(ix+1,width-1);y0=iy;y1=np.minimum(iy+1,height-1)
    return ((1-fx)*array[y0,x0]+fx*array[y0,x1])*(1-fy)+((1-fx)*array[y1,x0]+fx*array[y1,x1])*fy


def bake_base_color(terrain,size):
    if type(size) is not int or size not in (512,1024,2048,4096,8192):raise ValueError('Unsupported bounded base-colour bake size')
    sources=terrain['sources'];layers=[]
    for layer in terrain['layers']:
        f=layer['fields']
        if f['m_Metallic']!=0 or f['m_Smoothness']!=0 or f['m_MaskMapTexture']['fileID'] or any(f['m_Specular'][k]!=0 for k in 'rgba'):
            raise ValueError('Terrain preview profile requires rough nonmetallic layers without mask/specular features')
        if f['m_DiffuseRemapMin']!={k:0 for k in 'xyzw'} or f['m_DiffuseRemapMax']!={k:1 for k in 'xyzw'}:
            raise ValueError('Unsupported terrain diffuse remapping')
        tile=np.array([f['m_TileSize'][k] for k in 'xy']);offset=np.array([f['m_TileOffset'][k] for k in 'xy'])
        if not np.isfinite([tile,offset]).all() or (tile<=0).any() or np.abs([tile,offset]).max()>4096:raise ValueError('Invalid terrain texture tiling')
        name=layer['textures']['m_DiffuseTexture'];meta=sources.metadata(name)['TextureImporter'];settings=meta['textureSettings'];mips=meta['mipmaps']
        if meta['textureType']!=0 or (settings['filterMode'],settings['wrapU'],settings['wrapV'],mips['sRGBTexture'])!=(1,0,0,1):
            raise ValueError('Expected repeat/bilinear sRGB terrain diffuse texture')
        with Image.open(sources.checked(name)) as image:
            if max(image.size)>4096:raise ValueError('Terrain layer image exceeds dimension budget')
            pixels=linear(np.asarray(image.convert('RGB'),dtype=np.float32)[::-1]/255)
        layers.append((pixels,tile,offset))
    result=np.empty((size,size,3),dtype=np.uint8);extent=(terrain['heights'].shape[0]-1)*terrain['scale'][[0,2]]
    weights=terrain['weights'].astype(np.float32)/255;u=(np.arange(size)[None,:]+.5)/size
    for row in range(0,size,32):
        v=1-(np.arange(row,min(size,row+32))[:,None]+.5)/size
        control=bilinear(weights,u*(weights.shape[1]-1),v*(weights.shape[0]-1))
        colour=np.zeros((len(v),size,3),dtype=float)
        for index,(pixels,tile,offset) in enumerate(layers):
            x=(u*extent[0]+offset[0])/tile[0]*pixels.shape[1]-.5
            y=(v*extent[1]+offset[1])/tile[1]*pixels.shape[0]-.5
            colour+=bilinear(pixels,x,y,repeat=True)*control[:,:,index,None]
        result[row:row+len(v)]=np.rint(np.clip(srgb(colour),0,1)*255).astype(np.uint8)
    return result


def geometry(heights,scale,origin):
    n=heights.shape[0];z,x=np.mgrid[:n,:n];elevation=heights.astype(float)/HEIGHT_MAX*scale[1]
    positions=np.stack([-(origin[0]+x*scale[0]),origin[1]+elevation,origin[2]+z*scale[2]],axis=-1)
    dz,dx=np.gradient(elevation,scale[2],scale[0]);normals=np.stack([dx,np.ones_like(dx),-dz],axis=-1);normals/=np.linalg.norm(normals,axis=-1)[...,None]
    uv=np.stack([x/(n-1),1-z/(n-1)],axis=-1)
    a=(z[:-1,:-1]*n+x[:-1,:-1]).ravel();b=a+1;c=a+n+1;d=a+n
    indices=np.stack([a,b,c,a,c,d],axis=1).reshape(-1,3)
    return positions.reshape(-1,3).astype('<f4'),normals.reshape(-1,3).astype('<f4'),uv.reshape(-1,2).astype('<f4'),indices.astype('<u4')


def write_glb(path,positions,normals,uv,indices,image_bytes):
    binary=bytearray();views=[];accessors=[]
    def view(data,target=None):
        binary.extend(b'\0'*(-len(binary)%4));item={'buffer':0,'byteOffset':len(binary),'byteLength':len(data)}
        if target:item['target']=target
        views.append(item);binary.extend(data);return len(views)-1
    def attribute(values,kind,component=5126):
        item={'bufferView':view(values.tobytes(),34962 if component==5126 else 34963),'componentType':component,'count':len(values),'type':kind}
        if kind=='VEC3':item.update(min=values.min(0).tolist(),max=values.max(0).tolist())
        accessors.append(item);return len(accessors)-1
    p=attribute(positions,'VEC3');normal=attribute(normals,'VEC3');tex=attribute(uv,'VEC2');faces=attribute(indices.ravel(),'SCALAR',5125)
    image=view(image_bytes)
    document={'asset':{'version':'2.0','generator':'Theia source TerrainData static geometry/base-colour preview'},'scene':0,'scenes':[{'nodes':[0]}],
              'nodes':[{'name':'source-terrain','mesh':0}],'meshes':[{'primitives':[{'attributes':{'POSITION':p,'NORMAL':normal,'TEXCOORD_0':tex},'indices':faces,'material':0}]}],
              'materials':[{'name':'baked source base colour; geometric normals','pbrMetallicRoughness':{'baseColorTexture':{'index':0},'metallicFactor':0,'roughnessFactor':1}}],
              'images':[{'bufferView':image,'mimeType':'image/png'}],'textures':[{'source':0,'sampler':0}],
              'samplers':[{'magFilter':9729,'minFilter':9987,'wrapS':33071,'wrapT':33071}],
              'buffers':[{'byteLength':len(binary)}],'bufferViews':views,'accessors':accessors}
    binary.extend(b'\0'*(-len(binary)%4));text=json.dumps(document,separators=(',',':'),allow_nan=False).encode();text+=b' '*(-len(text)%4)
    with Path(path).open('xb') as destination:
        destination.write(struct.pack('<4sII',b'glTF',2,28+len(text)+len(binary)))
        destination.write(struct.pack('<I4s',len(text),b'JSON')+text+struct.pack('<I4s',len(binary),b'BIN\0')+binary)


def convert(root,scene,output,size):
    output=Path(output)
    if output.exists():raise FileExistsError('Use a new terrain output directory')
    terrain=read_terrain(root,scene)
    if terrain['material_template']!={'fileID':10652,'guid':'0000000000000000f000000000000000','type':0}:
        raise ValueError('Unqualified terrain material template')
    colour=bake_base_color(terrain,size);image=io.BytesIO();Image.fromarray(colour).save(image,format='PNG')
    positions,normals,uv,indices=geometry(terrain['heights'],terrain['scale'],terrain['origin'])
    output.mkdir(parents=True);(output/'base-color.png').write_bytes(image.getvalue())
    (output/'heights.u16').write_bytes(terrain['heights'].tobytes());(output/'weights.u8').write_bytes(terrain['weights'].tobytes())
    write_glb(output/'terrain.glb',positions,normals,uv,indices,image.getvalue())
    sums=terrain['weights'].astype(np.uint16).sum(2)
    report={'schema':1,'archive_sha256':terrain['sources'].manifest['archive_sha256'],'scene':scene,'terrain':terrain['terrain'],
            'terrain_id':terrain['terrain_id'],'terrain_data_id':terrain['terrain_data_id'],'unity_version':'2020.3.40f1','unitypy_version':'1.25.0',
            'source_bindings':terrain['source_bindings'],'origin':terrain['origin'].tolist(),'scale':terrain['scale'].tolist(),
            'resolution':terrain['heights'].shape[0],'vertices':len(positions),'triangles':len(indices),
            'bounds':[positions.min(0).tolist(),positions.max(0).tolist()],
            'height_denominator':HEIGHT_MAX,'patch_extrema_max_error':terrain['patch_extrema_error'],
            'control_resolution':terrain['weights'].shape[0],'weight_sum_range':[int(sums.min()),int(sums.max())],
            'layers':terrain['layers'],'base_color_size':size,
            'outputs':{p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(output.iterdir())},
            'approximations':['finite-resolution linear weighted base-colour bake; not Unity terrain shader or lighting equivalence',
                              'derived geometric normals; authored normal maps retained as bindings but not rendered',
                              'fixed regular grid diagonal; not Unity adaptive tessellation or TerrainCollider parity'],
            'scope':'all source height samples and explicit base-colour preview; not complete forest, collision package or map adoption'}
    (output/'conversion.json').write_text(json.dumps(report,indent=2)+'\n');return report


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('source',type=Path);parser.add_argument('--scene',required=True);parser.add_argument('--output',required=True,type=Path)
    parser.add_argument('--base-color-size',type=int,default=4096);args=parser.parse_args()
    result=convert(args.source,args.scene,args.output,args.base_color_size)
    print(json.dumps({k:v for k,v in result.items() if k not in ('source_bindings','layers')}))
