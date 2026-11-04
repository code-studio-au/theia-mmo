#!/usr/bin/env python3
"""Check that fourteen directional clips extend, rather than replace, player art."""
import argparse
import hashlib
import json
from pathlib import Path

import numpy as np
from static_geometry import local_matrix
from verify_synty_character import accessor, read, tracks, world

RETAINED = {'Idle', 'Walk', 'Run', 'Jump', 'Death', 'SwordIdle', 'SwordAttack', 'SwordHit'}
ADDED = {gait + direction for gait in ['Walk', 'Run']
         for direction in ['Back', 'Left', 'Right', 'ForwardLeft', 'ForwardRight', 'BackLeft', 'BackRight']}


def compare(before, after):
    old, old_binary = read(before)
    new, new_binary = read(after)
    old_names = {node['name']: n for n, node in enumerate(old['nodes'])}
    new_names = {node['name']: n for n, node in enumerate(new['nodes'])}
    assert len(old_names) == len(old['nodes']) and len(new_names) == len(new['nodes'])
    assert old_names.keys() == new_names.keys(), 'Changed node/skin contract'
    for name, index in old_names.items():
        a, b = old['nodes'][index], new['nodes'][new_names[name]]
        np.testing.assert_allclose(local_matrix(a), local_matrix(b), rtol=0, atol=1e-7,
                                   err_msg=f'Changed bind node {name}')
        assert sorted(old['nodes'][n]['name'] for n in a.get('children', [])) == sorted(
            new['nodes'][n]['name'] for n in b.get('children', [])), f'Changed parentage {name}'
    assert len(old['skins']) == len(new['skins']) == 1
    assert [old['nodes'][n]['name'] for n in old['skins'][0]['joints']] == [
        new['nodes'][n]['name'] for n in new['skins'][0]['joints']], 'Changed joint ordering'
    np.testing.assert_array_equal(accessor(old, old_binary, old['skins'][0]['inverseBindMatrices']),
                                  accessor(new, new_binary, new['skins'][0]['inverseBindMatrices']))
    assert len(old['meshes']) == len(new['meshes']) == 1
    a, b = old['meshes'][0]['primitives'], new['meshes'][0]['primitives']
    assert len(a) == len(b) == 1
    a, b = a[0], b[0]
    assert a.get('mode', 4) == b.get('mode', 4) and a.get('material') == b.get('material')
    assert a['attributes'].keys() == b['attributes'].keys()
    for name in a['attributes']:
        np.testing.assert_array_equal(accessor(old, old_binary, a['attributes'][name]),
                                      accessor(new, new_binary, b['attributes'][name]),
                                      err_msg=f'Changed mesh attribute {name}')
    np.testing.assert_array_equal(accessor(old, old_binary, a['indices']),
                                  accessor(new, new_binary, b['indices']))
    for field in ['materials', 'textures', 'samplers']:
        assert old.get(field) == new.get(field), f'Changed {field}'
    assert len(old['images']) == len(new['images'])
    for a, b in zip(old['images'], new['images']):
        assert a['mimeType'] == b['mimeType'] == 'image/png'
        va, vb = old['bufferViews'][a['bufferView']], new['bufferViews'][b['bufferView']]
        start_a, start_b = va.get('byteOffset', 0), vb.get('byteOffset', 0)
        assert old_binary[start_a:start_a+va['byteLength']] == new_binary[start_b:start_b+vb['byteLength']], 'Changed embedded palette'
    old_clips = {a['name']: a for a in old['animations']}
    new_clips = {a['name']: a for a in new['animations']}
    assert len(old_clips) == len(old['animations']) and len(new_clips) == len(new['animations'])
    assert old_clips.keys() == RETAINED and new_clips.keys() == RETAINED | ADDED
    maximum = 0.
    sampled = 0
    for name, clip in old_clips.items():
        old_curves = tracks(old, old_binary, clip)
        new_curves = tracks(new, new_binary, new_clips[name])
        contract = lambda doc, curves: {(doc['nodes'][n]['name'], path) for n, path, *_ in curves}
        assert contract(old, old_curves) == contract(new, new_curves), f'Changed tracks {name}'
        times = sorted({float(t) for _, _, values, _, _ in old_curves for t in values})
        new_times = sorted({float(t) for _, _, values, _, _ in new_curves for t in values})
        np.testing.assert_array_equal(times, new_times)
        # Include midpoints so unchanged key poses cannot conceal a changed
        # interpolation mode or curve between retained keys.
        times = sorted(set(times + [(a+b)/2 for a, b in zip(times, times[1:])]))
        for time in times:
            a, b = world(old, old_curves, time), world(new, new_curves, time)
            for bone, index in old_names.items():
                error = float(np.max(np.abs(a[index]-b[new_names[bone]])))
                assert error < 1e-5, (name, time, bone, error)
                maximum = max(maximum, error)
            sampled += 1
    return dict(retained_clips=len(old_clips), added_clips=len(ADDED), retained_pose_samples=sampled,
                maximum_retained_world_matrix_error=maximum, mesh_weights_binds_palette_unchanged=True,
                baseline_sha256=hashlib.sha256(before.read_bytes()).hexdigest(),
                candidate_sha256=hashlib.sha256(after.read_bytes()).hexdigest())


def verify(baseline, candidate, output):
    result = dict(passed=True, artifacts={name: compare(baseline/name, candidate/name)
                                        for name in ['character-source-skin.glb', 'character.glb']},
                  scope='Retained geometry/weights/binds/palette and eight original clips; no runtime or production-fidelity claim')
    with output.open('x') as target:
        target.write(json.dumps(result, indent=2)+'\n')
    return result


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('baseline', type=Path)
    parser.add_argument('candidate', type=Path)
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    print(json.dumps(verify(args.baseline, args.candidate, args.output), indent=2))
