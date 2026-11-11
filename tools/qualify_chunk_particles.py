#!/usr/bin/env python3
"""Qualify a three-chunk x-axis fixture using an existing looping particle package.

The source fixture must use initial chunks [-1, 0] and [0, 0], and offer
[1, 0] during the standard green-floor transition. The particle package must
contain one untextured looping effect and its catalog, as produced by the editor.
Runs against the caller's display and Vulkan validation environment.
"""
import argparse
import hashlib
import json
from pathlib import Path
import shutil

from package_world import identity
from verify_multiplayer import run


def fixture(source, particle_package, destination):
    shutil.copytree(source, destination)
    index_path = destination / 'world.chunks.json'
    index = json.loads(index_path.read_text())
    for x, position in [(-1, [125., 1.3, 32.]), (0, [122., 1.3, 32.]), (1, [3., 1.3, 32.])]:
        chunk = destination / f'chunk_{x}_0'
        shutil.copytree(particle_package / 'particles', chunk / 'particles')
        catalog_path = chunk / 'particles/catalog.json'
        catalog = json.loads(catalog_path.read_text())
        if len(catalog['placements']) != 1 or len(catalog['effects']) != 1 or catalog['textures']:
            raise ValueError('Expected one untextured effect and placement')
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
        require_particle_retirement=True, **common)
    run(output / 'tampered', args.binaries.resolve(), client_chunk_world=bad,
        chunk_particle_failure=True, **common)


if __name__ == '__main__':
    main()
