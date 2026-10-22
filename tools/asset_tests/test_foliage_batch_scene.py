"""Final scene verification must survive graph rebasing and catch rebound faults."""
import hashlib
import json
from pathlib import Path
import sys
import tempfile
import unittest
import numpy as np
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from test_unity_geometry import prototype
from unity_geometry import assemble
from merge_static_glb import read
from add_terrain_house import write_glb
from static_geometry import accessor
from assemble_foliage_batch import assemble_batch
from verify_foliage_batch_scene import verify


def fixture(root):
    batch=root/'batch';batch.mkdir();source=root/'source.glb';prototype(source)
    doc,binary=read(source);normals=accessor(doc,binary,1);tangents=[]
    for normal in normals:
        axis=np.eye(3)[np.abs(normal).argmin()];t=np.cross(normal,axis);t/=np.linalg.norm(t);tangents.append([*t,1])
    values=np.array(tangents,dtype='<f4');view=len(doc['bufferViews']);doc['bufferViews'].append({'buffer':0,'byteOffset':len(binary),'byteLength':values.nbytes,'target':34962})
    index=len(doc['accessors']);doc['accessors'].append({'bufferView':view,'componentType':5126,'type':'VEC4','count':len(values)})
    doc['meshes'][0]['primitives'][0]['attributes']['TANGENT']=index;binary+=values.tobytes();doc['buffers'][0]['byteLength']=len(binary);source.unlink();write_glb(doc,binary,source)
    draws=[]
    for instance in [101,102,103]:
        matrix=np.eye(4);matrix[0,3]=instance-101
        if instance==102:matrix[0,0]=-1
        draws.append({'instance':instance,'renderer':201,'mesh':{'guid':'f'*32,'fileID':43},'materials':[{'guid':('b' if instance==103 else 'a')*32,'fileID':2100000}],'matrix':matrix.tolist()})
    plan=root/'plan.json';plan.write_text(json.dumps({'draws':draws}));key=lambda d:hashlib.sha256(json.dumps([d['mesh'],d['materials']],sort_keys=True,separators=(',',':')).encode()).hexdigest()
    directory=batch/'prototypes'/key(draws[0]);directory.mkdir(parents=True);(directory/'preview.glb').write_bytes(source.read_bytes())
    instances=[{'id':str(d['instance'])+':201','matrix':d['matrix']} for d in draws[:2]];placed=assemble(source,instances,directory/'placements.glb')
    digest=hashlib.sha256(source.read_bytes()).hexdigest();(directory/'independent-verification.json').write_text(json.dumps({'passed':True,'asset_sha256':digest}));(directory/'canonical-reference.json').write_text(json.dumps({'passed':True}))
    report={'schema':1,'source_plan':str(plan),'plan_sha256':hashlib.sha256(plan.read_bytes()).hexdigest(),'planned_draws':3,'converted_draws':2,'unresolved_draws':1,
            'converted':[{'key':key(draws[0]),'draws':2,'prototype_sha256':digest,'placements_sha256':placed['asset_sha256'],'expected_bounds':placed['bounds']}],
            'unresolved':[{'key':key(draws[2]),'draws':1,'reason':'explicit unsupported test binding'}]}
    (batch/'batch.json').write_text(json.dumps(report));scene=root/'scene';assemble_batch(batch,source,scene);return batch,scene


def rebind(scene):
    path=scene/'assembly.json';data=json.loads(path.read_text());data['asset_sha256']=hashlib.sha256((scene/'forest.glb').read_bytes()).hexdigest();path.write_text(json.dumps(data))


class BatchSceneTests(unittest.TestCase):
    def test_all_converted_and_unresolved_bindings_pose_parity_and_base_survive(self):
        with tempfile.TemporaryDirectory() as directory:
            batch,scene=fixture(Path(directory));result=verify(batch,scene)
            self.assertTrue(result['passed']);self.assertEqual((result['source_renderers'],result['unresolved_source_draws'],result['triangles']),(2,1,12))
            with self.assertRaises(FileExistsError):assemble_batch(batch,Path(directory)/'source.glb',scene)

    def test_rebound_pose_and_mirrored_tangent_faults_reject(self):
        for kind in ['pose','tangent']:
            with tempfile.TemporaryDirectory() as directory:
                batch,scene=fixture(Path(directory));path=scene/'forest.glb';doc,binary=read(path)
                if kind=='pose':
                    node=next(v for v in doc['nodes'] if v.get('extras',{}).get('source_instance')=='101:201');node['translation'][0]+=.1
                else:
                    # The mirrored mesh is the final added mesh; corrupt its W.
                    primitive=doc['meshes'][-1]['primitives'][0];item=doc['accessors'][primitive['attributes']['TANGENT']];view=doc['bufferViews'][item['bufferView']]
                    binary=bytearray(binary);offset=view.get('byteOffset',0)+item.get('byteOffset',0)+12;binary[offset:offset+4]=np.float32(1).tobytes()
                path.unlink();write_glb(doc,binary,path);rebind(scene)
                with self.subTest(kind=kind),self.assertRaisesRegex(ValueError,'positions differ' if kind=='pose' else 'tangents/handedness'):verify(batch,scene)

    def test_rebound_report_cannot_silently_drop_unresolved_source_bindings(self):
        with tempfile.TemporaryDirectory() as directory:
            batch,scene=fixture(Path(directory));path=batch/'batch.json';report=json.loads(path.read_text());report['unresolved']=[];path.write_text(json.dumps(report))
            meta=scene/'assembly.json';data=json.loads(meta.read_text());data['batch_report_sha256']=hashlib.sha256(path.read_bytes()).hexdigest();meta.write_text(json.dumps(data))
            with self.assertRaisesRegex(ValueError,'classification omitted'):verify(batch,scene)
