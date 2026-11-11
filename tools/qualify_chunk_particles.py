#!/usr/bin/env python3
"""Qualify a three-chunk x-axis fixture using an existing looping particle package.

The source fixture must use initial chunks [-1, 0] and [0, 0], and offer
[1, 0] during the standard green-floor transition. The particle package must
contain one looping effect and its catalog, as produced by the editor.
Runs against the caller's display and Vulkan validation environment.
"""
import argparse
import binascii
import hashlib
import json
from pathlib import Path
import shutil
import struct
import zlib

from package_world import identity
from verify_multiplayer import run


def oversized_texture_fixture(source, destination):
    """Create an identity-bound PNG just over the client's 32 MiB RGBA limit."""
    shutil.copytree(source, destination)
    chunk = destination / 'chunk_1_0'
    catalog_path = chunk / 'particles/catalog.json'
    catalog = json.loads(catalog_path.read_text())
    if len(catalog['textures']) != 1:
        raise ValueError('The budget case needs exactly one texture')
    texture = catalog['textures'][0]
    old_hash = bytes(texture['sha256']).hex()
    role = 's' if texture['color_space'] == 'srgb' else 'l'
    old_name = f'texture-{old_hash}-{role}.png'
    def png_chunk(name, data):
        return struct.pack('>I', len(data)) + name + data + struct.pack('>I', binascii.crc32(name + data))
    compressor = zlib.compressobj()
    row = b'\0' + bytes(4096 * 4)
    compressed = b''.join(compressor.compress(row) for _ in range(2049)) + compressor.flush()
    png = (b'\x89PNG\r\n\x1a\n' + png_chunk(b'IHDR', struct.pack('>IIBBBBB', 4096, 2049, 8, 6, 0, 0, 0))
           + png_chunk(b'IDAT', compressed) + png_chunk(b'IEND', b''))
    digest = hashlib.sha256(png).digest()
    new_name = f'texture-{digest.hex()}-{role}.png'
    (chunk / 'particles/textures' / old_name).unlink()
    (chunk / 'particles/textures' / new_name).write_bytes(png)
    texture.update(sha256=list(digest), bytes=len(png))
    effect = catalog['effects'][0]
    effect_path = chunk / 'particles/effects' / f"effect-{effect['id']}.ron"
    original = effect_path.read_bytes()
    updated = original.replace(old_name.encode(), new_name.encode())
    if updated == original:
        raise ValueError('Texture reference absent from effect')
    effect_path.write_bytes(updated)
    effect.update(sha256=list(hashlib.sha256(updated).digest()), bytes=len(updated))
    payload = json.dumps(catalog, separators=(',', ':')).encode()
    catalog_path.write_bytes(payload)
    manifest_path = chunk / 'world.manifest.json'
    manifest = json.loads(manifest_path.read_text())
    manifest.update(particle_bytes=len(payload), particle_sha256=list(hashlib.sha256(payload).digest()))
    manifest_path.write_text(json.dumps(manifest))
    index_path = destination / 'world.chunks.json'
    index = json.loads(index_path.read_text())
    entry, = (entry for entry in index['chunks'] if entry['id'] == [1, 0])
    entry['package'] = list(bytes.fromhex(identity(manifest)))
    index_path.write_text(json.dumps(index))


