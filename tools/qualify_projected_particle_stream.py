#!/usr/bin/env python3
"""Build and qualify a bounded projected-material/particle streaming fixture.

The fixture uses the existing paired collision world with projected visual
geometry. This checks presentation and particle residency, not visual/collision
shape equivalence or production world streaming capacity.
"""
import argparse
import hashlib
import json
from pathlib import Path
import shutil

from create_bound_material_fixture import create
from package_world import identity
from qualify_chunk_particles import fixture
from verify_multiplayer import run


def qualify(output, paired, particle_package, binaries):
    output.mkdir(parents=True, exist_ok=False)
    projected = output / 'projected-source'
    create(projected, session=True)
    geometry = output / 'paired-geometry'
    shutil.copytree(paired, geometry)
    shutil.copytree(projected / 'shared', geometry / 'shared')
    index_path = geometry / 'world.chunks.json'
    index = json.loads(index_path.read_text())
    for row in index['chunks']:
        x, z = row['id']
        name = f'chunk_{x}_{z}'
        chunk, reference = geometry / name, projected / name
        visual = (reference / 'terrain.glb').read_bytes()
        (chunk / 'terrain.glb').write_bytes(visual)
        profile_path = chunk / 'terrain.json'
        profile = json.loads(profile_path.read_text())
        profile['visual_sha256'] = list(hashlib.sha256(visual).digest())
        collision = json.dumps(profile, separators=(',', ':')).encode()
        profile_path.write_bytes(collision)
        manifest_path = chunk / 'world.manifest.json'
        manifest = json.loads(manifest_path.read_text())
        shared = json.loads((reference / 'world.manifest.json').read_text())['shared_images']
        manifest.update(schema=3, shared_images=shared, visual_bytes=len(visual),
                        visual_sha256=list(hashlib.sha256(visual).digest()),
                        collision_bytes=len(collision),
                        collision_sha256=list(hashlib.sha256(collision).digest()))
        manifest_path.write_text(json.dumps(manifest))
        row['package'] = list(bytes.fromhex(identity(manifest)))
    index_path.write_text(json.dumps(index))
    world = output / 'fixture'
    fixture(geometry, particle_package, world)
    run(output / 'native', binaries, chunk_world=world, chunk_direction='x',
        chunk_stream=True, chunk_delay_ms=1000, render_span=2, replication_span=2,
        chunk_shared_images=True, require_world_particles=True,
        require_particle_retirement=True, require_particle_rebase=True)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('--paired', type=Path, default=Path('build/runtime-materials-paired-regressions-v1-20261021/fixture-good'))
    parser.add_argument('--particle-package', type=Path, default=Path('build/particle-world-trails-fixture-20261111/world'))
    parser.add_argument('--binaries', type=Path, default=Path('target/debug'))
    args = parser.parse_args()
    qualify(args.output.resolve(), args.paired.resolve(), args.particle_package.resolve(), args.binaries.resolve())
