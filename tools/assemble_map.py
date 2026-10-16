#!/usr/bin/env python3
"""Assemble one wow.export tile's static placements with an existing terrain GLB.

Install requirements-assets.txt. Outputs are new, local compatibility artifacts;
no gameplay colliders, lights, particles, animations or liquid surfaces are inferred.
"""
import argparse
import csv
import hashlib
import json
from pathlib import Path
import re

import numpy as np
import trimesh

from map_placement import adt_transform, doodad_transform
from merge_static_glb import merge


def csv_rows(path):
    with path.open(newline='', encoding='utf-8') as file:
        return list(csv.DictReader(file, delimiter=';'))


def placement_plan(csv_path):
    plan, inputs, ids, filtered = [], {csv_path.resolve()}, set(), 0
    for row in csv_rows(csv_path):
        identifier = row['ModelId']
        if not identifier or identifier in ids:
            raise ValueError('Missing/duplicate ADT model identity')
        ids.add(identifier)
        path = (csv_path.parent / row['ModelFile']).resolve()
        name = f"adt_{identifier}"
        plan.append((name, 'world', path, adt_transform(row)))
        sidecar = path.with_name(path.stem + '_ModelPlacementInformation.csv')
        if row['Type'] == 'wmo' and sidecar.exists():
            inputs.add(sidecar)
            selected = {s.strip() for s in row.get('DoodadSetNames', '').split(',')}
            selected.add('Set_$DefaultGlobal')
            for index, doodad in enumerate(csv_rows(sidecar)):
                if doodad['DoodadSet'] not in selected:
                    filtered += 1
                    continue
                child = (sidecar.parent / doodad['ModelFile']).resolve()
                plan.append((f'{name}_doodad_{index}', name, child, doodad_transform(doodad)))
    return plan, inputs, len(ids), filtered


def model_inputs(path):
    inputs, libraries, faces, used, groups = {path, path.with_suffix('.json')}, [], 0, set(), {}
    with path.open() as file:
        for line in file:
            parts = line.split()
            if not parts:
                continue
            if parts[0] == 'mtllib':
                libraries.extend(parts[1:])
            elif parts[0] == 'f':
                if len(parts) != 4:
                    raise ValueError(f'Non-triangulated model: {path}')
                faces += 1
            elif parts[0] == 'usemtl':
                used.add(parts[1])
            elif parts[0] == 'g':
                groups.setdefault(parts[1], len(groups))
    textures = {}
    for name in libraries:
        mtl = path.parent / name
        inputs.add(mtl)
        current = None
        with mtl.open() as file:
            for line in file:
                parts = line.strip().split(maxsplit=1)
                if parts and parts[0] == 'newmtl':
                    current = parts[1]
                elif parts and parts[0] == 'map_Kd':
                    texture = (mtl.parent / parts[1]).resolve()
                    textures[current] = texture
                    inputs.add(texture)
    if faces and not used.issubset(textures):
        raise ValueError(f'Missing material texture declaration: {path}')
    for source in inputs:
        if not source.is_file():
            raise ValueError(f'Missing model dependency: {source}')
    return inputs, faces, groups


def blend_mode(metadata, group, material, groups):
    if metadata['fileType'] == 'm2':
        match = re.match(r'Geoset_?(\d+)', group)
        section = int(match[1]) if match else groups.get(group)
        if section is None:
            raise ValueError(f'Unknown M2 geoset: {group}')
        units = [u for u in metadata['skin']['textureUnits'] if u['skinSectionIndex'] == section]
        if not units:
            raise ValueError('Missing M2 skin texture unit')
        primary = min(units, key=lambda u: u.get('materialLayer', 0))
        return metadata['materials'][primary['materialIndex']]['blendingMode']
    if metadata['fileType'] == 'wmo':
        texture = next(t['fileDataID'] for t in metadata['textures'] if t['mtlName'] == material)
        modes = {m['blendMode'] for m in metadata['materials'] if m['texture1'] == texture}
        if len(modes) != 1:
            raise ValueError('Ambiguous WMO material blend mode')
        return modes.pop()
    raise ValueError('Unsupported model metadata type')


