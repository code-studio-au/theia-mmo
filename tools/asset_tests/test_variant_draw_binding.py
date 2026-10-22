"""Canonical variant conversion must retain inherited IDs and raw scene bindings."""
import copy
import hashlib
import json
from pathlib import Path
import sys
import tempfile
import unittest
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
import test_unity_prefab_ids as inherited
from test_unity_prefab_ids import INSTANCE,SOURCE_RENDERER
from test_unity_prefabs import record,write_records,override
from unity_prefab_ids import compile_single_model_variant
from unity_prefabs import static_prefab
from unity_draw_bindings import select_draw
from verify_unity_static import variant_renderer
from verify_unity_foliage import reference_materials
from unity_scene import records


def fixture(root):
    sources,rr,mods=inherited.InheritedIdTests().fixture(root);sources.by_guid.update({'a'*32:'variant.prefab','d'*32:'alternate.mat'})
    renderer=(INSTANCE^SOURCE_RENDERER)&((1<<63)-1)
    mods.append(override(renderer,'m_Materials.Array.data[0]',reference={'guid':'d'*32,'fileID':2100000}))
    scene={501:record(1001,'PrefabInstance',{'m_SourcePrefab':{'guid':'a'*32,'fileID':100100000},'m_Modification':{'m_TransformParent':{'fileID':0},'m_Modifications':mods}})}
    write_records(root/'scene.unity',scene)
    sources.manifest={'archive_sha256':'0'*64,'selected':{name:{'guid':guid,'sha256':hashlib.sha256((root/name).read_bytes()).hexdigest()} for name,guid in [('scene.unity','f'*32),('variant.prefab','a'*32)]}}
    compiled=compile_single_model_variant(sources,'variant.prefab',mods,'a'*32);draw,=static_prefab(compiled,mods,'a'*32)['draws'];draw.update(instance=501,source='variant.prefab')
    plan={'archive_sha256':'0'*64,'scene':'scene.unity','lod_policy':'highest-detail first serialized LOD; no runtime crossfade','draws':[draw]};path=root/'plan.json';path.write_text(json.dumps(plan))
    return sources,path,renderer,scene,plan


class VariantDrawTests(unittest.TestCase):
    def test_explicit_variant_path_matches_independent_raw_alias_mesh_and_material_overrides(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);sources,path,renderer,scene,plan=fixture(root)
            with self.assertRaises(ValueError):select_draw(sources,path,501,renderer,'variant.prefab')
            draw,_,binding=select_draw(sources,path,501,renderer,'variant.prefab',allow_model_variant=True)
            self.assertEqual(draw['materials'],[{'guid':'d'*32,'fileID':2100000}]);self.assertEqual(binding['source_kind'],'single-model-variant')
            conversion={'prefab':'variant.prefab','renderer':renderer,'scene_binding':binding};r,m=variant_renderer(sources,conversion,records(root/'variant.prefab'))
            self.assertEqual(m['m_Mesh'],draw['mesh']);self.assertEqual(r['m_Materials'][0]['guid'],'c'*32)
            self.assertEqual(reference_materials(sources,conversion,r,m),draw['materials'])
            bad=dict(conversion,renderer=renderer+1)
            with self.assertRaisesRegex(ValueError,'inherited renderer'):variant_renderer(sources,bad,records(root/'variant.prefab'))
            plan['draws'][0]['materials'][0]['guid']='c'*32;path.write_text(json.dumps(plan));binding['plan_sha256']=hashlib.sha256(path.read_bytes()).hexdigest()
            with self.assertRaisesRegex(ValueError,'material slots'):reference_materials(sources,conversion,r,m)

    def test_missing_authored_trs_and_structural_override_reject_before_conversion(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);sources,path,renderer,scene,plan=fixture(root)
            for kind in ['trs','structure']:
                bad=copy.deepcopy(scene)
                if kind=='trs':bad[501]['fields']['m_Modification']['m_Modifications'].pop(0)
                else:bad[501]['fields']['m_Modification']['m_AddedComponents']=[{'fileID':99}]
                write_records(root/'scene.unity',bad);sources.manifest['selected']['scene.unity']['sha256']=hashlib.sha256((root/'scene.unity').read_bytes()).hexdigest()
                with self.subTest(kind=kind),self.assertRaises(ValueError):select_draw(sources,path,501,renderer,'variant.prefab',allow_model_variant=True)
