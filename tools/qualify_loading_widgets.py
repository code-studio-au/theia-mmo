#!/usr/bin/env python3
"""Owned native widget checks; failure is a declared UI fixture, not admission evidence."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import time
from PIL import Image
from edit_zone import build_environment
from gpu_validation import check_gpu_diagnostics
from qualify_world_input_os import Desktop, owned_window, wait, capture_ready


def run(output):
    output.mkdir(parents=True, exist_ok=False)
    proofs = []
    for height in [1400, 700]:
        for failed in [False, True]:
            folder = output / f'{height}-{"failed" if failed else "cancel"}'
            folder.mkdir()
            env = build_environment(os.environ)
            env.update(DISPLAY=':0', XDG_RUNTIME_DIR='/run/user/1000', WGPU_BACKEND='vulkan',
                LD_LIBRARY_PATH=str(Path('build/native-sdk/usr/lib/x86_64-linux-gnu').resolve()),
                VK_INSTANCE_LAYERS='VK_LAYER_KHRONOS_validation', WGPU_VALIDATION='1',
                THEIA_LOADING_WIDGET_QUALIFY=str(folder.resolve()), THEIA_LOADING_WIDGET_HEIGHT=str(height))
            desktop = Desktop()
            with (folder/'client.log').open('w') as log:
                child = subprocess.Popen(['cargo','test','--offline','-j2','-p','theia-client','--bin','theia-client',
                    'loading::tests::native_loading_cancel_and_failure_widgets','--','--ignored','--exact','--nocapture'],
                    env=env, stdout=log, stderr=subprocess.STDOUT, start_new_session=True)
                try:
                    def state():
                        assert child.poll() is None, f'Widget process exited; see {folder}'
                        value = json.loads((folder/'widget.json').read_text())
                        return value if value['geometry'].get('cancel',[0,0,0,0])[2] > 0 else None
                    first = wait(state, 45)
                    window = wait(lambda: owned_window(first['pid']), 10)
                    desktop.focus(window)
                    if failed:
                        (folder/'fail.request').touch()
                        wait(lambda: (v:=state()) and v['phase']=='Failed', 5)
                    time.sleep(2.)
                    (folder/'capture.request').touch()
                    wait(lambda: capture_ready(folder/'widget.png'), 10)
                    value = state()
                    # The fixture renders a magenta world under the loading
                    # camera. Inspect every early captured frame, including
                    # frames before the UI pipelines have finished compiling.
                    early = []
                    for frame in range(1, 9):
                        path = folder/f'startup-{frame:02}.png'
                        wait(lambda: capture_ready(path), 10)
                        with Image.open(path) as image:
                            pixels = image.convert('RGB')
                            leaked = sum(r > 180 and g < 60 and b > 180 for r,g,b in pixels.getdata())
                            assert leaked == 0, ('Unprepared world leaked', path, leaked)
                            early.append(dict(frame=frame, world_pixels=leaked))
                    for name in ['art','status','cancel']:
                        x,y,w,h = value['geometry'][name]
                        assert 0 <= x and 0 <= y and x+w <= 1721 and y+h <= height+1, (name,value)
                    x,y,w,h = value['geometry']['cancel']
                    desktop.click(window, [x+w/2,y+h/2], 1, 1)
                    assert child.wait(timeout=15) == 0, f'Cancel/Close failed; see {folder}'
                    check_gpu_diagnostics((folder/'client.log').read_text())
                    proofs.append(dict(height=height, failed_fixture=failed, state=value, startup=early, passed=True))
                finally:
                    desktop.close()
                    if child.poll() is None:
                        import signal
                        os.killpg(child.pid,signal.SIGTERM)
                        child.wait(timeout=10)
    (output/'qualification.json').write_text(json.dumps(dict(passed=True, scope='Native widgets; injected failure state; real admission tested separately', runs=proofs),indent=2))


if __name__=='__main__':
    parser=argparse.ArgumentParser()
    parser.add_argument('--output',type=Path,required=True)
    run(parser.parse_args().output)