def assemble(csv_path, terrain, output):
    objects = output.with_name(output.stem + '-objects.glb')
    if output.exists() or objects.exists():
        raise ValueError('Output files must be new')
    plan, inputs, top_count, filtered = placement_plan(csv_path)
    models = sorted({p for _, _, p, _ in plan})
    face_counts, groups = {}, {}
    for model in models:
        dependencies, faces, groups[model] = model_inputs(model)
        inputs.update(dependencies)
        face_counts[model] = faces
    inputs.add(terrain.resolve())
    hashes = {str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(inputs)}
    scene, pieces, modes = trimesh.Scene(), {}, {}
    for index, model in enumerate(models):
        if face_counts[model] == 0:
            pieces[model] = []
            continue
        loaded = trimesh.load_scene(model, process=False, group_material=False, split_groups=True)
        if sum(len(m.faces) for m in loaded.geometry.values()) != face_counts[model]:
            raise ValueError(f'Model triangle count changed: {model}')
        metadata = json.loads(model.with_suffix('.json').read_text())
        pieces[model] = []
        for part, (group, mesh) in enumerate(loaded.geometry.items()):
            if mesh.visual.kind != 'texture' or mesh.visual.uv is None or mesh.visual.material.image is None:
                raise ValueError(f'Model lost texture or UVs: {model}')
            mode = blend_mode(metadata, group, mesh.visual.material.name, groups[model])
            if mode not in (0, 1, 2, 4):
                raise ValueError(f'Unsupported blend mode {mode}: {model}')
            material = mesh.visual.material.to_pbr()
            material.baseColorFactor = [1., 1., 1., 1.]
            material.metallicFactor, material.roughnessFactor = 0., 1.
            material.doubleSided = True
            material.alphaMode = {0: 'OPAQUE', 1: 'MASK', 2: 'BLEND', 4: 'BLEND'}[mode]
            if mode == 1:
                material.alphaCutoff = 0.5
            mesh.visual.material = material
            modes[str(mode)] = modes.get(str(mode), 0) + 1
            key = f'model_{index}_part_{part}'
            scene.geometry[key] = mesh
            pieces[model].append(key)
    empty_instances, rendered, triangles = [], 0, 0
    for name, parent, model, matrix in plan:
        scene.graph.update(frame_to=name, frame_from=parent, matrix=matrix)
        if not pieces[model]:
            empty_instances.append({'instance': name, 'source': str(model)})
            continue
        rendered += 1
        triangles += face_counts[model]
        for index, key in enumerate(pieces[model]):
            scene.graph.update(frame_to=f'{name}_part_{index}', frame_from=name,
                               matrix=np.eye(4), geometry=key)
    data = trimesh.exchange.gltf.export_glb(scene, include_normals=True)
    if any(hashlib.sha256(Path(p).read_bytes()).hexdigest() != h for p, h in hashes.items()):
        raise ValueError('Source changed during assembly')
    with objects.open('xb') as file:
        file.write(data)
    merge(terrain, objects, output)
    return {'schema': 1, 'source_csv': str(csv_path.resolve()), 'source_hashes': hashes,
            'output': str(output.resolve()), 'sha256': hashlib.sha256(output.read_bytes()).hexdigest(),
            'bytes': output.stat().st_size, 'top_level_placements': top_count,
            'nested_placements': len(plan) - top_count, 'rendered_placements': rendered,
            'unique_model_files': len(models), 'unique_mesh_parts': len(scene.geometry),
            'instanced_object_triangles': triangles, 'object_bounds': scene.bounds.tolist(),
            'empty_geometry_instances': empty_instances, 'filtered_doodad_sets': filtered,
            'material_source_modes': modes, 'source_blend_4_approximation': 'standard PBR BLEND; no additive/glow shader',
            'scope': 'static visual assembly; no collision, navigation, lights, particles, liquids or gameplay authority'}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('placements', type=Path)
    parser.add_argument('terrain', type=Path)
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    print(json.dumps(assemble(args.placements, args.terrain, args.output), indent=2))
