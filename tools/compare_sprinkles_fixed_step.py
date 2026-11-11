#!/usr/bin/env python3
"""Capture pinned upstream and local Sprinkles previews at fixed simulation steps.

Build the patched pinned upstream and the local deterministic_capture example
first; see docs/SPRINKLES_PARITY.md. Runs on the current display.
"""
import argparse
import hashlib
import json
import os
import re
from pathlib import Path
import subprocess

import numpy as np
from PIL import Image, ImageDraw

FRAMES = (30, 60, 120, 180, 300)
VIEWPORT = (616, 80, 1366, 768)  # exclude inspector and FPS text


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def capture(binary, effect, output, original):
    output.mkdir(parents=True)
    (output/'data').mkdir()
    (output/'frames').mkdir()
    env = dict(os.environ, WGPU_BACKEND='vulkan', RUST_LOG='warn,bevy_render::renderer=info',
               SPRINKLES_PARITY_OUTPUT=str((output/'frames').resolve()))
    env['SPRINKLES_PARITY_DATA_DIR' if original else 'THEIA_PARTICLE_EDITOR_DATA_DIR'] = str((output/'data').resolve())
    with (output/'run.log').open('w') as log:
        subprocess.run([str(binary), str(effect)], env=env, stdout=log, stderr=log, timeout=180, check=True)
    for frame in FRAMES:
        assert (output/'frames'/f'frame-{frame}.png').is_file(), f'Missing frame {frame}'
    log=(output/'run.log').read_text(errors='replace')
    return {'validation_layer_active':'Insert instance layer "VK_LAYER_KHRONOS_validation"' in log,
            'vuid_mentions':len(re.findall(r'VUID-[A-Za-z0-9_-]+',log)),
            'validation_error_lines':sum('validation error' in line.lower() for line in log.splitlines()),
            'panic_lines':sum('panicked at' in line.lower() for line in log.splitlines())}


def compare(left, right, output, labels=('Pinned upstream','Theia port')):
    values=[]
    for frame in FRAMES:
        a=Image.open(left/'frames'/f'frame-{frame}.png').convert('RGB').crop(VIEWPORT)
        b=Image.open(right/'frames'/f'frame-{frame}.png').convert('RGB').crop(VIEWPORT)
        assert a.size == b.size == (750,688)
        delta=np.abs(np.asarray(a,dtype=np.int16)-np.asarray(b,dtype=np.int16))
        values.append({'frame':frame,'mean_absolute_rgb':float(delta.mean()),
                       'pixels_above_10':float(np.any(delta>10,axis=2).mean())})
        panel=Image.new('RGB',(a.width+b.width,a.height+24),(20,20,20))
        draw=ImageDraw.Draw(panel);draw.text((8,5),labels[0],fill='white');draw.text((a.width+8,5),labels[1],fill='white')
        panel.paste(a,(0,24));panel.paste(b,(a.width,24));panel.save(output/f'frame-{frame}-comparison.png')
    return values


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output',type=Path)
    parser.add_argument('--original',type=Path,default=Path('build/sprinkles-upstream-target/debug/sprinkles'))
    parser.add_argument('--local',type=Path,default=Path('target/debug/examples/deterministic_capture'))
    parser.add_argument('--effect',action='append',default=[])
    args=parser.parse_args();root=args.output.resolve();root.mkdir(parents=True,exist_ok=False)
    original,local=args.original.resolve(strict=True),args.local.resolve(strict=True)
    effects=args.effect or ['rain','tube-trails','3d-explosion']
    results={}
    for name in effects:
        if name not in ('rain','tube-trails','3d-explosion','acid-pool','loot','magic-puff','windy-snow'):
            raise ValueError('Unknown bundled effect')
        effect=Path('third_party/sprinkles/crates/bevy_sprinkles_editor/src/assets/examples',name+'.ron').resolve(strict=True)
        effect_before=sha(effect)
        dest=root/name;dest.mkdir()
        print(f'Capturing {name}: pinned upstream',flush=True)
        upstream_log=capture(original,effect,dest/'upstream',True)
        print(f'Capturing {name}: Theia',flush=True)
        local_log=capture(local,effect,dest/'local',False)
        print(f'Capturing {name}: Theia repeat',flush=True)
        repeat_log=capture(local,effect,dest/'local-repeat',False)
        assert sha(effect)==effect_before, 'Source effect file changed during comparison'
        results[name]={'effect_sha256':effect_before,'diagnostics':{'upstream':upstream_log,'local':local_log,'local_repeat':repeat_log},
            'upstream_vs_local':compare(dest/'upstream',dest/'local',dest),
            'local_repeat':compare(dest/'local',dest/'local-repeat',dest/'local-repeat',('Theia first run','Theia repeat'))}
    report={'fixed_step_seconds':1/60,'seed_by_emitter':'1000 + emitter index, overridden in memory for looping','warmup_wall_seconds':4,
            'viewport_crop':VIEWPORT,'original_binary_sha256':sha(original),'local_binary_sha256':sha(local),
            'scope':'Viewport image differences include engine/rendering, timing, overlays and nondeterminism; not a pixel-perfect parity verdict.',
            'effects':results}
    (root/'comparison.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps(report,indent=2))


if __name__=='__main__':
    main()
