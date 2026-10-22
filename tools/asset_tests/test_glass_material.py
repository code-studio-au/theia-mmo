"""Portable glass keeps explicit source tint/opacity and rejects hidden features."""
from pathlib import Path
import sys
import unittest
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from unity_materials import glass_material,standard_material
from verify_unity_static import verify_glass


def source():
    return {'m_Shader':{'fileID':46,'guid':'0000000000000000f000000000000000','type':0},
            'm_ShaderKeywords':'_ALPHAPREMULTIPLY_ON','m_CustomRenderQueue':3000,
            'm_SavedProperties':{'m_TexEnvs':[{'_MainTex':{'m_Texture':{'fileID':0}}}],
                'm_Colors':[{'_Color':dict(r=.6,g=.07,b=.58,a=.51)},{'_EmissionColor':dict(r=0,g=0,b=0,a=1)}],
                'm_Floats':[{k:v} for k,v in [('_Mode',3),('_SrcBlend',1),('_DstBlend',10),('_ZWrite',0),('_SpecularHighlights',1),('_GlossyReflections',1),('_SmoothnessTextureChannel',0),('_Metallic',0),('_Glossiness',.813)]]}}


def material():
    return {'alphaMode':'BLEND','doubleSided':False,'pbrMetallicRoughness':{'baseColorFactor':[.6,.07,.58,.51],'metallicFactor':0,'roughnessFactor':.187}}


class GlassTests(unittest.TestCase):
    def test_explicit_profile_preserves_factors_and_rejects_unsupported_source_state(self):
        plan=glass_material(source());self.assertEqual(plan['base_color_factor'],[.6,.07,.58,.51]);self.assertIsNone(plan['base_color'])
        verify_glass(source(),{},material())
        with self.assertRaises(ValueError):standard_material(source())
        for fault in ['shader','keyword','depth','texture','hdr','alpha','duplicate','nan']:
            f=source();s=f['m_SavedProperties']
            if fault=='shader':f['m_Shader']['fileID']=45
            if fault=='keyword':f['m_ShaderKeywords']+=' _EMISSION'
            if fault=='depth':s['m_Floats'][3]['_ZWrite']=1
            if fault=='texture':s['m_TexEnvs'][0]['_MainTex']['m_Texture']['fileID']=2800000
            if fault=='hdr':s['m_Colors'][0]['_Color']['r']=2
            if fault=='alpha':s['m_Colors'][0]['_Color']['a']=0
            if fault=='duplicate':s['m_Floats'].append({'_Metallic':0})
            if fault=='nan':s['m_Colors'][0]['_Color']['g']=float('nan')
            for check in [glass_material,lambda f:verify_glass(f,{},material())]:
                with self.subTest(fault=fault,check=check),self.assertRaises(ValueError):check(f)

    def test_independent_material_checker_rejects_wrong_blend_colour_depth_or_dependencies(self):
        for fault in ['opaque','culling','opacity','tint','roughness','texture','emission','dependency']:
            m=material();d={}
            if fault=='opaque':m['alphaMode']='OPAQUE'
            if fault=='culling':m['doubleSided']=True
            if fault=='opacity':m['pbrMetallicRoughness']['baseColorFactor'][3]=1
            if fault=='tint':m['pbrMetallicRoughness']['baseColorFactor'][0]=.1
            if fault=='roughness':m['pbrMetallicRoughness']['roughnessFactor']=.8
            if fault=='texture':m['normalTexture']={'index':0}
            if fault=='emission':m['emissiveFactor']=[1,0,0]
            if fault=='dependency':d['images']=[{}]
            with self.subTest(fault=fault),self.assertRaises(ValueError):verify_glass(source(),d,m)
