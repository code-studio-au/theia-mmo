"""Bound profile/image dependencies and explicit package capability version."""
import copy
import hashlib
import json
from pathlib import Path
import tempfile
import unittest
from add_terrain_house import write_glb
from asset_tests.test_unity_reframe import fixture
from index_chunk_contributors import package_bounds
from material_profiles import parse, profiles
from package_world import identity, package
from shared_images import filename

ROOT=Path(__file__).resolve().parents[2]


def extra(uris=None):
    if uris is None:uris=[f"../shared/{'0'*64}-{role}-10497-10497-9729-9729.png" for role in ['s','l']]
    return {'theia_triplanar':1,'theia_triplanar_profile':{'schema':1,'profile':'synty-triplanar-world-v1',
        'id':1,'tiling':.5,'falloff':4.1,'source_anchor':[128e9,0,-128e9],
        'textures':dict(top=uris[0],side=uris[0],top_normal=uris[1],side_normal=uris[1])}}


class BoundMaterialTests(unittest.TestCase):
    def test_projected_identity_matches_rust_golden_and_preserves_legacy_marks(self):
        path=ROOT/'crates/network/tests/fixtures/content-manifest-projected.json'
        self.assertEqual(identity(json.loads(path.read_text())),'3183cb4425e6bc0f9f7b069e1a8b909c1c5f7d845edf5852bc359168f493a61d')
        self.assertIsNone(parse({'theia_triplanar':1}))
        data=extra();self.assertEqual(parse(data)['source_anchor'],[128e9,0.,-128e9])
        alias=copy.deepcopy(data);alias['theia_triplanar']=alias['theia_triplanar_profile']['id']=2
        # Source-local numeric IDs do not change the material's content key.
        self.assertEqual(len(profiles({'materials':[{'extras':data},{'extras':alias}]})),2)
        materials=[]
        for n in range(17):
            item=copy.deepcopy(data);item['theia_triplanar']=item['theia_triplanar_profile']['id']=n%16+1
            materials.append({'extras':item})
        self.assertEqual(len(profiles({'materials':materials})),17)
        for n,item in enumerate(materials):item['extras']['theia_triplanar_profile']['tiling']=(n+1)*.1
        with self.assertRaisesRegex(ValueError,'profile budget'):profiles({'materials':materials})

    def test_rebound_unknown_fields_f32_underflow_frames_roles_and_urls_reject(self):
        for fault in ['unknown','schema','profile','marker','boolean_marker','zero_tiling','underflow','boolean_scalar','falloff','anchor','boolean_anchor','missing_role','role','sampler','path','url']:
            value=extra();p=value['theia_triplanar_profile']
            if fault=='unknown':p['shader']='arbitrary.wgsl'
            if fault=='schema':p['schema']=2
            if fault=='profile':p['profile']='arbitrary'
            if fault=='marker':value['theia_triplanar']=2
            if fault=='boolean_marker':value['theia_triplanar']=True
            if fault=='zero_tiling':p['tiling']=0
            if fault=='underflow':p['tiling']=1e-100
            if fault=='boolean_scalar':p['tiling']=True
            if fault=='falloff':p['falloff']=101
            if fault=='anchor':p['source_anchor']=[1e13,0,0]
            if fault=='boolean_anchor':p['source_anchor']=[True,0,0]
            if fault=='missing_role':p['textures'].pop('side')
            if fault=='role':p['textures']['top']=p['textures']['top_normal']
            if fault=='sampler':p['textures']['top']=p['textures']['top'].replace('9729-9729','9728-9728')
            if fault=='path':p['textures']['top']='../../private.png'
            if fault=='url':p['textures']['top']='https://example.invalid/image.png'
            with self.subTest(fault=fault),self.assertRaises(ValueError):parse(value)

    def test_actual_pngs_and_indexed_geometry_bind_to_v3_without_external_metadata(self):
        with tempfile.TemporaryDirectory() as temporary:
            root=Path(temporary);world=root/'chunk_0_0';world.mkdir();shared=root/'shared';shared.mkdir()
            png=(ROOT/'crates/network/tests/fixtures/shared-image.png').read_bytes()
            descriptors=[];uris=[]
            for color in ['srgb','linear']:
                descriptor=dict(sha256=list(hashlib.sha256(png).digest()),bytes=len(png),color_space=color,sampler=[10497,10497,9729,9729])
                name=filename(descriptor);(shared/name).write_bytes(png);descriptors.append(descriptor);uris.append('../shared/'+name)
            document,binary=fixture();document['materials'][0]['extras']=extra(uris)
            write_glb(document,binary,world/'terrain.glb')
            profile={'schema':4,'bounds':[[-8,-8],[8,8]],'source_origin':[0,0,0],
                'visual_sha256':list(hashlib.sha256((world/'terrain.glb').read_bytes()).digest()),'triangles':[],
                'spawns':[],'static_shapes':{'heightfields':[dict(rows=2,columns=2,heights_column_major=[0]*4,center=[0,0,0],scale=[16,1,16])]}}
            (world/'terrain.json').write_text(json.dumps(profile))
            result=package(world,'theia.test.projected',1,shared_images=True)
            manifest=json.loads((world/'world.manifest.json').read_text())
            self.assertEqual(manifest['schema'],3);self.assertEqual(len(manifest['shared_images']),2)
            bounds=package_bounds(world,list(bytes.fromhex(result['identity'])))
            self.assertLess(bounds[0][0],-8);self.assertGreater(bounds[1][0],8)
            (shared/filename(descriptors[1])).write_bytes(b'corrupt')
            with self.assertRaisesRegex(ValueError,'hash disagrees'):package_bounds(world,list(bytes.fromhex(result['identity'])))