def fixture(source, particle_package, destination):
    shutil.copytree(source, destination)
    index_path = destination / 'world.chunks.json'
    index = json.loads(index_path.read_text())
    for x, position in [(-1, [125., 1.3, 32.]), (0, [122., 1.3, 32.]), (1, [3., 1.3, 32.])]:
        chunk = destination / f'chunk_{x}_0'
        shutil.copytree(particle_package / 'particles', chunk / 'particles')
        catalog_path = chunk / 'particles/catalog.json'
        catalog = json.loads(catalog_path.read_text())
        if len(catalog['placements']) != 1 or len(catalog['effects']) != 1:
            raise ValueError('Expected one effect and placement')
        catalog['placements'][0]['position'] = position
        payload = json.dumps(catalog, separators=(',', ':')).encode()
        catalog_path.write_bytes(payload)
        manifest_path = chunk / 'world.manifest.json'
        manifest = json.loads(manifest_path.read_text())
        manifest.update(schema=4, particle_bytes=len(payload),
                        particle_sha256=list(hashlib.sha256(payload).digest()))
        manifest_path.write_text(json.dumps(manifest))
        entry, = (entry for entry in index['chunks'] if entry['id'] == [x, 0])
        entry['package'] = list(bytes.fromhex(identity(manifest)))
    index_path.write_text(json.dumps(index))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('source', type=Path)
    parser.add_argument('particle_package', type=Path)
    parser.add_argument('output', type=Path)
    parser.add_argument('--binaries', type=Path, default=Path('target/debug'))
    parser.add_argument('--require-particle-rebase', action='store_true', help='Require the supplied effect to use world-space simulation')
    args = parser.parse_args()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    good, bad = output / 'fixture', output / 'tampered-fixture'
    fixture(args.source.resolve(), args.particle_package.resolve(), good)
    shutil.copytree(good, bad)
    catalog = json.loads((bad / 'chunk_1_0/particles/catalog.json').read_text())
    effect = catalog['effects'][0]['id']
    (bad / 'chunk_1_0/particles/effects' / f'effect-{effect}.ron').write_bytes(b'tampered\n')
    common = dict(chunk_world=good, chunk_direction='x', chunk_stream=True,
                  render_span=2, replication_span=2, require_world_particles=True)
    run(output / 'retirement', args.binaries.resolve(), chunk_delay_ms=1000,
        require_particle_retirement=True, require_particle_rebase=args.require_particle_rebase, **common)
    arrival = output / 'arrival-fixture'
    shutil.copytree(good, arrival)
    for x in (-1, 0):
        chunk = arrival / f'chunk_{x}_0'
        shutil.rmtree(chunk)
        shutil.copytree(args.source.resolve() / f'chunk_{x}_0', chunk)
    index_path = arrival / 'world.chunks.json'
    index = json.loads(index_path.read_text())
    base = json.loads((args.source.resolve() / 'world.chunks.json').read_text())
    identities = {tuple(entry['id']):entry['package'] for entry in base['chunks']}
    for entry in index['chunks']:
        if entry['id'][0] in (-1, 0):
            entry['package'] = identities[tuple(entry['id'])]
    index_path.write_text(json.dumps(index))
    run(output / 'arrival', args.binaries.resolve(), chunk_world=arrival, chunk_direction='x',
        chunk_stream=True, chunk_delay_ms=1000, render_span=2, replication_span=2,
        require_world_particles=True, require_particle_control_arrival=True)
    departure = output / 'departure-fixture'
    shutil.copytree(good, departure)
    for x in (0, 1):
        chunk = departure / f'chunk_{x}_0'
        shutil.rmtree(chunk)
        shutil.copytree(args.source.resolve() / f'chunk_{x}_0', chunk)
    index_path = departure / 'world.chunks.json'
    index = json.loads(index_path.read_text())
    for entry in index['chunks']:
        if entry['id'][0] in (0, 1):
            entry['package'] = identities[tuple(entry['id'])]
    index_path.write_text(json.dumps(index))
    run(output / 'departure', args.binaries.resolve(), chunk_world=departure, chunk_direction='x',
        chunk_stream=True, chunk_delay_ms=1000, render_span=2, replication_span=2,
        require_no_world_particles=True, require_particle_control_departure=True)
    run(output / 'tampered', args.binaries.resolve(), client_chunk_world=bad,
        chunk_particle_failure=True, **common)
    if catalog['textures']:
        bad_texture = output / 'tampered-texture-fixture'
        shutil.copytree(good, bad_texture)
        textures = list((bad_texture / 'chunk_1_0/particles/textures').iterdir())
        if not textures:
            raise ValueError('Particle catalog has no texture file')
        textures[0].write_bytes(b'tampered\n')
        run(output / 'tampered-texture', args.binaries.resolve(), client_chunk_world=bad_texture,
            chunk_particle_texture_failure=True, **common)
        if len(catalog['textures']) == 1:
            oversized = output / 'oversized-fixture'
            oversized_texture_fixture(good, oversized)
            run(output / 'oversized', args.binaries.resolve(), chunk_world=oversized,
                chunk_direction='x', chunk_stream=True, render_span=2, replication_span=2,
                require_world_particles=True, chunk_particle_budget_failure=True)


if __name__ == '__main__':
    main()
