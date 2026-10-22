#!/usr/bin/env python3
"""Qualify the actual Bevy chunk diagnostic on the caller's native display.

Uses original assets only. Native display/validation setup remains caller-owned.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile

import numpy as np
from PIL import Image
from create_chunk_fixture import create
from package_world import package, MANIFEST


def color_counts(path):
    pixels = np.asarray(Image.open(path).convert('RGB')).astype(float)
    if pixels.shape != (640, 960, 3):
        raise AssertionError(f'Unexpected screenshot size: {pixels.shape}')
    r, g, b = np.moveaxis(pixels, 2, 0)
    return {'red': int(((r > 45) & (r > g * 1.3) & (r > b * 1.5)).sum()),
            'green': int(((g > 45) & (g > r * 1.2) & (g > b * 1.1)).sum()),
            'blue': int(((b > 50) & (b > r * 1.3) & (b > g * 1.1)).sum())}


def run(output, binaries, base_x=0, base_z=0, expect_failure=False):
    output.mkdir(parents=True, exist_ok=True)
    (output / 'verification.json').unlink(missing_ok=True)
    binary = binaries / ('theia-client.exe' if os.name == 'nt' else 'theia-client')
    with tempfile.TemporaryDirectory(prefix='fixture-', dir=output) as temporary:
        fixture = Path(temporary) / 'chunks'
        create(fixture, base_x, base_z)
        if expect_failure:
            # Consistent package bytes, invalid glTF: rejection must come from
            # Bevy's real asset lifecycle rather than only manifest validation.
            directory = fixture / f'chunk_{base_x + 1}_{base_z}'
            visual = b'valid package binding; deliberately invalid glTF'
            (directory / 'terrain.glb').write_bytes(visual)
            profile = json.loads((directory / 'terrain.json').read_text())
            profile['visual_sha256'] = list(hashlib.sha256(visual).digest())
            (directory / 'terrain.json').write_text(json.dumps(profile, separators=(',', ':')) + '\n')
            (directory / MANIFEST).unlink()
            published = package(directory, f'theia.fixture.chunk.x{base_x + 1}-z{base_z}', 2)
            index = json.loads((fixture / 'world.chunks.json').read_text())
            index['chunks'][2]['package'] = list(bytes.fromhex(published['identity']))
            (fixture / 'world.chunks.json').write_text(json.dumps(index, indent=2) + '\n')
        index_hash = hashlib.sha256((fixture / 'world.chunks.json').read_bytes()).hexdigest()
        with (output / 'run.log').open('w') as log:
            process = subprocess.Popen([str(binary), '--chunk-smoke', str(fixture), str(output),
                                        *(['--expect-failure'] if expect_failure else [])],
                                       stdout=log, stderr=subprocess.STDOUT)
            try:
                code = process.wait(timeout=150)
                assert code == 0, f'Chunk diagnostic exited {code}; see {output / "run.log"}'
            finally:
                if process.poll() is None:
                    process.terminate()
                    try:
                        process.wait(timeout=5)
                    except subprocess.TimeoutExpired:
                        process.kill()
                        process.wait()
        report = json.loads((output / 'report.json').read_text())
        assert report['passed'] and report['expected_failure'] == expect_failure
        assert report['query_origin'] == [base_x * 128., 0., base_z * 128.]
        assert report['frame_checks'] > 10
        a, b, c = report['records']
        initial = [[base_x - 1, base_z], [base_x, base_z]]
        assert a['loaded'] == initial and a['roots'] == 2
        assert b['loaded'] == (initial if expect_failure else [[base_x, base_z], [base_x + 1, base_z]])
        assert b['rejected'] == expect_failure and b['roots'] == 2
        assert c['loaded'] == [] and c['roots'] == 0
        counts = {name: color_counts(output / f'{name}.png') for name in ['initial', 'replacement', 'empty']}
        assert counts['initial']['red'] > 500 and counts['initial']['green'] > 500, counts
        assert counts['initial']['blue'] < 100, counts
        replacement = counts['replacement']
        if expect_failure:
            assert replacement['red'] > 500 and replacement['blue'] < 100, counts
        else:
            assert replacement['red'] < 100 and replacement['blue'] > 500, counts
        assert replacement['green'] > 500 and max(counts['empty'].values()) < 100, counts
        log = (output / 'run.log').read_text()
        assert not any(marker in log for marker in ['VUID-', 'Validation Error', 'panicked at']), 'GPU validation/panic marker'
        errors = [line for line in log.splitlines() if ' ERROR ' in line]
        assert not errors or (expect_failure and all('bevy_asset' in line and 'Failed to load asset' in line for line in errors)), errors
        validation = 'Insert instance layer "VK_LAYER_KHRONOS_validation"' in log
        assert validation, 'Requested Vulkan validation layer was not active in captured loader output'
        proof = {'schema': 1, 'passed': True, 'expected_asset_failure': expect_failure,
                 'gpu': report['gpu'], 'validation_active': validation, 'colors': counts,
                 'index_sha256': index_hash, 'client_sha256': hashlib.sha256(binary.read_bytes()).hexdigest(),
                 'scope': 'actual original GLB scenes + live Avian publication/rollback; no network or imported-tile streaming'}
        (output / 'verification.json').write_text(json.dumps(proof, indent=2) + '\n')
        print(json.dumps(proof, indent=2), flush=True)
        return proof


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('--binaries', type=Path, default=Path('target/debug'))
    parser.add_argument('--base-x', type=int, default=0)
    parser.add_argument('--base-z', type=int, default=0)
    parser.add_argument('--expect-failure', action='store_true')
    args = parser.parse_args()
    run(args.output.resolve(), args.binaries.resolve(), args.base_x, args.base_z, args.expect_failure)
