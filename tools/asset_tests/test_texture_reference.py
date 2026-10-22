"""Synthetic reference guards; these tests do not simulate Editor qualification."""
import copy
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch
from PIL import Image
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import unity_texture_reference as reference


class TextureReferenceTests(unittest.TestCase):
    def fixture(self, root):
        source=root/'source';image=source/'Assets/Test.png';image.parent.mkdir(parents=True)
        Image.new('RGBA',(4,4),(1,2,3,255)).save(image)
        meta=Path(str(image)+'.meta');meta.write_text('guid: '+'a'*32+'\nTextureImporter:\n  textureType: 0\n')
        manifest={'archive_sha256':'b'*64,'selected':{},'metadata':{}}
        for key,path in [('selected',image),('metadata',meta)]:
            manifest[key]['Assets/Test.png']={'guid':'a'*32,'sha256':reference.digest(path),'bytes':path.stat().st_size}
        (source/'extraction.json').write_text(json.dumps(manifest));return source

    def test_staging_excludes_source_scripts_and_rejects_reuse_escape_and_non_png(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);source=self.fixture(root);(source/'Assets/Unsafe.cs').write_text('source code')
            output=root/'build/project'
            with patch.object(reference,'BUILD',root/'build'):
                reference.stage(source,output,['Assets/Test.png'],'6000.6.6f1')
                for path,names in [(output,['Assets/Test.png']),(root/'outside',['Assets/Test.png']),(root/'build/other',['Assets/Unsafe.cs'])]:
                    with self.assertRaises(ValueError):reference.stage(source,path,names,'6000.6.6f1')
            self.assertFalse((output/'Assets/Unsafe.cs').exists())
            self.assertEqual([p.name for p in (output/'Assets/Editor').iterdir()],['TheiaTextureReference.cs'])

    def test_reference_rejects_incomplete_identity_mutated_metadata_and_invalid_sampler(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);source=self.fixture(root);project=root/'build/project'
            with patch.object(reference,'BUILD',root/'build'):
                request=reference.stage(source,project,['Assets/Test.png'],'6000.6.6f1')
            texture={**request['textures'][0],'file_id':'2800000','width':4,'height':4,'filter':1,'wrap_u':0,'wrap_v':0,'aniso':1,'mip_bias':0.,'mip_count':3,'srgb':True,'mipmaps':True,'convert_normal':False}
            report={'schema':1,'unity_version':request['unity_version'],'platform':'LinuxEditor','request_sha256':reference.digest(project/'Reference/request.json'),'collector_sha256':reference.digest(reference.COLLECTOR),'textures':[texture]}
            output=project/'Reference/textures.json';meta=project/'Assets/Test.png.meta';saved=meta.read_bytes()
            for fault in [None,'request','collector','guid','id','duplicate','missing','dimensions','filter','bool','meta']:
                value=copy.deepcopy(report)
                if fault=='request':value['request_sha256']='0'*64
                if fault=='collector':value['collector_sha256']='0'*64
                if fault=='guid':value['textures'][0]['guid']='c'*32
                if fault=='id':value['textures'][0]['file_id']='1'
                if fault=='duplicate':value['textures'].append(copy.deepcopy(texture))
                if fault=='missing':value['textures']=[]
                if fault=='dimensions':value['textures'][0]['width']=2
                if fault=='filter':value['textures'][0]['filter']=-1
                if fault=='bool':value['textures'][0]['srgb']=1
                if fault=='meta':meta.write_text('changed')
                output.write_text(json.dumps(value))
                if fault:
                    with self.subTest(fault=fault),self.assertRaises(ValueError):reference.inspect(source,project)
                else:self.assertTrue(reference.inspect(source,project)['passed'])
                meta.write_bytes(saved)
