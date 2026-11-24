#!/usr/bin/env python3
"""Publish a metre-valued float32 [z,x] NPY as immutable saved terrain v2.

Preserves every source sample. No smoothing, vertical offset, axis flip, or
resampling. Runtime world bounds remain within the existing 16 km domain.
"""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import struct

import numpy as np

from package_world import file_hash, valid_id

MAX_SOURCE_BYTES = 512 * 1024 * 1024


def source_array(source, spacing, origin):
    if spacing not in (1, 2, 4) or any(type(v) is not int for v in origin) or len(origin) != 2:
        raise ValueError('Use 1, 2 or 4 metre spacing and two integer chunk coordinates')
    _, digest = file_hash(source, MAX_SOURCE_BYTES)
    heights = np.load(source, mmap_mode='r', allow_pickle=False)
    if heights.ndim != 2 or heights.dtype != np.dtype('<f4'):
        raise ValueError('Expected little-endian float32 NPY indexed [z,x]')
    cells = 128 // spacing
    nz, nx = ((n - 1) // cells for n in heights.shape)
    if (not 1 <= nx <= 125 or not 1 <= nz <= 125
            or heights.shape != (nz * cells + 1, nx * cells + 1)
            or any(not 0 <= o or o + n > 125 for o, n in zip(origin, (nx, nz)))):
        raise ValueError('Grid must contain complete 128 m tiles inside the 16 km domain')
    if not np.isfinite(heights).all() or np.max(np.abs(heights)) > 1024:
        raise ValueError('Terrain heights must be finite metres in -1024..1024')
    return heights, digest, cells, nx, nz


def publish(source, output, content_id, revision=1, spacing=1, origin=(0, 0)):
    source = Path(source).resolve(strict=True)
    output = Path(output).resolve()
    if output.exists():
        raise FileExistsError('Use a fresh output directory; existing terrain is immutable')
    if not valid_id(content_id) or type(revision) is not int or not 1 <= revision <= 0xffffffff:
        raise ValueError('Invalid content ID or revision')
    heights, digest, cells, nx, nz = source_array(source, spacing, origin)
    output.mkdir(parents=True)
    try:
        hashes = []
        for z in range(nz):
            for x in range(nx):
                gx, gz = origin[0] + x, origin[1] + z
                samples = heights[z*cells:(z+1)*cells+1, x*cells:(x+1)*cells+1]
                data = b'THTILE01' + struct.pack('<qq', gx, gz) + samples.tobytes(order='C')
                with (output / f'tile_{gx}_{gz}.height').open('xb') as stream:
                    stream.write(data)
                hashes.append(list(hashlib.sha256(data).digest()))
        if file_hash(source, MAX_SOURCE_BYTES)[1] != digest:
            raise ValueError('Source changed during import')
        manifest = dict(schema=2, content_id=content_id, revision=revision, chunk_metres=128,
                        chunks_xz=[nx, nz], chunk_origin=list(origin), samples_per_axis=cells+1,
                        tiles=hashes)
        with (output / 'terrain.manifest.json').open('x') as stream:
            stream.write(json.dumps(manifest, separators=(',', ':'), allow_nan=False) + '\n')
        report = verify(source, output, spacing, origin)
        (output / 'import.json').write_text(json.dumps(report, indent=2) + '\n')
        return report
    except BaseException:
        shutil.rmtree(output)
        raise


def verify(source, output, spacing=1, origin=(0, 0)):
    """Compare every decoded tile sample with the input, including all seams."""
    source, output = Path(source), Path(output)
    heights, digest, cells, nx, nz = source_array(source, spacing, origin)
    encoded = (output / 'terrain.manifest.json').read_bytes()
    manifest = json.loads(encoded)
    if (manifest['schema'] != 2 or manifest['chunk_metres'] != 128
            or manifest['chunks_xz'] != [nx, nz] or manifest['chunk_origin'] != list(origin)
            or manifest['samples_per_axis'] != cells+1 or len(manifest['tiles']) != nx*nz):
        raise ValueError('Manifest layout does not match source')
    count = 0
    for z in range(nz):
        for x in range(nx):
            gx, gz = origin[0]+x, origin[1]+z
            data = (output / f'tile_{gx}_{gz}.height').read_bytes()
            if (len(data) != 24 + (cells+1)**2*4 or data[:8] != b'THTILE01'
                    or struct.unpack('<qq', data[8:24]) != (gx, gz)
                    or list(hashlib.sha256(data).digest()) != manifest['tiles'][z*nx+x]):
                raise ValueError('Tile identity/hash mismatch')
            actual = np.frombuffer(data, dtype='<u4', offset=24).reshape(cells+1, cells+1)
            expected = heights[z*cells:(z+1)*cells+1, x*cells:(x+1)*cells+1].view('<u4')
            if not np.array_equal(actual, expected):
                raise ValueError('Tile differs from source sample bits')
            count += actual.size
    return dict(schema=1, source_sha256=digest.hex(), source_shape_zx=list(heights.shape),
                spacing_m=spacing, chunk_origin=list(origin), chunks_xz=[nx, nz],
                tiles=nx*nz, checked_samples_including_shared_edges=count,
                unique_source_samples=int(heights.size), max_height_error_m=0,
                height_range_m=[float(heights.min()), float(heights.max())],
                source_payload_bytes=int(heights.nbytes), tile_bytes=nx*nz*(24+(cells+1)**2*4),
                manifest_sha256=hashlib.sha256(encoded).hexdigest(),
                scope='Complete bit-preserving offline import; runtime/GPU/movement require separate checks')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('source', type=Path)
    parser.add_argument('output', type=Path)
    parser.add_argument('--content-id', required=True)
    parser.add_argument('--revision', type=int, default=1)
    parser.add_argument('--spacing', type=int, choices=(1, 2, 4), default=1)
    parser.add_argument('--chunk-origin', type=int, nargs=2, default=(0, 0))
    args = parser.parse_args()
    print(json.dumps(publish(args.source, args.output, args.content_id, args.revision,
                             args.spacing, args.chunk_origin), indent=2))
