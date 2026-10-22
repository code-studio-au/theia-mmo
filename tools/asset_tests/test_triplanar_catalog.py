"""Catalog composition must preserve geometry and exact profile/texture roles."""
import json
from pathlib import Path
import shutil
import sys
import tempfile
import unittest
from unittest.mock import patch
from PIL import Image
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
import assemble_triplanar_catalog as author
from verify_triplanar_catalog import verify
from add_terrain_house import write_glb
from merge_static_glb import read
from asset_tests.test_unity_reframe import fixture


class CatalogTests(unittest.TestCase):
    def test_shared_profile_requires_identical_settings(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);first=self.source(root,1);second=self.source(root,2)
            metadata=json.loads((second/'triplanar.json').read_text());metadata['tiling']=.2
            (second/'triplanar.json').write_text(json.dumps(metadata))
            with patch.object(author,'__file__',str(root/'tools/assemble_triplanar_catalog.py')):
                output=root/'build/shared';author.build([(1,first),(1,second)],[],output,shared_profiles=True)
                self.assertEqual(verify(output)['profiles'],1)
                metadata['falloff']=9;(second/'triplanar.json').write_text(json.dumps(metadata))
                with self.assertRaisesRegex(ValueError,'different material settings'):
                    author.build([(1,first),(1,second)],[],root/'build/different',shared_profiles=True)

    def test_external_standard_images_keep_variants_and_detect_changed_bytes(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);first=self.source(root,1);doc,binary=fixture()
            (root/'shared').mkdir();Image.new('RGBA',(2,2),(10,20,30,255)).save(root/'shared/color.png')
            shutil.copyfile(root/'shared/color.png',root/'shared/linear.png')
            doc['images']=[{'uri':'shared/color.png'},{'uri':'shared/linear.png'}]
            doc['textures']=[{'source':0},{'source':1}]
            doc['materials'][0]['pbrMetallicRoughness']={'baseColorTexture':{'index':0}}
            doc['materials'][0]['normalTexture']={'index':1}
            standard=root/'standard.glb';write_glb(doc,binary,standard)
            with patch.object(author,'__file__',str(root/'tools/assemble_triplanar_catalog.py')):
                output=root/'build/catalog';author.build([(1,first)],[standard],output)
            verify(output)
            self.assertEqual(len(list((output/'shared').glob('*.png'))),2)
            asset=output/'preview.glb';saved=asset.read_bytes();d,b=read(asset,allow_external_images=True)
            d['images'][-1]['uri']=d['images'][-2]['uri'];asset.unlink();write_glb(d,b,asset)
            receipt_path=output/'catalog-assembly.json';saved_receipt=receipt_path.read_bytes();receipt=json.loads(saved_receipt)
            metadata_path=output/'triplanar.json';saved_metadata=metadata_path.read_bytes();metadata=json.loads(saved_metadata)
            receipt['asset_sha256']=author.sha(asset);metadata['geometry_sha256']=receipt['asset_sha256']
            metadata_path.write_text(json.dumps(metadata));receipt['metadata_sha256']=author.sha(metadata_path);receipt_path.write_text(json.dumps(receipt))
            with self.assertRaisesRegex(ValueError,'external image variants'):verify(output)
            asset.write_bytes(saved);receipt_path.write_bytes(saved_receipt);metadata_path.write_bytes(saved_metadata)
            Image.new('RGBA',(2,2),(1,2,3,255)).save(output/'shared/catalog-1-color.png')
            with self.assertRaisesRegex(ValueError,'external image bytes'):verify(output)

    def source(self, root, index):
        stage=root/str(index);stage.mkdir();doc,binary=fixture()
        doc['materials'][0]['extras']={'theia_triplanar':1}
        write_glb(doc,binary,stage/'preview.glb')
        textures={}
        for role in ['top','side','top_normal','side_normal']:
            name='triplanar-'+role+'.png';Image.new('RGBA',(2,2),(128,128,255,255)).save(stage/name)
            textures[role]={'file':name,'sha256':author.sha(stage/name),'source':'synthetic metadata fixture'}
        metadata={'schema':1,'profile':'synty-triplanar-world-v1','tiling':.2 if index==1 else .5,'falloff':4.1,'geometry_sha256':author.sha(stage/'preview.glb'),'textures':textures}
        (stage/'triplanar.json').write_text(json.dumps(metadata));(stage/'independent-verification.json').write_text(json.dumps({'passed':True,'asset_sha256':metadata['geometry_sha256'],'scope':'synthetic format fixture only'}))
        return stage

    def test_two_profiles_share_typed_images_and_preserve_standard_input(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);first=self.source(root,1);second=self.source(root,2);doc,binary=fixture();standard=root/'standard.glb';write_glb(doc,binary,standard)
            output=root/'catalog'
            # Isolate the test output-root guard, never actual source validation.
            with patch.object(author,'__file__',str(root/'tools/assemble_triplanar_catalog.py')):
                output=root/'build/catalog';r=author.build([(1,first),(2,second)],[standard],output)
            result=verify(output)
            self.assertEqual((result['profiles'],result['unique_images'],result['mesh_instances'],result['authored_materials']),(2,2,3,3))
            self.assertEqual(r['decoded_bytes'],32)
            metadata=json.loads((output/'triplanar.json').read_text());self.assertEqual(metadata['profiles'][0]['textures'],metadata['profiles'][1]['textures'])
            for fault in ['scalar','mapping','material','pose']:
                target=root/fault;shutil.copytree(output,target);receipt=json.loads((target/'catalog-assembly.json').read_text())
                if fault in ['scalar','mapping']:
                    p=target/'triplanar.json';v=json.loads(p.read_text())
                    if fault=='scalar':v['profiles'][1]['tiling']=.8
                    else:v['profiles'][1]['textures']['top']=v['profiles'][1]['textures']['top_normal']
                    p.write_text(json.dumps(v));receipt['metadata_sha256']=author.sha(p)
                else:
                    p=target/'preview.glb';d,b=read(p)
                    if fault=='material':d['materials'][1]['extras']['theia_triplanar']=1
                    else:d['nodes'][0]['translation']=[.2,0,0]
                    p.unlink();write_glb(d,b,p);receipt['asset_sha256']=author.sha(p);meta=json.loads((target/'triplanar.json').read_text());meta['geometry_sha256']=receipt['asset_sha256'];(target/'triplanar.json').write_text(json.dumps(meta));receipt['metadata_sha256']=author.sha(target/'triplanar.json')
                (target/'catalog-assembly.json').write_text(json.dumps(receipt))
                with self.subTest(fault=fault),self.assertRaises(ValueError):verify(target)

    def test_duplicate_profile_ids_reject_without_creating_output(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);first=self.source(root,1);output=root/'build/catalog'
            with patch.object(author,'__file__',str(root/'tools/assemble_triplanar_catalog.py')):
                with self.assertRaises(ValueError):author.build([(1,first),(1,first)],[],output)
            self.assertFalse(output.exists())
