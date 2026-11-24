"""An explicit material profile retains source normal maps and sampler evidence."""
import copy
import io
from pathlib import Path
import sys
import tempfile
import unittest
import numpy as np
from PIL import Image
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from asset_tests.test_unity_materials import material
from unity_materials import standard_material
from unity_full_foliage import normal_rg
from unity_standard_normal import sampler, verify_normal_pixels, pack_normal
from verify_unity_static import standard_source


class StandardNormalTests(unittest.TestCase):
    def test_normals_require_explicit_profile_and_preserve_validated_strength(self):
        fields=material();fields['m_ShaderKeywords']='_NORMALMAP'
        fields['m_SavedProperties']['m_TexEnvs'].append({'_BumpMap':{'m_Texture':{'fileID':2800000,'guid':'c'*32},'m_Scale':{'x':1,'y':1},'m_Offset':{'x':0,'y':0}}})
        fields['m_SavedProperties']['m_Floats'].append({'_BumpScale':.75})
        with self.assertRaisesRegex(ValueError,'keyword'):standard_material(fields)
        with self.assertRaises(ValueError):standard_source(fields)
        plan=standard_material(fields,normal_map=True)
        self.assertEqual(plan['normal_texture']['guid'],'c'*32)
        self.assertEqual(plan['normal_amount'],.75)
        self.assertTrue(standard_source(fields,normal_map=True))
        for invalid in [-1,1.01,float('nan'),True]:
            bad=copy.deepcopy(fields);bad['m_SavedProperties']['m_Floats'][-1]={'_BumpScale':invalid}
            with self.assertRaises(ValueError):standard_material(bad,normal_map=True)
            with self.assertRaises(ValueError):standard_source(bad,normal_map=True)

    def test_independent_pixel_oracle_detects_wrong_normals_and_pack_keeps_source_bytes(self):
        pixels=np.array([[[128,128,255,255],[190,100,255,255]],[[64,180,255,255],[150,150,255,255]]],dtype=np.uint8)
        baked=normal_rg(pixels,.75,False)
        stream=io.BytesIO();Image.fromarray(baked).save(stream,format='PNG');png=stream.getvalue()
        with tempfile.TemporaryDirectory() as folder:
            source=Path(folder)/'normal.png';Image.fromarray(pixels).save(source)
            self.assertLess(verify_normal_pixels(source,png,.75)['maximum_vector_error'],.007)
            wrong=io.BytesIO();Image.new('RGB',(2,2),(255,128,128)).save(wrong,format='PNG')
            with self.assertRaisesRegex(ValueError,'vector bound'):verify_normal_pixels(source,wrong.getvalue(),.75)
        doc={'bufferViews':[], 'images':[], 'textures':[{'source':0,'sampler':0}], 'materials':[{'pbrMetallicRoughness':{'baseColorTexture':{'index':0}}}], 'buffers':[{'byteLength':4}]}
        binary=pack_normal(doc,b'keep',png)
        self.assertEqual(binary[:4],b'keep')
        self.assertEqual(doc['materials'][0]['normalTexture'],{'index':1})
        self.assertEqual(binary[4:4+len(png)],png)

    def test_inherited_samplers_need_observed_source_settings(self):
        meta={'textureType':1,'textureSettings':dict(zip(('filterMode','wrapU','wrapV','aniso','mipBias'),[-1]*5)),
              'mipmaps':{'enableMipMap':1,'sRGBTexture':0},'bumpmap':{'convertToNormalMap':0}}
        class Source:
            def metadata(self,name):return {'TextureImporter':meta}
        with self.assertRaisesRegex(ValueError,'actual Unity'):sampler(Source(),'normal',None)
        ref={'textures':[{'path':'normal','filter':1,'wrap_u':0,'wrap_v':0,'aniso':1,'mip_bias':0,
                          'importer_type':1,'srgb':False,'mipmaps':True,'convert_normal':False}]}
        self.assertEqual(sampler(Source(),'normal',ref),meta)
        ref['textures'][0]['filter']=0
        with self.assertRaisesRegex(ValueError,'sampler'):sampler(Source(),'normal',ref)
