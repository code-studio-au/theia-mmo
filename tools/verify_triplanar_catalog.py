"""Check catalog input ownership, geometry, shading streams and role mappings.

Does not import the catalog assembler or material authoring helpers. Inputs must
already have their own source qualification; this proves faithful composition.
"""
import argparse
import copy
import hashlib
import json
import re
from pathlib import Path
import numpy as np
from merge_static_glb import read
from static_geometry import accessor, hierarchy
from verify_foliage_batch_scene import material_signature


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def verify(stage):
    stage = Path(stage)
    receipt = json.loads((stage / 'catalog-assembly.json').read_text())
    config = json.loads((stage / 'triplanar.json').read_text())
    if sha(stage / 'preview.glb') != receipt['asset_sha256'] or config['geometry_sha256'] != receipt['asset_sha256'] or sha(stage / 'triplanar.json') != receipt['metadata_sha256']:
        raise ValueError('Catalog geometry/metadata binding differs')
    if config['schema'] != 2 or config['profile'] != 'synty-triplanar-catalog-v1':
        raise ValueError('Catalog schema/profile differs')
    profiles = {v['id']: v for v in config['profiles']}
    if len(profiles) != len(config['profiles']) or set(profiles) != {v['profile_id'] for v in receipt['inputs'] if v['profile_id'] is not None}:
        raise ValueError('Catalog profile input mapping differs')
    doc, binary = read(stage / 'preview.glb', allow_external_images=True)
    parents, world = hierarchy(doc)
    roots = {}
    for root in doc['scenes'][0]['nodes']:
        identifier = doc['nodes'][root].get('extras', {}).get('theia_catalog_input')
        if type(identifier) is not int or identifier in roots:
            raise ValueError('Duplicate/missing catalog input owner')
        roots[identifier] = root
    if set(roots) != set(range(len(receipt['inputs']))):
        raise ValueError('Missing/extra catalog input root')
    owned = {i: [] for i in roots}
    for node, frame in world.items():
        if 'mesh' not in doc['nodes'][node]: continue
        root = node
        while parents[root] is not None: root = parents[root]
        owner, = [i for i, value in roots.items() if value == root]
        owned[owner].append((node, frame))
    material_cache = {}
    def signature(document, data, index, cache, root):
        if index not in cache: cache[index] = material_signature(document, data, index, root=root)
        return copy.deepcopy(cache[index])
    mesh_instances = triangles = 0
    bounds = []
    image_paths = set()
    external_paths = []
    for index, record in enumerate(receipt['inputs']):
        asset = Path(record['asset'])
        if sha(asset) != record['asset_sha256']:
            raise ValueError('Catalog source asset changed')
        source_doc, source_binary = read(asset, allow_external_images=True)
        external = [image['uri'] for image in source_doc.get('images', []) if 'uri' in image]
        expected_external = []
        for uri in external:
            if not re.fullmatch(r'shared/[A-Za-z0-9_.-]+\.png', uri):
                raise ValueError('Unqualified source external image path')
            original = (asset.parent / uri).resolve(strict=True)
            destination = f'shared/catalog-{index}-{Path(uri).name}'
            copied = (stage / destination).resolve(strict=True)
            if not original.is_relative_to(asset.parent.resolve()) or not copied.is_relative_to(stage.resolve()):
                raise ValueError('External image escapes root')
            if sha(original) != sha(copied):
                raise ValueError('Catalog external image bytes changed')
            expected_external.append({'source_uri': uri, 'uri': destination, 'sha256': sha(original)})
        if expected_external != record.get('external_images', []):
            raise ValueError('Catalog external image inventory changed')
        external_paths.extend(v['uri'] for v in expected_external)
        _, source_world = hierarchy(source_doc)
        source_nodes = [(n, m) for n, m in source_world.items() if 'mesh' in source_doc['nodes'][n]]
        if len(source_nodes) != len(owned[index]):
            raise ValueError('Catalog source mesh coverage differs')
        source_materials = {}
        identifier = record['profile_id']
        if identifier is not None:
            metadata = Path(record['metadata'])
            if sha(metadata) != record['metadata_sha256'] or sha(metadata.parent / 'independent-verification.json') != record['source_proof_sha256']:
                raise ValueError('Catalog source profile/proof changed')
            value = json.loads(metadata.read_text())
            original_proof = json.loads((metadata.parent / 'independent-verification.json').read_text())
            if not original_proof['passed'] or original_proof['asset_sha256'] != record['asset_sha256']:
                raise ValueError('Catalog source qualification differs')
            profile = profiles[identifier]
            for key in ['profile', 'tiling', 'falloff']:
                if profile[key] != value[key]: raise ValueError('Catalog source profile scalars differ')
            for role, original in value['textures'].items():
                expected = copy.deepcopy(original)
                kind = 'normal' if role.endswith('normal') else 'color'
                expected['file'] = f'triplanar-{kind}-{original["sha256"]}.png'
                if profile['textures'][role] != expected:
                    raise ValueError('Catalog source texture role mapping differs')
                path = stage / expected['file']
                if sha(path) != original['sha256'] or path.read_bytes() != (metadata.parent / original['file']).read_bytes():
                    raise ValueError('Catalog source texture bytes differ')
                image_paths.add(expected['file'])
        for (source_node, source_frame), (node, frame) in zip(source_nodes, owned[index]):
            if not np.array_equal(source_frame, frame):
                raise ValueError('Catalog source world matrix changed')
            expected = source_doc['meshes'][source_doc['nodes'][source_node]['mesh']]['primitives']
            observed = doc['meshes'][doc['nodes'][node]['mesh']]['primitives']
            if len(expected) != len(observed): raise ValueError('Catalog primitive set changed')
            for before, after in zip(expected, observed):
                if set(before['attributes']) != set(after['attributes']) or before.get('mode', 4) != after.get('mode', 4):
                    raise ValueError('Catalog vertex stream set changed')
                for key, accessor_id in before['attributes'].items():
                    if not np.array_equal(accessor(source_doc, source_binary, accessor_id), accessor(doc, binary, after['attributes'][key])):
                        raise ValueError('Catalog vertex stream changed')
                indices = accessor(source_doc, source_binary, before['indices'])
                if not np.array_equal(indices, accessor(doc, binary, after['indices'])):
                    raise ValueError('Catalog triangle winding changed')
                material = signature(source_doc, source_binary, before['material'], source_materials, asset.parent)
                if identifier is not None: material['extras']['theia_triplanar'] = identifier
                if signature(doc, binary, after['material'], material_cache, stage) != material:
                    raise ValueError('Catalog source material/image changed')
                positions = accessor(source_doc, source_binary, before['attributes']['POSITION']).astype(float)
                positions = positions @ source_frame[:3, :3].T + source_frame[:3, 3]
                bounds.extend([positions.min(0), positions.max(0)])
                triangles += indices.size // 3
                mesh_instances += 1
    if len(image_paths) != receipt['unique_images']:
        raise ValueError('Catalog unique image inventory differs')
    if [v['uri'] for v in doc.get('images', []) if 'uri' in v] != external_paths:
        raise ValueError('Catalog external image variants changed')
    bounds = np.array(bounds)
    return {'passed': True, 'asset_sha256': receipt['asset_sha256'], 'profiles': len(profiles), 'unique_images': len(image_paths),
            'mesh_instances': mesh_instances, 'triangles': triangles, 'authored_materials': len(doc['materials']),
            'images': len(doc.get('images', [])), 'source_meshes': len(doc['meshes']),
            'expected_bounds': [bounds.min(0).tolist(), bounds.max(0).tolist()],
            'scope': 'complete input mesh ownership/poses/topology/UV/normal/tangent/material/image/profile composition; source qualification remains in bound predecessors'}


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__); p.add_argument('stage', type=Path); p.add_argument('--report', type=Path, required=True)
    a = p.parse_args()
    if a.report.exists(): raise FileExistsError('Use a fresh catalog report')
    result = verify(a.stage); a.report.write_text(json.dumps(result, indent=2) + '\n'); print(json.dumps(result))
