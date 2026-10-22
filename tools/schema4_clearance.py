"""Independent capsule/source-shape clearance for native multiplayer evidence.

Uses float64 geometry and exact segment distances, not Avian. Camera-only shapes
are excluded. This does not infer navigation or a highest floor under overhangs.
"""
import itertools
import math
import numpy as np
from collision_geometry import mesh_capsule_clearance, segment_segment_sq
from static_shapes import validate
from qualify_collision_geometry import convex_surface

BOX_FACES = np.array([[0,1,3],[0,3,2],[4,6,7],[4,7,5],[0,4,5],[0,5,1],
                      [2,3,7],[2,7,6],[0,2,6],[0,6,4],[1,5,7],[1,7,3]])


def quaternion_matrix(values):
    x,y,z,w = np.array(values, dtype=float) / np.linalg.norm(values)
    return np.array([[1-2*(y*y+z*z),2*(x*y-z*w),2*(x*z+y*w)],
                     [2*(x*y+z*w),1-2*(x*x+z*z),2*(y*z-x*w)],
                     [2*(x*z-y*w),2*(y*z+x*w),1-2*(x*x+y*y)]])


def profile_capsule_clearance(profile, player):
    data = profile['static_shapes']
    validate(data, len(profile.get('triangles', [])), len(profile.get('solids', [])) + len(profile.get('camera_solids', [])) + bool(profile.get('triangles')))
    x,z = player['xz']
    feet = player['y'] - .9
    p,q = (x,feet+.35,z),(x,feet+1.45,z)
    middle = np.array([x,player['y'],z])
    triangles = list(profile.get('triangles', []))
    nearest = math.inf
    bodies = list(data.get('objects', []))
    bodies += [dict(kind='box',rotation=[0,0,0,1],**box) for box in profile.get('solids', [])]
    for body in bodies:
        if not body.get('queries',7) & 3: continue
        if body['kind'] == 'mesh':
            mesh = data['meshes'][body['mesh']]
            matrix = np.array(body['affine']).reshape(4,4)
            vertices = np.array(mesh['vertices']) @ matrix[:3,:3].T + matrix[:3,3]
            faces = np.array(mesh['indices'])
            surface = vertices[faces]
            if body['convex']:
                convex_surface(mesh['vertices'],mesh['indices'])
                n = np.cross(surface[:,1]-surface[:,0],surface[:,2]-surface[:,0])
                sign = np.einsum('ij,ij->i',n,vertices.mean(0)-surface[:,0]) > 0
                n[sign] *= -1
                if (np.einsum('ij,ij->i',n,middle-surface[:,0]) <= 0).all(): nearest = min(nearest,-.35)
            triangles.extend(surface.tolist())
        elif body['kind'] == 'box':
            rotation = quaternion_matrix(body['rotation'])
            half = np.array(body['size'])/2
            center = np.array(body['center'])
            if (abs(rotation.T @ (middle-center)) <= half).all(): nearest = min(nearest,-.35)
            corners = np.array(list(itertools.product([-1.,1.],repeat=3)))*half
            corners = corners @ rotation.T + center
            triangles.extend(corners[BOX_FACES].tolist())
        elif body['kind'] == 'capsule':
            center,half = np.array(body['center']),np.array(body['half_segment'])
            nearest = min(nearest,math.sqrt(segment_segment_sq(p,q,center-half,center+half))-.35-body['radius'])
    for field in data.get('heightfields', []):
        if not field.get('queries',7) & 3: continue
        rows,columns = field['rows'],field['columns']
        heights = np.array(field['heights_column_major']).reshape(columns,rows).T
        scale,center = np.array(field['scale']),np.array(field['center'])
        start = center[[0,2]]-scale[[0,2]]/2
        spacing = scale[[0,2]]/[columns-1,rows-1]
        low = np.floor((np.array([x,z])-.4-start)/spacing).astype(int)
        high = np.floor((np.array([x,z])+.4-start)/spacing).astype(int)
        def point(i,j):return [start[0]+j*spacing[0],heights[i,j]*scale[1]+center[1],start[1]+i*spacing[1]]
        for i in range(max(0,low[1]),min(rows-2,high[1])+1):
            for j in range(max(0,low[0]),min(columns-2,high[0])+1):
                a,b,c,d = point(i,j),point(i+1,j),point(i,j+1),point(i+1,j+1)
                triangles.extend([[a,b,c],[b,d,c]])
    return min(nearest,mesh_capsule_clearance(triangles,player))
