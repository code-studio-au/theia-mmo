"""Ground packaging retains every asymmetric sample and rejects incomplete coverage."""
import copy
import hashlib
import io
import json
from pathlib import Path
import sys
import tempfile
import unittest
import numpy as np
from PIL import Image
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from convert_unity_terrain import write_glb
from package_unity_terrain import publish,owners
from package_world import package,identity
from verify_terrain_packages import verify


def fixture(root):
    source=root/'source';source.mkdir();z,x=np.mgrid[-2:3,-2:3];points=np.stack([x,x*.02+z*.03,z],axis=-1).reshape(-1,3).astype('<f4')
    normals=np.tile(np.array([-.02,1,-.03])/np.linalg.norm([-.02,1,-.03]),(25,1)).astype('<f4');uv=np.column_stack([(x.ravel()+2)/4,(z.ravel()+2)/4]).astype('<f4')
    faces=[]
    for row in range(4):
        for col in range(4):
            a=row*5+col;faces.extend([[a,a+5,a+6],[a,a+6,a+1]])
    image=io.BytesIO();Image.new('RGB',(2,2),(90,140,45)).save(image,format='PNG')
    write_glb(source/'terrain.glb',points,normals,uv,np.array(faces,dtype='<u4'),image.getvalue())
    (source/'conversion.json').write_text(json.dumps({'vertices':25,'triangles':32,'outputs':{'terrain.glb':hashlib.sha256((source/'terrain.glb').read_bytes()).hexdigest()}}))
    return source


class TerrainPackageTests(unittest.TestCase):
    def test_all_signed_owners_exact_geometry_shared_image_and_original_bytes(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);source=fixture(root);before=(source/'terrain.glb').read_bytes()
            result=publish(source,root/'packages','theia.test.ground',[[-1,0,-1],[-1,0,1]])
            self.assertEqual((result['triangles'],result['shared_images']),(32,1));self.assertEqual(len(result['chunks']),4)
            proof=verify(source,root/'packages');self.assertTrue(proof['passed']);self.assertEqual(proof['source_vertices'],25)
            self.assertEqual((source/'terrain.glb').read_bytes(),before)
            with self.assertRaises(FileExistsError):publish(source,root/'packages','theia.test.ground',[[-1,0,-1]])

    def test_cross_cell_triangle_is_rejected_without_decimation(self):
        points=np.array([[-1,0,-1],[1,0,-1],[1,0,1]],dtype=float)
        with self.assertRaisesRegex(ValueError,'crosses a chunk plane'):owners(points,np.array([[0,1,2]]))

    def test_hash_mismatch_and_outside_spawn_do_not_publish(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);source=fixture(root)
            with self.assertRaisesRegex(ValueError,'Spawn footprint'):publish(source,root/'outside','theia.test.ground',[[4,0,0]])
            path=source/'conversion.json';data=json.loads(path.read_text());data['outputs']['terrain.glb']='0'*64;path.write_text(json.dumps(data))
            with self.assertRaisesRegex(ValueError,'source hash'):publish(source,root/'bad','theia.test.ground',[[0,0,0]])
            self.assertFalse((root/'bad').exists());self.assertFalse((root/'outside').exists())

    def test_independent_oracle_rejects_missing_chunk_and_rebound_collision_hole(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);source=fixture(root);out=root/'packages';publish(source,out,'theia.test.ground',[[-1,0,-1]])
            index_path=out/'world.chunks.json';index=json.loads(index_path.read_text());bad=copy.deepcopy(index);bad['chunks'].pop();index_path.write_text(json.dumps(bad))
            with self.assertRaisesRegex(ValueError,'terrain owner'):verify(source,out)
            index_path.write_text(json.dumps(index));chunk=out/'chunk_-1_-1';profile_path=chunk/'terrain.json';profile=json.loads(profile_path.read_text());profile['triangles'].pop();profile_path.write_text(json.dumps(profile))
            (chunk/'world.manifest.json').unlink();package(chunk,'theia.test.changed-ground',3,shared_images=True)
            manifest=json.loads((chunk/'world.manifest.json').read_text());index['chunks'][0]['package']=list(bytes.fromhex(identity(manifest)));index_path.write_text(json.dumps(index))
            with self.assertRaisesRegex(ValueError,'collision coverage'):verify(source,out)
