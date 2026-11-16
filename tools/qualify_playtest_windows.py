"""Measure real native client viewports without externally resizing their windows.

The small offline scene isolates window creation from account/gameplay checks.
Two and four simultaneous clients check physical viewport size and placement.
"""
import argparse
from contextlib import ExitStack
import json
import os
from pathlib import Path
import subprocess
import re
import time

from PIL import Image
from gpu_validation import check_gpu_diagnostics
from launch_environment import client_environment
from playtest_windows import work_area_options


def run(binary, output):
    output.mkdir(parents=True, exist_ok=False)
    env = client_environment(os.environ)
    for key in list(env):
        if key.startswith('THEIA_'):
            env.pop(key)
    env.update(DISPLAY=':0', XDG_RUNTIME_DIR='/run/user/1000',
               WGPU_BACKEND='vulkan', WGPU_VALIDATION='1',
               VK_INSTANCE_LAYERS='VK_LAYER_KHRONOS_validation')
    results = []
    for name, size, count in [('solo', (3440, 1440), 1),
                              ('paired', (1720, 1400), 2),
                              ('four', (1720, 700), 4),
                              ('diagnostic', (1280, 720), 1)]:
        children = []
        with ExitStack() as stack:
            try:
                for index in range(count):
                    folder = output/f'{name}-{index+1}'
                    folder.mkdir()
                    log = stack.enter_context((folder/'client.log').open('w'))
                    args = [str(binary), '--window-size', f'{size[0]}x{size[1]}']
                    if count > 1:
                        args += ['--test-window', f'{index+1}/{count}', *work_area_options()]
                    args += ['--smoke-test', str(folder)]
                    children.append((subprocess.Popen(args, env=env, stdout=log, stderr=log), folder))
                geometries = {}
                observed = {}
                if count > 1:
                    deadline = time.monotonic() + 60
                    while time.monotonic() < deadline:
                        listed = subprocess.run(['wmctrl', '-lp'], env=env, capture_output=True, text=True)
                        if listed.returncode:
                            time.sleep(.02)
                            continue
                        listing = listed.stdout
                        for index, (child, folder) in enumerate(children):
                            matches = [line.split()[0] for line in listing.splitlines()
                                       if len(line.split()) >= 3 and line.split()[2] == str(child.pid)]
                            if not matches:
                                continue
                            queried = subprocess.run(['xwininfo', '-id', matches[0]], env=env, capture_output=True, text=True)
                            if queried.returncode:
                                continue
                            raw = queried.stdout
                            geometry = [int(re.search(label+r':\s+(-?\d+)', raw)[1]) for label in
                                        ['Absolute upper-left X', 'Absolute upper-left Y', 'Width', 'Height']]
                            observed[folder.name] = geometry
                            (output/f'{name}-observed.json').write_text(json.dumps(observed, indent=2)+'\n')
                            expected = [(index % 2)*size[0], (index // 2)*size[1], *size]
                            if geometry == expected:
                                geometries[folder.name] = geometry
                        if len(geometries) == count:
                            break
                        time.sleep(.02)
                    assert len(geometries) == count, (name, observed)
                for child, folder in children:
                    assert child.wait(timeout=60) == 0, f'Client failed: {folder}'
                    report = json.loads((folder/'report.json').read_text())
                    assert report['completed'] and report['extent'] == list(size), report
                    with Image.open(folder/'scene.png') as picture:
                        assert picture.size == size
                    assert check_gpu_diagnostics((folder/'client.log').read_text()) == 0
                    results.append(dict(case=folder.name, extent=report['extent'], geometry=geometries.get(folder.name), gpu=report['gpu']))
            finally:
                for child, _ in children:
                    if child.poll() is None:
                        child.terminate()
                        try:
                            child.wait(timeout=5)
                        except subprocess.TimeoutExpired:
                            child.kill()
                            child.wait(timeout=5)
    result = dict(passed=True, results=results, external_resize=False,
                  scope='Linux native window/viewports; offline scene, not multiplayer gameplay')
    (output/'qualification.json').write_text(json.dumps(result, indent=2)+'\n')
    return result


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, default=Path('target/debug/theia-client'))
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    print(json.dumps(run(args.binary.resolve(), args.output.resolve()), indent=2))
