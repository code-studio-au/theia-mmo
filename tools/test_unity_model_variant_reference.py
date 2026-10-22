"""Model-variant reference completeness/fault tests in isolated Blender Python."""
import copy
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch
sys.path.insert(0,str(Path(__file__).resolve().parent/'asset_tests'))
import test_unity_prefab_ids as fixtures
from test_unity_prefab_ids import INSTANCE,SOURCE_RENDERER,TRANSFORM
from test_unity_prefabs import record,transform,write_records
from unity_model_ids import signed_hash
from verify_unity_model_variants import verify


class ModelVariantReferenceTests(unittest.TestCase):
    def test_source_pose_bindings_and_category_completeness_reject_corruption(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);sources,rr,mods=fixtures.InheritedIdTests().fixture(root)
            sources.manifest={'archive_sha256':'0'*64};sources.by_guid['a'*32]='variant.prefab'
            scene={100:record(1,'GameObject',{'m_IsActive':1}),101:record(4,'Transform',transform(100,position=[100,0,0])),
                501:record(1001,'PrefabInstance',{'m_SourcePrefab':{'guid':'a'*32,'fileID':100100000},
                    'm_Modification':{'m_TransformParent':{'fileID':101},'m_Modifications':mods}})}
            write_records(root/'scene.unity',scene)
            draw={'instance':501,'source':'variant.prefab','renderer':(INSTANCE^SOURCE_RENDERER)&((1<<63)-1),
                'transform':TRANSFORM,'mesh':{'guid':'b'*32,'fileID':signed_hash('Type:Mesh->Cube0'),'type':3},
                'materials':[{'guid':'c'*32,'fileID':2100000}],
                'matrix':[[2,0,0,110],[0,3,0,20],[0,0,4,30],[0,0,0,1]]}
            plan={'archive_sha256':'0'*64,'scene':'scene.unity','draws':[draw]};path=root/'plan.json'
            with patch('verify_unity_model_variants.Sources',return_value=sources):
                path.write_text(json.dumps(plan));self.assertEqual(verify(root,path)['model_placements'],1)
                for kind in ['missing','extra','position','mesh','material','transform','nan']:
                    bad=copy.deepcopy(plan)
                    if kind=='missing':bad['draws']=[]
                    if kind=='extra':bad['draws'].append(copy.deepcopy(draw))
                    if kind=='position':bad['draws'][0]['matrix'][0][3]+=1
                    if kind=='mesh':bad['draws'][0]['mesh']['guid']='d'*32
                    if kind=='material':bad['draws'][0]['materials'][0]['guid']='d'*32
                    if kind=='transform':bad['draws'][0]['transform']+=1
                    if kind=='nan':bad['draws'][0]['matrix'][0][0]=float('nan')
                    path.write_text(json.dumps(bad))
                    with self.assertRaises(ValueError,msg=kind):verify(root,path)


    def test_stripped_source_parent_uses_nonuniform_rotation_and_active_override(self):
        from test_unity_prefabs import prefab,override
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);sources,rr,mods=fixtures.InheritedIdTests().fixture(root)
            sources.manifest={'archive_sha256':'0'*64};sources.by_guid['a'*32]='variant.prefab';sources.by_guid['d'*32]='parent.prefab'
            parent=prefab();parent[1]['fields']['m_IsActive']=0;write_records(root/'parent.prefab',parent)
            active_mod=override(1,'m_IsActive','1');active_mod['target']['guid']='d'*32
            scene={100:record(1,'GameObject',{'m_IsActive':1}),101:record(4,'Transform',transform(100,position=[100,0,0])),
                501:record(1001,'PrefabInstance',{'m_SourcePrefab':{'guid':'a'*32,'fileID':100100000},
                    'm_Modification':{'m_TransformParent':{'fileID':701},'m_Modifications':mods}}),
                601:record(1001,'PrefabInstance',{'m_SourcePrefab':{'guid':'d'*32,'fileID':100100000},
                    'm_Modification':{'m_TransformParent':{'fileID':101},'m_Modifications':[active_mod]}}),
                701:record(4,'Transform',{'m_CorrespondingSourceObject':{'guid':'d'*32,'fileID':11},'m_PrefabInstance':{'fileID':601}})}
            scene[701]['stripped']=True;write_records(root/'scene.unity',scene)
            plan={'archive_sha256':'0'*64,'scene':'scene.unity','draws':[{'instance':501,'source':'variant.prefab',
                'renderer':(INSTANCE^SOURCE_RENDERER)&((1<<63)-1),'transform':TRANSFORM,
                'mesh':{'guid':'b'*32,'fileID':signed_hash('Type:Mesh->Cube0'),'type':3},
                'materials':[{'guid':'c'*32,'fileID':2100000}],
                'matrix':[[0,-9,0,44],[4,0,0,22],[0,0,4,30],[0,0,0,1]]}]}
            path=root/'plan.json';path.write_text(json.dumps(plan))
            with patch('verify_unity_model_variants.Sources',return_value=sources):
                self.assertTrue(verify(root,path)['passed'])
                scene[601]['fields']['m_Modification']['m_Modifications']=[];write_records(root/'scene.unity',scene)
                with self.assertRaises(ValueError):verify(root,path)


if __name__=='__main__':unittest.main()
