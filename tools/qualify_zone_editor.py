#!/usr/bin/env python3
"""Native Linux OS-input qualification for the bounded offline zone editor."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import struct
import time

from gpu_validation import check_gpu_diagnostics
from qualify_world_input_os import Desktop, owned_window, wait, capture_ready


def run(output, binary, size, agent=False, terrain=False, guides=False, markers=False, vegetation=False, grading=False):
    vegetation = vegetation or grading
    markers = markers or vegetation
    guides = guides or markers
    terrain = terrain or guides
    agent = agent or terrain
    output.mkdir(parents=True, exist_ok=True)
    report = output / 'state.json'
    saved = output / 'zone.json'
    for path in [report, report.with_suffix('.png'), saved]:
        path.unlink(missing_ok=True)
    env = dict(os.environ, WGPU_BACKEND='vulkan', VK_INSTANCE_LAYERS='VK_LAYER_KHRONOS_validation', VK_LOADER_DEBUG='layer')
    checks = []
    with (output / 'client.log').open('w') as log:
        command = [str(binary.resolve()), '--commands', 'content/zones/greybox.commands.jsonl',
                   '--output', str(saved), '--report', str(report)]
        if agent:
            command += ['--mailbox', str(output / 'mailbox')]
        process = subprocess.Popen(command, env=env, stdout=log, stderr=log)
        desktop = None
        try:
            window = wait(lambda: owned_window(process.pid), 45)
            desktop = Desktop()
            desktop.focus(window)
            def state():
                assert process.poll() is None, 'Editor exited during qualification'
                return json.loads(report.read_text())
            wait(lambda: report.exists() and len(state()['buttons']) == 12, 45)
            # Verify the tool's own fullscreen startup; do not resize it into compliance.
            width, height = map(int, size.split('x'))
            wait(lambda: state()['viewport'] == [width, height])
            assert all(0 <= b['centre'][1] - b['size'][1]/2 and b['centre'][1] + b['size'][1]/2 <= state()['viewport'][1] for b in state()['buttons'])
            def read_saved():
                value = json.loads(saved.read_text())
                # Files serialize f32 heights with shortest round-trippable decimals;
                # JSON Value telemetry expands them to f64. Compare declared f32 data.
                if value.get('terrain'):
                    value['terrain']['heights'] = [struct.unpack('<f', struct.pack('<f', h))[0] for h in value['terrain']['heights']]
                return value
            def button(action):
                entry = next(b for b in state()['buttons'] if b['action'] == action)
                desktop.click(window, entry['centre'], 1, 1)
            def key(name):
                desktop.key(name)
            def chord(name):
                desktop.key_state('Control_L', True)
                try:
                    key(name)
                finally:
                    desktop.key_state('Control_L', False)
            def revision(previous):
                return wait(lambda: state() if state()['revision'] == previous + 1 else None)
            next_request = 0
            def agent_request(command=None, request=None):
                nonlocal next_request
                inbox = output / 'mailbox'
                bridge = wait(lambda: json.loads((inbox/'state.json').read_text()))
                if request is None:
                    next_request += 1
                    request = dict(session=bridge['session'], request_id=next_request, command=command)
                (inbox/'agent.tmp').write_text(json.dumps(request))
                (inbox/'agent.tmp').replace(inbox/'request.json')
                wait(lambda: not (inbox/'request.json').exists())
                receipt = wait(lambda: json.loads((inbox/'receipt.json').read_text()) if json.loads((inbox/'receipt.json').read_text())['request_id'] == request['request_id'] else None)
                assert receipt['session'] == bridge['session']
                return receipt, request
            initial = state()
            assert initial['revision'] == 1 and len(initial['document']['objects']) == 5
            checks.append('actual startup commands populate five rendered primitives')
            button('Place(Box)')
            time.sleep(.3)
            desktop.click(window, state()['ground'][0], 1, 1)
            placed = revision(1)
            assert len(placed['document']['objects']) == 6 and placed['selected'] == 6
            checks.append('actual palette pointer placement shares command history')
            key('Escape')
            time.sleep(.3)
            start = next(o['screen'] for o in state()['objects'] if o['id'] == 6)
            before = state()
            desktop.move(window, start, 1)
            desktop.button(1, True)
            desktop.move(window, [start[0] + 70, start[1] - 12], 1)
            time.sleep(.4)
            assert state()['revision'] == before['revision'], 'Preview committed before release'
            desktop.button(1, False)
            moved = revision(before['revision'])
            assert moved['document']['objects'][-1]['pose']['position'] != before['document']['objects'][-1]['pose']['position']
            checks.append('native drag previews without revision and releases one transaction')
            for name in ['Right', 'Page_Up', 'r', 'equal']:
                previous = state()['revision']
                key(name)
                revision(previous)
            changed = state()
            shape = changed['document']['objects'][-1]['pose']
            assert shape['yaw_degrees'] == 15 and shape['size'] == [1.25]*3
            checks.append('actual keyboard translation, elevation, yaw and uniform scale')
            chord('z')
            revision(changed['revision'])
            previous = state()['revision']
            button('Redo')
            redone = revision(previous)
            assert redone['document'] == changed['document']
            checks.append('keyboard undo and widget redo restore exact shared document')
            previous = state()['revision']
            key('Delete')
            assert len(revision(previous)['document']['objects']) == 5
            previous = state()['revision']
            button('Undo')
            assert len(revision(previous)['document']['objects']) == 6
            point = next(o['screen'] for o in state()['objects'] if o['id'] == 6)
            desktop.click(window, point, 1, 1)
            wait(lambda: state()['selected'] == 6)
            chord('s')
            wait(lambda: read_saved() == state()['document'])
            saved_document = read_saved()
            previous = state()['revision']
            key('Left')
            altered = revision(previous)
            previous = altered['revision']
            button('Load')
            loaded = revision(previous)
            assert loaded['document'] == saved_document
            previous = loaded['revision']
            chord('z')
            assert revision(previous)['document'] == altered['document']
            previous = state()['revision']
            chord('y')
            assert revision(previous)['document'] == saved_document
            checks.append('save/load exact round trip and undoable reload')
            button('Snap')
            wait(lambda: not state()['snap'])
            key('g')
            wait(lambda: state()['snap'])
            if agent:
                previous = state()
                new_object = dict(id=100, name='Agent pillar', primitive='cylinder',
                                  pose=dict(position=[-3., 1., 4.], size=[1., 2., 1.], yaw_degrees=0.))
                receipt, request = agent_request(dict(command='apply', expected_revision=previous['revision'],
                                                     edits=[dict(op='place', object=new_object)]))
                assert receipt['accepted']
                placed_by_agent = revision(previous['revision'])
                assert len(placed_by_agent['document']['objects']) == 7
                chord('z')
                restored = revision(placed_by_agent['revision'])
                assert restored['document'] == previous['document']
                retry, _ = agent_request(request=request)
                assert retry == receipt
                assert state()['revision'] == restored['revision']
                receipt, _ = agent_request(dict(command='redo', expected_revision=restored['revision']))
                assert receipt['accepted']
                restored_by_agent = revision(restored['revision'])
                assert restored_by_agent['document'] == placed_by_agent['document']
                button('Undo')
                restored = revision(restored_by_agent['revision'])
                assert restored['document'] == previous['document']
                receipt, _ = agent_request(dict(command='undo', expected_revision=0))
                assert not receipt['accepted'] and receipt['revision'] == restored['revision']
                button('Place(Box)')
                desktop.click(window, state()['ground'][1], 1, 1)
                visual = revision(restored['revision'])
                assert visual['document']['objects'][-1]['id'] == 101
                chord('z')
                undone = revision(visual['revision'])
                assert undone['document'] == previous['document']
                key('Escape')
                checks.append('live agent edit, visual undo, exact retry, agent redo, stale refusal and collision-free visual IDs share history')
            before_play = state()
            key('F5')
            playing = wait(lambda: state() if state()['play_feet'] is not None else None)
            start_feet = playing['play_feet']
            if agent:
                receipt, _ = agent_request(dict(command='undo', expected_revision=playing['revision']))
                assert not receipt['accepted'] and 'busy' in receipt['error']
                assert state()['revision'] == playing['revision']
                checks.append('actual live agent request refuses during offline playtest')
            desktop.key_state('w', True)
            try:
                time.sleep(.65)
            finally:
                desktop.key_state('w', False)
            wait(lambda: state()['play_feet'][2] < start_feet[2] - 1.)
            key('F5')
            after_play = wait(lambda: state() if state()['play_feet'] is None else None)
            assert after_play['document'] == before_play['document']
            assert after_play['revision'] == before_play['revision']
            checks.append('F5 native shared-Avian walking returns without changing edits/history')
            if terrain:
                key('F2')
                wait(lambda: state()['terrain_mode'])
                previous = state()['revision']
                button('TerrainToggle')
                flat = revision(previous)
                assert flat['document']['schema'] == 2
                assert len(flat['document']['terrain']['heights']) == 1089
                time.sleep(.5)
                button('Brush(Raise)')
                desktop.move(window, state()['terrain_points'][0], 1)
                desktop.button(1, True)
                wait(lambda: state()['dragging'])
                key('Escape')
                desktop.button(1, False)
                wait(lambda: not state()['dragging'])
                assert state()['revision'] == flat['revision'] and state()['document'] == flat['document']
                checks.append('actual Escape cancels a terrain preview without changing document/history')
                desktop.move(window, state()['terrain_points'][0], 1)
                desktop.button(1, True)
                time.sleep(.3)
                desktop.move(window, state()['terrain_points'][1], 1)
                time.sleep(.3)
                assert state()['revision'] == flat['revision']
                assert state()['dragging']
                receipt, _ = agent_request(dict(command='undo', expected_revision=flat['revision']))
                assert not receipt['accepted'] and 'busy' in receipt['error']
                desktop.button(1, False)
                raised = revision(flat['revision'])
                assert max(raised['document']['terrain']['heights']) > 1.
                chord('z')
                assert revision(raised['revision'])['document'] == flat['document']
                previous = state()['revision']
                chord('y')
                assert revision(previous)['document'] == raised['document']
                checks.append('native terrain drag previews through shared validation and commits one undo entry; live agent refuses preview')
                previous = state()['revision']
                button('Brush(Lower)')
                desktop.click(window, state()['terrain_points'][2], 1, 1)
                lowered = revision(previous)
                assert min(lowered['document']['terrain']['heights']) < -1.
                for tool in ['Flatten', 'Smooth']:
                    previous = state()['revision']
                    before = state()['document']['terrain']['heights']
                    button(f'Brush({tool})')
                    desktop.click(window, state()['terrain_points'][0], 1, 1)
                    after = revision(previous)
                    assert after['document']['terrain']['heights'] != before
                button('Radius(1)')
                wait(lambda: state()['brush_radius'] == 16.)
                button('Radius(-1)')
                wait(lambda: state()['brush_radius'] == 12.)
                key('F1')
                wait(lambda: not state()['terrain_mode'])
                previous = state()['revision']
                button('Spawn')
                spawned = revision(previous)
                chord('s')
                wait(lambda: read_saved() == state()['document'])
                saved_terrain = state()['document']
                key('F5')
                playing = wait(lambda: state() if state()['play_feet'] is not None else None)
                assert playing['play_feet'][1] > .1
                start = playing['play_feet']
                desktop.key_state('w', True)
                try:
                    time.sleep(.6)
                finally:
                    desktop.key_state('w', False)
                wait(lambda: state()['play_feet'][2] < start[2]-1.)
                key('F5')
                wait(lambda: state()['play_feet'] is None)
                assert state()['document'] == saved_terrain
                previous = state()['revision']
                chord('o')
                assert revision(previous)['document'] == saved_terrain
                receipt, _ = agent_request(dict(command='apply', expected_revision=state()['revision'], edits=[dict(op='brush',brush=dict(centre=[80.,64.],radius_metres=8.,operation=dict(kind='lower',metres=2.)))]))
                assert receipt['accepted']
                changed = wait(lambda: state() if state()['revision'] == receipt['revision'] else None)
                assert changed['document']['terrain']['heights'] != saved_terrain['terrain']['heights']
                chord('z')
                assert revision(changed['revision'])['document'] == saved_terrain
                key('F2')
                wait(lambda: state()['terrain_mode'])
                checks.append('native lower/flatten/smooth, brush radius, terrain save/load, actual capsule terrain walk and live agent brush/visual undo')
            if guides:
                previous = state()
                mesh_baseline = previous['mesh_assets']
                key('F3')
                wait(lambda: state()['path_mode'])
                button('NewPath(Road)')
                wait(lambda: state()['path_draft_points'] == 0)
                desktop.click(window, state()['guide_points'][0], 1, 1)
                wait(lambda: state()['path_draft_points'] == 1)
                if grading:
                    button('GradePath')
                    wait(lambda: 'Finish or cancel' in state()['status'])
                    assert state()['path_draft_points'] == 1
                receipt, _ = agent_request(dict(command='undo', expected_revision=state()['revision']))
                assert not receipt['accepted'] and 'busy' in receipt['error']
                key('Escape')
                wait(lambda: state()['path_draft_points'] is None)
                assert state()['document'] == previous['document'] and state()['revision'] == previous['revision']
                button('NewPath(Road)')
                desktop.click(window, state()['guide_points'][0], 1, 1)
                desktop.click(window, state()['guide_points'][1], 1, 1)
                wait(lambda: state()['path_draft_points'] == 2)
                key('BackSpace')
                wait(lambda: state()['path_draft_points'] == 1)
                desktop.click(window, state()['guide_points'][1], 1, 1)
                wait(lambda: state()['path_draft_points'] == 2)
                chord('s')
                wait(lambda: 'draft still pending' in state()['status'])
                assert state()['path_draft_points'] == 2 and read_saved() == previous['document']
                assert state()['revision'] == previous['revision']
                key('Return')
                road = revision(previous['revision'])
                assert len(road['document']['paths']) == 1 and road['document']['schema'] == 3
                assert road['document']['paths'][0]['kind'] == 'road'
                key('Right')
                translated = revision(road['revision'])
                assert all(new[0] == old[0]+1. for new, old in zip(translated['document']['paths'][0]['points'],road['document']['paths'][0]['points']))
                chord('z')
                assert revision(translated['revision'])['document'] == road['document']
                before = state()['revision']
                chord('y')
                assert revision(before)['document'] == translated['document']
                before = state()['revision']
                key('equal')
                wider = revision(before)
                assert wider['document']['paths'][0]['width_metres'] == 4.5
                before = wider['revision']
                key('Delete')
                removed = revision(before)
                assert not removed['document'].get('paths')
                wait(lambda: state()['mesh_assets'] == mesh_baseline)
                chord('z')
                assert revision(removed['revision'])['document'] == wider['document']
                wait(lambda: state()['mesh_assets'] == mesh_baseline+1)
                button('NewPath(Stream)')
                for point in state()['guide_points'][2:]:
                    desktop.click(window, point, 1, 1)
                wait(lambda: state()['path_draft_points'] == 3)
                before = state()['revision']
                key('Return')
                both = revision(before)
                assert len(both['document']['paths']) == 2 and both['document']['paths'][1]['kind'] == 'stream'
                wait(lambda: state()['mesh_assets'] == mesh_baseline+2)
                checks.append('native guide point placement/backspace/cancel, pending-draft save, one-entry finish, translation, width, delete/undo and bounded mesh lifetime')
                agent_path = dict(id=500,name='Agent road',kind='road',width_metres=2.,points=[[48.,.03,92.],[56.,.03,92.]])
                receipt, request = agent_request(dict(command='apply',expected_revision=both['revision'],edits=[dict(op='put_path',path=agent_path)]))
                assert receipt['accepted']
                agent_added = revision(both['revision'])
                assert len(agent_added['document']['paths']) == 3
                chord('z')
                undone = revision(agent_added['revision'])
                assert undone['document'] == both['document']
                retry, _ = agent_request(request=request)
                assert retry == receipt and state()['revision'] == undone['revision']
                button('NewPath(Road)')
                desktop.click(window, state()['guide_points'][0], 1, 1)
                desktop.click(window, state()['guide_points'][1], 1, 1)
                wait(lambda: state()['path_draft_points'] == 2)
                before = state()['revision']
                key('Return')
                visual = revision(before)
                assert visual['document']['paths'][-1]['id'] == 501
                chord('z')
                assert revision(visual['revision'])['document'] == both['document']
                wait(lambda: state()['mesh_assets'] == mesh_baseline+2)
                chord('s')
                wait(lambda: read_saved() == state()['document'])
                stored = state()['document']
                before = state()['revision']
                chord('o')
                loaded = revision(before)
                assert loaded['document'] == stored
                wait(lambda: state()['mesh_assets'] == mesh_baseline+2)
                key('F5')
                playing = wait(lambda: state() if state()['play_feet'] is not None else None)
                start = playing['play_feet']
                desktop.key_state('w', True)
                try:
                    time.sleep(.5)
                finally:
                    desktop.key_state('w', False)
                wait(lambda: state()['play_feet'][2] < start[2]-1.)
                key('F5')
                wait(lambda: state()['play_feet'] is None)
                assert state()['document'] == stored
                checks.append('live agent guide/visual undo/exact retry, shared ID allocation, guide save/load and nonblocking playtest')
            if markers:
                key('F4')
                wait(lambda: state()['marker_mode'])
                baseline = state()['mesh_assets']
                previous = state()['revision']
                button('PlaceMarker(Npc)')
                desktop.click(window, state()['guide_points'][0], 1, 1)
                placed = revision(previous)
                npc = placed['document']['markers'][0]
                assert npc['kind'] == 'npc' and placed['marker_selected'] == npc['id']
                assert npc['position'][0] == round(npc['position'][0])
                assert 'definition' not in npc
                for name in ['Right','Page_Up','r','equal']:
                    previous = state()['revision']
                    key(name)
                    revision(previous)
                transformed = state()['document']['markers'][0]
                assert transformed['position'][0] == npc['position'][0]+1.
                assert transformed['position'][1] == npc['position'][1]+1.
                assert transformed['yaw_degrees'] == 15 and transformed['radius_metres'] == 1.5
                chord('z')
                wait(lambda: state()['document']['markers'][0]['radius_metres'] == 1.)
                previous = state()['revision']
                chord('y')
                revision(previous)
                button('PlaceMarker(Quest)')
                previous = state()['revision']
                desktop.click(window, state()['guide_points'][2], 1, 1)
                quest = revision(previous)['document']['markers'][-1]
                assert quest['kind'] == 'quest' and quest['radius_metres'] == 4.
                key('Escape')
                wait(lambda: state()['marker_palette'] == 'None')
                centre = next(m['screen'] for m in state()['markers'] if m['id'] == npc['id'])
                desktop.click(window, centre, 1, 1)
                wait(lambda: state()['marker_selected'] == npc['id'])
                previous = state()['revision']
                key('Delete')
                assert len(revision(previous)['document']['markers']) == 1
                previous = state()['revision']
                chord('z')
                assert len(revision(previous)['document']['markers']) == 2
                edited = dict(state()['document']['markers'][0], id=900, name='Agent guide', definition='human.guide')
                receipt, _ = agent_request(dict(command='apply',expected_revision=state()['revision'],edits=[dict(op='put_marker',marker=edited)]))
                assert receipt['accepted']
                wait(lambda: len(state()['document']['markers']) == 3)
                chord('z')
                wait(lambda: len(state()['document']['markers']) == 2)
                # The editor allocator advances beyond agent IDs even after visual undo.
                button('PlaceMarker(Npc)')
                previous = state()['revision']
                desktop.click(window, state()['guide_points'][4], 1, 1)
                assert revision(previous)['document']['markers'][-1]['id'] == 901
                previous = state()['revision']
                chord('z')
                revision(previous)
                button('SelectMarker')
                chord('s')
                wait(lambda: read_saved() == state()['document'])
                stored = state()['document']
                previous = state()['revision']
                chord('o')
                assert revision(previous)['document'] == stored
                assert state()['mesh_assets'] == baseline
                wait(lambda: state()['marker_visible_entities'] == 6)
                # Annotations disappear during play; authoritative collision is unchanged.
                key('F5')
                wait(lambda: state()['play_feet'] is not None and state()['marker_visible_entities'] == 0)
                time.sleep(.3)
                key('F5')
                wait(lambda: state()['play_feet'] is None)
                assert state()['document'] == stored
                wait(lambda: state()['marker_visible_entities'] == 6)
                checks.append('NPC/quest marker pointer placement/selection, transforms/radius, delete/undo/redo, agent IDs, save/load and playtest')
            if vegetation:
                key('F6')
                wait(lambda: state()['vegetation_mode'])
                mesh_baseline = state()['mesh_assets']
                button('Plant(Tree, false)')
                previous = state()['revision']
                desktop.click(window,state()['guide_points'][4],1,1)
                placed = revision(previous)
                tree = placed['document']['plants'][0]
                assert tree['kind'] == 'tree' and tree['scale'] == 1.
                for name in ['Right','Page_Up','r','equal']:
                    previous = state()['revision']
                    key(name)
                    revision(previous)
                transformed = state()['document']['plants'][0]
                assert transformed['position'][0] == tree['position'][0]+1.
                assert transformed['position'][1] == tree['position'][1]+1.
                assert transformed['yaw_degrees'] == 15 and transformed['scale'] == 1.25
                button('Plant(Shrub, false)')
                previous = state()['revision']
                desktop.click(window,state()['guide_points'][2],1,1)
                shrub = revision(previous)['document']['plants'][-1]
                assert shrub['kind'] == 'shrub'
                key('Escape')
                time.sleep(.3)
                centre = next(p['screen'] for p in state()['plants'] if p['id'] == tree['id'])
                desktop.click(window,centre,1,1)
                wait(lambda: state()['plant_selected'] == tree['id'])
                previous = state()['revision']
                key('Delete')
                assert len(revision(previous)['document']['plants']) == 1
                previous = state()['revision']
                chord('z')
                assert len(revision(previous)['document']['plants']) == 2
                button('Plant(Tree, true)')
                key('bracketright')
                key('period')
                wait(lambda: state()['scatter_radius'] == 9. and state()['scatter_count'] == 12)
                before = state()
                desktop.click(window,state()['guide_points'][3],1,1)
                scattered = revision(before['revision'])
                assert len(scattered['document']['plants']) == 14
                assert scattered['history'][0] == before['history'][0]+1
                previous = state()['revision']
                chord('z')
                assert len(revision(previous)['document']['plants']) == 2
                previous = state()['revision']
                chord('y')
                assert revision(previous)['document'] == scattered['document']
                agent_plant = dict(shrub,id=2000,name='Agent shrub')
                receipt, request = agent_request(dict(command='apply',expected_revision=state()['revision'],edits=[dict(op='put_plant',plant=agent_plant)]))
                assert receipt['accepted']
                wait(lambda: len(state()['document']['plants']) == 15)
                previous = state()['revision']
                chord('z')
                undone = revision(previous)
                retry, _ = agent_request(request=request)
                assert retry == receipt and state()['document'] == undone['document']
                button('Plant(Shrub, false)')
                previous = state()['revision']
                desktop.click(window,state()['guide_points'][0],1,1)
                assert revision(previous)['document']['plants'][-1]['id'] == 2001
                previous = state()['revision']
                chord('z')
                revision(previous)
                button('SelectPlant')
                chord('s')
                wait(lambda: read_saved() == state()['document'])
                stored = state()['document']
                previous = state()['revision']
                chord('o')
                assert revision(previous)['document'] == stored
                assert state()['mesh_assets'] == mesh_baseline
                key('F5')
                wait(lambda: state()['play_feet'] is not None)
                time.sleep(.3)
                key('F5')
                wait(lambda: state()['play_feet'] is None)
                assert state()['document'] == stored
                checks.append('vegetation pointer placement/selection/transforms, bounded deterministic scatter/one-entry undo, agent retry/IDs, save/load/playtest and shared meshes')
            if grading:
                key('F3')
                wait(lambda: state()['path_mode'])
                button('NextPath')
                wait(lambda: state()['path_selected'] == state()['document']['paths'][0]['id'])
                previous = state()['revision']
                key('Page_Up')
                elevated = revision(previous)
                previous = state()['revision']
                key('Page_Up')
                elevated = revision(previous)
                before = state()
                button('GradePath')
                graded = revision(before['revision'])
                assert graded['document']['paths'] == before['document']['paths']
                assert graded['document']['terrain']['heights'] != before['document']['terrain']['heights']
                assert graded['history'][0] == before['history'][0]+1
                previous = state()['revision']
                chord('z')
                assert revision(previous)['document'] == before['document']
                previous = state()['revision']
                chord('y')
                assert revision(previous)['document'] == graded['document']
                # Exercise the same operation from a live agent, with an explicit wider blend.
                receipt, request = agent_request(dict(command='apply',expected_revision=state()['revision'],edits=[dict(op='grade_path',grade=dict(path_id=state()['path_selected'],surface_offset_metres=-.03,falloff_metres=8.,blend=.5))]))
                assert receipt['accepted']
                changed = wait(lambda: state() if state()['revision'] == receipt['revision'] else None)
                assert changed['document']['terrain']['heights'] != graded['document']['terrain']['heights']
                previous = state()['revision']
                chord('z')
                undone = revision(previous)
                assert undone['document'] == graded['document']
                retry, _ = agent_request(request=request)
                assert retry == receipt and state()['document'] == undone['document']
                chord('s')
                wait(lambda: read_saved() == state()['document'])
                previous = state()['revision']
                chord('o')
                assert revision(previous)['document'] == graded['document']
                key('F5')
                wait(lambda: state()['play_feet'] is not None)
                time.sleep(.3)
                key('F5')
                wait(lambda: state()['play_feet'] is None)
                checks.append('selected guide terrain grading, single-entry visual undo/redo, mixed agent grade/exact retry, save/load and playtest')
            key('F12')
            wait(lambda: capture_ready(report.with_suffix('.png')), 30)
            checks.append('actual native Vulkan screenshot captured')
            final = state()
            summary = dict(checks=checks, gpu=final['gpu'], viewport=final['viewport'],
                           revision=final['revision'], history=final['history'],
                           scope='Linux synthetic OS input; offline editor and collision sandbox; no multiplayer or Windows evidence')
            (output/'qualification.json').write_text(json.dumps(summary, indent=2)+'\n')
            subprocess.run(['wmctrl', '-ic', hex(window)], check=True)
            process.wait(timeout=20)
            assert process.returncode == 0
        finally:
            if desktop:
                desktop.close()
            if process.poll() is None:
                process.terminate()
                process.wait(timeout=15)
    summary['validation_layer_active'] = 'Insert instance layer \"VK_LAYER_KHRONOS_validation\"' in (output/'client.log').read_text()
    assert summary['validation_layer_active'], 'Vulkan validation layer activation was not observed'
    summary['allowed_resize_diagnostics'] = check_gpu_diagnostics((output/'client.log').read_text(), allow_surface_resize=bool(size))
    (output/'qualification.json').write_text(json.dumps(summary, indent=2)+'\n')
    print(json.dumps(summary, indent=2))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--binary', type=Path, default=Path('target/debug/theia-zone-editor'))
    parser.add_argument('--size', choices=['3440x1440'], default='3440x1440')
    parser.add_argument('--agent', action='store_true')
    parser.add_argument('--terrain', action='store_true')
    parser.add_argument('--guides', action='store_true')
    parser.add_argument('--markers', action='store_true')
    parser.add_argument('--vegetation', action='store_true')
    parser.add_argument('--grading', action='store_true')
    args = parser.parse_args()
    run(args.output, args.binary, args.size, args.agent, args.terrain, args.guides, args.markers, args.vegetation, args.grading)
