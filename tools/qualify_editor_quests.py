"""Owned native NPC quest inspection; source content and zone remain unchanged."""
import argparse
import json
import os
from pathlib import Path
import signal
import subprocess
from launch_environment import client_environment
from qualify_world_input_os import Desktop, owned_window, wait, capture_ready
from gpu_validation import check_gpu_diagnostics

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--binary', type=Path, default=Path('target/debug/theia-zone-editor'))
parser.add_argument('--output', type=Path, required=True)
args = parser.parse_args()
out = args.output.resolve()
out.mkdir(parents=True, exist_ok=False)
source, report = out/'source.zone.json', out/'state.json'
template = Path('content/development-gameplay-classic-quest-areas.json').resolve()
template_bytes = template.read_bytes()
original = json.dumps(dict(schema=4, name='NPC quest inspector', origin=[8192,0,8192],
    spawn=[8192,.1,8198], objects=[], markers=[dict(id=1, name='Village guide', kind='npc',
    position=[8194,.03,8192], yaw_degrees=0, radius_metres=1, definition='theia.dev.guide'),
    dict(id=2, name='Unbound NPC', kind='npc', position=[8190,.03,8192], yaw_degrees=0,
    radius_metres=1)])).encode()
source.write_bytes(original)
env = client_environment(dict(os.environ, DISPLAY=':0', XDG_RUNTIME_DIR='/run/user/1000',
    WGPU_BACKEND='vulkan', VK_INSTANCE_LAYERS='VK_LAYER_KHRONOS_validation',
    THEIA_PARTICLE_EDITOR_DATA_DIR=str(out/'particle-data')))
x = None
with (out/'editor.log').open('w') as log:
    process = subprocess.Popen([str(args.binary.resolve()), '--zone', str(source),
        '--gameplay-template', str(template), '--report', str(report)], env=env, stdout=log, stderr=log)
    try:
        window = wait(lambda: owned_window(process.pid), 45)
        x = Desktop(); x.focus(window)
        def state():
            assert process.poll() is None, 'Editor exited'
            return wait(lambda: json.loads(report.read_text()), 2)
        wait(lambda: report.exists() and state().get('menus'), 45)
        def fresh():
            before = state()['input_elapsed']
            return wait(lambda: state() if state()['input_elapsed'] > before else None)
        def click(point):
            x.move(window, point, 1); fresh()
            x.button(1, True); x.button(1, False); fresh()
        def select(identifier):
            row = wait(lambda: next((r for r in state()['content_list']['controls']
                if r['target'] == f'Marker({identifier})' and min(r['size']) > 0), None))
            click(row['centre']); wait(lambda: state()['marker_selected'] == identifier)
        def buttons():
            return state()['npc_quests']['controls']
        def quest(title):
            return next(b for b in buttons() if b['title'] == title)
        def capture(name):
            picture = report.with_suffix('.png'); picture.unlink(missing_ok=True)
            x.key('F12'); wait(lambda: capture_ready(picture), 30); picture.rename(out/name)
        select(1)
        click(next(c['centre'] for c in state()['property_tabs']['controls'] if c['tab'] == 'Attributes'))
        wait(lambda: state()['property_tabs']['active'] == 'Attributes')
        titles = [b['title'] for b in buttons() if b['title']]
        assert len(titles) == 3, titles
        capture('npc-quests.png')
        for title in titles:
            click(quest(title)['centre']); wait(lambda: quest(title)['expanded'])
            assert state()['marker_selected'] == 1
            if title == 'Meet the guide': capture('quest-details.png')
            click(quest(title)['centre']); wait(lambda: not quest(title)['expanded'])
        click(next(b['centre'] for b in buttons() if b['action'] == 'Choose gameplay'))
        wait(lambda: state()['file_dialog'] is not None)
        capture('gameplay-file-browser.png')
        assert Path(state()['file_dialog']['text']).name == template.name
        x.key('Return'); wait(lambda: state()['file_dialog'] is None)
        wait(lambda: len([b for b in buttons() if b['title']]) == 3)
        click(next(b['centre'] for b in buttons() if b['action'] == 'Choose gameplay'))
        wait(lambda: state()['file_dialog'] is not None)
        x.key('Escape'); wait(lambda: state()['file_dialog'] is None)
        select(2); wait(lambda: state()['npc_quests']['marker'] == 2)
        assert not any(b['title'] for b in buttons())
        capture('unbound-npc.png')
        select(1); wait(lambda: len([b for b in buttons() if b['title']]) == 3)
        assert not state()['unsaved_changes']
        assert source.read_bytes() == original and template.read_bytes() == template_bytes
        gpu = state()['gpu']
    finally:
        if x: x.close()
        if process.poll() is None:
            process.send_signal(signal.SIGINT)
            try: process.wait(timeout=10)
            except subprocess.TimeoutExpired: process.terminate(); process.wait(timeout=10)
assert check_gpu_diagnostics((out/'editor.log').read_text()) == 0
(out/'qualification.json').write_text(json.dumps(dict(passed=True, gpu=gpu,
    quest_titles=titles, native_expansion=True, browser_load_cancel=True,
    unbound_npc_clears_quests=True, source_preserved=True,
    scope='Read-only content inspection, not quest authoring or live player progress'), indent=2)+'\n')
