"""Foliage adaptation must preserve alpha and refuse ambiguous branch selection."""
import copy
from pathlib import Path
import sys
import unittest
import numpy as np
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from unity_foliage import vegetation_material,colour_floor_rgba,face_branch,BASIC_SHADER_GUID,BASIC_SHADER_SHA256
from verify_unity_foliage import expected_colour_table, normal_error


def material():
    texture={'m_Texture':{'guid':'a'*32,'fileID':2800000},'m_Scale':dict(x=1,y=1),'m_Offset':dict(x=0,y=0)}
    return {'m_Shader':{'guid':BASIC_SHADER_GUID,'fileID':4800000},'m_CustomRenderQueue':-1,
        'm_SavedProperties':{'m_TexEnvs':[{key:copy.deepcopy(texture)} for key in ['_LeafTex','_TunkTex','_LeafNormalMap','_TrunkNormalMap']],
            'm_Floats':[{k:v} for k,v in [('_FrostingSwitch',0),('_EmissiveAmount',0),('_Cutoff',.5),
                ('_LeafMetallic',.1),('_TrunkMetallic',.3),('_LeafSmoothness',.2),('_TrunkSmoothness',.4)]],
            'm_Colors':[{k:dict(r=.1,g=.2,b=.3,a=0)} for k in ['_LeafBaseColour','_TrunkBaseColour']]}}


class FoliageTests(unittest.TestCase):
    def test_source_profile_keeps_branches_and_declares_normals_approximation(self):
        plan=vegetation_material(material(),BASIC_SHADER_SHA256)
        self.assertEqual(plan['branches']['leaf']['metallic'],.1)
        self.assertEqual(plan['branches']['trunk']['roughness'],.6)
        self.assertTrue(plan['double_sided']);self.assertEqual(plan['alpha_cutoff'],.5)
        self.assertEqual(set(plan['normal_maps_not_applied']),{'_LeafNormalMap','_TrunkNormalMap'})
        self.assertTrue(plan['approximations'])

    def test_all_byte_values_match_independent_scalar_reference_and_keep_alpha(self):
        pixels=np.zeros((1,256,4),dtype=np.uint8)
        pixels[0,:,0]=np.arange(256);pixels[0,:,1]=255-np.arange(256)
        pixels[0,:,2]=np.arange(256);pixels[0,:,3]=np.arange(256)
        tint=[.05,.25,.75];result=colour_floor_rgba(pixels,tint)
        np.testing.assert_array_equal(result[:,:,3],pixels[:,:,3])
        for channel in range(3):
            table=np.array(expected_colour_table(tint[channel]),dtype=np.uint8)
            np.testing.assert_array_equal(result[:,:,channel],table[pixels[:,:,channel]])
        np.testing.assert_array_equal(colour_floor_rgba(pixels,[0,0,0]),pixels)

    def test_mixed_fractional_or_invalid_vertex_selectors_reject(self):
        self.assertEqual(face_branch([0,0,0]),'trunk');self.assertEqual(face_branch([1,1,1,1]),'leaf')
        for values in [[0,1,1],[.5,.5,.5],[float('nan'),0,0],[2,2,2],[1,1]]:
            with self.assertRaises(ValueError):face_branch(values)

    def test_unsupported_shader_effects_invalid_surface_and_transforms_reject(self):
        for kind in ['shader','hash','frost','emission','cutoff','nan','transform','duplicate']:
            fields=material();sha=BASIC_SHADER_SHA256;saved=fields['m_SavedProperties']
            if kind=='shader':fields['m_Shader']['guid']='b'*32
            if kind=='hash':sha='0'*64
            if kind=='frost':saved['m_Floats'][0]['_FrostingSwitch']=1
            if kind=='emission':saved['m_Floats'][1]['_EmissiveAmount']=1
            if kind=='cutoff':saved['m_Floats'][2]['_Cutoff']=2
            if kind=='nan':saved['m_Colors'][0]['_LeafBaseColour']['r']=float('nan')
            if kind=='transform':saved['m_TexEnvs'][0]['_LeafTex']['m_Scale']['x']=2
            if kind=='duplicate':saved['m_Floats'].append({'_Cutoff':.3})
            with self.assertRaises(ValueError,msg=kind):vegetation_material(fields,sha)
        with self.assertRaises(ValueError):colour_floor_rgba(np.zeros((1,1,4),dtype=np.uint8),[0,0,2])

    def test_normal_reference_accepts_cyclic_export_rounding_and_rejects_changed_normals(self):
        triangle=np.array([[0,0,0,0,0,.3,.4,.8660254],[1,0,0,1,0,.3,.4,.8660254],[0,1,0,0,1,.3,.4,.8660254]])
        rounded=triangle.copy();rounded[:,5:]=np.round(rounded[:,5:],4)
        rounded[:,5:]/=np.linalg.norm(rounded[:,5:],axis=1)[:,None]
        self.assertLess(normal_error([triangle],[np.roll(rounded,1,axis=0)]),.00018)
        rounded[:,5:]=[1,0,0]
        with self.assertRaises(ValueError):normal_error([triangle],[rounded])
