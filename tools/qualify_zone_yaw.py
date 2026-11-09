#!/usr/bin/env python3
"""Owned 3440x1440 native yaw-slider acceptance; requires desktop authorization."""
import argparse
import copy
import hashlib
import json
import os
from pathlib import Path
import subprocess

from gpu_validation import check_gpu_diagnostics
from qualify_world_input_os import Desktop, owned_window, wait, capture_ready

ROOT = Path(__file__).resolve().parents[1]


def run(output, binary, selected_case):
    output = output.resolve(); output.mkdir(parents=True, exist_ok=False)
    binary = binary.resolve(); cases = []
    env = dict(os.environ, WGPU_BACKEND='vulkan', VK_INSTANCE_LAYERS='VK_LAYER_KHRONOS_validation', VK_LOADER_DEBUG='layer')
    seed = output / 'seed.json'
    subprocess.run([str(binary), '--headless', '--commands', str(ROOT / 'content/zones/selection.commands.jsonl'), '--output', str(seed)], check=True, capture_output=True)
    for name, origin in [('local', [.125, .25, -.5]), ('far', [128000000000.125, 20.25, -128000000000.5])]:
        if selected_case != 'both' and name != selected_case:
            continue
        folder = output / name; folder.mkdir()
        source, saved, report, logfile = [folder / part for part in ['source.json', 'saved.json', 'state.json', 'editor.log']]
        zone = json.loads(seed.read_text()); zone['origin'] = origin; zone['paths'] = []
        zone['spawn'] = [origin[0], origin[1] + .0625, origin[2] + 6]
        zone['objects'] = zone['objects'][:1]
        obj = zone['objects'][0]
        obj['name'] = 'Precise yaw fixture'; obj['pose'] = dict(position=[origin[0] + 8.125, origin[1] + 1.25, origin[2] - 8.5], yaw_degrees=225.123456789, size=[3.5, 2.5, 4.25])
        for entries, offset in [(zone['plants'], [-6, 0, -4]), (zone['markers'], [6, 0, 4])]:
            for entry in entries:
                entry['position'] = [a + b for a, b in zip(origin, offset)]
                entry['yaw_degrees'] = 225.123456789
        zone['plants'][0]['scale'] = 1.25; zone['markers'][0]['radius_metres'] = 2.5
        zone['markers'][0]['definition'] = 'quest.precision-fixture'
        source.write_text(json.dumps(zone)); original_bytes = source.read_bytes()
        with logfile.open('w') as log:
            process = subprocess.Popen([str(binary), '--zone', str(source), '--save-path', str(saved), '--report', str(report)], env=env, stdout=log, stderr=log)
            desktop = None
            try:
                window = wait(lambda: owned_window(process.pid), 45); desktop = Desktop(); desktop.focus(window)
                def state():
                    assert process.poll() is None, 'Editor exited'
                    return json.loads(report.read_text())
                wait(lambda: report.exists() and state()['viewport'] == [3440, 1440], 45)
                desktop.move(window, [1737, 731], 1); desktop.move(window, [1720, 720], 1)
                def fresh():
                    elapsed = state()['input_elapsed']; wait(lambda: state()['input_elapsed'] > elapsed); return state()
                def entries(group):
                    return state()['content_list']['controls'] if group == 'content' else state()[group]
                def control(target, group='workspace_controls'):
                    entry = wait(lambda: next((c for c in entries(group) if c['target'] == target and all(v > 0 for v in c['size'])), None))
                    desktop.move(window, entry['centre'], 1)
                    wait(lambda: any(c['target'] == target and c['hovered'] for c in entries(group)))
                    desktop.button(1, True)
                    try:
                        wait(lambda: state()['left_mouse_down'])
                    finally:
                        desktop.button(1, False)
                    wait(lambda: not state()['left_mouse_down']); fresh()
                def capture(label):
                    picture = report.with_suffix('.png'); picture.unlink(missing_ok=True)
                    desktop.key('F12'); wait(lambda: capture_ready(picture), 30)
                    picture.replace(folder / f'{label}.png')
                records = []
                for index, (target, key, selection, desired, snap) in enumerate([
                    ('Object(1)', 'objects', 'selected', 90., True),
                    ('Plant(1001)', 'plants', 'plant_selected', 23., False),
                    ('Marker(1000)', 'markers', 'marker_selected', 45., True),
                ]):
                    control(target, 'content')
                    wait(lambda: state()[selection] == (1 if index == 0 else 1001 if index == 1 else 1000))
                    if state()['snap'] != snap:
                        control('Snap'); wait(lambda: state()['snap'] == snap)
                    control('Frame'); before = fresh(); document = copy.deepcopy(before['document'])
                    control('Transform'); wait(lambda: state()['transform_edit'] is not None)
                    original_fields = state()['transform_edit']['fields']
                    assert original_fields[3] == '225.123456789'
                    item = wait(lambda: next((c for c in state()['transform_controls'] if c['target'] == 'YawSlider' and all(v > 0 for v in c['size'])), None))
                    assert item['range'] == [-180., 180.]
                    x, y = item['centre']; width = item['size'][0]
                    thumb = [x - width / 2 + 7 + (width - 14) * (item['value'] + 180) / 360, y]
                    point = [x - width / 2 + 7 + (width - 14) * (desired + 180) / 360, y]
                    desktop.move(window, thumb, 1); desktop.button(1, True)
                    try:
                        wait(lambda: state()['left_mouse_down'])
                        desktop.move(window, point, 1); during = fresh()
                        assert during['document'] == document and during['revision'] == before['revision'] and during['history'] == before['history']
                    finally:
                        desktop.button(1, False)
                    wait(lambda: not state()['left_mouse_down']); current = fresh()
                    fields = current['transform_edit']['fields']; yaw = float(fields[3]); step = 15 if snap else 1
                    assert yaw % step == 0 and abs(yaw - desired) <= (0 if snap else 2)
                    assert fields[:3] == original_fields[:3] and fields[4:] == original_fields[4:]
                    assert current['document'] == document and current['revision'] == before['revision']
                    capture(f'{index}-pending-yaw')
                    control('Apply', 'transform_controls')
                    wait(lambda: state()['revision'] == before['revision'] + 1 and state()['transform_edit'] is None)
                    expected = copy.deepcopy(document)
                    if key == 'objects':
                        expected[key][0]['pose']['yaw_degrees'] = yaw
                        edit = dict(op='transform', id=1, pose=expected[key][0]['pose'])
                    else:
                        expected[key][0]['yaw_degrees'] = yaw
                        edit = dict(op='put_plant' if key == 'plants' else 'put_marker', **({'plant': expected[key][0]} if key == 'plants' else {'marker': expected[key][0]}))
                    assert state()['document'] == expected and state()['history'] == [index + 1, 0]
                    prior, commands, oracle = [folder / f'{index}.{suffix}' for suffix in ['prior.json', 'commands.jsonl', 'oracle.json']]
                    prior.write_text(json.dumps(document)); commands.write_text(json.dumps(dict(command='apply', expected_revision=0, edits=[edit])) + '\n')
                    subprocess.run([str(binary), '--headless', '--zone', str(prior), '--commands', str(commands), '--output', str(oracle)], check=True, capture_output=True)
                    assert json.loads(oracle.read_text()) == expected
                    control('Command(Undo)'); wait(lambda: state()['document'] == document and state()['revision'] == before['revision'] + 2)
                    # Opening and applying the initialized slider without touching
                    # it must retain the exact f64 original and pending redo.
                    control('Transform'); wait(lambda: state()['transform_edit'] is not None)
                    assert state()['transform_edit']['fields'][3] == '225.123456789'
                    control('Apply', 'transform_controls'); wait(lambda: state()['transform_edit'] is None)
                    assert state()['revision'] == before['revision'] + 2 and state()['history'] == [index, 1]
                    control('Command(Redo)'); wait(lambda: state()['document'] == expected and state()['revision'] == before['revision'] + 3)
                    records.append(dict(target=target, yaw=yaw, snap=snap, complete_headless_document_equal=True, history_roundtrip=True))
                control('Command(Save)'); wait(lambda: not state()['unsaved_changes'])
                assert json.loads(saved.read_text()) == expected and source.read_bytes() == original_bytes
                final = fresh(); subprocess.run(['wmctrl', '-ic', hex(window)], check=True); process.wait(timeout=20); assert process.returncode == 0
            finally:
                if desktop: desktop.close()
                if process.poll() is None: process.terminate(); process.wait(timeout=15)
        text = logfile.read_text(); assert 'Insert instance layer "VK_LAYER_KHRONOS_validation"' in text
        case = dict(name=name, viewport=final['viewport'], gpu=final['gpu'], validation_layer_active=True, gpu_diagnostics=check_gpu_diagnostics(text, allow_surface_resize=False), records=records)
        (folder / 'qualification.json').write_text(json.dumps(case, indent=2) + '\n'); cases.append(case)
    result = dict(cases=cases, editor_sha256=hashlib.sha256(binary.read_bytes()).hexdigest(), scope='Linux owned synthetic input, native 3440x1440 only; no Windows or multiplayer claim')
    (output / 'qualification.json').write_text(json.dumps(result, indent=2) + '\n'); print(json.dumps(result, indent=2))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--binary', type=Path, default=Path('target/debug/theia-zone-editor'))
    parser.add_argument('--case', choices=['both', 'local', 'far'], default='both')
    args = parser.parse_args(); run(args.output, args.binary, args.case)
