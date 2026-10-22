"""Explicitly omit unsupported receiver-linked lights from an inspector copy.

The source lighting stage remains intact. No replacement sun is introduced.
Run with Blender's Python for independent source pose verification.
"""
import argparse
import copy
import hashlib
import json
from pathlib import Path
import shutil
from verify_unity_lighting import verify


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def supported(document):
    result = copy.deepcopy(document)
    omitted = [light for light in result['lights'] if light['mask'] != 0xffffffff]
    result['lights'] = [light for light in result['lights'] if light['mask'] == 0xffffffff]
    if not result['lights']:
        raise ValueError('No supported source lights remain')
    return result, omitted


def prepare(source, output):
    source = Path(source).resolve(strict=True)
    output = Path(output).resolve()
    build = Path(__file__).resolve().parents[1] / 'build'
    if output.exists() or output == build or not output.is_relative_to(build) or output.is_relative_to(source) or source.is_relative_to(output):
        raise ValueError('Use a fresh ignored output outside the source')
    checked = verify(source)
    document = json.loads((source / 'lighting.json').read_text())
    filtered, omitted = supported(document)
    files = {str(p.relative_to(source)): sha(p) for p in source.rglob('*') if p.is_file()}
    try:
        shutil.copytree(source, output)
        (output / 'lighting.json').write_text(json.dumps(filtered, indent=2, allow_nan=False) + '\n')
        receipt = {
            'schema': 1, 'profile': 'standard-unlinked-lighting-preview-v1',
            'source': str(source), 'source_files': files,
            'source_verification': checked, 'lighting_sha256': sha(output / 'lighting.json'),
            'omitted_lights': omitted,
            'reason': 'Standard Bevy RenderLayers filter lights per camera, not per PBR receiver; selective source lights are omitted without broadening their masks.',
            'scope': 'Explicit inspector approximation; original source stage preserved; no Unity lighting parity or gameplay adoption.'}
        (output / 'standard-lighting.json').write_text(json.dumps(receipt, indent=2) + '\n')
        return receipt
    except BaseException:
        if output.exists():
            shutil.rmtree(output)
        raise


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('source', type=Path)
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    result = prepare(args.source, args.output)
    print(json.dumps({'lights_omitted': len(result['omitted_lights']), 'source_lights': result['source_verification']['lights']}))
