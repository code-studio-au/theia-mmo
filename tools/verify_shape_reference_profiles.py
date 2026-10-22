"""Independent complete source/profile shape comparison, beyond sampled rays.

Compares every affine mesh vertex/index, each box's complete corner set, capsule
endpoints/radius and every field sample with the source fixtures. Does not import
the profile author, its bounds calculation, quaternion library or Avian.
"""
import argparse
import itertools
import json
from pathlib import Path
import numpy as np
from unity_editor_reference import digest, read_json


def require(condition, message):
    if not condition: raise ValueError(message)


def verify(objects, terrain, directory):
    directory = Path(directory)
    reference = read_json(directory/'reference.json',2*1024*1024)
    require(reference['schema']==1 and reference['profile']=='schema4-source-shape-reference-v1','Unknown shape reference')
    require(reference['objects_sha256']==digest(objects) and reference['heightfield_sha256']==digest(terrain),'Source bindings differ')
    source = read_json(objects,128*1024*1024)
    ground = read_json(terrain,128*1024*1024)
    bodies = {body['id']:body for body in source['colliders']}
    require(len(bodies)==len(source['colliders']),'Repeated source identity')
    profiles = {}
    for entry in reference['profiles']:
        owner = tuple(entry['owner'])
        require(len(owner)==2 and all(type(v) is int for v in owner) and owner not in profiles,'Invalid profile owner')
        require(entry['file']==f'profile_{owner[0]}_{owner[1]}.json','Unexpected profile path')
        path = directory/entry['file']
        require(entry['sha256']==digest(path),'Profile binding differs')
        profile = read_json(path,32*1024*1024)
        require(profile['schema']==4 and profile['source_origin']==[owner[0]*128,0,owner[1]*128],'Profile frame differs')
        require(not any(profile.get(k,[]) for k in ['triangles','solids','camera_solids','walk_surfaces','spawns']),'Unexpected reference-only legacy geometry/spawn')
        profiles[owner] = profile['static_shapes']
    seen,addresses = set(),set()
    maximum = 0.
    vertices_compared = 0
    signs = np.array(list(itertools.product([-1.,1.],repeat=3)))
    for address in reference['objects']:
        identity,owner,index = address['id'],tuple(address['owner']),address['collider']
        require(identity in bodies and identity not in seen and (owner,index) not in addresses,'Unbound/repeated object address')
        seen.add(identity);addresses.add((owner,index))
        body = bodies[identity]
        shapes = profiles[owner]
        shape = shapes['objects'][index]
        require(shape.get('queries',7)==7,'Source object query layers differ')
        origin = np.array([owner[0]*128,0,owner[1]*128])
        matrix = np.array(body['matrix']).reshape(4,4)
        if body['kind']=='MeshCollider':
            require(shape['kind']=='mesh' and shape['convex']==body['convex'],'Mesh kind/hull differs')
            geometry = source['meshes'][body['mesh']]
            mesh = shapes['meshes'][shape['mesh']]
            require(mesh['indices']==geometry['indices'] and mesh['vertices']==geometry['vertices'],'Complete mesh pool geometry differs')
            transform = np.array(shape['affine']).reshape(4,4)
            require(np.array_equal(transform[3],[0,0,0,1]),'Projective mesh transform')
            points = np.array(geometry['vertices'])
            actual = points @ transform[:3,:3].T + transform[:3,3] + origin
            expected = points @ matrix[:3,:3].T + matrix[:3,3]
            error = float(abs(actual-expected).max())
            vertices_compared += len(points)
        elif body['kind']=='BoxCollider':
            require(shape['kind']=='box','Box kind differs')
            # Native primitive rotation is the orthogonal source frame; native
            # extents use its recorded lossy scale, not a mesh affine/shear.
            axes = matrix[:3,:3]/np.linalg.norm(matrix[:3,:3],axis=0)
            require(abs(axes.T@axes-np.eye(3)).max()<1e-5 and np.linalg.det(axes)>0,'Unqualified primitive source frame')
            expected_center = matrix[:3,:3] @ body['center']+matrix[:3,3]
            expected = (signs*np.array(body['size'])*abs(np.array(body['lossy_scale']))/2) @ axes.T + expected_center
            quaternion = np.array(shape['rotation'])
            require(abs(np.dot(quaternion,quaternion)-1)<1e-4,'Nonunit box quaternion')
            quaternion /= np.linalg.norm(quaternion)
            vectors = signs*np.array(shape['size'])/2
            cross = np.cross(quaternion[:3],vectors)
            actual = vectors + 2*quaternion[3]*cross + 2*np.cross(quaternion[:3],cross) + np.array(shape['center']) + origin
            distances = np.linalg.norm(actual[:,None,:]-expected[None,:,:],axis=2)
            error = float(max(distances.min(0).max(),distances.min(1).max()))
        elif body['kind']=='CapsuleCollider':
            require(shape['kind']=='capsule','Capsule kind differs')
            axis = body['direction']; scale = abs(np.array(body['lossy_scale']))
            radius = body['radius']*max(np.delete(scale,axis))
            require(abs(shape['radius']-radius)<1e-5,'Capsule radius differs')
            half = max(body['height']*scale[axis]/2-radius,0)
            direction = matrix[:3,axis]/np.linalg.norm(matrix[:3,axis])
            center = matrix[:3,:3] @ body['center']+matrix[:3,3]
            expected = np.array([center-direction*half,center+direction*half])
            actual = np.array([np.array(shape['center'])-shape['half_segment'],np.array(shape['center'])+shape['half_segment']])+origin
            distances = np.linalg.norm(actual[:,None,:]-expected[None,:,:],axis=2)
            error = float(max(distances.min(0).max(),distances.min(1).max()))
        else: raise ValueError('Unknown source collider kind')
        require(error <= .0001,f'Complete source shape differs: {identity}: {error}')
        maximum = max(maximum,error)
    require(seen==set(bodies),'Source object coverage differs')
    require(len(addresses)==sum(len(s.get('objects',[])) for s in profiles.values()),'Unexpected profile object')
    require(len(reference['terrain'])==4,'Full terrain requires four source quarters')
    rows,columns = ground['rows'],ground['columns']
    samples = np.array(ground['heights_column_major'],dtype=np.float32).reshape(columns,rows).T
    coverage = np.zeros((rows,columns),dtype=np.uint8)
    field_addresses = set()
    for address in reference['terrain']:
        owner = tuple(address['owner']); shapes = profiles[owner]
        index = address['collider']-len(shapes['objects'])
        require((owner,index) not in field_addresses,'Repeated terrain address')
        field_addresses.add((owner,index))
        field = shapes['heightfields'][index]
        require(field.get('queries',7)==7,'Source field query layers differ')
        require(field['rows']==257 and field['columns']==257 and field['scale']==[75,1,75],'Full field dimensions differ')
        center = np.array(field['center'])+np.array([owner[0]*128,0,owner[1]*128])
        start = center[[0,2]]-np.array(field['scale'])[[0,2]]/2
        source_start = np.array(ground['center'])[[0,2]]-np.array(ground['scale'])[[0,2]]/2
        offset = (start-source_start)/(np.array(ground['scale'])[[0,2]]/[columns-1,rows-1])
        x,z = np.rint(offset).astype(int)
        require(abs(offset-[x,z]).max()<1e-8 and x in (0,256) and z in (0,256) and center[1]==ground['center'][1],'Field source origin differs')
        values = np.array(field['heights_column_major'],dtype=np.float32).reshape(257,257).T
        require(np.array_equal(values,samples[z:z+257,x:x+257]),'Full height sample data differs')
        coverage[z:z+257,x:x+257]+=1
    require(coverage.min()==1 and coverage.max()==4,'Incomplete/repeated full terrain coverage')
    require(len(field_addresses)==sum(len(s.get('heightfields',[])) for s in profiles.values()),'Unexpected profile field')
    return {'passed':True,'objects':len(seen),'affine_vertices_compared':vertices_compared,
            'max_shape_error_metres':maximum,'unique_height_samples':int(coverage.size),
            'reference_sha256':digest(directory/'reference.json'),
            'scope':'Independent complete geometry/primitive/sample source binding; native query tests remain separate'}


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('objects',type=Path);parser.add_argument('terrain',type=Path);parser.add_argument('directory',type=Path)
    args=parser.parse_args()
    print(json.dumps(verify(args.objects,args.terrain,args.directory),indent=2))
