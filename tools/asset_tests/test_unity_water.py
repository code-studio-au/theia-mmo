"""Water approximation sample and fail-closed source shader eligibility."""
from pathlib import Path
import sys
import unittest
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from unity_water import material, SHADERS


def fields():
    guid=next(iter(SHADERS))
    return {'m_Shader':{'fileID':4800000,'guid':guid,'type':3},'m_ShaderKeywords':None,
        'm_SavedProperties':{'m_Floats':[{k:v} for k,v in {'_Opacity':.7,'_OpacityMin':.2,'_OpacityFalloff':1,'_Smoothness':.8,'_ReflectionPower':.5,'_OverallFalloff':.54,'_ShallowFalloff':.2}.items()],
            'm_Colors':[{k:dict(r=r,g=g,b=b,a=0)} for k,(r,g,b) in {'_ShallowColour':(1,0,0),'_DeepColour':(0,0,1),'_VeryDeepColour':(0,1,0)}.items()], 'm_TexEnvs':[]}}


class WaterTests(unittest.TestCase):
    def test_unit_depth_selects_deep_or_shallow_mix_and_clamps_surface(self):
        f=fields();digest=SHADERS[f['m_Shader']['guid']];result=material(f,digest)
        self.assertEqual(result['base_color_factor'],[0,0,1,.7]);self.assertAlmostEqual(result['roughness'],.6)
        f['m_SavedProperties']['m_Floats'][-1]['_ShallowFalloff']=-.75
        f['m_SavedProperties']['m_Floats'][3]['_Smoothness']=4
        result=material(f,digest);self.assertEqual(result['base_color_factor'],[.75,0,.25,.7]);self.assertEqual(result['roughness'],0)
        self.assertEqual(result['source_properties'],f['m_SavedProperties']);self.assertEqual(result['unapplied_textures'],{})

    def test_unknown_shader_dynamic_wave_invalid_opacity_and_nonfinite_reject(self):
        for fault in ['hash','shader','keyword','wave','opacity','opacity-falloff','nan','tint']:
            f=fields();digest=SHADERS[f['m_Shader']['guid']];floats=f['m_SavedProperties']['m_Floats']
            if fault=='hash':digest='0'*64
            if fault=='shader':f['m_Shader']['guid']='0'*32
            if fault=='keyword':f['m_ShaderKeywords']='UNKNOWN'
            if fault=='wave':floats.append({'_WaveAmplitude':.1})
            if fault=='opacity':floats[0]['_Opacity']=1.2
            if fault=='opacity-falloff':floats[2]['_OpacityFalloff']=0
            if fault=='nan':floats[5]['_OverallFalloff']=float('nan')
            if fault=='tint':f['m_SavedProperties']['m_Colors'][0]['_ShallowColour']['r']=2
            with self.subTest(fault=fault),self.assertRaises(ValueError):material(f,digest)
