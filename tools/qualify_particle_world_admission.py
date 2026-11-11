#!/usr/bin/env python3
"""Check actual client pre-launch admission against a copied particle world."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile


def digest(data):
    return list(hashlib.sha256(data).digest())


def check(client, world, expected):
    run = subprocess.run([str(client), '--terrain-world', str(world)],
                         text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                         env=os.environ.copy(), timeout=30)
    assert run.returncode != 0 and expected in run.stdout, run.stdout


def rewrite_effect(world, change):
    root = world/'particles'
    catalog_path = root/'catalog.json'
    catalog = json.loads(catalog_path.read_bytes())
    entry = catalog['effects'][0]
    effect_path = root/'effects'/('effect-'+entry['id']+'.ron')
    original = effect_path.read_bytes()
    changed = change(original)
    assert changed != original
    effect_path.write_bytes(changed)
    entry.update(bytes=len(changed), sha256=digest(changed))
    encoded = json.dumps(catalog, separators=(',', ':')).encode()
    catalog_path.write_bytes(encoded)
    manifest_path = world/'world.manifest.json'
    manifest = json.loads(manifest_path.read_bytes())
    manifest.update(particle_bytes=len(encoded), particle_sha256=digest(encoded))
    manifest_path.write_text(json.dumps(manifest))


def run(client, source):
    with tempfile.TemporaryDirectory(prefix='theia-particle-admission-') as temporary:
        root = Path(temporary)
        def copied(name):
            path = root/name
            shutil.copytree(source, path)
            return path
        valid = copied('valid')
        check(client, valid, 'World walking requires')
        catalog = copied('catalog')
        (catalog/'particles/catalog.json').write_bytes(b'changed')
        check(client, catalog, 'Particle catalog does not match')
        effect = copied('effect')
        next((effect/'particles/effects').iterdir()).write_bytes(b'changed')
        check(client, effect, 'does not match its descriptor')
        texture = copied('texture')
        next((texture/'particles/textures').iterdir()).write_bytes(b'changed')
        check(client, texture, 'does not match its descriptor')
        world_space = copied('world-space')
        rewrite_effect(world_space, lambda b: b.replace(b'use_local_coords:true', b'use_local_coords:false'))
        check(client, world_space, 'must use local coordinates')
        budget = copied('budget')
        rewrite_effect(budget, lambda b: b.replace(b'particles_amount:48', b'particles_amount:9000'))
        check(client, budget, 'exceeds the supplied count limits')
    print('Particle package admission checks passed')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('client', type=Path)
    parser.add_argument('world', type=Path)
    args = parser.parse_args()
    run(args.client.resolve(), args.world.resolve())
