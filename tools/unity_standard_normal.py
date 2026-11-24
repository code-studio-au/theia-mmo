"""Explicit static Standard normal-map adaptation; ordinary glTF output."""
import io
import math
import numpy as np
from PIL import Image


def sampler(sources, name, reference):
    meta = sources.metadata(name)['TextureImporter']
    settings = meta['textureSettings']
    values = [settings.get(k) for k in ('filterMode','wrapU','wrapV','aniso','mipBias')]
    if values == [-1]*5:
        if reference is None:
            raise ValueError('Inherited sampler settings require an actual Unity texture reference')
        matches = [v for v in reference['textures'] if v['path'] == name]
        if len(matches) != 1:
            raise ValueError('Texture reference does not cover this image')
        value = matches[0]
        values = [value[k] for k in ('filter','wrap_u','wrap_v','aniso','mip_bias')]
        if value['importer_type'] != meta['textureType'] or value['srgb'] != bool(meta['mipmaps']['sRGBTexture']) or not value['mipmaps'] or value['convert_normal']:
            raise ValueError('Effective texture importer differs from supported source policy')
    if values != [1,0,0,1,0] or meta['mipmaps'].get('enableMipMap') != 1 or meta.get('bumpmap',{}).get('convertToNormalMap'):
        raise ValueError('Unsupported Standard texture sampler or conversion')
    return meta


def pack_normal(document, binary, png):
    packed=bytearray(binary);packed.extend(b'\0'*(-len(packed)%4))
    view=len(document['bufferViews']);document['bufferViews'].append({'buffer':0,'byteOffset':len(packed),'byteLength':len(png)})
    packed.extend(png);packed.extend(b'\0'*(-len(packed)%4))
    image=len(document['images']);document['images'].append({'bufferView':view,'mimeType':'image/png','name':'source-normal'})
    material,=document['materials'];base=document['textures'][material['pbrMetallicRoughness']['baseColorTexture']['index']]
    texture=len(document['textures']);document['textures'].append({'source':image,'sampler':base['sampler']})
    material['normalTexture']={'index':texture}
    document['buffers'][0]['byteLength']=len(packed)
    return bytes(packed)


def bake_normal(sources, material_plan, reference):
    from unity_full_foliage import normal_rg
    name=sources.by_guid[material_plan['normal_texture']['guid']]
    meta=sampler(sources,name,reference)
    if meta['textureType'] != 1 or meta['mipmaps']['sRGBTexture'] != 0:
        raise ValueError('Standard normal adaptation needs a linear imported normal map')
    with Image.open(sources.checked(name)) as image:
        if image.format!='PNG' or max(image.size)>4096:
            raise ValueError('Normal map exceeds bounded PNG profile')
        result=normal_rg(np.asarray(image.convert('RGBA')),material_plan['normal_amount'],False)
    stream=io.BytesIO();Image.fromarray(result).save(stream,format='PNG')
    return name,stream.getvalue()


def verify_normal_pixels(original, embedded, amount):
    """Independent vector oracle; does not call the converter's normal_rg."""
    if type(amount) not in (int, float) or not math.isfinite(amount) or not 0<=amount<=1:
        raise ValueError('Invalid reference normal amount')
    with Image.open(original) as source, Image.open(io.BytesIO(embedded)) as image:
        if source.size != image.size or max(source.size)>4096:
            raise ValueError('Normal map dimensions differ')
        raw=np.asarray(source.convert('RGBA'),dtype=np.float64)/255.
        x=(raw[:,:,0]*raw[:,:,3]*2.-1.)*amount;y=(raw[:,:,1]*2.-1.)*amount
        z=np.sqrt(np.maximum(0.,1.-x*x-y*y))
        length=np.sqrt(x*x+y*y+z*z)
        expected=np.stack([x/length,y/length,z/length],axis=2)
        actual=np.asarray(image.convert('RGB'),dtype=np.float64)/127.5-1.
        actual/=np.linalg.norm(actual,axis=2)[:,:,None]
        error=float(np.linalg.norm(actual-expected,axis=2).max())
        if error>.007:raise ValueError('Normal reconstruction exceeds RGB8 vector bound')
        return {'source_dimensions':list(source.size),'maximum_vector_error':error,'limit':.007}
