#!/usr/bin/env python3
"""Align buffer views in a self-contained, unextended GLB; never overwrite input."""
import argparse
import json
import math
import struct
from pathlib import Path


def realign(data, repair_metadata=False):
    if len(data) < 20 or struct.unpack_from('<4sII', data) != (b'glTF', 2, len(data)):
        raise ValueError('Expected a complete GLB 2.0 file')
    chunks = []
    offset = 12
    while offset < len(data):
        if offset + 8 > len(data):
            raise ValueError('Truncated chunk header')
        size, kind = struct.unpack_from('<I4s', data, offset)
        offset += 8
        if size % 4 or offset + size > len(data):
            raise ValueError('Invalid chunk size')
        chunks.append((kind, data[offset:offset + size]))
        offset += size
    if [k for k, _ in chunks] != [b'JSON', b'BIN\0']:
        raise ValueError('Expected exactly JSON and BIN chunks')
    doc = json.loads(chunks[0][1])
    if doc.get('extensionsUsed') or doc.get('extensionsRequired'):
        raise ValueError('Extended GLBs are outside this repair scope')
    buffers = doc.get('buffers', [])
    if len(buffers) != 1 or 'uri' in buffers[0]:
        raise ValueError('Expected a single embedded buffer')
    if any('uri' in image for image in doc.get('images', [])):
        raise ValueError('External images are outside this repair scope')
    binary = chunks[1][1]
    length = buffers[0]['byteLength']
    if not 0 <= len(binary) - length <= 3:
        raise ValueError('Buffer length disagrees with BIN chunk')
    packed = bytearray()
    for view in doc.get('bufferViews', []):
        start, size = view.get('byteOffset', 0), view['byteLength']
        if view.get('buffer', 0) != 0 or start < 0 or size <= 0 or start + size > length:
            raise ValueError('Buffer view outside embedded data')
        packed.extend(b'\0' * (-len(packed) % 4))
        view['byteOffset'] = len(packed)
        packed.extend(binary[start:start + size])
    buffers[0]['byteLength'] = len(packed)
    if repair_metadata:
        # Only recompute existing extrema; geometry, samples and indices stay intact.
        for accessor in doc.get('accessors', []):
            if 'min' not in accessor and 'max' not in accessor:
                continue
            if 'sparse' in accessor or 'bufferView' not in accessor:
                raise ValueError('Sparse/implicit extrema repair is unsupported')
            components = {'SCALAR': 1, 'VEC2': 2, 'VEC3': 3, 'VEC4': 4, 'MAT4': 16}.get(accessor['type'])
            if components is None or (accessor['type'] == 'MAT4' and accessor['componentType'] != 5126):
                raise ValueError('Unsupported matrix extrema layout')
            fmt = {5120: 'b', 5121: 'B', 5122: 'h', 5123: 'H', 5125: 'I', 5126: 'f'}[accessor['componentType']]
            decoder = struct.Struct('<' + fmt * components)
            view = doc['bufferViews'][accessor['bufferView']]
            stride = view.get('byteStride', decoder.size)
            start = accessor.get('byteOffset', 0)
            count = accessor['count']
            if count <= 0 or stride < decoder.size or start < 0 or start + (count - 1) * stride + decoder.size > view['byteLength']:
                raise ValueError('Accessor outside buffer view')
            low, high = [math.inf] * components, [-math.inf] * components
            for i in range(count):
                sample = decoder.unpack_from(packed, view['byteOffset'] + start + i * stride)
                if not all(math.isfinite(v) for v in sample):
                    raise ValueError('Nonfinite accessor data')
                low = [min(a, b) for a, b in zip(low, sample)]
                high = [max(a, b) for a, b in zip(high, sample)]
            if 'min' in accessor:
                accessor['min'] = low
            if 'max' in accessor:
                accessor['max'] = high
        # No extensions are allowed, so there are no extension-held clip indices.
        if 'animations' in doc:
            doc['animations'] = [a for a in doc['animations'] if a.get('channels') and a.get('samplers')]
    packed.extend(b'\0' * (-len(packed) % 4))
    text = json.dumps(doc, separators=(',', ':'), allow_nan=False).encode()
    text += b' ' * (-len(text) % 4)
    size = 12 + 8 + len(text) + 8 + len(packed)
    return (struct.pack('<4sII', b'glTF', 2, size)
            + struct.pack('<I4s', len(text), b'JSON') + text
            + struct.pack('<I4s', len(packed), b'BIN\0') + packed)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('source', type=Path)
    parser.add_argument('output', type=Path)
    parser.add_argument('--repair-metadata', action='store_true',
                        help='Recompute authored accessor extrema and remove empty clips')
    args = parser.parse_args()
    if args.source.resolve() == args.output.resolve():
        parser.error('Output must be a separate new file')
    result = realign(args.source.read_bytes(), args.repair_metadata)
    # Exclusive creation also rejects existing outputs and symlinks.
    with args.output.open('xb') as output:
        output.write(result)
    print(f'Wrote {len(result):,} bytes; validate separately before use')


if __name__ == '__main__':
    main()
