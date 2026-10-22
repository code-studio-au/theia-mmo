"""Batch draw selection must bind actual source overrides and highest LOD."""
import copy
import hashlib
import json
from pathlib import Path
import sys
import tempfile
import unittest
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from test_unity_prefabs import prefab,record,override,write_records,GUID
from unity_scene import Sources
from unity_prefabs import static_prefab
from unity_draw_bindings import select_draw
from verify_unity_foliage import reference_materials
from unity_scene import records


def fixture(root):
    source=root/'source';(source/'Assets').mkdir(parents=True);prefab_name='Assets/tree.prefab';scene_name='Assets/demo.unity'
    changes=[override(12,'m_Materials.Array.data[0]',reference={'fileID':2100000,'guid':'d'*32})]
    rr=prefab();write_records(source/prefab_name,rr)
    scene={101:record(1001,'PrefabInstance',{'m_SourcePrefab':{'fileID':100100000,'guid':GUID},'m_Modification':{'m_TransformParent':{'fileID':0},'m_Modifications':changes,'m_RemovedComponents':[]}})}
    write_records(source/scene_name,scene);manifest={'archive_sha256':'e'*64,'selected':{},'metadata':{}}
    for name,guid in [(prefab_name,GUID),(scene_name,'f'*32)]:
        path=source/name;meta=Path(str(path)+'.meta');meta.write_text('guid: '+guid+'\n')
        for category,p in [('selected',path),('metadata',meta)]:manifest[category][name]={'guid':guid,'bytes':p.stat().st_size,'sha256':hashlib.sha256(p.read_bytes()).hexdigest()}
    (source/'extraction.json').write_text(json.dumps(manifest))
    draw=static_prefab(rr,changes,GUID)['draws'][0];draw.update(instance=101,source=prefab_name)
    plan={'archive_sha256':'e'*64,'scene':scene_name,'lod_policy':'highest-detail first serialized LOD; no runtime crossfade','draws':[draw]}
    path=root/'plan.json';path.write_text(json.dumps(plan));return source,path,plan


class DrawBindingTests(unittest.TestCase):
    def test_material_override_and_child_pose_are_bound_to_scene_and_plan_hash(self):
        with tempfile.TemporaryDirectory() as directory:
            source,path,plan=fixture(Path(directory));draw,static,binding=select_draw(Sources(source),path,101,12,'Assets/tree.prefab')
            self.assertEqual(draw['materials'][0]['guid'],'d'*32)
            self.assertEqual(draw['matrix'],plan['draws'][0]['matrix']);self.assertEqual(binding['world_matrix'],draw['matrix'])
            self.assertEqual(binding['plan_sha256'],hashlib.sha256(path.read_bytes()).hexdigest())
            self.assertEqual([d['renderer'] for d in static['draws']],[12])

    def test_independent_reference_resolves_overrides_and_rejects_rebound_material_change(self):
        with tempfile.TemporaryDirectory() as directory:
            source,path,plan=fixture(Path(directory));sources=Sources(source)
            _,_,binding=select_draw(sources,path,101,12,'Assets/tree.prefab')
            rr=records(source/'Assets/tree.prefab');renderer=rr[12]['fields'];mesh=rr[13]['fields'];conversion={'prefab':'Assets/tree.prefab','renderer':12,'scene_binding':binding}
            refs=reference_materials(sources,conversion,renderer,mesh);self.assertEqual(refs[0]['guid'],'d'*32);self.assertEqual(renderer['m_Materials'][0]['guid'],'b'*32)
            plan['draws'][0]['materials'][0]['guid']='b'*32;path.write_text(json.dumps(plan));binding['plan_sha256']=hashlib.sha256(path.read_bytes()).hexdigest()
            with self.assertRaisesRegex(ValueError,'scene material slots'):reference_materials(sources,conversion,renderer,mesh)

    def test_planned_mesh_material_transform_or_prefab_tampering_rejects(self):
        with tempfile.TemporaryDirectory() as directory:
            source,path,plan=fixture(Path(directory))
            for kind in ['mesh','materials','transform','source']:
                bad=copy.deepcopy(plan)
                if kind=='mesh':bad['draws'][0]['mesh']['fileID']=999
                if kind=='materials':bad['draws'][0]['materials'][0]['guid']='b'*32
                if kind=='transform':bad['draws'][0]['transform']=21
                if kind=='source':bad['draws'][0]['source']='Assets/other.prefab'
                path.write_text(json.dumps(bad))
                with self.subTest(kind=kind),self.assertRaises(ValueError):select_draw(Sources(source),path,101,12,'Assets/tree.prefab')

    def test_duplicate_archives_nonhighest_lod_and_changed_source_reject(self):
        with tempfile.TemporaryDirectory() as directory:
            source,path,plan=fixture(Path(directory))
            for kind in ['duplicate','archive','lod']:
                bad=copy.deepcopy(plan)
                if kind=='duplicate':bad['draws']*=2
                if kind=='archive':bad['archive_sha256']='0'*64
                if kind=='lod':bad['draws'][0]['renderer']=22
                path.write_text(json.dumps(bad))
                with self.subTest(kind=kind),self.assertRaises(ValueError):select_draw(Sources(source),path,101,22 if kind=='lod' else 12,'Assets/tree.prefab')
            path.write_text(json.dumps(plan));select_draw(Sources(source),path,101,12,'Assets/tree.prefab')
            (source/'Assets/tree.prefab').write_text('changed source')
            with self.assertRaisesRegex(ValueError,'hash/size'):select_draw(Sources(source),path,101,12,'Assets/tree.prefab')
