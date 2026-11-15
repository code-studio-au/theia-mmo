"""Measure real native client viewports without externally resizing their windows.

The small offline scene isolates window creation from account/gameplay checks.
Two simultaneous clients check the per-player viewport size independently.
"""
import argparse
from contextlib import ExitStack
import json
import os
from pathlib import Path
import subprocess

from PIL import Image
from gpu_validation import check_gpu_diagnostics
from launch_environment import client_environment


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
                              ('paired', (1920, 1440), 2),
                              ('diagnostic', (1280, 720), 1)]:
        children = []
        with ExitStack() as stack:
            try:
                for index in range(count):
                    folder = output/f'{name}-{index+1}'
                    folder.mkdir()
                    log = stack.enter_context((folder/'client.log').open('w'))
                    args = [str(binary), '--window-size', f'{size[0]}x{size[1]}']
                    args += ['--smoke-test', str(folder)]
                    children.append((subprocess.Popen(args, env=env, stdout=log, stderr=log), folder))
                for child, folder in children:
                    assert child.wait(timeout=60) == 0, f'Client failed: {folder}'
                    report = json.loads((folder/'report.json').read_text())
                    assert report['completed'] and report['extent'] == list(size), report
                    with Image.open(folder/'scene.png') as picture:
                        assert picture.size == size
                    assert check_gpu_diagnostics((folder/'client.log').read_text()) == 0
                    results.append(dict(case=folder.name, extent=report['extent'], gpu=report['gpu']))
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
