"""Check owned native client window placement and captures during a real session."""
import argparse
import json
import os
from pathlib import Path
import re
import subprocess
import sys
from PIL import ImageGrab
sys.path.insert(0, 'tools')
from launch_environment import client_environment
from qualify_world_input_os import Desktop, owned_window, wait
from gpu_validation import check_gpu_diagnostics

parser = argparse.ArgumentParser(description=__doc__)
for name in ('scenario', 'world', 'players', 'output'):
    parser.add_argument('--' + name, type=Path, required=True)
args = parser.parse_args()
out = args.output.resolve()
out.mkdir(parents=True, exist_ok=False)
session = out / 'session'
env = client_environment(dict(os.environ, DISPLAY=':0', XDG_RUNTIME_DIR='/run/user/1000',
    WGPU_BACKEND='vulkan', VK_INSTANCE_LAYERS='VK_LAYER_KHRONOS_validation'))
p = None
x = None
with (out / 'launcher.log').open('w') as log:
    try:
        p = subprocess.Popen([sys.executable, 'tools/network_playtest.py',
            '--scenario', str(args.scenario.resolve()), '--world', str(args.world.resolve()),
            '--players', str(args.players.resolve()), '--output', str(session), '--stop-on-stdin-eof'],
            stdin=subprocess.PIPE, stdout=log, stderr=log, env=env)
        def clients():
            assert p.poll() is None, 'Launcher exited unexpectedly'
            children = subprocess.run(['ps', '--ppid', str(p.pid), '-o', 'pid=,comm='], capture_output=True, text=True, check=False).stdout
            ids = [int(line.split()[0]) for line in children.splitlines() if line.split()[1] == 'theia-client']
            windows = [(pid, owned_window(pid)) for pid in ids]
            return windows if len(windows) == 2 and all(w for _, w in windows) else None
        windows = wait(clients, 60)
        def ready():
            return all((session / f'client-{i}.log').exists() and '"finished":true' in (session / f'client-{i}.log').read_text() for i in (1, 2))
        wait(ready, 90)
        x = Desktop()
        results = []
        for pid, w in windows:
            x.focus(w)
            x.owned_focus()
            info = subprocess.check_output(['xwininfo', '-id', hex(w)], text=True)
            def number(label):
                return int(re.search(re.escape(label) + r':\s+(-?\d+)', info)[1])
            left, top = number('Absolute upper-left X'), number('Absolute upper-left Y')
            width, height = number('Width'), number('Height')
            title = subprocess.check_output(['xprop', '-id', hex(w), '_NET_WM_NAME'], text=True)
            assert 'Theia network playtest - Player ' in title, title
            index = int(re.search(r'view ([12])/2', title)[1])
            layout = next(json.loads(line)['test_window'] for line in (session / f'client-{index}.log').read_text().splitlines() if line.startswith('{"test_window":'))
            mx, my = layout['monitor_position']
            mw, mh = layout['monitor_size']
            assert mx <= left and my <= top and left + width <= mx + mw and top + height <= my + mh
            assert [width, height] == layout['physical_size']
            ImageGrab.grab(bbox=(left, top, left + width, top + height), xdisplay=':0').save(out / f'view-{index}.png')
            results.append(dict(pid=pid, title=title.strip(), bounds=[left, top, width, height], requested=layout))
        a, b = (r['bounds'] for r in sorted(results, key=lambda r: r['bounds'][0]))
        assert a[0] + a[2] <= b[0] or a[1] + a[3] <= b[1] or b[1] + b[3] <= a[1], results
        p.stdin.close()
        assert p.wait(timeout=30) == 0
        result = json.loads((session / 'session.json').read_text())
        assert not result['cleanup_errors'] and result['reason'] == 'stop requested'
        assert not list(session.glob('tickets-*'))
        for path in session.glob('*.log'):
            assert check_gpu_diagnostics(path.read_text()) == 0
        (out / 'qualification.json').write_text(json.dumps(dict(passed=True, windows=results,
            two_independent_scripts_finished=True, cleanup=True), indent=2))
    finally:
        if x:
            x.close()
        if p and p.poll() is None:
            if p.stdin and not p.stdin.closed:
                p.stdin.close()
            try:
                p.wait(timeout=40)
            except subprocess.TimeoutExpired:
                p.terminate()
                p.wait(timeout=20)
