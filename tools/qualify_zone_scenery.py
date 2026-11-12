#!/usr/bin/env python3
"""Compare published static scenery against its exact staged source, without a GPU."""
import argparse
import copy
import hashlib
import json
from pathlib import Path
import numpy as np
from merge_static_glb import read
from static_geometry import accessor, hierarchy, local_matrix


def image_bytes(doc, binary, index):
    image = doc['images'][index]
    view = doc['bufferViews'][image['bufferView']]
    start = view.get('byteOffset', 0)
    return binary[start:start + view['byteLength']]


def material(doc, binary, index):
    value = copy.deepcopy(doc['materials'][index])
    for owner, names in [(value, ('normalTexture', 'occlusionTexture', 'emissiveTexture')),
                         (value.get('pbrMetallicRoughness', {}), ('baseColorTexture', 'metallicRoughnessTexture'))]:
        for name in names:
            if name in owner:
                texture = doc['textures'][owner[name]['index']]
                owner[name]['index'] = dict(image_sha256=hashlib.sha256(image_bytes(doc, binary, texture['source'])).hexdigest(),
                                           sampler=doc['samplers'][texture['sampler']] if 'sampler' in texture else None)
    return value


def verify(zone_path, catalog_path, world):
    zone, catalog = json.loads(zone_path.read_bytes()), json.loads(catalog_path.read_bytes())
    assets = {a['id']: a for a in catalog['assets']}
    manifest = json.loads((world / 'world.manifest.json').read_bytes())
    profile_bytes = (world / 'terrain.json').read_bytes()
    visual_bytes = (world / 'terrain.glb').read_bytes()
    assert list(hashlib.sha256(profile_bytes).digest()) == manifest['collision_sha256']
    assert list(hashlib.sha256(visual_bytes).digest()) == manifest['visual_sha256']
    profile = json.loads(profile_bytes)
    assert profile['visual_sha256'] == manifest['visual_sha256']
    assert json.loads((world / 'authoring.zone.json').read_bytes()) == zone
    doc, binary = read(world / 'terrain.glb')
    _, matrices = hierarchy(doc)
    placements = zone['scenery']
    assert placements
    indices = doc['scenes'][0]['nodes'][-len(placements):]
    solids = [p for p in placements if assets[p['asset']]['collision']['kind'] == 'triangle_mesh']
    bodies = iter(profile['static_shapes']['objects'][-len(solids):] if solids else [])
    mesh_uses, checked = {}, []
    for placement, node_index in zip(placements, indices):
        asset = assets[placement['asset']]
        source_path = catalog_path.parent / asset['file']
        assert hashlib.sha256(source_path.read_bytes()).hexdigest() == asset['sha256']
        source, source_binary = read(source_path)
        node = doc['nodes'][node_index]
        assert node['name'] == placement['name']
        previous = mesh_uses.setdefault(asset['sha256'], node['mesh'])
        assert node['mesh'] == previous, 'Repeated asset must share one mesh'
        yaw = np.deg2rad(placement['yaw_degrees']) / 2
        # The renderer/collision contract uses bounded f32 local transforms after
        # the exact f64 global subtraction. Keep this independent of Rust export.
        local = (np.array(placement['position'], dtype=np.float64) - zone['origin']).astype(np.float32)
        expected = local_matrix(dict(translation=local.tolist(),
                                     rotation=[0, np.sin(yaw), 0, np.cos(yaw)], scale=[placement['scale']]*3))
        assert np.allclose(matrices[node_index], expected, atol=2e-6, rtol=0)
        original = source['meshes'][0]['primitives']
        published = doc['meshes'][node['mesh']]['primitives']
        assert len(original) == len(published)
        for before, after in zip(original, published):
            assert before['attributes'].keys() == after['attributes'].keys()
            for name, attribute in before['attributes'].items():
                assert np.array_equal(accessor(source, source_binary, attribute), accessor(doc, binary, after['attributes'][name]))
                for key in ('componentType', 'count', 'type', 'normalized'):
                    assert source['accessors'][attribute].get(key) == doc['accessors'][after['attributes'][name]].get(key)
            if 'indices' in before:
                assert np.array_equal(accessor(source, source_binary, before['indices']), accessor(doc, binary, after['indices']))
            assert material(source, source_binary, before['material']) == material(doc, binary, after['material'])
        if placement in solids:
            body = next(bodies)
            assert body['kind'] == 'mesh' and body['convex'] is False
            assert np.allclose(np.array(body['affine']).reshape((4,4)), expected, atol=2e-6, rtol=0)
            data = (catalog_path.parent / asset['collision']['file']).read_bytes()
            assert hashlib.sha256(data).hexdigest() == asset['collision']['sha256']
            assert profile['static_shapes']['meshes'][body['mesh']] == json.loads(data)
        checked.append(dict(id=placement['id'], asset=placement['asset'], mesh=node['mesh']))
    return dict(passed=True, placements=checked, unique_visuals=len(mesh_uses), solid_instances=len(solids),
                checks=['exact vertex attributes', 'exact indices', 'embedded image bytes', 'material parameters',
                        'sampler references', 'local transforms', 'shared meshes', 'collision surfaces', 'package hashes'],
                scope='CPU artifact comparison; rendering and connected gameplay not exercised')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--zone', type=Path, required=True)
    parser.add_argument('--catalog', type=Path, required=True)
    parser.add_argument('--world', type=Path, required=True)
    parser.add_argument('--report', type=Path, required=True)
    args = parser.parse_args()
    report = verify(args.zone, args.catalog, args.world)
    args.report.write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(report))
