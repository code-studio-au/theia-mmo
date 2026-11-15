"""Inspect starter travel quests through ordinary input in an owned editor."""
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
parser.add_argument('--zone', type=Path, required=True)
parser.add_argument('--scenery-library', type=Path)
parser.add_argument('--output', type=Path, required=True)
args = parser.parse_args()
out = args.output.resolve()
out.mkdir(parents=True, exist_ok=False)
source, report = out/'source.zone.json', out/'state.json'
original = args.zone.read_bytes()
source.write_bytes(original)
template = Path('content/development-gameplay-starter-economy.json').resolve()
template_bytes = template.read_bytes()
env = client_environment(dict(os.environ, DISPLAY=':0', XDG_RUNTIME_DIR='/run/user/1000',
    WGPU_BACKEND='vulkan', VK_INSTANCE_LAYERS='VK_LAYER_KHRONOS_validation',
    THEIA_PARTICLE_EDITOR_DATA_DIR=str(out/'particle-data')))
x = None
with (out/'editor.log').open('w') as log:
    process = subprocess.Popen([str(args.binary.resolve()), '--zone', str(source),
        '--gameplay-template', str(template), '--report', str(report),
        *(['--scenery-library', str(args.scenery_library.resolve())] if args.scenery_library else [])],
        env=env, stdout=log, stderr=log)
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
        def capture(name):
            picture = report.with_suffix('.png'); picture.unlink(missing_ok=True)
            x.key('F12'); wait(lambda: capture_ready(picture), 30); picture.rename(out/name)
        def select(identifier):
            row = wait(lambda: next((r for r in state()['content_list']['controls']
                if r['target'] == f'Marker({identifier})' and min(r['size']) > 0), None))
            click(row['centre']); wait(lambda: state()['marker_selected'] == identifier)
        def control(title=None, action=None):
            return next(c for c in state()['npc_quests']['controls']
                if (title is not None and c['title'] == title)
                or (action is not None and c['action'] == action))
        def reveal(title=None, action=None):
            # Scroll only within the actual properties pane; never click clipped rows.
            for _ in range(50):
                c = control(title, action)
                left, top, width, height = state()['ui_layout']['inspector']
                if top + 65 < c['centre'][1] < top + height - 40 and min(c['size']) > 0:
                    return c
                x.move(window, [left + width/2, top + height/2], 1); fresh()
                wheel = 5 if c['centre'][1] >= top + height - 40 else 4
                x.button(wheel, True); x.button(wheel, False); fresh()
            raise AssertionError(f'Cannot reveal {title or action}')
        initial = state()
        click(next(m['centre'] for m in state()['menus']
            if m['name'] == 'Content filter' and m['role'] == 'button' and min(m['size']) > 0))
        click(wait(lambda: next((c for c in state()['content_list']['controls']
            if c['target'] == 'Filter(Markers)' and min(c['size']) > 0), None))['centre'])
        select(1004)
        click(next(c['centre'] for c in state()['property_tabs']['controls'] if c['tab'] == 'Attributes'))
        wait(lambda: state()['property_tabs']['active'] == 'Attributes')
        follow = next(c for c in state()['npc_quests']['cards'] if c['title'] == 'Follow the NPC')
        assert follow['destination'] == 1202
        for text in ['7.0 m/s', '10 m', 'Speak with Ranger Elowen']:
            assert text in '\n'.join(part for pair in follow['details'] for part in pair), (text, follow)
        click(reveal(title='Follow the NPC')['centre'])
        wait(lambda: control(title='Follow the NPC')['expanded'])
        destination = reveal(action='Select destination')
        capture('escort-details.png')
        click(destination['centre']); wait(lambda: state()['marker_selected'] == 1202)
        capture('destination-selected.png')
        select(1005)
        compass = next(c for c in state()['npc_quests']['cards'] if c['title'] == "Rowan's lost compass")
        for text in ['Ground collection', 'Lost Compass', 'Northern Meadow', 'Deliver Lost Compass to Rowan']:
            assert text in '\n'.join(part for pair in compass['details'] for part in pair), (text, compass)
        click(reveal(title="Rowan's lost compass")['centre'])
        wait(lambda: control(title="Rowan's lost compass")['expanded'])
        capture('collection-details.png')
        final = state()
        assert final['document'] == initial['document']
        assert final['revision'] == initial['revision'] and final['history'] == initial['history']
        assert not final['unsaved_changes']
        assert source.read_bytes() == original and template.read_bytes() == template_bytes
        gpu = final['gpu']
    finally:
        if x: x.close()
        if process.poll() is None:
            process.send_signal(signal.SIGINT)
            try: process.wait(timeout=10)
            except subprocess.TimeoutExpired: process.terminate(); process.wait(timeout=10)
assert check_gpu_diagnostics((out/'editor.log').read_text()) == 0
(out/'qualification.json').write_text(json.dumps(dict(passed=True, gpu=gpu,
    escort=follow, collection=compass, native_destination_selection=True,
    source_history_preserved=True, scope='Read-only quest content, not live progress'), indent=2)+'\n')
