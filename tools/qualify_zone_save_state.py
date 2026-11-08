#!/usr/bin/env python3
"""Check saved/unsaved authoring state with owned native Linux input at 3440x1440."""
import argparse
import json
import os
from pathlib import Path
import subprocess

from gpu_validation import check_gpu_diagnostics
from qualify_world_input_os import Desktop, owned_window, wait, capture_ready


def run(output, binary):
    output.mkdir(parents=True, exist_ok=False)
    saved = output / 'draft.json'
    blocked = output / 'blocked'
    blocked.mkdir()
    checks = []
    cases = []
    env = dict(os.environ, WGPU_BACKEND='vulkan',
               VK_INSTANCE_LAYERS='VK_LAYER_KHRONOS_validation', VK_LOADER_DEBUG='layer')

    def case(name, arguments, verify):
        report = output / f'{name}.state.json'
        log_path = output / f'{name}.log'
        with log_path.open('w') as log:
            process = subprocess.Popen([str(binary.resolve()), *arguments, '--report', str(report)],
                                       env=env, stdout=log, stderr=log)
            desktop = None
            try:
                window = wait(lambda: owned_window(process.pid), 45)
                desktop = Desktop()
                desktop.focus(window)

                def state():
                    assert process.poll() is None, 'Editor exited during qualification'
                    return json.loads(report.read_text())

                wait(lambda: report.exists() and state()['viewport'] == [3440, 1440], 45)
                desktop.move(window, [1720, 720], 1)

                def key(name):
                    desktop.key(name)

                def chord(name):
                    desktop.key_state('Control_L', True)
                    try:
                        key(name)
                    finally:
                        desktop.key_state('Control_L', False)

                def button(action):
                    entry = next(b for b in state()['buttons'] if b['action'] == action)
                    desktop.move(window, entry['centre'], 1)
                    wait(lambda: any(b['action'] == action and b['hovered'] for b in state()['buttons']))
                    desktop.button(1, True)
                    try:
                        wait(lambda: state()['left_mouse_down'])
                    finally:
                        desktop.button(1, False)
                    wait(lambda: not state()['left_mouse_down'])

                verify(state, key, chord, button, desktop, window)
                key('F12')
                wait(lambda: capture_ready(report.with_suffix('.png')), 30)
                final = state()
                subprocess.run(['wmctrl', '-ic', hex(window)], check=True)
                process.wait(timeout=20)
                assert process.returncode == 0
            finally:
                if desktop:
                    desktop.close()
                if process.poll() is None:
                    process.terminate()
                    process.wait(timeout=15)
        log_text = log_path.read_text()
        assert 'Insert instance layer "VK_LAYER_KHRONOS_validation"' in log_text
        diagnostics = check_gpu_diagnostics(log_text, allow_surface_resize=False)
        cases.append(dict(name=name, gpu=final['gpu'], viewport=final['viewport'],
                          validation_layer_active=True, allowed_resize_diagnostics=diagnostics))

    def fresh(state, key, chord, button, desktop, window):
        assert not saved.exists() and state()['unsaved_changes']
        initial = state()['document']
        chord('s')
        wait(lambda: saved.exists() and not state()['unsaved_changes'])
        assert state()['document'] == initial
        button('Place(Box)')
        point = state()['ground'][0]
        desktop.move(window, point, 1)
        desktop.button(1, True)
        try:
            wait(lambda: state()['left_mouse_down'])
        finally:
            desktop.button(1, False)
        wait(lambda: state()['revision'] == 2 and state()['unsaved_changes'])
        chord('z')
        wait(lambda: state()['revision'] == 3 and not state()['unsaved_changes'])
        assert state()['document'] == initial
        chord('y')
        wait(lambda: state()['revision'] == 4 and state()['unsaved_changes'])
        edited = state()['document']
        chord('s')
        wait(lambda: not state()['unsaved_changes'])
        assert json.loads(saved.read_text()) == edited
        chord('z')
        wait(lambda: state()['revision'] == 5 and state()['unsaved_changes'])
        chord('o')
        wait(lambda: state()['revision'] == 6 and not state()['unsaved_changes'])
        assert state()['document'] == edited
        chord('z')
        wait(lambda: state()['revision'] == 7 and state()['unsaved_changes'])
        chord('y')
        wait(lambda: state()['revision'] == 8 and not state()['unsaved_changes'])
        checks.append('new draft writes only on Save; visual edits and exact undo/redo/load compare with last saved content')
        mailbox = output / 'mailbox'
        bridge = json.loads((mailbox / 'state.json').read_text())
        pose = dict(edited['objects'][-1]['pose'])
        pose['position'] = list(pose['position'])
        pose['position'][1] += 1.
        request = dict(session=bridge['session'], request_id=1,
                       command=dict(command='apply', expected_revision=8,
                                    edits=[dict(op='transform', id=edited['objects'][-1]['id'], pose=pose)]))
        (mailbox / 'agent.tmp').write_text(json.dumps(request))
        (mailbox / 'agent.tmp').replace(mailbox / 'request.json')
        wait(lambda: state()['revision'] == 9 and state()['unsaved_changes'])
        receipt = wait(lambda: json.loads((mailbox / 'receipt.json').read_text()) if (mailbox / 'receipt.json').exists() else None)
        assert receipt['accepted']
        assert json.loads(saved.read_text()) == edited
        chord('z')
        wait(lambda: state()['revision'] == 10 and not state()['unsaved_changes'])
        checks.append('accepted agent edit is unsaved until Save; visual undo restores the saved baseline')
        key('F3')
        wait(lambda: state()['path_mode'])
        button('NewPath(Road)')
        wait(lambda: state()['path_draft_points'] == 0 and state()['unsaved_changes'])
        chord('s')
        wait(lambda: state()['status'].endswith('(guide draft still pending)'))
        assert state()['unsaved_changes'] and json.loads(saved.read_text()) == edited
        key('Escape')
        wait(lambda: state()['path_draft_points'] is None and not state()['unsaved_changes'])
        key('F5')
        wait(lambda: state()['play_feet'] is not None)
        assert not state()['unsaved_changes']
        key('F5')
        wait(lambda: state()['play_feet'] is None)
        checks.append('pending guide draft stays unsaved across Save; cancellation restores clean content; playtest preserves it')

    case('fresh', ['--commands', 'content/zones/greybox.commands.jsonl', '--save-path', str(saved), '--mailbox', str(output / 'mailbox')], fresh)
    stored_bytes = saved.read_bytes()

    def reopen(state, key, chord, button, desktop, window):
        assert not state()['unsaved_changes']
        assert saved.read_bytes() == stored_bytes
        checks.append('saved draft reopens clean without rewriting source bytes')

    case('reopen', ['--zone', str(saved)], reopen)

    def failure(state, key, chord, button, desktop, window):
        assert state()['unsaved_changes']
        before = state()
        chord('s')
        wait(lambda: 'directory' in state()['status'].lower())
        assert state()['unsaved_changes']
        assert state()['document'] == before['document']
        assert state()['revision'] == before['revision']
        assert blocked.is_dir() and list(blocked.iterdir()) == []
        assert saved.read_bytes() == stored_bytes
        checks.append('failed save keeps unsaved state, document, history revision and existing source intact')

    case('failure', ['--zone', str(saved), '--save-path', str(blocked)], failure)
    summary = dict(checks=checks, cases=cases,
                   scope='Linux owned OS input, standard Bevy UI at 3440x1440; no Windows or multiplayer claim')
    (output / 'qualification.json').write_text(json.dumps(summary, indent=2) + '\n')
    print(json.dumps(summary, indent=2))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--binary', type=Path, default=Path('target/debug/theia-zone-editor'))
    args = parser.parse_args()
    run(args.output, args.binary)
