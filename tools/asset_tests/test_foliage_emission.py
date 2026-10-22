"""Explicit preview preserves raw properties, masks and HDR energy eligibility."""
import copy
from pathlib import Path
import sys
import unittest
import numpy as np
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from test_full_foliage import full_fields
from unity_full_foliage import full_material, SHADER_SHA256
from unity_foliage_emission import material, pixels


def fields():
    f=full_fields();s=f['m_SavedProperties']
    for row in s['m_Floats']:
        if '_EmissiveAmount' in row:row['_EmissiveAmount']=1.3
    for row in s['m_TexEnvs']:
        for key in ['_LeafNormalMap','_TrunkNormalMap']:
            if key in row:row[key]['m_Texture']={'fileID':0}
    s['m_Colors'] += [{k:dict(r=r,g=g,b=b,a=1)} for k,(r,g,b) in {'_EmissiveColour':(1,0,1),'_Emissive2Colour':(0,1,1),'_TrunkEmissiveColour':(0,0,0)}.items()]
    for k in ['_EmissiveMask','_Emissive2Mask','_EmissivePulseMap','_TrunkEmissiveMask']:
        s['m_TexEnvs'].append({k:{'m_Texture':{'fileID':2800000,'guid':'a'*32} if 'Pulse' not in k and 'Trunk' not in k else {'fileID':0},'m_Scale':dict(x=1,y=1),'m_Offset':dict(x=0,y=0)}})
    return f


class EmissiveTests(unittest.TestCase):
    def test_opt_in_keeps_original_fields_and_declares_omitted_spatial_noise(self):
        f=fields();before=copy.deepcopy(f);result=material(f,SHADER_SHA256)
        self.assertEqual(f,before);self.assertEqual(result['source_properties'],f['m_SavedProperties']);self.assertEqual(result['emission']['strength'],2.6)
        self.assertIn('_LeafNoiseColour',result['omitted_spatial_noise'])
        with self.assertRaises(ValueError):full_material(f,SHADER_SHA256)

    def test_source_default_pulse_trunk_colour_and_mask_uv_eligibility(self):
        for fault in ['amount','pulse','trunk','uv','hdr','normal']:
            f=fields();s=f['m_SavedProperties']
            if fault=='amount':
                next(row for row in s['m_Floats'] if '_EmissiveAmount' in row)['_EmissiveAmount']=3
            if fault=='pulse':next(row for row in s['m_TexEnvs'] if '_EmissivePulseMap' in row)['_EmissivePulseMap']['m_Texture']['fileID']=2800000
            if fault=='trunk':next(row for row in s['m_Colors'] if '_TrunkEmissiveColour' in row)['_TrunkEmissiveColour']['b']=.1
            if fault=='uv':next(row for row in s['m_TexEnvs'] if '_EmissiveMask' in row)['_EmissiveMask']['m_Scale']['x']=2
            if fault=='normal':next(row for row in s['m_TexEnvs'] if '_LeafNormalMap' in row)['_LeafNormalMap']['m_Texture']['fileID']=2800000
            if fault=='hdr':next(row for row in s['m_Colors'] if '_EmissiveColour' in row)['_EmissiveColour']['r']=2
            with self.subTest(fault=fault),self.assertRaises(ValueError):material(f,SHADER_SHA256)

    def test_all_mask_bytes_use_linear_energy_before_srgb_encoding(self):
        a=np.zeros((1,256,4),dtype=np.uint8);b=a.copy();a[0,:,0]=np.arange(256);b[0,:,0]=255-np.arange(256)
        actual=pixels(a,b,{'_EmissiveColour':[1,0,1],'_Emissive2Colour':[0,1,1]})
        def decode(v):
            v=v/255;return v/12.92 if v<=.04045 else ((v+.055)/1.055)**2.4
        def encode(v):return round((12.92*v if v<=.0031308 else 1.055*v**(1/2.4)-.055)*255)
        for v in range(256):
            x,y=decode(v),decode(255-v)
            self.assertEqual(actual[0,v].tolist(),[encode(x/2),encode(y/2),encode((x+y)/2)])
        with self.assertRaises(ValueError):pixels(a,b[:,:1],{'_EmissiveColour':[1,0,1],'_Emissive2Colour':[0,1,1]})
