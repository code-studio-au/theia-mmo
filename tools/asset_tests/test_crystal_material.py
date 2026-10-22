"""Crystal previews keep bounded source factors and explicit omitted effects."""
from pathlib import Path
import sys
import unittest
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from unity_crystal import material,GUID,SHA256


def fields():
    return {'m_Shader':{'fileID':4800000,'guid':GUID,'type':3},'m_ShaderKeywords':None,'m_CustomRenderQueue':-1,
            'm_SavedProperties':{'m_Floats':[{k:v} for k,v in [('_ColourFalloff',2),('_Opacity',.3),('_Metallic',.7),('_Smoothness',.6),('_Refraction',.5),('_ChromaticAberration',.2)]],
                'm_Colors':[{'_ShallowColor':dict(r=1,g=0,b=0,a=1)},{'_DeepColor':dict(r=0,g=0,b=1,a=1)},{'_Emission':dict(r=.1,g=.2,b=.3,a=0)}],
                'm_TexEnvs':[{'_UnusedTexture':{'m_Texture':{'fileID':2800000,'guid':'a'*32}}}]}}


class CrystalTests(unittest.TestCase):
    def test_normal_incidence_interpolation_and_surface_state_are_explicit(self):
        p=material(fields(),SHA256)
        self.assertEqual(p['base_color_factor'],[.5,0,.5,.3]);self.assertAlmostEqual(p['roughness'],.4)
        self.assertEqual(p['unapplied_refraction'],.5);self.assertEqual(p['unapplied_chromatic_aberration'],.2)
        self.assertEqual(p['emissive_factor'],[.1,.2,.3]);self.assertIn('_UnusedTexture',p['unused_saved_texture_bindings'])
        f=fields();f['m_SavedProperties']['m_Floats'][0]['_ColourFalloff']=.5
        self.assertEqual(material(f,SHA256)['base_color_factor'],[0,0,1,.3])

    def test_unknown_shader_variants_nonfinite_hdr_and_unsupported_ranges_reject(self):
        for fault in ['shader','hash','keyword','falloff','opacity','chromatic','hdr','nan','duplicate']:
            f=fields();digest=SHA256;s=f['m_SavedProperties']
            if fault=='shader':f['m_Shader']['guid']='0'*32
            if fault=='hash':digest='0'*64
            if fault=='keyword':f['m_ShaderKeywords']='_UNKNOWN'
            if fault=='falloff':s['m_Floats'][0]['_ColourFalloff']=0
            if fault=='opacity':s['m_Floats'][1]['_Opacity']=1
            if fault=='chromatic':s['m_Floats'][5]['_ChromaticAberration']=.5
            if fault=='hdr':s['m_Colors'][2]['_Emission']['r']=2
            if fault=='nan':s['m_Colors'][0]['_ShallowColor']['g']=float('nan')
            if fault=='duplicate':s['m_Floats'].append({'_Opacity':.2})
            with self.subTest(fault=fault),self.assertRaises(ValueError):material(f,digest)
