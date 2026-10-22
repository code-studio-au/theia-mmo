#!/usr/bin/env python3
"""Measure verified package costs and conservative XZ residency demand.

This is offline budget evidence, not a simulation, reachable-route proof or a
resident CPU/GPU memory measurement. No runtime cap or active metadata changes.
"""
import argparse
import itertools
import json
import math
from pathlib import Path
import struct
import time

from index_chunk_contributors import overlaps, package_bounds
from merge_static_glb import read
from package_world import file_hash, identity, MAX_COLLISION_BYTES, MAX_VISUAL_BYTES
from shared_images import canonical, filename, MAX_IMAGE_BYTES, png_pixels, validate_references


def f32(value):
    return struct.unpack('<f', struct.pack('<f', value))[0]


# Match the shared controller's f32 radius + skin and authority's four-metre
# lookahead. This is a policy-specific oracle; changes to authority policy need
# remeasurement before using this result to choose a runtime budget.
RADIUS = f32(f32(.35) + f32(.005))
LOOKAHEAD = 4.


def proposed_control_sizes(capacity=32):
    """Conservative JSON sizing only; these messages are not runtime admission."""
    chunks = [[-7_812_500_000 + n, -7_812_500_000] for n in range(capacity)]
    maximum = 2**64 - 1
    base = {'version': 2**32 - 1, 'epoch': maximum}
    messages = {'ready': dict(base, sequence=maximum, chunks=chunks),
                'ack': {'kind': 'Ack', 'payload': dict(base, sequence=maximum, chunks=chunks, tick=maximum)},
                'offer': {'kind': 'Offer', 'payload': dict(base, generation=maximum, chunks=chunks)}}
    return {'proposed_capacity': capacity, 'bytes': {key: len(json.dumps(value, separators=(',', ':')).encode()) for key, value in messages.items()},
            'scope': 'JSON size estimate using full-width headers and checked-global-range chunk IDs; runtime version/count/address admission unqualified'}


def contributors(entries):
    ids = {tuple(entry['id']) for entry in entries}
    if not ids or len(ids) != len(entries) or len(ids) > 64:
        raise ValueError('Invalid or over-budget candidate entry IDs')
    return {cell: {cell} | {tuple(entry['id']) for entry in entries if overlaps(entry['id'], entry['bounds'], cell)} for cell in ids}


def footprint(point, closure):
    cells = {(math.floor((point[0] + dx) / 128), math.floor((point[1] + dz) / 128))
             for dx, dz in itertools.product([-RADIUS, 0., RADIUS], repeat=2)}
    return cells, set().union(*(closure.get(cell, set()) for cell in cells))


def axis_samples(cells, axis):
    """One representative per open interval plus every ownership breakpoint."""
    planes = {128 * (cell[axis] + side) for cell in cells for side in [0, 1]}
    offsets = {ahead + edge for ahead in [-LOOKAHEAD, 0., LOOKAHEAD] for edge in [-RADIUS, 0., RADIUS]}
    edges = sorted({plane - offset for plane in planes for offset in offsets})
    return sorted(set(edges) | {(a + b) / 2 for a, b in zip(edges, edges[1:])})


