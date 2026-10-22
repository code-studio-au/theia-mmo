"""Opt-in static emissive foliage preview, retaining source effect limitations."""
import copy
import math
import numpy as np
from unity_full_foliage import full_material
from unity_materials import properties


def material(fields, digest):
    saved=fields['m_SavedProperties'];f=properties(saved['m_Floats']);c=properties(saved['m_Colors']);t=properties(saved['m_TexEnvs'])
    amount=f['_EmissiveAmount']
    if type(amount) not in (int,float) or not math.isfinite(amount) or not 0<amount<=2 or f['_FrostingSwitch']!=0:
        raise ValueError('Unsupported bounded emissive foliage amount/frost')
    if t['_EmissivePulseMap']['m_Texture'].get('fileID') or t['_TrunkEmissiveMask']['m_Texture'].get('fileID') or any(c['_TrunkEmissiveColour'][axis]!=0 for axis in 'rgb'):
        raise ValueError('Only default-white pulse and zero trunk emission are qualified')
    for key in ['_EmissiveMask','_Emissive2Mask','_EmissivePulseMap','_TrunkEmissiveMask']:
        if t[key]['m_Scale']!={'x':1,'y':1} or t[key]['m_Offset']!={'x':0,'y':0}:
            raise ValueError('Emissive mask UV transform needs separate qualification')
    for key in ['_EmissiveMask','_Emissive2Mask']:
        if t[key]['m_Texture'].get('fileID')!=2800000:raise ValueError('Explicit leaf emissive masks required')
    colours={key:[c[key][axis] for axis in 'rgb'] for key in ['_EmissiveColour','_Emissive2Colour']}
    if any(type(v) not in (int,float) or not math.isfinite(v) or not 0<=v<=1 for rgb in colours.values() for v in rgb):
        raise ValueError('Unsupported emissive foliage colour')
    for key in ['_LeafBaseColour','_LeafNoiseColour','_LeafNoiseLargeColour','_TrunkBaseColour','_TrunkNoiseColour']:
        if any(type(c[key][axis]) not in (int,float) or not math.isfinite(c[key][axis]) or not 0<=c[key][axis]<=1 for axis in 'rgb'):raise ValueError('Unbounded original spatial colour record')
    # Reuse the qualified base-floor/surface constructor with an explicit surrogate;
    # original varying noise and emissive properties stay in the final profile.
    surrogate=copy.deepcopy(fields)
    for row in surrogate['m_SavedProperties']['m_Floats']:
        if '_EmissiveAmount' in row:row['_EmissiveAmount']=0
    for row in surrogate['m_SavedProperties']['m_Colors']:
        for noise,base in [('_LeafNoiseColour','_LeafBaseColour'),('_LeafNoiseLargeColour','_LeafBaseColour'),('_TrunkNoiseColour','_TrunkBaseColour')]:
            if noise in row:row[noise]=copy.deepcopy(c[base])
    result=full_material(surrogate,digest)
    if any(branch['normal_texture']['fileID'] and branch['normal_amount'] for branch in result['branches'].values()):raise ValueError('Emissive preview currently requires no active normal maps')
    result.update(profile='synty-full-emissive-preview-v1',source_properties=saved,
        emission={'amount':amount,'strength':2*amount,'colours':colours,'masks':{key:t[key]['m_Texture'] for key in ['_EmissiveMask','_Emissive2Mask']},'pulse':'source shader default white forces multiplier one'},
        omitted_spatial_noise={key:c[key] for key in ['_LeafNoiseColour','_LeafNoiseLargeColour','_TrunkNoiseColour']})
    result['approximations'].append('explicit base-colour-floor preview omits spatial albedo noise; static default-white pulse, quantized normalized HDR emission through standard glTF strength')
    return result


def pixels(first, second, colours):
    a=np.asarray(first);b=np.asarray(second)
    if a.dtype!=np.uint8 or b.dtype!=np.uint8 or a.ndim!=3 or a.shape!=b.shape or a.shape[2]!=4 or a.shape[0]*a.shape[1]>4096**2:
        raise ValueError('Emissive masks require equal bounded RGBA8 dimensions')
    basis=np.asarray([colours['_EmissiveColour'],colours['_Emissive2Colour']],dtype=float)
    if basis.shape!=(2,3) or not np.isfinite(basis).all() or (basis<0).any() or (basis>1).any():raise ValueError('Invalid bounded emissive colours')
    masks=np.stack([a[:,:,0],b[:,:,0]],axis=-1).astype(float)/255
    masks=np.where(masks<=.04045,masks/12.92,((masks+.055)/1.055)**2.4)
    # Strength = 2*amount, hence normalized emission cancels the common amount.
    rgb=(masks[:,:,0,None]*basis[0]+masks[:,:,1,None]*basis[1])/2
    encoded=np.where(rgb<=.0031308,rgb*12.92,1.055*rgb**(1/2.4)-.055)
    return np.rint(np.clip(encoded,0,1)*255).astype(np.uint8)
