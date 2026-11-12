#!/usr/bin/env python3
"""Stage selected verified static prototypes for the editor; no native window.

Source payloads and outputs remain local. This first catalog accepts embedded
standard-material GLBs; foliage, projected shaders and animation remain separate.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import shutil
import tempfile

import numpy as np
from merge_static_glb import read
from static_geometry import hierarchy, transformed_bounds

MAX_ASSET_BYTES = 32 * 1024 * 1024
MAX_LIBRARY_BYTES = 128 * 1024 * 1024


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def stage(batch, selection, output):
    batch = batch.resolve(strict=True)
    selection = selection.resolve(strict=True)
    output = output.absolute()
    if output.exists() or batch.is_relative_to(output) or selection.is_relative_to(output):
        raise ValueError('Use a new output folder outside the inputs')
    chosen = json.loads(selection.read_bytes())
    if chosen['schema'] != 1 or not 1 <= len(chosen['assets']) <= 64:
        raise ValueError('Unsupported scenery selection')
    records = json.loads((batch / 'batch.json').read_bytes())['converted']
    prepared, used, total = [], set(), 0
    for item in chosen['assets']:
        identifier = item['id']
        if not re.fullmatch(r'[a-z][a-z0-9._-]{0,94}[a-z0-9]', identifier) or '..' in identifier or identifier in used:
            raise ValueError('Invalid or duplicate scenery ID')
        used.add(identifier)
        matches = [r for r in records if Path(r['source']).stem == item['prefab'] and r.get('profile') == 'standard-opaque']
        if len(matches) != 1:
            raise ValueError(f'Expected one qualified standard-material binding: {item["prefab"]}')
        record = matches[0]
        if not re.fullmatch('[0-9a-f]{64}', record['key']):
            raise ValueError('Invalid prototype key')
        root = batch / 'prototypes' / record['key']
        asset = root / 'preview.glb'
        size = asset.stat().st_size
        total += size
        if not 0 < size <= MAX_ASSET_BYTES or total > MAX_LIBRARY_BYTES:
            raise ValueError('Scenery byte budget exceeded')
        proof = json.loads((root / 'independent-verification.json').read_bytes())
        sha = digest(asset)
        if proof.get('passed') is not True or proof.get('asset_sha256') != sha or record['prototype_sha256'] != sha:
            raise ValueError('Prototype verification no longer matches source bytes')
        doc, binary = read(asset)
        _, transforms = hierarchy(doc)
        if len(doc['nodes']) != 1 or len(doc['meshes']) != 1 or list(transforms) != [0] or not np.array_equal(transforms[0], np.eye(4)):
            raise ValueError('Scenery prototype must have one canonical mesh node')
        if doc['nodes'][0].get('mesh') != 0 or doc.get('extensionsUsed') or doc.get('extensionsRequired'):
            raise ValueError('Scenery requires ordinary standard-material glTF')
        bounds = transformed_bounds(doc, binary, 0, np.eye(4))
        if not np.isfinite(bounds).all() or np.abs(bounds).max() > 128:
            raise ValueError('Scenery bounds exceed local prototype limits')
        if not np.allclose(bounds[0], proof['bounds_y_up_metres']['min'], atol=1e-5, rtol=0) or not np.allclose(bounds[1], proof['bounds_y_up_metres']['max'], atol=1e-5, rtol=0):
            raise ValueError('Scenery bounds differ from independent verification')
        conversion = json.loads((root / 'conversion.json').read_bytes())
        entry = {k: item[k] for k in ('id', 'name', 'category')}
        entry.update(file=f'{sha}.glb', bytes=size, sha256=sha, bounds=bounds.tolist(),
                     triangles=proof['triangles'], source=dict(prefab=record['source'],
                     archive_sha256=conversion['archive_sha256'],
                     verification_sha256=digest(root / 'independent-verification.json'),
                     profile=record['profile']))
        prepared.append((entry, asset))
    output.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='.scenery-', dir=output.parent) as folder:
        temporary = Path(folder)
        for entry, asset in prepared:
            shutil.copyfile(asset, temporary / entry['file'])
            if digest(temporary / entry['file']) != entry['sha256']:
                raise ValueError('Source asset changed during staging')
        catalog = dict(schema=1, id=chosen['id'], assets=[entry for entry, _ in prepared])
        (temporary / 'catalog.json').write_text(json.dumps(catalog, indent=2) + '\n')
        output.mkdir()  # Refuse a racing/existing destination.
        try:
            for file in temporary.iterdir():
                file.rename(output / file.name)
        except Exception:
            shutil.rmtree(output)
            raise
    return dict(assets=len(prepared), bytes=total, catalog=str(output / 'catalog.json'),
                collision='not inferred', desktop_used=False)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('batch', type=Path)
    parser.add_argument('selection', type=Path)
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    print(json.dumps(stage(args.batch, args.selection, args.output), indent=2))
