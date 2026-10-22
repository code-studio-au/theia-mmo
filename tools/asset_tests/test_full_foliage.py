"""Full foliage keeps explicit effects, source cutouts and bounded normal decoding."""
import copy
import io
from pathlib import Path
import sys
import unittest
import tempfile
import numpy as np
from PIL import Image
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from test_unity_foliage import material
from unity_full_foliage import full_material,full_colour,normal_rg,SHADER_GUID,SHADER_SHA256
from verify_unity_foliage import expected_colour_table,normal_error,verify_auxiliary


def full_fields():
    fields=material();fields['m_Shader']['guid']=SHADER_GUID;saved=fields['m_SavedProperties'];base=dict(r=.1,g=.2,b=.3,a=0)
    saved['m_Colors'] += [{k:base.copy()} for k in ['_LeafNoiseColour','_LeafNoiseLargeColour','_TrunkNoiseColour','_GustHighlight']]
    saved['m_Floats'] += [{k:v} for k,v in [('_LeafFlatColourSwitch',0),('_TrunkFlatColourSwitch',0),('_LeafNormalAmount',.5),('_TrunkNormalAmount',.25),('_TotalWindAmount',.008)]]
    saved['m_TexEnvs'] += [{k:{'m_Texture':{'fileID':0},'m_Scale':{'x':1,'y':1},'m_Offset':{'x':0,'y':0}}} for k in ['_LeafAmbientOcclusion','_TrunkAmbientOcclusion']]
    return fields


