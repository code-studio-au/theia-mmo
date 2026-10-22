"""Authoring pipeline preserves node transforms, separate visuals and source bytes."""
import hashlib
import json
from pathlib import Path
import sys
import tempfile
import unittest
import numpy as np
import trimesh
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from prepare_collision_world import prepare
from package_world import identity

class CollisionImportTests(unittest.TestCase):
    def write(self,path,scene):path.write_bytes(trimesh.exchange.gltf.export_glb(scene))

    def test_instances_transforms_spawns_and_package_binding(self):
        with tempfile.TemporaryDirectory() as temporary:
            root=Path(temporary);source=root/'source.glb';visual=root/'visual.glb';out=root/'world'
            scene=trimesh.Scene()
            mesh=trimesh.creation.box(extents=[4,1,4])
            scene.add_geometry(mesh,geom_name='floor',node_name='a',transform=trimesh.transformations.translation_matrix([24,-.5,24]))
            scene.graph.update(frame_to='b',frame_from='world',matrix=trimesh.transformations.translation_matrix([30,2.5,24]),geometry='floor')
            self.write(source,scene);self.write(visual,trimesh.Scene(mesh))
            original={p:p.read_bytes() for p in [source,visual]}
            report=prepare(visual,source,out,[[4,0,4]],'theia.test.mesh',3,origin=[20,0,20])
            profile=json.loads((out/'terrain.json').read_text());manifest=json.loads((out/'world.manifest.json').read_text())
            self.assertEqual(profile['schema'],3);self.assertEqual(manifest['collision_schema'],3)
            self.assertEqual(profile['spawns'],[[4,0,4]])
            self.assertEqual(report['triangles'],24);self.assertEqual(report['identity'],identity(manifest))
            points=np.array(profile['triangles']).reshape((-1,3))
            np.testing.assert_allclose(points.min(axis=0),[2,-1,2]);np.testing.assert_allclose(points.max(axis=0),[12,3,6])
            self.assertEqual(profile['visual_sha256'],list(hashlib.sha256((out/'terrain.glb').read_bytes()).digest()))
            for p,data in original.items():self.assertEqual(p.read_bytes(),data)
            with self.assertRaisesRegex(ValueError,'must be new'):prepare(visual,source,out,[],'theia.test.mesh',4)

    def test_chunk_has_no_implicit_spawns_and_invalid_input_is_not_published(self):
        with tempfile.TemporaryDirectory() as temporary:
            root=Path(temporary);source=root/'source.glb'
            self.write(source,trimesh.Scene(trimesh.creation.box(extents=[10,1,10])))
            out=root/'chunk';prepare(source,source,out,[],'theia.test.chunk',1)
            self.assertEqual(json.loads((out/'terrain.json').read_text())['spawns'],[])
            for spawns,origin in [([[99,0,0]],[0,0,0]),([[0,0,0]]*9,[0,0,0]),([],[99999,0,0])]:
                bad=root/'bad'
                with self.assertRaises(ValueError):prepare(source,source,bad,spawns,'theia.test.bad',1,origin)
                self.assertFalse(bad.exists())

    def test_verified_migration_keeps_geometry_and_visuals_changes_revision_and_preserves_sources(self):
        from upgrade_collision_world import upgrade
        from package_world import package
        with tempfile.TemporaryDirectory() as temporary:
            root=Path(temporary);source=root/'old';source.mkdir();out=root/'new'
            self.write(source/'terrain.glb',trimesh.Scene(trimesh.creation.box()))
            digest=list(hashlib.sha256((source/'terrain.glb').read_bytes()).digest())
            profile={'schema':2,'source_origin':[0,0,0],'bounds':[[-10,-10],[10,10]],'visual_sha256':digest,
                     'triangles':[[[-10,0,-10],[10,0,10],[10,0,-10]]],
                     'walk_surfaces':[[[-10,0,-10],[-10,0,10],[10,0,10]]],'step_height':.2,'solids':[]}
            (source/'terrain.json').write_text(json.dumps(profile));package(source,'theia.test.upgrade',1)
            original={p.name:p.read_bytes() for p in source.iterdir()}
            result=upgrade(source,out,[[0,0,0]],2)
            p=json.loads((out/'terrain.json').read_text());self.assertEqual(p['triangles'],profile['triangles']+profile['walk_surfaces'])
            self.assertNotIn('walk_surfaces',p);self.assertNotIn('step_height',p)
            self.assertEqual(result['revision'],2)
            self.assertEqual((out/'terrain.glb').read_bytes(),original['terrain.glb'])
            for name,data in original.items():self.assertEqual((source/name).read_bytes(),data)
            with self.assertRaisesRegex(ValueError,'newer revision'):upgrade(source,root/'stale',[[0,0,0]],1)
            (source/'terrain.glb').write_bytes(b'corrupt')
            with self.assertRaisesRegex(ValueError,'match manifest'):upgrade(source,root/'bad',[[0,0,0]],2)
