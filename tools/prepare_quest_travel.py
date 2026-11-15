"""Add one navmesh destination and a separate sparse collection test path to a saved valley."""
import argparse
import json
import math
from pathlib import Path


def prepare(source):
    draft = json.loads(source)
    terrain = draft['terrain']
    axis = math.isqrt(len(terrain['heights']))
    assert axis * axis == len(terrain['heights'])
    spacing = terrain['span_chunks'] * 128 / (axis - 1)

    def ground(x, z):
        fx = (x - terrain['chunk'][0] * 128) / spacing
        fz = (z - terrain['chunk'][1] * 128) / spacing
        ix, iz = int(fx), int(fz)
        assert 0 <= ix < axis - 1 and 0 <= iz < axis - 1
        u, v = fx - ix, fz - iz
        a, b = terrain['heights'][iz * axis + ix:iz * axis + ix + 2]
        c, d = terrain['heights'][(iz + 1) * axis + ix:(iz + 1) * axis + ix + 2]
        y = a * (1-u-v) + b*u + c*v if u+v <= 1 else b*(1-v) + c*(1-u) + d*(u+v-1)
        return [x, y, z]

    assert not any(m['id'] == 1202 for m in draft['markers']), 'Destination already exists'
    assert not any(p['id'] == 1203 for p in draft['paths']), 'Collection path already exists'
    draft['markers'].append(dict(id=1202, name='Ancient tree - Elowen destination',
        kind='quest', position=ground(8276, 8128), yaw_degrees=0., radius_metres=1.))
    draft['paths'].append(dict(id=1203, name='Rowan to northern meadow collection test',
        kind='road', width_metres=1., points=[ground(8194, 8168), ground(8194, 8260)]))
    return draft


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    result = prepare(args.source.read_text())
    with args.output.open('x') as target:
        target.write(json.dumps(result, indent=2) + '\n')
