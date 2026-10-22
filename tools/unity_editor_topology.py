"""Use recorded Editor triangle diagonals without changing source corner data.

Only source triangles/quads are supported. Original source loops supply UVs and
normals; actual Editor geometry supplies the triangulation and material slots.
"""
from collections import Counter
import bpy
import numpy as np


def key(points):
    rows=[tuple(v) for v in points]
    return min(tuple(rows[i:]+rows[:i]) for i in range(3))


def triangulate(mesh,reference,transform):
    vertices=np.array([v.co[:] for v in mesh.vertices],dtype=np.float32)
    canonical=vertices[:,[0,2,1]].copy();canonical[:,2]*=-1
    transform=np.asarray(transform,dtype=float)
    if transform.shape!=(4,4) or not np.isfinite(transform).all() or not np.array_equal(transform[3],[0,0,0,1]):raise ValueError('Invalid recorded topology frame')
    actual=np.asarray(canonical@transform[:3,:3].T+transform[:3,3],dtype=np.float32)
    editor=np.asarray(reference['positions'],dtype=np.float32).reshape(-1,3).copy();editor[:,0]*=-1
    remaining={}
    for slot,submesh in enumerate(reference['submeshes']):
        if submesh['topology']!=0 or len(submesh['indices'])%3:raise ValueError('Editor topology requires triangles')
        indices=np.asarray(submesh['indices'])
        if indices.dtype.kind not in 'iu' or indices.min()<0 or indices.max()>=len(editor):raise ValueError('Invalid Editor topology indices')
        remaining[slot]=Counter(key(t) for t in editor[indices.reshape(-1,3)[:,[0,2,1]]])
    polygons=list(mesh.polygons);normals=[v.vector[:] for v in mesh.corner_normals]
    active_uv=mesh.uv_layers.active_index
    layers=[(layer.name,[v.uv[:] for v in layer.data],layer.active_render) for layer in mesh.uv_layers]
    mesh.calc_loop_triangles();old_quads={}
    for t in mesh.loop_triangles:
        if len(polygons[t.polygon_index].vertices)==4:old_quads.setdefault(t.polygon_index,Counter()).update([key(actual[list(t.vertices)])])
    faces={};quads=changed=0
    # Consume existing triangular faces first; they can share a quad's corners.
    for p in polygons:
        if len(p.vertices) not in (3,4) or p.material_index not in remaining:raise ValueError('Topology repair accepts bound source triangles/quads only')
        if len(p.vertices)!=3:continue
        signature=key(actual[list(p.vertices)])
        if remaining[p.material_index][signature]<1:raise ValueError('Original source triangle is absent from Editor slot')
        remaining[p.material_index][signature]-=1;faces[p.index]=[list(p.loop_indices)]
    for p in polygons:
        if len(p.vertices)!=4:continue
        corners={tuple(actual[mesh.loops[i].vertex_index]):i for i in p.loop_indices}
        if len(corners)!=4:raise ValueError('Ambiguous coincident source quad corners')
        candidates=[k for k,n in remaining[p.material_index].items() if n and set(k)<=set(corners)]
        if len(candidates)!=2:raise ValueError('Editor quad triangles are missing/ambiguous')
        faces[p.index]=[[corners[point] for point in signature] for signature in candidates]
        for signature in candidates:remaining[p.material_index][signature]-=1
        quads+=1;changed+=int(old_quads[p.index]!=Counter(candidates))
    if any(sum(values.values()) for values in remaining.values()):raise ValueError('Editor triangles remain outside source polygons')
    output_faces=[];output_loops=[];slots=[]
    for p in polygons:
        for loops in faces[p.index]:
            output_faces.append([mesh.loops[i].vertex_index for i in loops]);output_loops.extend(loops);slots.append(p.material_index)
    new=bpy.data.meshes.new(mesh.name+'-EditorTriangles');new.from_pydata(vertices.tolist(),[],output_faces)
    for material in mesh.materials:new.materials.append(material)
    for polygon,slot in zip(new.polygons,slots):polygon.material_index=slot;polygon.use_smooth=True
    for name,values,render in layers:
        layer=new.uv_layers.new(name=name)
        for i,source in enumerate(output_loops):layer.data[i].uv=values[source]
        layer.active_render=render
    if layers:new.uv_layers.active_index=active_uv
    expected_normals=[normals[i] for i in output_loops]
    new.normals_split_custom_set(expected_normals);new.update()
    return new,{'source_polygons':len(polygons),'source_quads':quads,'changed_quad_diagonals':changed,'output_triangles':len(output_faces),
                'scope':'actual Editor triangle/slot selection; original source vertex, per-corner UV and normal data retained'},expected_normals
