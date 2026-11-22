#!/usr/bin/env python3
"""Bake Millhaven cartography from the verified imported samples and source features."""
import argparse
import hashlib
import json
from pathlib import Path
import numpy as np
from PIL import Image, ImageDraw


def bake(terrain, features, output):
    encoded = (terrain/'terrain.manifest.json').read_bytes()
    manifest = json.loads(encoded)
    assert (manifest['schema'], manifest['chunks_xz'], manifest['chunk_origin'],
            manifest['samples_per_axis']) == (2, [32, 24], [0, 0], 129)
    heights = np.empty((769, 1025), dtype=np.float32)
    for z in range(24):
        for x in range(32):
            data = (terrain/f'tile_{x}_{z}.height').read_bytes()
            assert list(hashlib.sha256(data).digest()) == manifest['tiles'][z*32+x]
            tile = np.frombuffer(data, dtype='<f4', offset=24).reshape(129, 129)
            heights[z*32:z*32+33, x*32:x*32+33] = tile[::4, ::4]
    # Pixel centres average adjacent 4 m samples. +Z north requires -X east
    # in the game's right-handed Y-up frame; reverse BOTH image axes.
    h = ((heights[:-1, :-1]+heights[1:, :-1]+heights[:-1, 1:]+heights[1:, 1:])*.25)[::-1, ::-1]
    dy, dx = np.gradient(h, 4.)
    light = np.clip((.8-dx*.6-dy*.5)/np.sqrt(1+dx*dx+dy*dy), .35, 1.1)
    ramp = np.clip((h-169)/157, 0, 1)[..., None]
    rgb = np.array([130, 151, 100])*(1-ramp)+np.array([186, 166, 126])*ramp
    rgb *= (.72+.38*light)[..., None]
    # Subtle 10 m contours preserve landform readability without invented scenery.
    contour = np.floor(h/10) != np.floor(np.roll(h, 1, axis=0)/10)
    contour[0] = False
    rgb[contour] *= .8
    image = Image.fromarray(np.uint8(np.clip(rgb, 0, 255)))
    draw = ImageDraw.Draw(image)
    source = json.loads(features.read_text())
    project = lambda p: (1024-p[0]/4, 768-p[1]/4)
    for feature in source['features']:
        kind = feature['type']
        points = feature.get('centreline_xz_m', feature.get('centreline_xz_bed_m'))
        if points:
            points = [project(p) for p in points]
            if kind in ('river', 'stream'):
                draw.line(points, fill=(50, 82, 90), width=7 if kind == 'river' else 3, joint='curve')
                draw.line(points, fill=(103, 151, 153), width=4 if kind == 'river' else 1, joint='curve')
            elif kind == 'graded_path':
                draw.line(points, fill=(100, 74, 42), width=5, joint='curve')
                draw.line(points, fill=(223, 194, 137), width=2, joint='curve')
    names = ['Village centre', 'Healing house', 'Militia ground', 'Merchant yard',
             'Southeast farm', 'Southwest farm', 'Bandit caves']
    descriptions = ['Village clearing and central meeting point.', 'Planned healing-house site.',
                    'Militia training clearing.', 'Merchant and trade-yard site.',
                    'Southeastern farmstead.', 'Southwestern farmstead.',
                    'Cave entrance terrace. The cave interior is not built yet.']
    pads = [f for f in source['features'] if f['type'] == 'level_pad']
    assert len(pads) == len(names)
    landmarks = [dict(name=n, description=d, xz=[f['x'], f['z']], major=i in (0, 4, 5, 6))
                 for i, (f, n, d) in enumerate(zip(pads, names, descriptions))]
    output.mkdir(parents=True, exist_ok=True)
    image.save(output/'overview.png')
    identity = hashlib.sha256(b'theia-saved-height-terrain-v2\0'+encoded).digest()
    record = dict(schema=2, title='Millhaven', terrain_identity=list(identity),
                  bounds=[[0., 0.], [4096., 3072.]], north_positive_z=True, landmarks=landmarks)
    (output/'overview.json').write_text(json.dumps(record, indent=2)+'\n')
    provenance = dict(terrain_manifest_sha256=hashlib.sha256(encoded).hexdigest(),
                      features_sha256=hashlib.sha256(features.read_bytes()).hexdigest(),
                      overview_sha256=hashlib.sha256((output/'overview.png').read_bytes()).hexdigest(),
                      dimensions=[1024,768], method='4 m sample cartography, pixel-centre average, shaded relief and 10 m contours; source route/watercourse lines; +Z north, -X east (Y-up game orientation)')
    (output/'provenance.json').write_text(json.dumps(provenance,indent=2)+'\n')


if __name__ == '__main__':
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('terrain',type=Path)
    p.add_argument('features',type=Path)
    p.add_argument('output',type=Path)
    a=p.parse_args()
    bake(a.terrain,a.features,a.output)
