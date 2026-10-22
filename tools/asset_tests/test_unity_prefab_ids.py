"""Inherited aliases must bind actual source IDs without guessing imported TRS."""
import copy
from pathlib import Path
import sys
import tempfile
import unittest
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from unity_prefab_ids import inherited_id,alias_table,variant_bindings,compile_single_model_variant
from unity_model_ids import signed_hash
from unity_prefabs import static_prefab
from test_unity_model_ids import fbx
from test_unity_prefabs import record,write_records,override

INSTANCE=8796525071321904866
SOURCE_TRANSFORM=-8679921383154817045
SOURCE_GO=919132149155446097
SOURCE_RENDERER=-7511558181221131132
GO=8562153384742259635
TRANSFORM=9050319417422434569


class InheritedIdTests(unittest.TestCase):
    def test_signed_ids_reproduce_actual_variant_and_stripped_object_anchors(self):
        self.assertEqual(inherited_id(INSTANCE,SOURCE_TRANSFORM),TRANSFORM)
        self.assertEqual(inherited_id(INSTANCE,SOURCE_GO),GO)
        self.assertEqual(inherited_id(8,3),11)
        self.assertEqual(inherited_id(-1,3),(1<<63)-4)

    def test_bad_ranges_nulls_and_alias_collisions_reject(self):
        for instance,source in [(0,1),(1,0),(True,1),(1,True),(1<<63,1),(1,1<<63),(1,1)]:
            with self.assertRaises(ValueError):inherited_id(instance,source)
        with self.assertRaisesRegex(ValueError,'collision'):
            alias_table(8,{1:{'kind':'Transform'},1-(1<<63):{'kind':'GameObject'}})
        self.assertEqual(alias_table(8,{1:{'kind':'Mesh'}}),{})

    def fixture(self,root):
        fbx(root/'model.fbx',single=True)
        rr={INSTANCE:record(1001,'PrefabInstance',{'m_SourcePrefab':{'guid':'b'*32,'fileID':100100000},
                'm_Modification':{'m_TransformParent':{'fileID':0},'m_Modifications':[
                    {'target':{'fileID':SOURCE_RENDERER,'guid':'b'*32},'propertyPath':'m_Materials.Array.data[0]',
                     'value':None,'objectReference':{'fileID':2100000,'guid':'c'*32}}],'m_RemovedComponents':[]}}),
            GO:record(1,'GameObject',{'m_CorrespondingSourceObject':{'guid':'b'*32,'fileID':SOURCE_GO},
                'm_PrefabInstance':{'fileID':INSTANCE}}),
            211244591:record(64,'MeshCollider',{'m_GameObject':{'fileID':GO},'m_Enabled':1})}
        rr[GO]['stripped']=True;write_records(root/'variant.prefab',rr)
        class Sources:
            by_guid={'b'*32:'model.fbx','c'*32:'material.mat'}
            def checked(self,name):return root/name
            def metadata(self,name):return {'ModelImporter':{'meshes':{'fileIdsGeneration':2,'importVisibility':0,'importBlendShapes':0},
                'importAnimation':0,'materials':{'materialImportMode':0}}}
        values={**{f'm_LocalPosition.{k}':v for k,v in zip('xyz',[10,20,30])},
                **{f'm_LocalRotation.{k}':v for k,v in zip('xyzw',[0,0,0,1])},
                **{f'm_LocalScale.{k}':v for k,v in zip('xyz',[2,3,4])}}
        mods=[override(TRANSFORM,k,str(v)) for k,v in values.items()]
        return Sources(),rr,mods

    def test_compile_requires_complete_outer_trs_and_retains_source_collider(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);sources,rr,mods=self.fixture(root)
            bindings=variant_bindings(sources,'variant.prefab')
            self.assertEqual(bindings['stripped'],[{'fileID':GO,'source_fileID':SOURCE_GO,'kind':'GameObject'}])
            compiled=compile_single_model_variant(sources,'variant.prefab',mods,'a'*32)
            resolved=static_prefab(compiled,mods,'a'*32);draw,=resolved['draws']
            self.assertEqual(draw['matrix'],[[2,0,0,10],[0,3,0,20],[0,0,4,30],[0,0,0,1]])
            self.assertEqual(draw['materials'],[{'fileID':2100000,'guid':'c'*32}])
            self.assertIn(211244591,compiled)
            self.assertEqual(compiled[211244591],rr[211244591])
            with self.assertRaisesRegex(ValueError,'complete outer root TRS'):
                compile_single_model_variant(sources,'variant.prefab',mods[:-1],'a'*32)
            with self.assertRaisesRegex(ValueError,'Duplicate'):
                compile_single_model_variant(sources,'variant.prefab',mods+[mods[0]],'a'*32)

    def test_incorrect_stripped_id_source_or_added_component_owner_reject(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);sources,rr,mods=self.fixture(root)
            for kind in ['id','source','owner']:
                bad=copy.deepcopy(rr)
                if kind=='id':bad[GO+1]=bad.pop(GO)
                if kind=='source':bad[GO]['fields']['m_CorrespondingSourceObject']['fileID']=SOURCE_GO+1
                if kind=='owner':bad[211244591]['fields']['m_GameObject']['fileID']=999
                write_records(root/'variant.prefab',bad)
                with self.assertRaises(ValueError,msg=kind):variant_bindings(sources,'variant.prefab')