def demand_patterns(closure, recovery):
    """All stencil breakpoints in this finite XZ index, ignoring walkability.

    All eight falling lookahead endpoints dominate an intentional endpoint
    within four metres: cells are 128m wide. Unknown future cells add no provider,
    matching authority policy. Current body footprints must be fully indexed.
    """
    ids = sorted(closure)
    bits = {cell: 1 << n for n, cell in enumerate(ids)}
    mask = lambda required: sum(bits[cell] for cell in required)
    recovery_mask = mask(recovery)
    required_patterns, falling_patterns = {}, {}
    for point in itertools.product(axis_samples(ids, 0), axis_samples(ids, 1)):
        cells, current = footprint(point, closure)
        if not cells.issubset(closure):
            continue
        required = mask(current)
        required_patterns.setdefault(required, list(point))
        future = set(current)
        for dx, dz in itertools.product([-LOOKAHEAD, 0., LOOKAHEAD], repeat=2):
            future.update(footprint([point[0] + dx, point[1] + dz], closure)[1])
        falling_patterns.setdefault(mask(future), list(point))
    if not required_patterns:
        raise ValueError('No complete actor footprint in candidate index')

    def maxima(patterns):
        values = list(patterns)
        one = max(values, key=lambda value: (recovery_mask | value).bit_count())
        pair = max(itertools.combinations_with_replacement(values, 2), key=lambda pair: (recovery_mask | pair[0] | pair[1]).bit_count())
        state, witnesses = recovery_mask, []
        for _ in range(8):
            chosen = max(values, key=lambda value: (state | value).bit_count())
            if state | chosen == state:
                break
            state |= chosen
            witnesses.append(patterns[chosen])
        total = (1 << len(ids)) - 1
        return {'one_actor_packages': (recovery_mask | one).bit_count(),
                'one_actor_point': patterns[one], 'two_actor_packages': (recovery_mask | pair[0] | pair[1]).bit_count(),
                'two_actor_points': [patterns[value] for value in pair],
                'eight_actor_greedy_packages': state.bit_count(), 'eight_actor_greedy_points': witnesses,
                'eight_actor_maximum_proven': state == total,
                'indexed_package_upper_bound': len(ids), 'distinct_actor_provider_sets': len(values)}
    return {'footprint_radius_metres': RADIUS, 'lookahead_metres': LOOKAHEAD,
            'scope': 'conservative indexed-XZ stencil union; no height, obstacles, reachability or live timing proof',
            'current_footprints': maxima(required_patterns), 'falling_lookahead': maxima(falling_patterns)}


