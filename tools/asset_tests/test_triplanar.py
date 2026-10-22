"""Source triplanar signs, lower-Y binding, epsilon and repeat sampling matter."""
from pathlib import Path
import sys
import unittest
import numpy as np
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from unity_triplanar import projection,repeat_bilinear,evaluate,material,GUID,SHA256

class TriplanarTests(unittest.TestCase):
    def test_pinned_source_profile_records_unused_st_and_rejects_unqualified_states(self):
        import copy
        textures=[{key:{'m_Texture':{'fileID':2800000,'guid':'a'*32},'m_Scale':{'x':5,'y':5},'m_Offset':{'x':0,'y':0}}} for key in ['_Top','_Sides','_TopNormal','_SidesNormal']]
        fields={'m_Shader':{'guid':GUID,'fileID':4800000},'m_SavedProperties':{'m_Floats':[{'_FallOff':4.1},{'_Tiling':.2}],'m_TexEnvs':textures}}
        result=material(fields,SHA256);self.assertEqual(result['roughness'],1);self.assertEqual(result['unused_texture_st']['_Sides']['m_Scale']['x'],5)
        for kind in ['hash','shader','keyword','zero','nan','underflow','texture']:
            bad=copy.deepcopy(fields);digest=SHA256
            if kind=='hash':digest='0'*64
            if kind=='shader':bad['m_Shader']['fileID']=46
            if kind=='keyword':bad['m_ShaderKeywords']='UNQUALIFIED'
            if kind=='zero':bad['m_SavedProperties']['m_Floats'][0]['_FallOff']=0
            if kind=='nan':bad['m_SavedProperties']['m_Floats'][1]['_Tiling']=float('nan')
            if kind=='underflow':bad['m_SavedProperties']['m_Floats'][1]['_Tiling']=1e-100
            if kind=='texture':bad['m_SavedProperties']['m_TexEnvs'][0]['_Top']['m_Texture']['fileID']=0
            with self.subTest(kind=kind),self.assertRaises(ValueError):material(bad,digest)

    def test_six_signed_axes_have_source_uvs_and_bottom_uses_side(self):
        p=np.tile([2,3,5],(6,1));n=np.concatenate([np.eye(3),-np.eye(3)])
        uv,w=projection(p,n,4.1,.5)
        np.testing.assert_allclose(w*np.float64(1.00001),[[1,0,0,0],[0,1,0,0],[0,0,0,1],[1,0,0,0],[0,0,1,0],[0,0,0,1]])
        np.testing.assert_array_equal(uv[[0,1,2,3,4,5],[0,1,3,0,2,3]],[[2.5,1.5],[1,2.5],[-1,1.5],[-2.5,1.5],[-1,2.5],[1,1.5]])
    def test_epsilon_is_not_ordinary_normalized_blend_and_rejects_bad_inputs(self):
        _,w=projection([0,0,0],[.5,.5,.5],10,.5);self.assertLess(w.sum(),1);self.assertAlmostEqual(w.sum(),3*.5**10/(3*.5**10+1e-5))
        for n,f,t in [([0,0,0],10,1),([1,0,0],0,1),([1,0,0],10,float('nan'))]:
            with self.assertRaises(ValueError):projection([0,0,0],n,f,t)
    def test_png_v_flip_repeat_and_linear_light_filtering(self):
        pixels=np.array([[[255,0,0,255],[0,255,0,255]],[[0,0,255,255],[255,255,255,255]]],np.uint8)
        np.testing.assert_array_equal(repeat_bilinear(pixels,[.25,.75]),[1,0,0,1])
        np.testing.assert_allclose(repeat_bilinear(pixels,[-.75,-.25]),[1,0,0,1])
        np.testing.assert_allclose(repeat_bilinear(pixels,[.5,.5],True),[.5,.5,.5,1])
    def test_normal_channels_are_blended_before_desktop_unpack(self):
        images={'top':np.tile([255,0,0,255],(1,1,1)),'side':np.tile([0,0,255,255],(1,1,1)),
                'top_normal':np.tile([128,128,255,255],(1,1,1)),'side_normal':np.tile([128,128,255,255],(1,1,1))}
        color,normal=evaluate(images,[2,3,4],[0,-1,0],4.1,.5)
        np.testing.assert_allclose(color,[0,0,1/1.00001]);self.assertGreater(normal[2],.99)