class FullFoliageTests(unittest.TestCase):
    def test_exact_profile_collapses_constant_colour_and_retains_unapplied_motion(self):
        plan=full_material(full_fields(),SHADER_SHA256)
        self.assertEqual(plan['branches']['leaf']['normal_amount'],.5)
        self.assertEqual(plan['source_motion_properties']['_TotalWindAmount'],.008)
        self.assertIn('animated gust highlights',plan['approximations'][0])
        self.assertEqual(plan['branches']['trunk']['occlusion_texture'],{'fileID':0})

    def test_known_saved_flags_preserve_cutout_normals_and_disabled_emission(self):
        fields=full_fields();fields['m_ShaderKeywords']='_NORMALMAP _EMISSION _ALPHATEST_ON'
        plan=full_material(fields,SHADER_SHA256)
        self.assertEqual(plan['source_keywords'],['_ALPHATEST_ON','_EMISSION','_NORMALMAP'])
        fields['m_ShaderKeywords']+=' _UNREVIEWED_EFFECT'
        with self.assertRaisesRegex(ValueError,'variant/pass'):full_material(fields,SHADER_SHA256)
        fields=full_fields();fields['m_ShaderKeywords']=None
        fields['m_SavedProperties']['m_Colors'].append({'_CameraFadeParams':{'r':0,'g':'Infinity','b':0,'a':0}})
        plan=full_material(fields,SHADER_SHA256)
        self.assertEqual(plan['source_keywords'],[]);self.assertEqual(plan['unused_saved_colour_keys'],['_CameraFadeParams'])

    def test_unknown_shader_spatial_colour_frost_emission_and_transformed_normals_reject(self):
        for kind in ['shader','hash','noise','emission','normal_uv','nan']:
            fields=full_fields();saved=fields['m_SavedProperties'];sha=SHADER_SHA256
            if kind=='shader':fields['m_Shader']['guid']='0'*32
            if kind=='hash':sha='0'*64
            if kind=='noise':saved['m_Colors'][2]['_LeafNoiseColour']['r']=.9
            if kind=='emission':saved['m_Floats'][1]['_EmissiveAmount']=.1
            if kind=='normal_uv':saved['m_TexEnvs'][2]['_LeafNormalMap']['m_Offset']['x']=1
            if kind=='nan':saved['m_Floats'].append({'_TotalWindAmount':float('nan')})
            with self.subTest(kind=kind),self.assertRaises(ValueError):full_material(fields,sha)

    def test_flat_blend_matches_independent_all_byte_tables_and_preserves_alpha(self):
        source=np.stack([np.arange(256,dtype=np.uint8)]*4,axis=-1)[None,:,:]
        for flat in [0,.3,1]:
            baked=full_colour(source,[.1,.2,.3],flat);np.testing.assert_array_equal(baked[:,:,3],source[:,:,3])
            for channel,floor in enumerate([.1,.2,.3]):np.testing.assert_array_equal(baked[0,:,channel],expected_colour_table(floor,flat))

    def test_normal_strength_alpha_product_and_srgb_import_are_explicit(self):
        source=np.array([[[255,255,255,255],[255,128,255,0],[128,128,255,255]]],dtype=np.uint8)
        np.testing.assert_array_equal(normal_rg(source,0,False),np.array([[[128,128,255]]*3],dtype=np.uint8))
        baked=normal_rg(source,.5,False)
        np.testing.assert_array_equal(baked[0,0],[191,191,218]);self.assertLess(baked[0,1,0],128)
        self.assertLess(normal_rg(source,.5,True)[0,2,0],baked[0,2,0])
        with self.assertRaises(ValueError):normal_rg(source,float('nan'),False)

    def test_geometry_uses_exact_cyclic_positions_and_bounded_uv_rounding(self):
        source=np.array([[0,0,0,.50000501,0,0,0,1],[1,0,0,1,0,0,0,1],[0,1,0,0,1,0,0,1]])
        actual=source.copy();actual[0,3]=.50000499
        self.assertEqual(normal_error([source],[actual]),0)
        for kind in ['winding','position','uv']:
            bad=actual.copy()
            if kind=='winding':bad=bad[[0,2,1]]
            if kind=='position':bad[0,0]=.0001
            if kind=='uv':bad[0,3]+=.0001
            with self.subTest(kind=kind),self.assertRaises(ValueError):normal_error([source],[bad])

    def test_auxiliary_oracle_checks_linear_normal_type_and_exact_occlusion(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory)
            Image.new('RGBA',(1,1),(128,128,255,255)).save(root/'normal.png')
            Image.new('RGB',(1,1),(64,128,255)).save(root/'ao.png')
            class Sources:
                by_guid={'a':'normal.png','b':'ao.png'}
                def checked(self,name):return root/name
                def metadata(self,name):return {'TextureImporter':{'textureType':1 if name=='normal.png' else 0,'mipmaps':{'sRGBTexture':1 if name=='normal.png' else 0}}}
            textures={'_LeafNormalMap':{'m_Texture':{'fileID':2800000,'guid':'a'}},'_LeafAmbientOcclusion':{'m_Texture':{'fileID':2800000,'guid':'b'}}}
            material={'normalTexture':{'index':0},'occlusionTexture':{'index':1}}
            def document(normal=(128,128,255),occlusion=(64,64,64)):
                packed=bytearray();views=[]
                for colour in [normal,occlusion]:
                    image=io.BytesIO();Image.new('RGB',(1,1),colour).save(image,format='PNG');value=image.getvalue();views.append({'byteOffset':len(packed),'byteLength':len(value)});packed.extend(value)
                return {'textures':[{'source':i,'sampler':0} for i in range(2)],'samplers':[{'magFilter':9729,'minFilter':9985,'wrapS':10497,'wrapT':10497}],
                        'images':[{'bufferView':i} for i in range(2)],'bufferViews':views},packed
            doc,packed=document();images,details=verify_auxiliary(Sources(),textures,{'_LeafNormalAmount':.5},'_Leaf',material,doc,packed)
            self.assertEqual(images,{0,1});self.assertFalse(details['normal']['source_sample_srgb']);self.assertTrue(details['occlusion']['pixels_exact'])
            for normal,occlusion in [((255,0,0),(64,64,64)),((128,128,255),(65,65,65))]:
                doc,packed=document(normal,occlusion)
                with self.assertRaises(ValueError):verify_auxiliary(Sources(),textures,{'_LeafNormalAmount':.5},'_Leaf',material,doc,packed)
