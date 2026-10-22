"""Actual Blender mesh API checks for source-corner-preserving quad splits."""
import copy
from pathlib import Path
import sys
import unittest
import bpy
import numpy as np
sys.path.insert(0,str(Path(__file__).resolve().parent))
from unity_editor_topology import triangulate


def fixture():
    bpy.ops.wm.read_factory_settings(use_empty=True)
    mesh=bpy.data.meshes.new('quad');mesh.from_pydata([(0,0,0),(1,0,0),(1,1,0),(0,1,0)],[],[(0,1,2,3)])
    for name,scale in [('primary',1),('lightmap',.5)]:
        uv=mesh.uv_layers.new(name=name)
        for i,value in enumerate([(0,0),(1,0),(1,1),(0,1)]):uv.data[i].uv=np.array(value)*scale
    mesh.uv_layers.active_index=0;mesh.uv_layers['primary'].active_render=True
    mesh.calc_loop_triangles();original={frozenset(t.vertices) for t in mesh.loop_triangles}
    chosen=[[0,1,3],[1,2,3]] if frozenset([0,1,2]) in original else [[0,1,2],[0,2,3]]
    ref={'positions':[0,0,0,-1,0,0,-1,0,-1,0,0,-1],
         'submeshes':[{'topology':0,'indices':[i for face in chosen for i in [face[0],face[2],face[1]]]}]}
    return mesh,ref,chosen


class EditorTopologyTests(unittest.TestCase):
    def test_opposite_diagonal_keeps_positions_both_uv_layers_and_source_normals(self):
        mesh,ref,chosen=fixture();new,report,normals=triangulate(mesh,ref,np.eye(4))
        self.assertEqual(report['changed_quad_diagonals'],1);self.assertEqual(report['source_quads'],1)
        self.assertEqual(new.uv_layers.active.name,mesh.uv_layers.active.name)
        self.assertEqual([u.active_render for u in new.uv_layers],[u.active_render for u in mesh.uv_layers])
        self.assertEqual({frozenset(p.vertices) for p in new.polygons},{frozenset(t) for t in chosen})
        np.testing.assert_array_equal([v.co[:] for v in new.vertices],[v.co[:] for v in mesh.vertices])
        for scale,layer in [(1,new.uv_layers['primary']),(.5,new.uv_layers['lightmap'])]:
            for loop in new.loops:np.testing.assert_array_equal(layer.data[loop.index].uv[:],np.array(new.vertices[loop.vertex_index].co[:2])*scale)
        np.testing.assert_allclose(normals,[[0,0,1]]*6,atol=1e-7)
        np.testing.assert_allclose([n.vector[:] for n in new.corner_normals],normals,atol=1e-6)
        self.assertEqual(len(mesh.polygons),1)

    def test_missing_ambiguous_and_wrong_slot_reference_triangles_reject(self):
        for fault in ['missing','duplicate','slot','nonfinite']:
            mesh,ref,_=fixture();value=copy.deepcopy(ref)
            if fault=='missing':value['submeshes'][0]['indices']=value['submeshes'][0]['indices'][:3]
            if fault=='duplicate':value['submeshes'][0]['indices']=value['submeshes'][0]['indices'][:3]*2
            if fault=='slot':mesh.polygons[0].material_index=1
            transform=np.eye(4)
            if fault=='nonfinite':transform[0,0]=float('nan')
            with self.subTest(fault=fault),self.assertRaises(ValueError):triangulate(mesh,value,transform)


if __name__=='__main__':unittest.main()
