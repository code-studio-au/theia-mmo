#!/usr/bin/env python3
"""Add static liquid geometry from wow.export JSON to a new local map GLB.

Geometry follows the upstream liquid renderer. Standard translucent PBR is a
preview material, without source-game liquid textures, refraction or physics.
"""
import argparse
import hashlib
import json
import math
from pathlib import Path

import numpy as np
import trimesh

from merge_static_glb import merge

UNIT_SIZE = (1600 / 3) / 16 / 8


def surface(instance):
    width, height = instance['width'], instance['height']
    x_offset, y_offset = instance['xOffset'], instance['yOffset']
    for value in (width, height, x_offset, y_offset):
        if type(value) is not int or value < 0:
            raise ValueError('Liquid dimensions/offsets must be nonnegative integers')
    if width + x_offset > 8 or height + y_offset > 8:
        raise ValueError('Liquid rectangle exceeds the chunk grid')
    if not width or not height:
        return None
    position = instance['terrainChunkPosition']
    if len(position) != 3 or not all(math.isfinite(v) for v in position):
        raise ValueError('Invalid liquid terrain chunk position')
    bitmap = instance.get('bitmap', [])
    if any(type(v) is not int or not 0 <= v <= 255 for v in bitmap):
        raise ValueError('Liquid bitmap must contain bytes')
    if bitmap and len(bitmap) < math.ceil(width * height / 8):
        raise ValueError('Truncated liquid visibility bitmap')
    count = (width + 1) * (height + 1)
    heights = instance.get('vertexData', {}).get('height', [])
    if not heights:
        low, high = instance['minHeightLevel'], instance['maxHeightLevel']
        if not math.isfinite(low) or low != high:
            raise ValueError('Missing nonconstant liquid vertex heights')
        heights = [low] * count
    if len(heights) != count or not all(math.isfinite(v) for v in heights):
        raise ValueError('Liquid vertex heights are incomplete or nonfinite')
    # Keep only referenced vertices: masked-out height samples may contain zero.
    vertices, faces, remap = [], [], {}

    def vertex(row, col):
        index = row * (width + 1) + col
        if index not in remap:
            remap[index] = len(vertices)
            vertices.append([position[1] - (col + x_offset) * UNIT_SIZE,
                             heights[index],
                             position[0] - (row + y_offset) * UNIT_SIZE])
        return remap[index]

    for row in range(height):
        for col in range(width):
            bit = row * width + col
            if bitmap and not (bitmap[bit // 8] & (1 << (bit % 8))):
                continue
            tl, tr = vertex(row, col), vertex(row, col + 1)
            bl, br = vertex(row + 1, col), vertex(row + 1, col + 1)
            # Upward-facing Y-up triangles, using the renderer's same diagonal.
            faces.extend([[tl, bl, tr], [tr, bl, br]])
    if not faces:
        return None
    points, triangles = np.asarray(vertices), np.asarray(faces)
    normals = np.zeros_like(points)
    face_normals = np.cross(points[triangles[:, 1]] - points[triangles[:, 0]],
                            points[triangles[:, 2]] - points[triangles[:, 0]])
    for corner in range(3):
        np.add.at(normals, triangles[:, corner], face_normals)
    lengths = np.linalg.norm(normals, axis=1)
    if not np.isfinite(lengths).all() or (lengths <= 0).any():
        raise ValueError('Degenerate liquid surface')
    # Area-weighted normals need only the pinned numpy dependency, not scipy.
    return trimesh.Trimesh(vertices=points, faces=triangles,
                           vertex_normals=normals / lengths[:, None], process=False)


def build_scene(document):
    chunks = document['liquidChunks']
    if not isinstance(chunks, list) or len(chunks) != 256:
        raise ValueError('Expected 256 liquid chunk slots')
    scene, records, empty = trimesh.Scene(), [], 0
    materials = {}
    for chunk_index, chunk in enumerate(chunks):
        if not chunk:
            continue
        for index, instance in enumerate(chunk.get('instances', [])):
            if not instance:
                continue
            if instance['chunkIndex'] != chunk_index or instance['instanceIndex'] != index:
                raise ValueError('Liquid identity disagrees with its slot')
            mesh = surface(instance)
            if mesh is None:
                empty += 1
                continue
            key = (instance['liquidType'], instance['liquidObject'])
            if key not in materials:
                materials[key] = trimesh.visual.material.PBRMaterial(
                    name=f'LiquidPreview_{key[0]}_{key[1]}',
                    baseColorFactor=[0.08, 0.28, 0.45, 0.72],
                    metallicFactor=0., roughnessFactor=0.2,
                    alphaMode='BLEND', doubleSided=True)
            mesh.visual = trimesh.visual.TextureVisuals(material=materials[key])
            name = f'liquid_chunk_{chunk_index}_instance_{index}'
            scene.add_geometry(mesh, node_name=name, geom_name=name)
            records.append({'chunk': chunk_index, 'instance': index,
                            'liquid_type': key[0], 'liquid_object': key[1],
                            'vertices': len(mesh.vertices), 'quads': len(mesh.faces) // 2,
                            'triangles': len(mesh.faces), 'bounds': mesh.bounds.tolist()})
    if not records:
        raise ValueError('No visible liquid geometry in export')
    return scene, records, empty


def add_liquids(source, map_asset, output):
    liquid_asset = output.with_name(output.stem + '-surfaces.glb')
    if output.exists() or liquid_asset.exists():
        raise ValueError('Output files must be new')
    hashes = {str(p.resolve()): hashlib.sha256(p.read_bytes()).hexdigest() for p in (source, map_asset)}
    scene, records, empty = build_scene(json.loads(source.read_text()))
    data = trimesh.exchange.gltf.export_glb(scene, include_normals=True)
    if any(hashlib.sha256(Path(p).read_bytes()).hexdigest() != h for p, h in hashes.items()):
        raise ValueError('Source changed during liquid conversion')
    with liquid_asset.open('xb') as file:
        file.write(data)
    merge(map_asset, liquid_asset, output)
    return {'schema': 1, 'source_hashes': hashes, 'output': str(output.resolve()),
            'sha256': hashlib.sha256(output.read_bytes()).hexdigest(), 'bytes': output.stat().st_size,
            'surfaces': records, 'empty_instances': empty,
            'quads': sum(r['quads'] for r in records), 'triangles': sum(r['triangles'] for r in records),
            'bounds': scene.bounds.tolist(), 'preview_material': 'static generic blue translucent PBR',
            'scope': 'exported liquid geometry only; no source-game texture/DB classification, refraction, flow, swimming, collision or authority'}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('source', type=Path)
    parser.add_argument('map', type=Path)
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    print(json.dumps(add_liquids(args.source, args.map, args.output), indent=2))
