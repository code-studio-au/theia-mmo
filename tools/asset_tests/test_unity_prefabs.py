"""Prefab instances must preserve source hierarchy, overrides and LOD membership."""
import copy
import math
from pathlib import Path
import sys
import tempfile
import unittest
import numpy as np
import yaml
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from unity_prefabs import static_prefab, local_matrix, scene_plan

GUID='a'*32


def record(kind,name,fields):
    return {'kind':kind,'name':name,'fields':fields,'stripped':False}


def transform(go,parent=0,children=(),position=(0,0,0)):
    return {'m_GameObject':{'fileID':go},'m_Father':{'fileID':parent},
            'm_Children':[{'fileID':i} for i in children],
            'm_LocalPosition':dict(zip('xyz',position)), 'm_LocalScale':dict(x=1,y=1,z=1),
            'm_LocalRotation':dict(x=0,y=0,z=0,w=1), 'm_LocalEulerAnglesHint':dict(x=0,y=0,z=0)}


def prefab():
    rr={}
    for go,t,components in [(1,2,[2,3]),(10,11,[11,12,13]),(20,21,[21,22,23])]:
        rr[go]=record(1,'GameObject',{'m_Name':str(go),'m_IsActive':1,'m_Layer':0,
            'm_Component':[{'component':{'fileID':c}} for c in components]})
    rr[2]=record(4,'Transform',transform(1,children=[11,21],position=[10,0,0]))
    rr[2]['fields']['m_LocalScale']=dict(x=2,y=3,z=1)
    rr[2]['fields']['m_LocalRotation']=dict(x=0,y=0,z=math.sqrt(.5),w=math.sqrt(.5))
    for go,t,r,f,pos in [(10,11,12,13,[1,2,0]),(20,21,22,23,[9,0,0])]:
        rr[t]=record(4,'Transform',transform(go,2,position=pos))
        rr[r]=record(23,'MeshRenderer',{'m_GameObject':{'fileID':go},'m_Enabled':1,
            'm_Materials':[{'fileID':2100000,'guid':'b'*32}], 'm_CastShadows':1})
        rr[f]=record(33,'MeshFilter',{'m_GameObject':{'fileID':go},'m_Mesh':{'fileID':43,'guid':'c'*32}})
    rr[3]=record(205,'LODGroup',{'m_GameObject':{'fileID':1},'m_Enabled':1,
        'm_LODs':[{'renderers':[{'renderer':{'fileID':i}}]} for i in (12,22)]})
    return rr


def override(i,path,value='',reference=None):
    return {'target':{'fileID':i,'guid':GUID},'propertyPath':path,'value':value,
            'objectReference':reference or {'fileID':0}}


def write_records(path,rr):
    path.write_text(''.join(f'--- !u!{v["kind"]} &{i}'+(' stripped' if v['stripped'] else '')+'\n'+
        yaml.safe_dump({v['name']:v['fields']}) for i,v in rr.items()))


