#!/usr/bin/env python3
"""Independently compare exported wolf skinning with sampled Blender source poses."""
import argparse
import hashlib
import json
from pathlib import Path
import numpy as np
from verify_synty_character import read, skin, tracks, world, deformed


def verify(directory):
    proof = directory / 'verification.json'
    proof.unlink(missing_ok=True)
    path = directory / 'wolf.glb'
    doc, binary = read(path)
    reference = json.loads((directory / 'source-pose-reference.json').read_bytes())
    data = skin(doc, binary, expected_bones=41)
    assert data[1].shape[1] == 4
    assert {a['name'] for a in doc['animations']} == {'Idle', 'Walk', 'Run', 'Attack', 'Hit', 'Death'}
    assert len(doc['meshes']) == len(doc['materials']) == len(doc['images']) == 1
    conversion = np.array([[1,0,0,0],[0,0,1,0],[0,-1,0,0],[0,0,0,1]], dtype=float)
    bind = np.array(reference['bind_vertices_blender_world']) @ conversion[:3,:3].T
    positions = deformed(doc, data, world(doc))
    distances = np.linalg.norm(positions[:,None,:] - bind[None,:,:], axis=2)
    mapping = distances.argmin(axis=1)
    assert distances.min(axis=1).max() < 1e-5
    names = {n['name']: i for i,n in enumerate(doc['nodes'])}
    results = {}
    for animation in doc['animations']:
        name = animation['name']
        curves = tracks(doc, binary, animation)
        joint_error = skin_error = 0.
        low, high = np.full(3,np.inf), np.full(3,-np.inf)
        for frame in reference['clips'][name]:
            matrices = world(doc, curves, frame['time'])
            for bone, expected in frame['bones_blender_world'].items():
                wanted = (conversion @ np.array(expected))[:3,3]
                joint_error = max(joint_error, float(np.linalg.norm(matrices[names[bone]][:3,3]-wanted)))
            vertices = deformed(doc, data, matrices)
            wanted = np.array(frame['vertices_blender_world']) @ conversion[:3,:3].T
            skin_error = max(skin_error, float(np.linalg.norm(vertices - wanted[mapping], axis=1).max()))
            low, high = np.minimum(low, vertices.min(axis=0)), np.maximum(high, vertices.max(axis=0))
        assert joint_error < .0001, (name, joint_error)
        assert skin_error < .0001, (name, skin_error)
        assert max(high-low) < 4., (name, low, high)
        results[name] = {'frames':len(reference['clips'][name]), 'maximum_joint_error_metres':joint_error,
                         'maximum_skin_error_metres':skin_error,'animated_bounds_metres':[low.tolist(),high.tolist()]}
    result = {'passed':True,'sha256':hashlib.sha256(path.read_bytes()).hexdigest(),
              'bones':41,'vertices':len(data[0]),'clips':results,
              'scope':'Independent every-source-frame exported skin and joint comparison; not native gameplay'}
    proof.write_text(json.dumps(result,indent=2)+'\n')
    return result


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('directory', type=Path)
    print(json.dumps(verify(parser.parse_args().directory),indent=2))
