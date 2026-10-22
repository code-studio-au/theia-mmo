"""Source materials must not silently lose active shader features or texture slots."""
import copy
from pathlib import Path
import struct
import sys
import tempfile
import unittest

from PIL import Image
import yaml
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from unity_materials import standard_material, standard_sampler_glb
from verify_unity_static import verify_material,standard_source
from merge_static_glb import read


def material():
    def texture(guid):
        return {'m_Texture':{'guid':guid,'fileID':2800000},
                'm_Scale':{'x':1,'y':1},'m_Offset':{'x':0,'y':0}}
    return {'m_Shader':{'guid':'0000000000000000f000000000000000','fileID':46},
            'm_ShaderKeywords':'_EMISSION','m_CustomRenderQueue':-1,
            'm_SavedProperties':{'m_TexEnvs':[{'_MainTex':texture('a'*32)},{'_EmissionMap':texture('b'*32)}],
                'm_Colors':[{'_Color':dict(r=1,g=1,b=1,a=1)},{'_EmissionColor':dict(r=1,g=1,b=1,a=1)}],
                'm_Floats':[{'_Mode':0},{'_Metallic':.3},{'_Glossiness':.2}]}}


class MaterialTests(unittest.TestCase):
    def test_independent_source_profile_rejects_active_unsupported_features(self):
        self.assertTrue(standard_source(material()))
        for kind in ['shader','alpha','keyword','tint','normal','transform','duplicate','nan','range','blend']:
            fields=material();saved=fields['m_SavedProperties']
            if kind=='shader':fields['m_Shader']['fileID']=45
            if kind=='alpha':saved['m_Floats'][0]['_Mode']=2
            if kind=='keyword':fields['m_ShaderKeywords']='_NORMALMAP'
            if kind=='tint':saved['m_Colors'][0]['_Color']['r']=.5
            if kind=='normal':saved['m_TexEnvs'].append({'_BumpMap':saved['m_TexEnvs'][0]['_MainTex']})
            if kind=='transform':saved['m_TexEnvs'][0]['_MainTex']['m_Scale']['x']=2
            if kind=='duplicate':saved['m_Floats'].append({'_Metallic':.4})
            if kind=='nan':saved['m_Floats'][1]['_Metallic']=float('nan')
            if kind=='range':saved['m_Floats'][2]['_Glossiness']=1.1
            if kind=='blend':saved['m_Floats'].append({'_SrcBlend':5})
            with self.subTest(kind=kind),self.assertRaises(ValueError):standard_source(fields)

    def test_preserves_surface_emission_and_explicit_profile(self):
        plan=standard_material(material())
        self.assertEqual((plan['metallic'],plan['roughness']),(.3,.8))
        self.assertEqual(plan['emissive']['guid'],'b'*32)
        self.assertEqual(plan['emissive_factor'],[1,1,1])

    def test_unsupported_features_and_bad_scalars_reject(self):
        for kind in ['shader','alpha','keyword','tint','normal','transform','duplicate','nan','range','blend']:
            fields=material();saved=fields['m_SavedProperties']
            if kind=='shader':fields['m_Shader']['guid']='c'*32
            if kind=='alpha':saved['m_Floats'][0]['_Mode']=2
            if kind=='keyword':fields['m_ShaderKeywords']='_NORMALMAP'
            if kind=='tint':saved['m_Colors'][0]['_Color']['r']=.5
            if kind=='normal':saved['m_TexEnvs'].append({'_BumpMap':saved['m_TexEnvs'][0]['_MainTex']})
            if kind=='transform':saved['m_TexEnvs'][0]['_MainTex']['m_Scale']['x']=2
            if kind=='duplicate':saved['m_Floats'].append({'_Metallic':.4})
            if kind=='nan':saved['m_Floats'][1]['_Metallic']=float('nan')
            if kind=='range':saved['m_Floats'][2]['_Glossiness']=1.1
            if kind=='blend':saved['m_Floats'].append({'_SrcBlend':5})
            with self.assertRaises(ValueError,msg=kind):standard_material(fields)

    def test_independent_oracle_rejects_swapped_images_surface_culling_and_sampling(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory)
            (root/'prefab').write_text('--- !u!23 &1\n'+yaml.safe_dump({'MeshRenderer':{'m_Materials':[{'guid':'c'*32}]}}))
            (root/'mat').write_text('--- !u!21 &1\n'+yaml.safe_dump({'Material':material()}))
            blobs=[]
            for name,color in [('base',(200,20,10,255)),('emissive',(10,200,20,255))]:
                image=Image.new('RGBA',(2,2),color);image.save(root/name,format='PNG')
                blobs.append((root/name).read_bytes())
            class Sources:
                by_guid={'a'*32:'base','b'*32:'emissive','c'*32:'mat'}
                def checked(self,name):return root/name
            doc={'materials':[{'pbrMetallicRoughness':{'metallicFactor':.3,'roughnessFactor':.8,
                  'baseColorTexture':{'index':0}},'emissiveFactor':[1,1,1],'emissiveTexture':{'index':1}}],
                 'textures':[{'source':0,'sampler':0},{'source':1,'sampler':0}],
                 'images':[{'bufferView':0},{'bufferView':1}],
                 'bufferViews':[{'byteOffset':0,'byteLength':len(blobs[0])},
                                {'byteOffset':len(blobs[0]),'byteLength':len(blobs[1])}],
                 'samplers':[{'magFilter':9729,'minFilter':9985}]}
            conversion={'prefab':'prefab','material':'mat','base_color_texture':'base',
                        'emissive_texture':'emissive','material_plan':{'profile':'unity-standard-opaque-v1'}}
            self.assertEqual(set(verify_material(Sources(),conversion,doc,b''.join(blobs))),{'base_color','emissive'})
            for kind in ['swapped','surface','culling','sampling','factor','uv','nan']:
                bad=copy.deepcopy(doc);m=bad['materials'][0]
                if kind=='swapped':bad['textures'][0]['source']=1
                if kind=='surface':m['pbrMetallicRoughness']['roughnessFactor']=.2
                if kind=='culling':m['doubleSided']=True
                if kind=='sampling':bad['samplers'][0]['minFilter']=9987
                if kind=='factor':m['emissiveFactor']=[0,0,0]
                if kind=='uv':m['emissiveTexture']['texCoord']=1
                if kind=='nan':m['pbrMetallicRoughness']['metallicFactor']=float('nan')
                with self.assertRaises(ValueError,msg=kind):verify_material(Sources(),conversion,bad,b''.join(blobs))

    def test_sampler_finalization_preserves_embedded_bytes(self):
        import json
        with tempfile.TemporaryDirectory() as directory:
            path=Path(directory)/'asset.glb'
            binary=b'\0\1\2\3'
            doc={'asset':{'version':'2.0'},'scenes':[{}],'buffers':[{'byteLength':4}],
                 'samplers':[{'magFilter':9729,'minFilter':9987}]}
            text=json.dumps(doc).encode();text+=b' '*(-len(text)%4)
            path.write_bytes(struct.pack('<4sII',b'glTF',2,28+len(text)+len(binary))+
                struct.pack('<I4s',len(text),b'JSON')+text+struct.pack('<I4s',len(binary),b'BIN\0')+binary)
            standard_sampler_glb(path);converted,data=read(path)
            self.assertEqual(data,binary)
            self.assertEqual(converted['samplers'],[dict(magFilter=9729,minFilter=9985,wrapS=10497,wrapT=10497)])