class PrefabTests(unittest.TestCase):
    def test_nested_nonuniform_rotation_preserves_matrix_and_selects_lod0(self):
        result=static_prefab(prefab())
        self.assertEqual([d['renderer'] for d in result['draws']],[12])
        self.assertEqual(result['excluded_lod_renderers'],1)
        matrix=np.array(result['draws'][0]['matrix'])
        np.testing.assert_allclose(matrix@[0,0,0,1],[4,2,0,1],atol=1e-12)
        np.testing.assert_allclose(matrix@[1,0,0,1],[4,4,0,1],atol=1e-12)

    def test_overrides_preserve_source_and_apply_child_active_material_and_transform(self):
        rr=prefab();before=copy.deepcopy(rr)
        mods=[override(2,'m_LocalPosition.x','20'),override(12,'m_Materials.Array.data[0]',reference={'guid':'d'*32,'fileID':2100000})]
        result=static_prefab(rr,mods,GUID)
        self.assertEqual(result['draws'][0]['materials'][0]['guid'],'d'*32)
        self.assertAlmostEqual(result['draws'][0]['matrix'][0][3],14)
        self.assertEqual(rr,before)
        for mod in [override(1,'m_IsActive','0'),override(10,'m_IsActive','0'),override(12,'m_Enabled','0')]:
            self.assertEqual(static_prefab(rr,[mod],GUID)['draws'],[])

    def test_bad_overrides_and_unresolved_nested_models_reject(self):
        rr=prefab()
        for mod in [override(2,'m_LocalPosition.x','nan'),
                    override(2,'m_Something','1'),override(12,'m_Materials.Array.data[2]',reference={'guid':'d'*32,'fileID':1})]:
            with self.assertRaises(ValueError):static_prefab(rr,[mod],GUID)
        mod=override(2,'m_LocalPosition.x',1);mod['target']['guid']='d'*32
        with self.assertRaises(ValueError):static_prefab(rr,[mod],GUID)
        rr[2]['stripped']=True
        with self.assertRaisesRegex(ValueError,'Nested/model'):static_prefab(rr)

    def test_bad_hierarchy_lod_and_numeric_data_reject(self):
        for kind in ['children','parent','lod','component','quaternion','nan','disabled_lod']:
            rr=prefab()
            if kind=='children':rr[2]['fields']['m_Children']=[]
            if kind=='parent':rr[11]['fields']['m_Father']['fileID']=999
            if kind=='lod':rr[3]['fields']['m_LODs'][0]['renderers'][0]['renderer']['fileID']=999
            if kind=='component':rr[10]['fields']['m_Component']=[]
            if kind=='quaternion':rr[2]['fields']['m_LocalRotation']=dict(x=0,y=0,z=0,w=0)
            if kind=='nan':rr[2]['fields']['m_LocalScale']['x']=float('nan')
            if kind=='disabled_lod':rr[3]['fields']['m_Enabled']=0
            with self.assertRaises(ValueError,msg=kind):static_prefab(rr)

    def test_disconnected_transform_cycle_rejects(self):
        rr=prefab()
        for go,t,parent,child in [(30,31,41,41),(40,41,31,31)]:
            rr[go]=record(1,'GameObject',{'m_IsActive':1,'m_Name':str(go),'m_Component':[{'component':{'fileID':t}}]})
            rr[t]=record(4,'Transform',transform(go,parent,[child]))
        with self.assertRaisesRegex(ValueError,'Cyclic'):static_prefab(rr)

    def test_real_serialized_scene_parent_and_stripped_transform_bindings(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);write_records(root/'source.prefab',prefab())
            rr={100:record(1,'GameObject',{'m_Name':'parent','m_IsActive':1}),
                101:record(4,'Transform',transform(100,position=[100,0,0]))}
            for i,parent in [(501,101),(502,601)]:
                rr[i]=record(1001,'PrefabInstance',{'m_SourcePrefab':{'guid':GUID,'fileID':100100000},
                    'm_Modification':{'m_TransformParent':{'fileID':parent},'m_RemovedComponents':[],
                        'm_Modifications':[override(2,'m_LocalPosition.x','20')] if i==501 else []}})
            rr[601]=record(4,'Transform',{'m_CorrespondingSourceObject':{'guid':GUID,'fileID':11},
                                        'm_PrefabInstance':{'fileID':501}});rr[601]['stripped']=True
            write_records(root/'scene.unity',rr)
            class Sources:
                by_guid={GUID:'source.prefab'}
                manifest={'archive_sha256':'0'*64}
                def checked(self,name):return root/name
                def metadata(self,name):pass
            result=scene_plan(Sources(),'scene.unity')
            self.assertEqual(result['unresolved'],[])
            self.assertEqual(result['counts']['resolved_instances'],2)
            points=[np.array(d['matrix'])@[0,0,0,1] for d in result['draws']]
            np.testing.assert_allclose(points,[[114,2,0,1],[108,10,0,1]],atol=1e-12)
            rr[501]['fields']['m_Modification']['m_TransformParent']['fileID']=601
            write_records(root/'scene.unity',rr)
            broken=scene_plan(Sources(),'scene.unity')
            self.assertEqual(broken['draws'],[])
            self.assertEqual(broken['counts']['unresolved_instances'],2)

    def test_unused_invalid_local_target_is_reported_only_for_flat_sources(self):
        mod=override(999,'m_Materials.Array.data[0]',reference={'guid':'d'*32,'fileID':2100000})
        rr=prefab();result=static_prefab(rr,[mod],GUID)
        self.assertEqual(result['draws'],static_prefab(rr)['draws'])
        self.assertEqual(result['unused_target_overrides'],[{'target':mod['target'],'propertyPath':mod['propertyPath']}])
        rr[900]=record(1001,'PrefabInstance',{})
        with self.assertRaisesRegex(ValueError,'Nested/model'):static_prefab(rr,[mod],GUID)

    def test_explicit_preview_lod_selection_keeps_default_highest_detail(self):
        self.assertEqual([d['renderer'] for d in static_prefab(prefab(),lod_level=1)['draws']],[22])
        self.assertEqual([d['renderer'] for d in static_prefab(prefab())['draws']],[12])
        for invalid in [-1,2,True]:
            with self.assertRaises(ValueError):static_prefab(prefab(),lod_level=invalid)
