#!/usr/bin/env python3
"""Compare real matching captures of this map with and without liquid geometry.

Uses requirements-assets.txt. Supplement the image check with visual inspection.
"""
import argparse
import hashlib
import json
from pathlib import Path

import numpy as np
from PIL import Image


def verify(water, baseline):
    proof = water / 'liquid-verification.json'
    proof.unlink(missing_ok=True)
    reports = [json.loads((p / 'asset.json').read_text()) for p in (water, baseline)]
    actual, reference = reports
    for report in reports:
        assert report['completed']
        state = report['pipelines']
        assert state['pending'] == state['failed'] == 0 and state['compiled'] > 0
        assert state['clear_frames'] >= 3, 'Capture must wait for renderer readiness'
    assert actual['view'] == reference['view'], 'Use exactly matching camera framing'
    assert actual['gpu'] == reference['gpu'], 'Use the same adapter/backend'
    assert actual['stats']['bounds'] == reference['stats']['bounds']
    assert actual['stats']['triangles'] > reference['stats']['triangles']
    assert actual['stats']['mesh_instances'] > reference['stats']['mesh_instances']
    pixels = []
    for directory in (water, baseline):
        with Image.open(directory / 'scene.png') as image:
            assert image.size == (1280, 720)
            pixels.append(np.asarray(image.convert('RGB')).astype(np.int16))
    added, original = pixels
    change = np.max(np.abs(added - original), axis=2)
    change[:130, :] = 0  # Filename/count HUD changes are not water evidence.
    changed = change > 10
    blue = (added[:, :, 2] - original[:, :, 2] > 25) & changed
    assert changed.sum() > 5000 and blue.sum() > 5000, 'Expected a substantial visible blue liquid surface'
    result = {'passed': True, 'changed_pixels_excluding_hud': int(changed.sum()),
              'pixels_with_blue_increase_over_25': int(blue.sum()), 'view': actual['view'],
              'image_hashes': [hashlib.sha256((p / 'scene.png').read_bytes()).hexdigest() for p in (water, baseline)],
              'pipelines': [r['pipelines'] for r in reports],
              'scope': 'actual matching map captures with/without blue preview liquids; not source-shader fidelity or performance'}
    proof.write_text(json.dumps(result, indent=2) + '\n')
    return result


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('water', type=Path)
    parser.add_argument('baseline', type=Path)
    args = parser.parse_args()
    print(json.dumps(verify(args.water, args.baseline), indent=2))
