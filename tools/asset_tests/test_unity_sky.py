"""Clamped source-height lookup endpoints and eligibility stay explicit."""
from pathlib import Path
import sys
import unittest
import numpy as np
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from unity_sky import material,lut,GUID,SHA256


def fields():
    return {'m_Shader':{'fileID':4800000,'guid':GUID,'type':3},'m_ShaderKeywords':None,'m_CustomRenderQueue':-1,
            'm_SavedProperties':{'m_Floats':[{'_Offset':-10},{'_Distance':20},{'_Falloff':2},{'_UV_Based_Toggle':0}],
                'm_Colors':[{'_ColorBottom':dict(r=0,g=0,b=1,a=0)},{'_ColorTop':dict(r=1,g=0,b=0,a=0)}],
                'm_TexEnvs':[{'_texcoord':{'m_Texture':{'fileID':0}}}]}}


class SkyTests(unittest.TestCase):
    def test_lookup_preserves_exact_endpoints_opaque_alpha_and_linear_colour_power(self):
        profile=material(fields(),SHA256);pixels=lut(profile)
        np.testing.assert_array_equal(pixels[0],[[255,0,0,255]]*2);np.testing.assert_array_equal(pixels[-1],[[0,0,255,255]]*2)
        np.testing.assert_array_equal(pixels[:,0],pixels[:,1]);self.assertTrue((pixels[:,:,3]==255).all())
        def linear(channel):
            v=channel/255;return v/12.92 if v<=.04045 else ((v+.055)/1.055)**2.4
        for row in [1,256,512,768,1022]:
            weight=((1023-row)/1023)**2
            self.assertLess(abs(linear(pixels[row,0,0])-weight),.0045)
            self.assertLess(abs(linear(pixels[row,0,2])-(1-weight)),.0045)

    def test_shader_variant_uv_mode_zero_distance_hdr_and_nonfinite_reject(self):
        for fault in ['shader','hash','keyword','uv','texture','distance','hdr','nan']:
            f=fields();digest=SHA256;s=f['m_SavedProperties']
            if fault=='shader':f['m_Shader']['guid']='0'*32
            if fault=='hash':digest='0'*64
            if fault=='keyword':f['m_ShaderKeywords']='_UV_BASED_TOGGLE_ON'
            if fault=='uv':s['m_Floats'][3]['_UV_Based_Toggle']=1
            if fault=='texture':s['m_TexEnvs'][0]['_texcoord']['m_Texture']['fileID']=2800000
            if fault=='distance':s['m_Floats'][1]['_Distance']=0
            if fault=='hdr':s['m_Colors'][1]['_ColorTop']['b']=2
            if fault=='nan':s['m_Floats'][2]['_Falloff']=float('nan')
            with self.subTest(fault=fault),self.assertRaises(ValueError):material(f,digest)
