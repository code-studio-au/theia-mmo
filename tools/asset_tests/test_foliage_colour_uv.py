"""Colour tiling must preserve normal/AO sampling, geometry and handedness."""
import copy
from pathlib import Path
import sys
import unittest
import numpy as np
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from asset_tests.test_unity_reframe import fixture
from asset_tests.test_full_foliage import full_fields
from unity_full_foliage import full_material,SHADER_SHA256
from foliage_colour_uv import apply
from verify_unity_foliage import verify_colour_uv
from static_geometry import accessor
from qualify_foliage_group import source_group


class ColourUvTests(unittest.TestCase):
    def test_group_includes_exact_bindings_across_prefabs_and_excludes_other_renderers(self):
        a={'instance':1,'renderer':10,'source':'Tree.prefab','mesh':{'guid':'a','fileID':1},'materials':[{'guid':'b'}]}
        other=copy.deepcopy(a);other.update(instance=2,renderer=20);other['mesh']['fileID']=2
        override=copy.deepcopy(a);override.update(instance=3,renderer=30);override['materials'][0]['guid']='c'
        same=copy.deepcopy(a);same.update(instance=4,renderer=40,source='OtherTree.prefab')
        _,_,selected=source_group([a,other,override,same],a,'Tree.prefab')
        self.assertEqual(selected,[a,same])
        with self.assertRaises(ValueError):source_group([a,a],a,'Tree.prefab')
        with self.assertRaises(ValueError):source_group([a],a,'Wrong.prefab')

    def test_only_explicit_colour_transforms_are_eligible(self):
        f=full_fields();entry=f['m_SavedProperties']['m_TexEnvs'][0]['_LeafTex'];entry['m_Scale']={'x':2,'y':2}
        with self.assertRaises(ValueError):full_material(f,SHADER_SHA256)
        plan=full_material(f,SHADER_SHA256,colour_uv=True)
        self.assertEqual(plan['branches']['leaf']['colour_uv'],{'scale':[2,2],'offset':[0,0]})
        for fault in ['normal','zero','negative','offset','nan']:
            bad=copy.deepcopy(f)
            if fault=='normal':bad['m_SavedProperties']['m_TexEnvs'][2]['_LeafNormalMap']['m_Scale']['x']=2
            if fault=='zero':bad['m_SavedProperties']['m_TexEnvs'][0]['_LeafTex']['m_Scale']['x']=0
            if fault=='negative':bad['m_SavedProperties']['m_TexEnvs'][0]['_LeafTex']['m_Scale']['x']=-2
            if fault=='offset':bad['m_SavedProperties']['m_TexEnvs'][0]['_LeafTex']['m_Offset']['y']=100
            if fault=='nan':bad['m_SavedProperties']['m_TexEnvs'][0]['_LeafTex']['m_Scale']['x']=float('nan')
            with self.subTest(fault=fault),self.assertRaises(ValueError):full_material(bad,SHADER_SHA256,colour_uv=True)

    def test_second_channel_reflects_v_and_retains_original_streams(self):
        doc,binary=fixture();doc['materials'][0]={'name':'leaf','normalTexture':{'index':1},'occlusionTexture':{'index':2},'pbrMetallicRoughness':{'baseColorTexture':{'index':0}}}
        result,packed=apply(doc,binary,{'leaf':{'colour_uv':{'scale':[2,3],'offset':[.25,.5]}}})
        self.assertEqual(packed[:len(binary)],binary)
        primitive=result['meshes'][0]['primitives'][0]
        for name,index in doc['meshes'][0]['primitives'][0]['attributes'].items():
            np.testing.assert_array_equal(accessor(result,packed,primitive['attributes'][name]),accessor(doc,binary,index))
        np.testing.assert_array_equal(accessor(result,packed,primitive['attributes']['TEXCOORD_1']),[[.25,-2.5],[2.25,-2.5],[.25,.5]])
        self.assertEqual(result['materials'][0]['normalTexture'],doc['materials'][0]['normalTexture'])
        self.assertEqual(result['materials'][0]['occlusionTexture'],doc['materials'][0]['occlusionTexture'])
        environment={'m_Scale':{'x':2,'y':3},'m_Offset':{'x':.25,'y':.5}}
        verify_colour_uv(result,packed,primitive,environment)
        for fault in ['primary','channel','offset']:
            bad=copy.deepcopy(result)
            if fault=='primary':bad['meshes'][0]['primitives'][0]['attributes']['TEXCOORD_1']=2
            if fault=='channel':bad['materials'][0]['pbrMetallicRoughness']['baseColorTexture']['texCoord']=0
            if fault=='offset':bad['accessors'][-1]['bufferView']=2
            with self.subTest(fault=fault),self.assertRaises(ValueError):verify_colour_uv(bad,packed,bad['meshes'][0]['primitives'][0],environment)

    def test_identity_keeps_document_and_binary_unchanged(self):
        doc,binary=fixture();doc['materials'][0]['name']='leaf'
        result,packed=apply(doc,binary,{'leaf':{}})
        self.assertEqual(result,doc);self.assertEqual(packed,binary)
