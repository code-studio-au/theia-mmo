#!/usr/bin/env python3
"""Owned native content-list paging at the supported 3440x1440 authoring size."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess

from gpu_validation import check_gpu_diagnostics
from qualify_world_input_os import Desktop, owned_window, wait, capture_ready


def run(output, binary, selected_case):
    output = output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    binary = binary.resolve()
    cases = []
    env = dict(os.environ, WGPU_BACKEND='vulkan', VK_INSTANCE_LAYERS='VK_LAYER_KHRONOS_validation', VK_LOADER_DEBUG='layer')
    for name, origin in [('local', [.125, .25, -.5]), ('far', [128000000000.125, 20.25, -128000000000.5])]:
        if selected_case != 'both' and name != selected_case:
            continue
        folder = output / name
        folder.mkdir()
        source, report, mailbox, logfile = [folder / part for part in ['source.json', 'state.json', 'mailbox', 'editor.log']]
        objects = [dict(id=i, name=f'Block {i} ' + ('long named content ' * 2 if i % 17 == 0 else ''), primitive='box',
                        pose=dict(position=[origin[0] + 12 + i % 8 * 4, origin[1] + 1, origin[2] - 12 - i // 8 * 4], yaw_degrees=0, size=[2, 2, 2])) for i in range(1, 44)]
        source.write_text(json.dumps(dict(schema=1, name='Content browsing', origin=origin, spawn=[origin[0], origin[1] + .0625, origin[2] + 6], objects=objects)))
        original = source.read_bytes()
        with logfile.open('w') as log:
            desktop = Desktop()
            desktop.prepare_wheel()
            process = subprocess.Popen([str(binary), '--zone', str(source), '--save-path', str(folder / 'unused.json'), '--report', str(report), '--mailbox', str(mailbox)], env=env, stdout=log, stderr=log)
            try:
                window = wait(lambda: owned_window(process.pid), 45)
                desktop.focus(window)
                def state():
                    assert process.poll() is None, 'Editor exited'
                    return json.loads(report.read_text())
                wait(lambda: report.exists() and state()['viewport'] == [3440, 1440], 45)
                desktop.move(window, [1737, 731], 1)
                desktop.move(window, [1720, 720], 1)
                def fresh():
                    elapsed = state()['input_elapsed']
                    wait(lambda: state()['input_elapsed'] > elapsed)
                    return state()
                def click(target, group='content_list'):
                    def controls():
                        return state()['content_list']['controls'] if group == 'content_list' else state()['workspace_controls']
                    entry = wait(lambda: next((c for c in controls() if c['target'] == target and all(v > 0 for v in c['size'])), None))
                    desktop.move(window, entry['centre'], 1)
                    wait(lambda: any(c['target'] == target and c['hovered'] for c in controls()))
                    desktop.button(1, True)
                    try:
                        wait(lambda: state()['left_mouse_down'])
                    finally:
                        desktop.button(1, False)
                    wait(lambda: not state()['left_mouse_down'])
                initial = fresh()
                assert initial['content_list']['count'] == 44 and initial['content_list']['pages'] == 3
                seen = []
                def page(index):
                    current = fresh()
                    content = current['content_list']
                    assert content['page'] == index
                    rows = [c for c in content['controls'] if c['target'] not in ['Previous', 'Next', 'Search', 'ClearSearch'] and not c['target'].startswith('Filter(')]
                    rows.sort(key=lambda row: row['centre'][1])
                    expected = (['Spawn'] + [f'Object({i})' for i in range(1, 44)])[index * 18:(index + 1) * 18]
                    assert [c['target'] for c in rows] == expected
                    for row in rows:
                        x, y = row['centre']; w, h = row['size']
                        assert x - w / 2 >= 3140 and x + w / 2 <= 3440
                        assert y - h / 2 >= 92 and y + h / 2 <= 840, row
                        assert not row['disabled']
                    buttons = {c['target']: c for c in content['controls']}
                    assert buttons['Previous']['disabled'] == (index == 0)
                    assert buttons['Next']['disabled'] == (index == 2)
                    assert current['view'] == initial['view'] and current['document'] == initial['document']
                    assert current['revision'] == 0 and current['history'] == [0, 0]
                    seen.extend(expected)
                    return rows
                def wheel(lines, expected):
                    desktop.move(window, [3290, 350], 1)
                    before = fresh()['content_list']['scroll_events']
                    assert state()['focused'] and state()['cursor'] is not None, 'Owned editor lost focus/cursor before wheel delivery'
                    desktop.scroll(lines)
                    # Never retry a mutating event. Retain the observed routing
                    # record to distinguish OS delivery from editor gating.
                    try:
                        wait(lambda: state()['content_list']['scroll_events'] > before)
                        wait(lambda: state()['content_list']['page'] == expected)
                    except TimeoutError as error:
                        (folder / 'wheel-failure.json').write_text(json.dumps(state()['content_list'], indent=2) + '\n')
                        raise AssertionError('Wheel delivery/routing failed; see wheel-failure.json') from error
                    fresh()
                def capture(label):
                    picture = report.with_suffix('.png'); picture.unlink(missing_ok=True)
                    desktop.key('F12'); wait(lambda: capture_ready(picture), 30)
                    picture.replace(folder / f'{label}.png')
                rows = page(0)
                click(rows[-1]['target']); wait(lambda: state()['selected'] == 17)
                capture('first-page')
                click('Previous'); assert fresh()['content_list']['page'] == 0
                wheel(-1, 1)
                rows = page(1)
                click(rows[-1]['target']); wait(lambda: state()['selected'] == 35)
                wheel(-1, 2)
                rows = page(2)
                click(rows[-1]['target']); wait(lambda: state()['selected'] == 43)
                capture('last-page')
                wheel(-1, 2)
                click('Next'); assert fresh()['content_list']['page'] == 2
                wheel(1, 1)
                click('Filter(Objects)')
                wait(lambda: state()['content_list']['filter'] == 'Objects')
                for _ in range(3):
                    filtered = fresh()['content_list']
                    assert filtered['page'] == 0 and filtered['count'] == 43 and filtered['total'] == 44
                click('Filter(Markers)')
                wait(lambda: state()['content_list']['filter'] == 'Markers')
                filtered = fresh()['content_list']
                assert filtered['count'] == 0 and filtered['pages'] == 1
                assert all(c['disabled'] for c in filtered['controls'] if c['target'] in ['Previous', 'Next'])
                click('Filter(All)')
                wait(lambda: state()['content_list']['filter'] == 'All')
                click('Search')
                wait(lambda: state()['name_edit'] is not None)
                assert state()['text_form_kind'] == 'search'
                assert state()['unsaved_changes'] == initial['unsaved_changes'], 'Browser search must preserve the saved-state indicator'
                for character in 'block 17':
                    desktop.key('space' if character == ' ' else character)
                wait(lambda: state()['name_edit'] == 'block 17')
                desktop.key('Return')
                wait(lambda: state()['name_edit'] is None and state()['content_list']['query'] == 'block 17')
                assert fresh()['content_list']['count'] == 1
                click('Object(17)'); wait(lambda: state()['selected'] == 17)
                click('Search'); wait(lambda: state()['name_edit'] is not None)
                desktop.key('Escape'); wait(lambda: state()['name_edit'] is None)
                assert state()['content_list']['query'] == 'block 17'
                click('ClearSearch')
                wait(lambda: state()['content_list']['query'] == '' and state()['content_list']['count'] == 44)
                wheel(-1, 1)
                click('SpawnTransform', 'workspace_controls')
                wait(lambda: state()['transform_edit'] is not None)
                wheel(-1, 1)
                assert all(c['disabled'] for c in fresh()['content_list']['controls'])
                desktop.key('Escape'); wait(lambda: state()['transform_edit'] is None)
                click('Previous'); wait(lambda: state()['content_list']['page'] == 0)
                assert state()['document'] == initial['document'] and source.read_bytes() == original
                entry = next(c for c in state()['content_list']['controls'] if c['target'] == 'Object(17)')
                desktop.move(window, entry['centre'], 1)
                wait(lambda: any(c['target'] == 'Object(17)' and c['hovered'] for c in state()['content_list']['controls']))
                for _ in range(2):
                    desktop.button(1, True)
                    desktop.button(1, False)
                expected = [objects[16]['pose']['position'][i] - origin[i] for i in range(3)]
                wait(lambda: max(abs(a - b) for a, b in zip(state()['view']['target'], expected)) < 1e-4)
                assert state()['document'] == initial['document'] and state()['revision'] == 0
                capture('framed-selection')
                final = fresh()
                subprocess.run(['wmctrl', '-ic', hex(window)], check=True)
                process.wait(timeout=20)
            finally:
                if desktop:
                    desktop.close()
                if process.poll() is None:
                    process.terminate(); process.wait(timeout=15)
        text = logfile.read_text()
        assert 'Insert instance layer "VK_LAYER_KHRONOS_validation"' in text
        diagnostics = check_gpu_diagnostics(text, allow_surface_resize=False)
        record = dict(name=name, gpu=final['gpu'], viewport=final['viewport'], visible_targets=seen, gpu_diagnostics=diagnostics, validation_layer_active=True,
                      checks=['all 44 targets reachable with every row wholly above Properties', 'last row selected on all three pages; disabled boundary buttons', 'Linux wheel paging, type filters and search preserve source/history and isolate exact-properties modal', 'double-click frames an entry through the existing checked local-coordinate path'])
        (folder / 'qualification.json').write_text(json.dumps(record, indent=2) + '\n')
        cases.append(record)
    result = dict(cases=cases, binary_sha256=hashlib.sha256(binary.read_bytes()).hexdigest(), scope='Linux native synthetic input, 3440x1440; no Windows or multiplayer claim')
    (output / 'qualification.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps(result, indent=2))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--binary', type=Path, default=Path('target/debug/theia-zone-editor'))
    parser.add_argument('--case', choices=['both', 'local', 'far'], default='both')
    args = parser.parse_args()
    run(args.output, args.binary, args.case)
