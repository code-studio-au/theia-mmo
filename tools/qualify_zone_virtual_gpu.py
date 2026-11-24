#!/usr/bin/env python3
"""Render an isolated editor startup on a virtual X server without OS input."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import select
import shutil
import subprocess

from launch_environment import client_environment
from gpu_validation import check_gpu_diagnostics

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--output', required=True, type=Path)
parser.add_argument('--xvfb', default=shutil.which('Xvfb'))
parser.add_argument('--case', choices=['local', 'far'], default='local')
parser.add_argument('--view', choices=['startup', 'spawn', 'property-actions'], default='startup')
args = parser.parse_args()
if not args.xvfb:
    parser.error('Provide --xvfb or install Xvfb')
out = args.output.resolve()
out.mkdir(parents=True, exist_ok=False)
origin = [0.125, 0.25, -0.5] if args.case == 'local' else [128000000000.125, 20.25, -128000000000.5]
document = dict(schema=1, name='Virtual GPU smoke', origin=origin,
                spawn=[origin[0], origin[1] + 0.0625, origin[2] + 6],
                objects=[dict(id=1, name='Authored box', primitive='box',
                              pose=dict(position=[origin[0] + 8.125, origin[1] + 1.25, origin[2] - 8.5],
                                        yaw_degrees=17.25, size=[2, 2.5, 3]))])
source = out / 'source.json'
source.write_text(json.dumps(document))
original = source.read_bytes()
data = out / 'particle-data'
data.mkdir()
binary = Path('target/debug/theia-zone-editor').resolve()
read_fd, write_fd = os.pipe()
server = None
editor = None
try:
    with (out / 'xvfb.log').open('w') as server_log:
        server = subprocess.Popen([str(Path(args.xvfb).resolve()), '-displayfd', str(write_fd),
                                   '-screen', '0', '1920x1080x24', '-nolisten', 'tcp', '-noreset'],
                                  pass_fds=(write_fd,), stdout=server_log, stderr=server_log)
        os.close(write_fd)
        write_fd = None
        ready, _, _ = select.select([read_fd], [], [], 15)
        if not ready:
            raise RuntimeError('Virtual X server did not publish a display number')
        display = os.read(read_fd, 32).decode('ascii').strip()
        if not display.isdecimal():
            raise RuntimeError('Virtual X server failed to start')
        env = client_environment(dict(os.environ, DISPLAY=':' + display, WINIT_UNIX_BACKEND='x11',
                                      WGPU_BACKEND='vulkan', VK_INSTANCE_LAYERS='VK_LAYER_KHRONOS_validation',
                                      THEIA_PARTICLE_EDITOR_DATA_DIR=str(data)))
        env.pop('WAYLAND_DISPLAY', None)
        with (out / 'editor.log').open('w') as log:
            command = [str(binary), '--zone', str(source), '--capture', str(out / 'capture.png'),
                       '--report', str(out / 'state.json')]
            if args.view != 'startup':
                command.extend(['--capture-view', args.view])
            editor = subprocess.Popen(command, env=env, stdout=log, stderr=log)
            editor.wait(timeout=120)
        if editor.returncode != 0:
            raise RuntimeError(f'Editor exited with code {editor.returncode}')
        gpu_diagnostics = check_gpu_diagnostics((out / 'editor.log').read_text())
        assert source.read_bytes() == original
        state = json.loads((out / 'state.json').read_text())
        assert state['document'] == document
        assert state['revision'] == 0
        if args.view == 'spawn':
            assert len(state['inline_attributes']) == 3 and not state['inline_names']
            actual = {field['field']: float(field['value']) for field in state['inline_attributes']}
            assert [actual[i] for i in range(3)] == document['spawn']
        if args.view == 'property-actions':
            assert state['selected'] == 1
            assert any(menu['name'] == 'Property actions' and menu['role'] == 'popup'
                       and min(menu['size']) > 0 for menu in state['menus'])
        assert (out / 'capture.png').stat().st_size > 0
        result = dict(case=args.case, view=args.view, binary_sha256=hashlib.sha256(binary.read_bytes()).hexdigest(),
                      gpu=state['gpu'], viewport=state['viewport'], source_preserved=True,
                      input='none', scope='rendered capture with programmatic setup; no native interaction',
                      gpu_diagnostics=gpu_diagnostics)
        (out / 'qualification.json').write_text(json.dumps(result, indent=2))
        print(json.dumps(result))
finally:
    if write_fd is not None:
        os.close(write_fd)
    os.close(read_fd)
    for process in (editor, server):
        if process is not None and process.poll() is None:
            process.terminate()
            try:
                process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait(timeout=10)
