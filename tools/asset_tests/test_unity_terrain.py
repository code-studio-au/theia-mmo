"""Terrain conversion rejects incomplete samples and retains asymmetric orientation."""
import copy
import sys
from pathlib import Path
import unittest
import numpy as np
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from unity_terrain import height_grid,alpha_weights,serialized_guid
from convert_unity_terrain import geometry,bilinear,linear,srgb


def heightmap():
    n=33;z,x=np.mgrid[:n,:n];h=(x+2*z)*100
    extrema=[]
    for step in [16,32]:
        for row in range(0,32,step):
            for col in range(0,32,step):
                a=h[row:row+step+1,col:col+step+1]/32766
                extrema.extend([float(a.min()),float(a.max())])
    return {'m_Resolution':n,'m_Heights':h.ravel().tolist(),'m_Holes':[],'m_HolesLOD':[],
            'm_MinMaxPatchHeights':extrema,'m_Scale':{'x':.5,'y':100,'z':.25}}


def control():
    values=np.zeros((32,32,4),dtype=np.uint8);values[:,:,0]=255
    blob=values.tobytes()+b'\0'*sum((32>>level)**2*4 for level in range(1,6))
    return {'m_Width':32,'m_Height':32,'m_TextureFormat':4,'m_ImageCount':1,'m_TextureDimension':2,'m_ColorSpace':0,
            'm_StreamData':{'size':0,'path':''},'m_MipCount':6,'m_MipsStripped':0,'image data':blob,
            'm_CompleteImageSize':len(blob),'m_TextureSettings':{'m_FilterMode':1,'m_WrapU':1,'m_WrapV':1}}


class TerrainTests(unittest.TestCase):
    def test_raw_height_samples_use_32766_and_source_patch_extrema(self):
        heights,scale,error=height_grid(heightmap())
        self.assertEqual(heights.shape,(33,33));self.assertEqual(int(heights[-1,0]),6400)
        self.assertLess(error,1e-12);np.testing.assert_array_equal(scale,[.5,100,.25])
        value=heightmap();value['m_MinMaxPatchHeights']=[v*32766/32767 for v in value['m_MinMaxPatchHeights']]
        with self.assertRaisesRegex(ValueError,'extrema disagree'):height_grid(value)

    def test_holes_truncated_negative_and_nonfinite_height_profiles_reject(self):
        for field,value in [('m_Holes',[1]),('m_Heights',[1]),('m_Heights',[-1]*1089),('m_MinMaxPatchHeights',[float('nan')]*10),('m_Scale',{'x':0,'y':1,'z':1})]:
            data=heightmap();data[field]=value
            with self.subTest(field=field),self.assertRaises(ValueError):height_grid(data)

    def test_grid_orientation_winding_uv_and_asymmetric_heights(self):
        raw,scale,_=height_grid(heightmap());positions,normals,uv,faces=geometry(raw,scale,np.array([-7,-20,2]))
        np.testing.assert_allclose(positions[1],[6.5,-20+100/32766*100,2],atol=1e-6)
        np.testing.assert_allclose(positions[33],[7,-20+200/32766*100,2.25],atol=1e-6)
        np.testing.assert_array_equal(uv[[0,32,-1]],[[0,1],[1,1],[1,0]])
        triangles=positions[faces];cross=np.cross(triangles[:,1]-triangles[:,0],triangles[:,2]-triangles[:,0])
        self.assertTrue((cross[:,1]>0).all());self.assertTrue((normals[:,1]>0).all())
        np.testing.assert_allclose(np.linalg.norm(normals,axis=1),1,atol=1e-6)
        self.assertEqual(len(faces),2048)

    def test_control_pixels_preserved_and_bad_layout_or_missing_layers_reject(self):
        texture=control();weights=alpha_weights([texture],1,32)
        self.assertEqual(weights.shape,(32,32,1));self.assertTrue((weights==255).all())
        for key,value in [('m_TextureFormat',5),('m_ColorSpace',1),('image data',b'x'),('m_StreamData',{'size':4,'path':'external'})]:
            bad=copy.deepcopy(texture);bad[key]=value
            with self.subTest(key=key),self.assertRaises(ValueError):alpha_weights([bad],1,32)
        bad=control();bad['image data']=bytes([255,1,0,0])+bad['image data'][4:]
        with self.assertRaisesRegex(ValueError,'missing terrain layers'):alpha_weights([bad],1,32)

    def test_bilinear_clamp_repeat_and_linear_colour(self):
        pixels=np.array([[[1.,0,0],[0,1,0]],[[0,0,1],[1,1,1]]])
        np.testing.assert_allclose(bilinear(pixels,np.array([[.5]]),np.array([[.5]])),[[[.5,.5,.5]]])
        np.testing.assert_allclose(bilinear(pixels,np.array([[-1.]]),np.array([[-1.]])),[[[1,0,0]]])
        np.testing.assert_allclose(bilinear(pixels,np.array([[-1.]]),np.array([[0.]]),True),[[[0,1,0]]])
        values=np.linspace(0,1,256);np.testing.assert_allclose(srgb(linear(values)),values,atol=1e-12)

    def test_serialized_guid_nibbles_retain_asymmetric_identity(self):
        raw=bytes.fromhex('0e90185f5ebd22488a87f32a4df6d0c8')
        self.assertEqual(serialized_guid(raw),'e00981f5e5db2284a8783fa2d46f0d8c')
        with self.assertRaises(ValueError):serialized_guid(raw[:-1])
