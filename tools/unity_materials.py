"""Explicit, bounded Unity material adaptations for ordinary glTF/Bevy PBR.

This is a source-property contract, not Unity shader execution or BRDF parity.
Unsupported active features reject rather than silently disappearing.
"""
import math


def properties(rows):
    result = {}
    for row in rows:
        if not isinstance(row, dict) or len(row) != 1:
            raise ValueError('Malformed material property')
        key, value = next(iter(row.items()))
        if key in result:
            raise ValueError('Duplicate material property')
        result[key] = value
    return result


def standard_material(fields):
    shader = fields['m_Shader']
    if shader.get('guid') != '0000000000000000f000000000000000' or shader.get('fileID') != 46:
        raise ValueError('Expected explicitly identified built-in Standard shader')
    saved = fields['m_SavedProperties']
    textures = properties(saved['m_TexEnvs'])
    colors = properties(saved['m_Colors'])
    floats = properties(saved['m_Floats'])
    if (floats.get('_Mode') != 0 or fields.get('m_CustomRenderQueue', -1) not in (-1, 2000)
            or fields.get('disabledShaderPasses') or fields.get('m_DoubleSidedGI', 0)):
        raise ValueError('Only opaque default-pass Standard material is qualified')
    keywords = set(fields.get('m_ShaderKeywords', '').split())
    if keywords - {'_EMISSION'}:
        raise ValueError('Unsupported Standard shader keyword')
    if floats.get('_SpecularHighlights', 1) != 1 or floats.get('_GlossyReflections', 1) != 1:
        raise ValueError('Disabled Standard reflection features are unsupported')
    if any(floats.get(key, expected) != expected for key, expected in
           [('_SrcBlend', 1), ('_DstBlend', 0), ('_ZWrite', 1)]):
        raise ValueError('Custom Standard blend/depth settings are unsupported')
    if floats.get('_SmoothnessTextureChannel', 0) != 0:
        raise ValueError('Albedo-alpha smoothness needs explicit texture conversion')
    if colors.get('_Color') != dict(r=1, g=1, b=1, a=1):
        raise ValueError('Nonwhite Standard tint needs qualified colour conversion')
    for key, texture in textures.items():
        if texture['m_Texture'].get('fileID', 0) and key not in ('_MainTex', '_EmissionMap'):
            raise ValueError(f'Unsupported active Standard texture: {key}')
    def texture(key, required):
        value = textures.get(key)
        if value is None or not value['m_Texture'].get('fileID', 0):
            if required:
                raise ValueError(f'Missing required Standard texture: {key}')
            return None
        if (value['m_Scale'] != dict(x=1, y=1) or value['m_Offset'] != dict(x=0, y=0)
                or value['m_Texture'].get('fileID') != 2800000
                or not isinstance(value['m_Texture'].get('guid'), str)):
            raise ValueError('Unsupported Standard texture transform/reference')
        return dict(value['m_Texture'])
    metallic, smoothness = floats['_Metallic'], floats['_Glossiness']
    for value in (metallic, smoothness):
        if isinstance(value, bool) or not isinstance(value, (int, float)) or not math.isfinite(value) or not 0 <= value <= 1:
            raise ValueError('Standard surface scalar must be finite in [0,1]')
    emission = [0., 0., 0.]
    emissive_texture = None
    if '_EMISSION' in keywords:
        color = colors['_EmissionColor']
        # White/black avoid an unqualified Unity colour-space or HDR adaptation.
        if color not in (dict(r=1, g=1, b=1, a=1), dict(r=0, g=0, b=0, a=1)):
            raise ValueError('Tinted/HDR emission needs qualified colour conversion')
        emission = [color[k] for k in ('r', 'g', 'b')]
        emissive_texture = texture('_EmissionMap', False)
    return {'profile': 'unity-standard-opaque-v1',
            'base_color': texture('_MainTex', True), 'base_color_factor': [1., 1., 1., 1.],
            'metallic': metallic, 'roughness': 1. - smoothness,
            'emissive': emissive_texture, 'emissive_factor': emission,
            'scope': 'source scalar/albedo/emission adaptation; no Unity BRDF, lightmap or shader equivalence'}


def glass_material(fields):
    """Explicit untextured Standard-premultiplied to portable glTF BLEND adaptation."""
    shader=fields['m_Shader'];saved=fields['m_SavedProperties']
    if shader!={'fileID':46,'guid':'0000000000000000f000000000000000','type':0}:raise ValueError('Glass requires the built-in Standard shader')
    floats=properties(saved['m_Floats']);colors=properties(saved['m_Colors']);textures=properties(saved['m_TexEnvs'])
    if fields.get('m_ShaderKeywords')!='_ALPHAPREMULTIPLY_ON' or fields.get('m_CustomRenderQueue')!=3000 or fields.get('disabledShaderPasses') or fields.get('m_DoubleSidedGI',0):raise ValueError('Unsupported glass variant/pass')
    for key,wanted in [('_Mode',3),('_SrcBlend',1),('_DstBlend',10),('_ZWrite',0),('_SpecularHighlights',1),('_GlossyReflections',1),('_SmoothnessTextureChannel',0)]:
        if floats.get(key)!=wanted:raise ValueError('Unsupported glass blend/depth/reflection state')
    if any(t['m_Texture'].get('fileID') for t in textures.values()) or colors.get('_EmissionColor')!=dict(r=0,g=0,b=0,a=1):raise ValueError('Textured/emissive glass requires separate adaptation')
    color=[colors['_Color'][k] for k in 'rgba'];surface=[floats['_Metallic'],floats['_Glossiness']]
    if any(type(v) not in (int,float) or not math.isfinite(v) or not 0<=v<=1 for v in color+surface) or not 0<color[3]<1:raise ValueError('Invalid glass colour/surface scalar')
    return {'profile':'unity-standard-glass-blend-v1','base_color':None,'base_color_factor':color,
            'metallic':surface[0],'roughness':1-surface[1],'emissive':None,'emissive_factor':[0.,0.,0.],
            'scope':'explicit linear-tint glTF BLEND approximation of untextured Standard premultiplied glass; no Unity colour-space/BRDF/premultiplied-specular or refraction parity'}


def standard_sampler_glb(path):
    """Preserve Blender BIN bytes, setting qualified repeat/bilinear mip sampling.

    Blender's default Linear export uses trilinear minification. Unity Bilinear
    samples the nearest mip, so this explicit finalization is required.
    """
    import json
    import struct
    from merge_static_glb import read
    document, binary = read(path)
    if not document.get('samplers'):
        raise ValueError('Expected exported texture samplers')
    for sampler in document['samplers']:
        sampler.update(magFilter=9729, minFilter=9985, wrapS=10497, wrapT=10497)
    text = json.dumps(document, separators=(',', ':')).encode()
    text += b' ' * (-len(text) % 4)
    path.write_bytes(struct.pack('<4sII', b'glTF', 2, 28 + len(text) + len(binary))
                     + struct.pack('<I4s', len(text), b'JSON') + text
                     + struct.pack('<I4s', len(binary), b'BIN\0') + binary)
