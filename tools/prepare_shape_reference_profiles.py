"""Route verified object/heightfield fixtures through real schema-4 profiles.

Reference-only output: no visual manifests or spawns are emitted. The downward
water plane remains in this complete geometry comparison; gameplay admission
requires a separate explicit water policy.
"""
import argparse
import collections
import copy
import json
from pathlib import Path
import shutil
import numpy as np
from trimesh.transformations import quaternion_from_matrix
from unity_editor_reference import digest, read_json
from analyze_collision_layout import bounds
from static_shapes import validate


def prepare(objects, terrain, output):
    output = Path(output).resolve()
    build = Path(__file__).resolve().parents[1] / 'build'
    if output.exists() or output == build or not output.is_relative_to(build):
        raise ValueError('Use a fresh ignored schema-4 reference directory')
    for path in [objects, terrain]:
        if read_json(Path(path).with_suffix('.receipt.json'), 1024 * 1024)['fixture_sha256'] != digest(path):
            raise ValueError('Verified source fixture binding differs')
    document = read_json(objects, 128 * 1024 * 1024)
    ground = read_json(terrain, 128 * 1024 * 1024)
    if document['profile'] != 'unity-object-collision-comparison-v1' or ground['profile'] != 'source-full-grid-heightfield-reference-v1' or (ground['rows'], ground['columns']) != (513, 513):
        raise ValueError('Unsupported source shape references')
    cells = {}
    object_map = []
    terrain_map = []

    def cell(owner):
        if owner not in cells:
            cells[owner] = {'mesh_map': {}, 'shapes': {'meshes': [], 'objects': [], 'heightfields': []}}
        return cells[owner]

    for body in document['colliders']:
        box = bounds(body, document['meshes'])
        owner = tuple(int(v) for v in np.floor(box.mean(0)[[0, 2]] / 128))
        entry = cell(owner)
        shapes = entry['shapes']
        origin = np.array([owner[0] * 128, 0, owner[1] * 128], dtype=float)
        matrix = np.array(body['matrix'], dtype=float).reshape(4, 4)
        if body['kind'] == 'MeshCollider':
            index = body['mesh']
            if index not in entry['mesh_map']:
                entry['mesh_map'][index] = len(shapes['meshes'])
                shapes['meshes'].append(copy.deepcopy(document['meshes'][index]))
            matrix[:3, 3] -= origin
            shape = {'kind': 'mesh', 'mesh': entry['mesh_map'][index],
                     'affine': matrix.astype(np.float32).reshape(-1).tolist(), 'convex': body['convex']}
        elif body['kind'] == 'BoxCollider':
            linear = matrix[:3, :3]
            axes = linear / np.linalg.norm(linear, axis=0)
            if abs(axes.T @ axes - np.eye(3)).max() > .0001 or np.linalg.det(axes) <= 0:
                raise ValueError('Source box shear/mirroring needs explicit qualification')
            rotation = np.eye(4)
            rotation[:3, :3] = axes
            q = quaternion_from_matrix(rotation)
            q /= np.linalg.norm(q)
            center = linear @ body['center'] + matrix[:3, 3] - origin
            shape = {'kind': 'box', 'center': center.astype(np.float32).tolist(),
                     'size': (np.array(body['size']) * abs(np.array(body['lossy_scale']))).astype(np.float32).tolist(),
                     'rotation': q[[1,2,3,0]].astype(np.float32).tolist()}
        elif body['kind'] == 'CapsuleCollider':
            axis = body['direction']
            scale = abs(np.array(body['lossy_scale']))
            radius = body['radius'] * max(scale[i] for i in range(3) if i != axis)
            half = max(body['height'] * scale[axis] - 2 * radius, 0) / 2
            direction = matrix[:3, axis] / np.linalg.norm(matrix[:3, axis])
            center = matrix[:3, :3] @ body['center'] + matrix[:3, 3] - origin
            shape = {'kind': 'capsule', 'center': center.astype(np.float32).tolist(),
                     'radius': float(np.float32(radius)), 'half_segment': (direction * half).astype(np.float32).tolist()}
        else:
            raise ValueError('Unknown source object shape')
        object_map.append({'id': body['id'], 'owner': list(owner), 'collider': len(shapes['objects'])})
        shapes['objects'].append(shape)
    heights = np.array(ground['heights_column_major'], dtype=np.float32).reshape(513, 513)
    for x in range(2):
        for z in range(2):
            center = np.array(ground['center'], dtype=float)
            center[[0,2]] += (np.array([x,z]) - .5) * np.array(ground['scale'])[[0,2]] / 2
            owner = tuple(int(v) for v in np.floor(center[[0,2]] / 128))
            shapes = cell(owner)['shapes']
            local = center - np.array([owner[0]*128, 0, owner[1]*128])
            field = {'rows':257,'columns':257,
                     'heights_column_major':heights[x*256:x*256+257,z*256:z*256+257].reshape(-1).tolist(),
                     'center':local.astype(np.float32).tolist(),
                     'scale':[ground['scale'][0]/2, ground['scale'][1], ground['scale'][2]/2]}
            terrain_map.append({'owner':list(owner),'collider':len(shapes['objects'])+len(shapes['heightfields'])})
            shapes['heightfields'].append(field)
    output.mkdir(parents=True)
    try:
        profiles = []
        for owner, entry in sorted(cells.items()):
            checked = validate(entry['shapes'])
            profile = {'schema':4,'bounds':[[0,0],[128,128]],
                       'source_origin':[owner[0]*128,0,owner[1]*128],
                       'visual_sha256':[0]*32,'triangles':[],'spawns':[],
                       'static_shapes':entry['shapes']}
            name = f'profile_{owner[0]}_{owner[1]}.json'
            (output/name).write_text(json.dumps(profile,separators=(',',':'),allow_nan=False)+'\n')
            profiles.append({'owner':list(owner),'file':name,'sha256':digest(output/name),'metrics':checked})
        result = {'schema':1,'profile':'schema4-source-shape-reference-v1',
                  'objects_sha256':digest(objects),'heightfield_sha256':digest(terrain),
                  'profiles':profiles,'objects':object_map,'terrain':terrain_map,
                  'scope':'Reference-only complete source shape data through schema-4 reader; zero visual hash/no spawns/no manifests; no runtime map adoption or water policy'}
        (output/'reference.json').write_text(json.dumps(result,indent=2)+'\n')
        return {'profiles':len(profiles),'objects':len(object_map),'heightfields':len(terrain_map)}
    except BaseException:
        shutil.rmtree(output)
        raise


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('objects',type=Path)
    parser.add_argument('terrain',type=Path)
    parser.add_argument('output',type=Path)
    args = parser.parse_args()
    print(json.dumps(prepare(args.objects,args.terrain,args.output)))
