"""Explicit static water PBR adaptation; source depth/refraction/foam remain separate."""
import math
from unity_materials import properties

SHADERS = {
    'a14ab9faa19eaeb4b8b41c78635b23bd': 'ab01bd729b9d7e76281573b27b72e22bfec1ca5e72047c554e7f7b3b4c4f702b',
    '276a39a5a210b124885f0eff3e80f1f3': 'd1798bf9cc2e6e4622cb1f6812c08f951778bcc380b8f0d0565fa37c0c7da6b8',
}


def material(fields, digest):
    shader = fields['m_Shader']; guid = shader.get('guid')
    if guid not in SHADERS or shader != {'fileID': 4800000, 'guid': guid, 'type': 3} or SHADERS[guid] != digest:
        raise ValueError('Unqualified water shader')
    keywords = fields.get('m_ShaderKeywords')
    if (keywords not in (None, '', '_GLOSSYREFLECTIONS_OFF _SPECULARHIGHLIGHTS_OFF') or
            fields.get('disabledShaderPasses') or fields.get('m_CustomRenderQueue', -1) != -1):
        raise ValueError('Unsupported water shader pass/variant')
    saved = fields['m_SavedProperties']; floats = properties(saved['m_Floats']); colours = properties(saved['m_Colors']); textures = properties(saved['m_TexEnvs'])
    if any(type(v) not in (int, float) or not math.isfinite(v) or abs(v) > 10000 for v in floats.values()):
        raise ValueError('Invalid bounded water scalar')
    if not 0 < floats['_Opacity'] <= 1 or not 0 <= floats['_OpacityMin'] <= 1 or floats['_OpacityFalloff'] != 1 or floats.get('_WaveAmplitude', 0) != 0:
        raise ValueError('Only static unit-depth opacity profile is qualified')
    if not 0 <= floats['_Smoothness'] <= 4 or not 0 <= floats['_ReflectionPower'] <= 1 or not 0 <= floats['_OverallFalloff'] <= 16 or abs(floats['_ShallowFalloff']) > 16:
        raise ValueError('Unsupported water surface/depth scalar')
    rgb = {k: [colours[k][c] for c in 'rgb'] for k in ['_ShallowColour', '_DeepColour', '_VeryDeepColour']}
    if any(type(v) not in (int, float) or not math.isfinite(v) or not 0 <= v <= 1 for values in rgb.values() for v in values):
        raise ValueError('Unsupported water colour')
    # Explicit representative sample: normalized depth = 1, before screen/foam effects.
    weight = 1 + floats['_ShallowFalloff']
    tint = [a + (b-a)*weight for a,b in zip(rgb['_ShallowColour'], rgb['_DeepColour'])] if weight < 1 else rgb['_DeepColour']
    if any(not 0 <= v <= 1 for v in tint): raise ValueError('Representative water tint escapes SDR')
    smoothness = min(1, max(0, floats['_Smoothness'] * floats['_ReflectionPower']))
    return {'profile': 'synty-unit-depth-static-water-v1', 'shader_sha256': digest,
            'base_color_factor': tint + [floats['_Opacity']], 'metallic': 0, 'roughness': 1-smoothness,
            'unapplied_textures': textures, 'source_properties': saved,
            'scope': 'static normalized-depth-one tint and one-metre opacity without foam; clamped water smoothness times reflection power; standard dielectric specular and geometry-derived flat normals; no source UV/normal, scrolling, refraction, depth, foam, caustics, waves or Unity BRDF parity'}
