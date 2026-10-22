"""Independent raw-source eligibility and whole-image HDR emission oracle."""
import hashlib
import io
import math
import numpy as np
from PIL import Image
from verify_unity_mixed import properties


def source(fields, conversion, shader_hash):
    enabled=conversion.get('profile',{}).get('profile')=='synty-full-emissive-preview-v1'
    if not enabled:return False
    if shader_hash!='87a2b6f6206262674de9b2ff05d3743ee2a62e0d1bc7eff5b06575ab4c56abe4':raise ValueError('Emissive source shader differs')
    saved=fields['m_SavedProperties'];f=properties(saved['m_Floats']);c=properties(saved['m_Colors']);t=properties(saved['m_TexEnvs']);profile=conversion['profile']
    if any(t[prefix+'NormalMap']['m_Texture']['fileID'] and f[prefix+'NormalAmount'] for prefix in ['_Leaf','_Trunk']):raise ValueError('Emissive preview requires no active source normal maps')
    if profile['source_properties']!=saved or not conversion['canonical_mesh'] or not conversion['scene_binding']:raise ValueError('Emissive source properties/draw record differs')
    if any(type(v) not in (int,float) or not math.isfinite(v) for v in f.values()) or not 0<f['_EmissiveAmount']<=2 or f['_FrostingSwitch']!=0:raise ValueError('Unqualified emissive amount/frost')
    for key in ['_EmissiveMask','_Emissive2Mask','_EmissivePulseMap','_TrunkEmissiveMask']:
        if t[key]['m_Scale']!={'x':1,'y':1} or t[key]['m_Offset']!={'x':0,'y':0}:raise ValueError('Emissive source UV transform differs')
    if any(t[key]['m_Texture'].get('fileID')!=2800000 for key in ['_EmissiveMask','_Emissive2Mask']) or any(t[key]['m_Texture'].get('fileID') for key in ['_EmissivePulseMap','_TrunkEmissiveMask']) or any(c['_TrunkEmissiveColour'][axis]!=0 for axis in 'rgb'):
        raise ValueError('Emissive default-white pulse/zero-trunk profile differs')
    if any(type(v) not in (int,float) or not math.isfinite(v) or not 0<=v<=1 for key in ['_EmissiveColour','_Emissive2Colour'] for v in [c[key][axis] for axis in 'rgb']):raise ValueError('Emissive colours exceed SDR source profile')
    for key in ['_LeafBaseColour','_LeafNoiseColour','_LeafNoiseLargeColour','_TrunkBaseColour','_TrunkNoiseColour']:
        if any(type(c[key][axis]) not in (int,float) or not math.isfinite(c[key][axis]) or not 0<=c[key][axis]<=1 for axis in 'rgb'):raise ValueError('Unbounded original spatial colour record')
    e=profile['emission']
    if e['amount']!=f['_EmissiveAmount'] or e['strength']!=2*f['_EmissiveAmount'] or e['masks']!={key:t[key]['m_Texture'] for key in ['_EmissiveMask','_Emissive2Mask']} or e['colours']!={key:[c[key][axis] for axis in 'rgb'] for key in ['_EmissiveColour','_Emissive2Colour']}:raise ValueError('Emissive profile record differs')
    if profile['omitted_spatial_noise']!={key:c[key] for key in ['_LeafNoiseColour','_LeafNoiseLargeColour','_TrunkNoiseColour']}:raise ValueError('Omitted spatial noise record differs')
    return True


def material(sources, fields, branch, m, doc, binary):
    if branch=='trunk':
        if m.get('emissiveFactor',[0,0,0])!=[0,0,0] or 'emissiveTexture' in m or m.get('extensions'):raise ValueError('Unexpected trunk emission')
        return set(),{'emission':'zero source trunk'}
    saved=fields['m_SavedProperties'];f=properties(saved['m_Floats']);c=properties(saved['m_Colors']);t=properties(saved['m_TexEnvs'])
    if doc.get('extensionsUsed')!=['KHR_materials_emissive_strength'] or doc.get('extensionsRequired')!=['KHR_materials_emissive_strength'] or m.get('extensions')!={'KHR_materials_emissive_strength':{'emissiveStrength':2*f['_EmissiveAmount']}} or m.get('emissiveFactor')!=[1,1,1]:raise ValueError('HDR emissive strength/factor differs')
    entry=m['emissiveTexture']
    if set(entry)!={'index','texCoord'} or entry['texCoord']!=0:raise ValueError('Emissive UV channel differs')
    texture=doc['textures'][entry['index']];sampler=doc['samplers'][texture['sampler']]
    if [sampler.get(k,v) for k,v in [('magFilter',9729),('minFilter',9987),('wrapS',10497),('wrapT',10497)]]!=[9729,9985,10497,10497]:raise ValueError('Emissive sampler differs')
    masks=[];inputs=[]
    for key in ['_EmissiveMask','_Emissive2Mask']:
        name=sources.by_guid[t[key]['m_Texture']['guid']];path=sources.checked(name);importer=sources.metadata(name)['TextureImporter'];sampling=importer['textureSettings']
        if importer['textureType']!=0 or importer['mipmaps']['sRGBTexture']!=1 or importer['mipmaps']['enableMipMap']!=1 or [sampling[k] for k in ('filterMode','wrapU','wrapV','aniso','mipBias')]!=[1,0,0,1,0]:raise ValueError('Emissive source sampling differs')
        with Image.open(path) as original:
            if max(original.size)>4096:raise ValueError('Emissive source dimensions exceed budget')
            masks.append(np.asarray(original.convert('RGBA'))[:,:,0])
        inputs.append({'source':name,'sha256':hashlib.sha256(path.read_bytes()).hexdigest()})
    if masks[0].shape!=masks[1].shape:raise ValueError('Emissive source dimensions differ')
    decode=np.array([v/255/12.92 if v/255<=.04045 else ((v/255+.055)/1.055)**2.4 for v in range(256)])
    expected=np.zeros((*masks[0].shape,3),dtype=float)
    for mask,key in zip(masks,['_EmissiveColour','_Emissive2Colour']):expected+=decode[mask][:,:,None]*np.array([c[key][axis] for axis in 'rgb'])/2
    encoded=np.where(expected<=.0031308,expected*12.92,1.055*expected**(1/2.4)-.055);wanted=np.rint(encoded*255).astype(np.uint8)
    image=doc['images'][texture['source']];view=doc['bufferViews'][image['bufferView']];offset=view.get('byteOffset',0)
    with Image.open(io.BytesIO(binary[offset:offset+view['byteLength']])) as actual:
        if actual.format!='PNG' or actual.mode!='RGB' or actual.size!=tuple(reversed(masks[0].shape)) or not np.array_equal(np.asarray(actual),wanted):raise ValueError('Derived source HDR emissive pixels differ')
    return {texture['source']},{'emission_pixels_exact':True,'emission_dimensions':list(reversed(masks[0].shape)),'emissive_strength':2*f['_EmissiveAmount'],'source_masks':inputs,'source_pulse':'default white forces multiplier one; no time dependence for this binding'}
