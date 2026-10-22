"""Explicit static normal-incidence PBR preview of one pinned crystal shader."""
import math
from unity_materials import properties

GUID='006c4dbb387622342a1397597c7b3e66'
SHA256='ca667b1dc83a0f02e1059694197ff954ee2b065b0e8c78601f35a42aad2d6aa7'


def material(fields,digest):
    if digest!=SHA256 or fields['m_Shader']!={'fileID':4800000,'guid':GUID,'type':3}:raise ValueError('Unqualified crystal source shader')
    if fields.get('m_ShaderKeywords') or fields.get('disabledShaderPasses') or fields.get('m_CustomRenderQueue',-1) not in (-1,3000):raise ValueError('Unsupported crystal source variant/pass')
    saved=fields['m_SavedProperties'];floats=properties(saved['m_Floats']);colours=properties(saved['m_Colors']);textures=properties(saved['m_TexEnvs'])
    def bounded(key,low,high):
        value=floats[key]
        if type(value) not in (int,float) or not math.isfinite(value) or not low<=value<=high:raise ValueError('Invalid bounded crystal scalar')
        return value
    falloff=bounded('_ColourFalloff',.0001,100)
    opacity=bounded('_Opacity',0,1)
    if not 0<opacity<1:raise ValueError('Crystal preview requires partial source opacity')
    metallic=bounded('_Metallic',0,1);smoothness=bounded('_Smoothness',0,1)
    refraction=bounded('_Refraction',0,1);chromatic=bounded('_ChromaticAberration',0,.3)
    for name in ['_ShallowColor','_DeepColor','_Emission']:
        if any(type(colours[name][c]) not in (int,float) or not math.isfinite(colours[name][c]) or not 0<=colours[name][c]<=1 for c in 'rgb'):raise ValueError('Unsupported crystal colour/HDR emission')
    weight=min(1,1/falloff)
    colour=[colours['_ShallowColor'][c]*(1-weight)+colours['_DeepColor'][c]*weight for c in 'rgb']
    active={'_ColourFalloff','_Opacity','_Metallic','_Smoothness','_Refraction','_ChromaticAberration'}
    return {'profile':'synty-crystal-normal-incidence-pbr-preview-v1','shader_sha256':digest,
            'base_color_factor':colour+[opacity],'metallic':metallic,'roughness':1-smoothness,
            'emissive_factor':[colours['_Emission'][c] for c in 'rgb'],'alpha_mode':'BLEND','double_sided':True,
            'unused_saved_float_keys':sorted(set(floats)-active),'unused_saved_texture_bindings':textures,
            'unapplied_refraction':refraction,'unapplied_chromatic_aberration':chromatic,
            'scope':'explicit constant normal-incidence tint through standard PBR; no source view-dependent colour, GrabPass/refraction/chromatic effect, final alpha, Unity colour-space or BRDF parity'}