def measure(directory, source_world=None, global_origin=(0., 0.)):
    directory = directory.resolve(strict=True)
    index_path = directory / 'world.chunks.candidate.json'
    if not index_path.exists():
        index_path = directory / 'world.chunks.json'
    index = json.loads(index_path.read_bytes())
    if index['schema'] != 2 or index['chunk_metres'] != 128:
        raise ValueError('Measurement requires schema-2 contributor metadata')
    closure = contributors(index['chunks'])
    report = json.loads((directory / 'partition-report.json').read_bytes())
    recovery = {tuple(cell) for cell in report['initial_required_packages']}
    if not recovery.issubset(closure):
        raise ValueError('Recovery providers outside index')
    spawn_provenance = {'verified': False, 'scope': 'Recovery provider IDs declared by the partition report'}
    if source_world is not None:
        manifest = json.loads((source_world / 'world.manifest.json').read_bytes())
        size, digest = file_hash(source_world / 'terrain.json', MAX_COLLISION_BYTES)
        if size != manifest['collision_bytes'] or list(digest) != manifest['collision_sha256']:
            raise ValueError('Source spawn collision binding mismatch')
        profile = json.loads((source_world / 'terrain.json').read_bytes())
        expected = set()
        for point in profile['spawns']:
            cells, required = footprint([point[0] + global_origin[0], point[2] + global_origin[1]], closure)
            if not cells.issubset(closure):
                raise ValueError('Source spawn footprint outside candidate index')
            expected.update(required)
        if expected != recovery:
            raise ValueError('Recovery providers disagree with source spawn footprints')
        spawn_provenance = {'verified': True, 'source_collision_sha256': digest.hex(),
                            'global_origin_xz': list(global_origin), 'spawns': len(profile['spawns'])}
    images, packages = {}, {}
    started = time.perf_counter()
    for entry in index['chunks']:
        cell = tuple(entry['id'])
        package = directory / f'chunk_{cell[0]}_{cell[1]}'
        began = time.perf_counter()
        manifest = json.loads((package / 'world.manifest.json').read_bytes())
        if manifest['schema'] != 2 or bytes.fromhex(identity(manifest)) != bytes(entry['package']):
            raise ValueError('Candidate package identity mismatch')
        for name, field, limit in [('terrain.glb', 'visual', MAX_VISUAL_BYTES), ('terrain.json', 'collision', MAX_COLLISION_BYTES)]:
            count, digest = file_hash(package / name, limit)
            if count != manifest[field + '_bytes'] or list(digest) != manifest[field + '_sha256']:
                raise ValueError('Package artifact binding mismatch')
        document, _ = read(package / 'terrain.glb', allow_external_images=True)
        validate_references(document, manifest['shared_images'])
        profile = json.loads((package / 'terrain.json').read_bytes())
        names = []
        for image in canonical(manifest['shared_images']):
            name = filename(image)
            names.append(name)
            if name not in images:
                path = directory / 'shared' / name
                if path.resolve(strict=True) != path:
                    raise ValueError('Shared image alias is unsupported')
                count, digest = file_hash(path, MAX_IMAGE_BYTES)
                if count != image['bytes'] or list(digest) != image['sha256']:
                    raise ValueError('Shared image binding mismatch')
                images[name] = {'encoded_bytes': count, 'rgba8_base_estimate': png_pixels(path) * 4}
            elif images[name]['encoded_bytes'] != image['bytes']:
                raise ValueError('Conflicting shared image descriptor')
        if package_bounds(package, entry['package']) != entry['bounds']:
            raise ValueError('Candidate contributor bounds disagree with actual package geometry')
        packages[cell] = {'id': list(cell), 'visual_bytes': manifest['visual_bytes'], 'collision_bytes': manifest['collision_bytes'],
                          'collision_triangles': len(profile['triangles']),
                          'static_colliders': bool(profile['triangles']) + len(profile.get('solids', [])) + len(profile.get('camera_solids', [])),
                          'images': names, 'verification_ms': (time.perf_counter() - began) * 1000}

    def costs(cells):
        selected = [packages[cell] for cell in cells]
        names = set().union(*(set(item['images']) for item in selected))
        return {'packages': len(selected), 'visual_bytes': sum(p['visual_bytes'] for p in selected),
                'collision_bytes': sum(p['collision_bytes'] for p in selected),
                'collision_triangles': sum(p['collision_triangles'] for p in selected),
                'static_colliders': sum(p['static_colliders'] for p in selected), 'shared_image_variants': len(names),
                'shared_image_encoded_bytes': sum(images[name]['encoded_bytes'] for name in names),
                'shared_image_rgba8_base_estimate': sum(images[name]['rgba8_base_estimate'] for name in names)}
    result = {'schema': 1, 'scope': 'offline verified encoded costs and base RGBA8 estimates; no resident memory or live load benchmark',
              'index_path': str(index_path), 'spawn_provenance': spawn_provenance,
              'recovery': costs(recovery), 'all_indexed': costs(packages),
              'maximum_cell_contributors': max(map(len, closure.values())),
              'demand': demand_patterns(closure, recovery), 'packages': list(packages.values()),
              'proposed_control_sizes': proposed_control_sizes(),
              'verification_wall_ms': (time.perf_counter() - started) * 1000,
              'staging_upper_bound': 'Active and one candidate can retain two collision scenes. The candidate instantiates only added package roots, so their asset dependency union stays within the finite index. Identical PNG URI keys share handles. This does not measure engine allocations, cache lifetimes, mipmaps or driver copies.'}
    return result


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('directory', type=Path)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--source-world', type=Path)
    parser.add_argument('--global-origin', type=float, nargs=2, default=(0., 0.), metavar=('X', 'Z'))
    args = parser.parse_args()
    result = measure(args.directory, args.source_world, args.global_origin)
    with args.output.open('x') as file:
        file.write(json.dumps(result, indent=2) + '\n')
    print(json.dumps({key: result[key] for key in ('recovery', 'all_indexed', 'demand')}))
