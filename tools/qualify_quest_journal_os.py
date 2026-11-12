#!/usr/bin/env python3
"""Owned normal launcher/OIDC/SQL journal flow using PID-bound X11 input."""
import argparse
import json
import math
import os
from pathlib import Path
import re
import shutil
import signal
import subprocess
import sys
import time
import urllib.parse
from gpu_validation import check_gpu_diagnostics
from private_identity import Identity
from private_postgres import Cluster
from play import fantasy_ui_options
from qualify_world_input_os import Desktop, wait, owned_window, capture_ready

FIRST = 'theia.dev.quest-introduction'
SECOND = 'theia.dev.quest-follow-up'
WOLVES = SECOND + '.objective.wolves'


def flow(desktop, window, state, folder, resumed=False, repeat_interaction=False, minimap=False, reach_areas=False, corpse_loot=False, item_quest=False, quest_only=False, intro_only=False, authored_trail=None, authored_area=None):
    scale = state()['window'][2]
    base_head = 13 if repeat_interaction else 11
    expected_head = base_head + (3 if reach_areas else 0)
    if resumed and item_quest:
        expected_head = 27 if quest_only else 24
    expected_xp = (250 if quest_only else 200) if resumed and item_quest else (150 if reach_areas else 125)

    def journal():
        return state()['quest_journal']

    def quest(name):
        return next((q for q in journal()['quests'] if q['id'] == name), None)

    def capture(name):
        if minimap:
            expected = {
                'quest-offer.png': 0, 'quest-accepted.png': 1,
                'quest-ready.png': 2, 'follow-up-accepted.png': 1,
                'follow-up-ready.png': 2, 'completed-journal.png': 3,
                'resumed-completed-journal.png': 3,
            }.get(name)
            if expected is not None:
                # The normal guide also opens the vendor backpack. Keep its
                # behavior intact; deliberately close that local pane to inspect
                # the map beside the still-open quest dialogue.
                if state()['inventory_open']:
                    wait(lambda: not state()['minimap']['visible'])
                    desktop.key('i')
                    wait(lambda: not state()['inventory_open'])
                observed = wait(lambda: state()['minimap'] if state()['minimap'] and
                                state()['minimap']['visible'] and
                                state()['minimap']['guide_phase'] == expected else None)
                (folder/(name+'.minimap.json')).write_text(json.dumps(observed, indent=2)+'\n')
        (folder/'capture.request').write_text(name+'\n')
        wait(lambda: capture_ready(folder/'input-os.png'))
        (folder/'input-os.png').rename(folder/name)

    def close_panels():
        if (state().get('loot') or {}).get('ui',{}).get('open'):
            desktop.key('Escape')
            wait(lambda:not state()['loot']['ui']['open'])
        if state()['dialogue_open']:
            desktop.key('Escape')
            wait(lambda: not state()['dialogue_open'] and not state().get('dialogue_pending', False) and not any(state().get('dialogue_ui', {}).get(key, False) for key in ['open', 'pending', 'close_when_open']))
        if state()['inventory_open']:
            desktop.key('Escape')
            wait(lambda: not state()['inventory_open'])

    def speak():
        if not state()['dialogue_open']:
            close_panels()
            desktop.click(window, wait(lambda: state()['guide_screen']), scale, 3)
            wait(lambda: state()['dialogue_open'])

    def action(name, status):
        before = journal()['ui']['applied']
        point = wait(lambda: next((b['point'] for b in journal()['buttons'] if b['quest'] == name), None))
        desktop.click(window, point, scale, 1)
        wait(lambda: quest(name) and quest(name)['status'] == status and journal()['ui']['applied'] == before+1 and not journal()['ui']['pending'])

    def ask(name):
        if not journal()['choices']:
            close_panels()
            speak()
        # The small-window dialogue body scrolls. Its last choice can have a
        # computed centre outside the clipping rectangle; expose it with real
        # wheel input before reading/clicking the settled painted location.
        wanted = 'theia.dev.interaction.'+name
        # Wait for this reopened dialogue's actual branch before scrolling;
        # its presentation resets scroll when the branch arrives.
        wait(lambda: any(c['choice']==wanted for c in journal()['choices']))
        time.sleep(.2)
        if name=='training':(folder/'dialogue-training-before.json').write_text(json.dumps(journal(),indent=2)+'\n')
        width, height, _ = state()['window']
        desktop.move(window, wait(lambda:journal()['dialogue_body']['point']), scale)
        for _ in range(4):
            desktop.scroll(-1 if name == 'training' else 1)
        time.sleep(.2)
        if name=='training':(folder/'dialogue-training-after.json').write_text(json.dumps(journal(),indent=2)+'\n')
        point = wait(lambda: next((c['point'] for c in journal()['choices'] if c['choice'] == wanted), None))
        desktop.click(window, point, scale, 1)
        wait(lambda: not journal()['choices'])

    def feet():
        return state()['dodge']['authority_feet']

    def face(point):
        player = feet()
        yaw = math.atan2(-(point[0]-player[0]), -(point[2]-player[2]))
        for _ in range(6):
            error = math.atan2(math.sin(yaw-state()['classic_facing']*math.tau/65536), math.cos(yaw-state()['classic_facing']*math.tau/65536))
            if abs(error) < .05:
                return
            width, height, _ = state()['window']
            desktop.move(window, [width*.55, height*.42], scale)
            desktop.button(3, True)
            try:
                error = math.atan2(math.sin(yaw-state()['camera_angles'][0]), math.cos(yaw-state()['camera_angles'][0]))
                desktop.look(round(max(-400, min(400, -error/.004))), 0)
            finally:
                desktop.button(3, False)
            time.sleep(.2)
        raise AssertionError('Manual facing did not reach the requested direction')

    def approach(point, tolerance=2.1):
        for _ in range(45):
            target = point()
            face(target)
            distance = math.hypot(target[0]-feet()[0], target[2]-feet()[2])
            if distance < tolerance:
                return
            desktop.key_state('w', True)
            try:
                time.sleep(min(.35, max(.02, (distance-max(.1, tolerance-.2))/5)))
            finally:
                desktop.key_state('w', False)
            time.sleep(.15)
        raise AssertionError('Normal movement did not reach the contact')

    travel_count = 0

    def walk_trail(reverse=False):
        nonlocal travel_count
        travel_count += 1
        points = list(reversed(authored_trail)) if reverse else authored_trail
        observations = []
        for point in points:
            approach(lambda: point, tolerance=.6)
            observations.append(dict(requested=point, feet=feet(), tick=state()['dodge']['tick']))
        (folder / f'trail-{travel_count}-{"return" if reverse else "outward"}.json').write_text(
            json.dumps(observations, indent=2)+'\n')

    def engage():
        clicks = state()['controls']['enemy_clicks']
        for dx, dy in [(20, -25), (-20, -25), (0, -30), (0, 0)]:
            point = wait(lambda: state()['enemy_screen'])
            desktop.click(window, [point[0]+dx, point[1]+dy], scale, 3)
            try:
                wait(lambda:state()['controls']['enemy_clicks']>clicks,.75)
                break
            except TimeoutError:
                pass
        if state()['controls']['enemy_clicks']==clicks:
            # An earlier corpse can be the actual frontmost pointer hit.
            # Close that ordinary pane and use the supported Tab target cycle.
            if (state().get('loot') or {}).get('ui',{}).get('open'):
                desktop.key('Escape');wait(lambda:not state()['loot']['ui']['open'])
            if state()['dialogue_open']:
                close_panels()
            desktop.key('Tab')
            wait(lambda:state()['selected_hostile'] is not None)
    def await_respawn():
        wait(lambda: state()['combat']['player_health'] > 0 and
             math.hypot(feet()[0]-authored_trail[0][0], feet()[2]-authored_trail[0][2]) < 3, 30)
        assert state()['combat']['player_deaths'] <= 3, 'Bounded quest test death budget exceeded'

    wait(lambda: journal() and journal()['tracker'] and journal()['tracker']['glyphs'] > 0)
    if resumed and intro_only:
        assert quest(FIRST)['status'] == 'Completed' and quest(SECOND) is None
        assert journal()['xp'] == 25 and journal()['ui']['submitted'] == 0
        capture('intro-restored.png')
        speak()
        assert not any(button.get('quest') == FIRST for button in journal()['buttons'])
        capture('intro-resumed.png')
        return dict(passed=True, resumed=True, revision=journal()['revision'], xp=journal()['xp'])
    if resumed:
        assert all(quest(q)['status'] == 'Completed' for q in [FIRST, SECOND])
        assert journal()['revision'] == expected_head and journal()['xp'] == expected_xp
        if reach_areas:
            assert quest('theia.dev.quest-overlook')['status'] == 'Completed'
        assert journal()['ui']['submitted'] == 0
        if corpse_loot or item_quest:
            # Pickup legitimately changed the saved logout position.
            approach(lambda: [8004., 0., 8000.])
        speak()
        if authored_trail and not reach_areas:
            assert not any(b.get('quest') in [FIRST, SECOND] for b in journal()['buttons'])
        else:
            assert not journal()['buttons'], 'Completed quests were offered again'
            assert not state()['quest_marker']['visible_now']
        capture('resumed-completed-journal.png')
        return dict(passed=True, resumed=True, revision=expected_head, xp=expected_xp)

    assert journal()['revision'] == 0 and not journal()['quests']
    wait(lambda: state()['quest_marker']['visible_now'])
    capture('quest-offer.png')
    if repeat_interaction:
        for receipt in [1, 2]:
            speak()
            ask('road')
            wait(lambda: journal()['revision'] == receipt)
            assert not journal()['quests'] and journal()['xp'] == 0
            capture(f'preaccept-repeat-{receipt}.png')
    speak()
    action(FIRST, 'Active')
    capture('quest-accepted.png')
    for name in ['road', 'supplies', 'report']:
        ask(name)
    wait(lambda: quest(FIRST)['status'] == 'Ready')
    capture('quest-ready.png')
    action(FIRST, 'Completed')
    wait(lambda: journal()['xp'] == 25)
    if intro_only:
        capture('intro-completed.png')
        return dict(passed=True, resumed=False, revision=journal()['revision'], xp=journal()['xp'])
    action(SECOND, 'Active')
    capture('follow-up-accepted.png')
    if item_quest:
        ask('training')
    close_panels()

    if not state()['sword_equipped']:
        desktop.key('i')
        wait(lambda: state()['inventory_open'])
        desktop.click(window, wait(lambda: state()['sword_slot_screen']), scale, 1)
        desktop.click(window, wait(lambda: state()['equip_screen']), scale, 1)
        wait(lambda: state()['sword_equipped'])
        close_panels()
    if authored_trail:
        walk_trail()
    initial_deaths = state()['combat']['deaths']
    for count in range(1, 4):
        wait(lambda: state().get('enemy_health') is not None and state()['enemy_health'] > 0, 15)
        approach(lambda: state()['enemy_global'])
        engage()
        deadline = time.monotonic()+60
        while quest(SECOND)['counts'][WOLVES] < count:
            assert time.monotonic() < deadline, 'Normal wolf fight timed out'
            if authored_trail and state()['combat']['player_health'] == 0:
                await_respawn()
                walk_trail()
                approach(lambda: state()['enemy_global'])
                engage()
                deadline = time.monotonic()+60
            elif not authored_trail:
                assert state()['combat']['player_deaths'] == 0, 'Quest fixture player died'
            desktop.key('2')
            time.sleep(.25)
        wait(lambda: journal()['xp'] == 25+25*count)
    assert state()['combat']['deaths'] == initial_deaths+3
    capture('three-kills.png')
    returned_via_respawn = False
    if authored_trail:
        if state()['combat']['player_health'] == 0 or (state()['combat']['player_deaths'] > 0 and
                math.hypot(feet()[0]-authored_trail[0][0], feet()[2]-authored_trail[0][2]) < 3):
            await_respawn()
            returned_via_respawn = True
            capture('respawn-after-combat.png')
        else:
            walk_trail(reverse=True)
    else:
        approach(lambda: [8004., 0., 8000.])
    speak()
    if not item_quest:
        ask('training')
    wait(lambda: quest(SECOND)['status'] == 'Ready')
    capture('follow-up-ready.png')
    action(SECOND, 'Completed')
    wait(lambda: journal()['xp'] == 125 and journal()['revision'] == base_head)
    assert journal()['ui']['submitted'] == 4 and journal()['ui']['applied'] == 4
    assert journal()['ui']['rejected'] == 0
    assert state()['combat']['event_gaps'] == 0
    capture('completed-journal.png')
    area_proof = None
    if reach_areas:
        from quest_reach_area_os import flow as area_flow
        area_proof = area_flow(desktop, state, folder, quest, journal, feet, face,
                               close_panels, speak, action, capture,
                               area=authored_area, travel=walk_trail if authored_trail else None)
        assert journal()['revision'] == expected_head
        assert journal()['ui']['submitted'] == 6 and journal()['ui']['applied'] == 6
        assert journal()['ui']['rejected'] == 0
    return dict(passed=True, resumed=False, revision=expected_head, xp=expected_xp,
                actions=6 if reach_areas else 4, reach_area=area_proof,
                player_deaths=state()['combat']['player_deaths'], returned_via_respawn=returned_via_respawn,
                semantic_choices=4, preaccept_noop_choices=2 if repeat_interaction else 0, kills=3, ui=journal()['ui'], adapter=state()['adapter'])



def check_minimap(desktop, window, state, folder):
    if state()['dialogue_open']:
        desktop.key('Escape')
        wait(lambda: not state()['dialogue_open'])
    if state()['inventory_open']:
        desktop.key('i')
        wait(lambda: not state()['inventory_open'])
    first = wait(lambda: state() if state()['minimap'] and
                 state()['minimap']['visible'] and state()['minimap']['bounds'] and
                 state()['minimap']['known_pixels'] == 4096 and
                 state()['minimap_streaming']['loaded'] == 81 else None)
    width, height, scale = first['window']
    left, top, map_width, map_height = first['minimap']['bounds']
    assert 0 <= left < left+map_width <= width
    assert 0 <= top < top+map_height <= height
    assert first['minimap']['texture_bytes'] == 17408
    assert first['minimap']['resident_tiles_read'] <= 81
    assert first['minimap']['guide_phase'] == 3
    # Stationary presentation may sample resident grids, but must not read any
    # saved tiles, create more terrain roots or request another collision patch.
    before_streaming = first['minimap_streaming']
    before_updates = first['minimap']['raster_updates']
    time.sleep(4)
    stable = state()
    updates = stable['minimap']['raster_updates']-before_updates
    assert 12 <= updates <= 18, updates
    assert stable['minimap_streaming'] == before_streaming
    assert stable['minimap']['max_raster_us'] <= 5000, stable['minimap']
    point = [left+map_width/2, top+map_height/2]
    desktop.move(window, point, scale)
    before = state()
    desktop.click(window, point, scale, 1)
    desktop.drag(window, point, scale)
    desktop.move(window, point, scale)
    desktop.button(4, True)
    desktop.button(4, False)
    time.sleep(.4)
    after = state()
    for counter in ['enemy_clicks', 'friendly_clicks', 'basic_requests', 'interaction_requests']:
        assert after['controls'][counter] == before['controls'][counter], counter
    assert math.dist(after['camera_angles'], before['camera_angles']) < .001
    assert abs(after['camera_distance']-before['camera_distance']) < .001
    assert after['cursor']['visible'] and after['cursor']['grab'] == 'None'
    desktop.key('i')
    wait(lambda: state()['inventory_open'] and not state()['minimap']['visible'])
    (folder/'capture.request').write_text('minimap-backpack.png\n')
    wait(lambda: capture_ready(folder/'input-os.png'))
    (folder/'input-os.png').rename(folder/'minimap-backpack.png')
    desktop.key('i')
    wait(lambda: not state()['inventory_open'] and state()['minimap']['visible'])
    old = state()['classic_facing']
    try:
        desktop.key_state('a', True)
        # Keep the real key held until an authoritative turn is observed. A
        # fixed short tap can fall between input samples during a debug/GPU
        # hitch; waiting only after release cannot recover that missed sample.
        turned = wait(lambda: state() if abs((state()['classic_facing']-old+32768)%65536-32768) > 1000 else None, 3)
    finally:
        desktop.key_state('a', False)
    def agrees():
        s = state()
        radians = s['classic_facing']*math.tau/65536
        arrow = s['minimap']['arrow_forward']
        return s if arrow and math.dist(arrow, [-math.sin(radians), -math.cos(radians)]) < .02 else None
    turned = wait(agrees)
    (folder/'capture.request').write_text('minimap-controls.png\n')
    wait(lambda: capture_ready(folder/'input-os.png'))
    (folder/'input-os.png').rename(folder/'minimap-controls.png')
    proof = dict(passed=True, bounds=first['minimap']['bounds'],
                 known_pixels=first['minimap']['known_pixels'], texture_bytes=17408,
                 raster_updates_over_four_seconds=updates,
                 max_raster_us=stable['minimap']['max_raster_us'],
                 max_present_us=stable['minimap']['max_present_us'],
                 mean_present_us=(stable['minimap']['total_present_us']-first['minimap']['total_present_us'])/max(1, stable['minimap']['present_samples']-first['minimap']['present_samples']),
                 streaming_unchanged_during_idle=True, streaming=before_streaming,
                 pointer_and_wheel_owned_by_ui=True, backpack_accessible=True,
                 actual_arrow_forward=turned['minimap']['arrow_forward'],
                 authoritative_facing=turned['classic_facing'])
    (folder/'minimap-proof.json').write_text(json.dumps(proof, indent=2)+'\n')
    return proof



def check_minimap_walk(desktop, window, state, folder):
    start = state()
    start_position = start['dodge']['authority_feet']
    start_revision = start['render_frame']['revision']
    chunks = set()
    samples = []
    maximum_error = 0.
    began = time.monotonic()
    try:
        desktop.key_state('w', True)
        while time.monotonic()-began < 90:
            desktop.owned_focus()
            current = state()
            assert current['connected']
            position = current['dodge']['authority_feet']
            chunks.add((math.floor(position[0]/128), math.floor(position[2]/128)))
            m = current['minimap']
            assert m['visible'] and m['texture_bytes'] == 17408
            assert m['resident_tiles_read'] <= 81 and 0 <= m['known_pixels'] <= 4096
            centre, point = m['centre'], m['player_point']
            projected = [centre[0]+(point[0]/176-.5)*256,
                         centre[2]+(point[1]/176-.5)*256]
            if current['rendered_body'] is not None:
                rendered = current['rendered_body']
                error = math.dist(projected, [rendered[0], rendered[2]])
                maximum_error = max(maximum_error, error)
                assert error <= .003, (error, current['render_frame'])
            if not samples or time.monotonic()-samples[-1]['seconds_absolute'] >= 1:
                samples.append(dict(seconds_absolute=time.monotonic(), position=position,
                                    centre=centre, player_point=point,
                                    render_frame=current['render_frame'], streaming=current['minimap_streaming']))
            distance = math.dist([position[0],position[2]], [start_position[0],start_position[2]])
            if len(chunks) >= 3 and current['render_frame']['revision'] > start_revision and distance >= 160:
                break
            time.sleep(.1)
        else:
            raise AssertionError('Normal movement did not cross three chunk owners and a render-origin boundary')
    finally:
        desktop.key_state('w', False)
    wait(lambda: state()['movement']['direction'] == [0., 0.])
    end = state()
    assert end['minimap_streaming']['unloaded'] > start['minimap_streaming']['unloaded']
    assert end['quest_journal']['revision'] == start['quest_journal']['revision'] == 13
    assert end['quest_journal']['xp'] == start['quest_journal']['xp'] == 125
    assert end['minimap']['guide_point'] is None, 'Distant guide retained a local map icon'
    (folder/'capture.request').write_text('minimap-chunk-rebase.png\n')
    wait(lambda: capture_ready(folder/'input-os.png'))
    (folder/'input-os.png').rename(folder/'minimap-chunk-rebase.png')
    for sample in samples:
        sample['seconds'] = sample.pop('seconds_absolute')-began
    proof = dict(passed=True, normal_speed=True, synthetic_os_keyboard=True,
                 elapsed_seconds=time.monotonic()-began, chunks=sorted(chunks),
                 start_position=start_position, end_position=end['dodge']['authority_feet'],
                 start_render_revision=start_revision, end_render_frame=end['render_frame'],
                 max_map_position_error_metres=maximum_error,
                 start_streaming=start['minimap_streaming'], end_streaming=end['minimap_streaming'],
                 far_guide_filtered=True, quest_rewards_unchanged=True, samples=samples)
    (folder/'minimap-walk-proof.json').write_text(json.dumps(proof, indent=2)+'\n')
    return proof


def run(output, binaries, sizes, production=False, repeat_interaction=False, gameplay_manifest=Path("content/development-gameplay-classic-journal.json"), minimap=False, minimap_traversal=False, reach_areas=False, destroy_items=False, corpse_loot=False, item_quest=False, documents=False, item_rewards=False, reward_capacity=False, quest_offers=False, interface_exit=False, fantasy_ui=None, ui_scale=None, ui_preferences=None):
    abilities = gameplay_manifest.parent/json.loads(gameplay_manifest.read_text())['abilities'] if fantasy_ui else None
    ui_options = fantasy_ui_options(fantasy_ui, True, abilities, ui_scale, ui_preferences)
    if quest_offers and (not documents or not item_rewards or reward_capacity):
        raise ValueError('Quest offer fixture requires the separate normal fixed-reward document flow')
    if reward_capacity and not item_rewards:
        raise ValueError('Capacity fixture requires fixed rewards')
    if item_rewards and not documents:
        raise ValueError('Fixed reward walkthrough requires the document flow')
    if documents:
        manifest=json.loads(gameplay_manifest.read_text())
        if any((production,repeat_interaction,minimap,minimap_traversal,reach_areas,destroy_items,corpse_loot,item_quest)) or manifest['schema']!=(21 if item_rewards else 20):
            raise ValueError('Choose the separate guaranteed document fixture')
        if any(r.get('chance_basis_points')!=10000 for r in json.loads((gameplay_manifest.parent/manifest['rewards']).read_text())['rewards'] if r['scope']=='per_creature_life'):
            raise ValueError('Document walkthrough requires guaranteed per-life fixture chances')
    quest_only = item_quest and json.loads(gameplay_manifest.read_text())['schema']==19
    if item_quest:
        manifest=json.loads(gameplay_manifest.read_text())
        if not reach_areas or corpse_loot or destroy_items or minimap or manifest['schema'] not in (18,19) or json.loads((gameplay_manifest.parent/manifest['rewards']).read_text())['schema']!=(4 if quest_only else 2):
            raise ValueError('Item quest walkthrough requires a schema-18/19 guaranteed-loot fixture and area regression')
        if quest_only and any(r.get('chance_basis_points')!=10000 for r in json.loads((gameplay_manifest.parent/manifest['rewards']).read_text())['rewards'] if r['scope']=='per_creature_life'):
            raise ValueError('Conditional walkthrough requires explicit guaranteed per-life fixture chances')
    if corpse_loot and (not reach_areas or destroy_items or minimap or json.loads(gameplay_manifest.read_text())["schema"] != 17):
        raise ValueError("Corpse loot requires schema 17, area regression and separate inventory checks")
    if minimap_traversal and not minimap:
        raise ValueError("Map traversal requires --minimap")
    if reach_areas and (minimap or json.loads(gameplay_manifest.read_text())['schema'] not in ((18,19) if item_quest else ((16,17) if corpse_loot else (16,)))):
        raise ValueError('Area flow requires schema 16 and its separate marker assertions')
    output = output.resolve()
    binaries = binaries.resolve()
    output.mkdir(mode=0o700, parents=True, exist_ok=False)
    bundle = output/'content'
    shutil.copytree('content/gameplay', bundle/'gameplay')
    shutil.copytree('content/quests', bundle/'quests')
    shutil.copy2(gameplay_manifest, bundle)
    if quest_offers:
        # Only this owned UI fixture changes prerequisites and adds offers.
        path=bundle/json.loads(gameplay_manifest.read_text())['quest']
        if not path.resolve().is_relative_to(bundle.resolve()):
            raise ValueError('Quest fixture must stay inside its owned content copy')
        quests=json.loads(path.read_text())
        for quest in quests['quests']:quest['prerequisites']=[]
        remaining=32-len(quests['quests'])
        assert remaining>0
        for index in range(remaining):
            key='theia.dev.zz-offer-tail' if index==remaining-1 else f'theia.dev.offer-fixture-{index:02}'
            quests['quests'].append(dict(id=key,version=1,title=f'Offer fixture {index:02}',description='Owned bounded-menu fixture; ordinary quest content is unchanged.',
                prerequisites=[],giver='theia.dev.guide',turn_in='theia.dev.guide',repeat_policy='once',credit_scope='character',experience=1,
                start_stage=key+'.stage',stages=[dict(id=key+'.stage',completion=dict(kind='all'),next=None,
                objectives=[dict(kind='npc_interaction',id=key+'.objective',label='Ask about the road',npc='theia.dev.guide',interaction='theia.dev.interaction.road')])]))
        path.write_text(json.dumps(quests)+'\n')
    if reward_capacity:
        # Deliberately add two nonstackable rewards to this owned fixture only.
        # Normal UI splitting will fill the backpack; consuming one note cannot
        # make both swords fit. No database or client state is injected.
        reward_path=bundle/json.loads(gameplay_manifest.read_text())['quest_item_rewards']
        if not reward_path.resolve().is_relative_to(bundle.resolve()):
            raise ValueError('Reward fixture must stay inside its owned content copy')
        rewards=json.loads(reward_path.read_text())
        recipe=next(r for r in rewards['rewards'] if r['quest']=='theia.dev.quest-roadside-orders')
        recipe['grants'].append(dict(item='theia.dev.training-sword',count=2))
        reward_path.write_text(json.dumps(rewards)+'\n')
    if not production:
        path = bundle/'gameplay/development-creature-deflect-pursuit.json'
        creature = json.loads(path.read_text())
        creature['aggro'].update(perception_metres=1.0, chase_speed=0.1)
        creature['speed_metres_per_second'] = 0.1
        path.write_text(json.dumps(creature)+'\n')
        path = bundle/'gameplay/development-creature-retaliation.json'
        policy = json.loads(path.read_text())
        policy['damage'] = 1
        path.write_text(json.dumps(policy)+'\n')
        if quest_only or documents:
            path=bundle/'gameplay/development-physical-damage-definitions.json'
            physical=json.loads(path.read_text())
            for attack in physical['attacks']:
                if attack['kind']=='creature':attack.update(minimum=1,maximum=1)
            path.write_text(json.dumps(physical)+'\n')

    os.environ.setdefault('DISPLAY', ':0')
    desktop = Desktop()
    results = {}
    launcher = None
    query = '''SELECT n.display_name,h.revision,p.experience,
        (SELECT count(*) FROM theia.quest_journal_entries q WHERE q.realm_id=n.realm_id AND q.character_id=n.character_id AND q.status=2),
        (SELECT count(*) FROM theia.quest_journal_actions a WHERE a.realm_id=n.realm_id AND a.character_id=n.character_id),
        (SELECT count(*) FROM theia.quest_journal_events e WHERE e.realm_id=n.realm_id AND e.character_id=n.character_id)
        FROM theia.character_names n JOIN theia.quest_journal_heads h USING(realm_id,character_id)
        JOIN theia.progression p USING(realm_id,character_id) ORDER BY n.display_name;'''
    inventory_query = """SELECT n.display_name,b.revision,encode(b.state,'hex')
        FROM theia.character_names n JOIN theia.backpacks b USING(realm_id,character_id)
        ORDER BY n.display_name;"""
    rolls_query = """SELECT n.display_name,r.roll_key,encode(r.victim_life,'hex'),r.ticket,
        r.generation_eligible,r.generation_bag_revision,r.generation_journal_revision,
        encode(r.profile_hash,'hex'),r.corpse_x,r.corpse_y,r.corpse_z
        FROM theia.character_names n JOIN theia.loot_rolls r USING(realm_id,character_id)
        ORDER BY n.display_name,r.roll_key,r.victim_life;"""
    try:
        with Identity(output/'provider', Path('build/identity-sdk'), token_seconds=300) as identity, Cluster(output/'cluster', Path('build/postgres-sdk')) as cluster:
            for index, size in enumerate([*sizes, sizes[0]]):
                resumed = index == len(sizes)
                if resumed:
                    before = cluster.query(query)
                    assert before.returncode == 0, before.stderr
                    (output/'before-restart.txt').write_text(before.stdout)
                    if quest_only or documents:
                        rolls_before=cluster.query(rolls_query)
                        assert rolls_before.returncode==0
                        (output/'rolls-before-restart.txt').write_text(rolls_before.stdout)
                    if item_rewards:
                        bags_before=cluster.query(inventory_query)
                        assert bags_before.returncode==0
                        (output/'bags-before-restart.txt').write_text(bags_before.stdout)
                    cluster.restart()
                    if item_rewards:
                        bags_after=cluster.query(inventory_query)
                        assert bags_after.returncode==0 and bags_after.stdout==bags_before.stdout
                        (output/'bags-after-restart.txt').write_text(bags_after.stdout)
                    if quest_only or documents:
                        rolls_after=cluster.query(rolls_query)
                        assert rolls_after.returncode==0 and rolls_after.stdout==rolls_before.stdout
                        (output/'rolls-after-restart.txt').write_text(rolls_after.stdout)
                    after = cluster.query(query)
                    assert after.returncode == 0 and after.stdout == before.stdout
                    (output/'after-restart.txt').write_text(after.stdout)
                folder = output/('resumed' if resumed else size)
                folder.mkdir()
                env = {k:v for k,v in os.environ.items() if not k.startswith('THEIA_') and k not in ['VK_INSTANCE_LAYERS','VK_LOADER_DEBUG','WGPU_VALIDATION','WGPU_DEBUG']}
                env.update(DISPLAY=':0', XDG_RUNTIME_DIR='/run/user/1000', LD_LIBRARY_PATH=str(Path('build/native-sdk/usr/lib/x86_64-linux-gnu').resolve()),
                           THEIA_DATABASE_URL=cluster.test_environment()['THEIA_TEST_DATABASE_URL'], THEIA_OIDC_ISSUER=identity.issuer, THEIA_OIDC_CLIENT_ID=identity.client_id,
                           THEIA_ACCOUNT_BIND=urllib.parse.urlsplit(identity.redirect).netloc, THEIA_TEST_BROWSER_ISSUER=identity.issuer, THEIA_TEST_BROWSER_REDIRECT=identity.redirect,
                           THEIA_TEST_BROWSER_USER=identity.users[0][0], THEIA_TEST_BROWSER_PASSWORD=identity.users[0][1],
                           BROWSER=f'{sys.executable} {Path("tools/qualify_account_launcher.py").resolve()} --owned-browser %s', THEIA_WORLD_INPUT_OS_QUALIFY=str(folder))
                with (folder/'launcher.log').open('w') as log:
                    command = [sys.executable, 'tools/play.py', '--account-login', '--classic', '--binaries', str(binaries), '--gameplay-manifest', str(bundle/gameplay_manifest.name),
                               '--players', str(binaries/'art/player'), '--creature-asset', str(binaries/'art/wolf/wolf.glb'), '--dev-spawn', '8002,8002', '--guide-position','8004,8000', '--window-size',size]
                    if size == '3440x1440':
                        command.append('--fullscreen')
                    command += ui_options
                    launcher = subprocess.Popen(command, env=env, stdin=subprocess.PIPE, stdout=log, stderr=subprocess.STDOUT, text=True)
                    try:
                        launcher.stdin.write('1\n' if resumed else (('' if index == 0 else 'n\n')+'JournalArden'+chr(65+index)+'\n'))
                        launcher.stdin.flush()
                        def state():
                            assert launcher.poll() is None, 'Owned launcher exited early'
                            return wait(lambda: json.loads((folder/'input-os.json').read_text()), 2)
                        wait(lambda: (folder/'input-os.json').is_file() or launcher.poll() is not None, 60)
                        assert launcher.poll() is None, 'Owned launcher exited before client telemetry; inspect launcher/server logs'
                        initial = wait(lambda: state() if state()['connected'] and state()['quest_journal'] and state()['guide_screen'] else None, 60)
                        status = Path(f'/proc/{initial["pid"]}/status').read_text()
                        assert int(re.search(r'^PPid:\s+(\d+)', status, re.M)[1]) == launcher.pid
                        window = wait(lambda: owned_window(initial['pid']))
                        desktop.focus(window)
                        if size != '3440x1440':
                            # A WM can restore a partially off-screen decorated
                            # frame. Place only this PID-verified client fully
                            # within the desktop before any synthetic input.
                            subprocess.run(['wmctrl','-i','-r',hex(window),'-b','remove,maximized_vert,maximized_horz'],check=True,timeout=3)
                            width,height=[int(v) for v in size.split('x')]
                            subprocess.run(['wmctrl','-i','-r',hex(window),'-e',f'0,40,40,{width},{height}'],check=True,timeout=3)
                        time.sleep(1)
                        wanted=[int(v) for v in size.split('x')]
                        initial=wait(lambda:state() if state()['window'][:2]==wanted else None,20)
                        wait(lambda:state()['cursor'].get('focused',True),20)
                        (folder/'window-geometry.txt').write_text(subprocess.check_output(['xwininfo','-id',hex(window)],text=True))
                        desktop.move(window,[initial['window'][0]*.5,initial['window'][1]*.5],initial['window'][2])
                        time.sleep(.2)
                        if documents:
                            from quest_document_os import flow as document_flow
                            proof=document_flow(desktop,window,state,folder,resumed,item_rewards=item_rewards,reward_capacity=reward_capacity,quest_offers=quest_offers)
                        else:
                            proof = flow(desktop, window, state, folder, resumed, repeat_interaction, minimap, reach_areas, corpse_loot, item_quest, quest_only)
                        if minimap:
                            proof['minimap'] = check_minimap(desktop, window, state, folder)
                            if minimap_traversal and resumed:
                                proof['minimap_walk'] = check_minimap_walk(desktop, window, state, folder)
                        if corpse_loot:
                            from quest_corpse_loot_os import flow as loot_flow
                            proof['corpse_loot'] = loot_flow(desktop, window, state, folder, resumed)
                        if item_quest:
                            from quest_corpse_loot_os import flow as loot_flow
                            proof['item_quest'] = loot_flow(desktop, window, state, folder, resumed, item_quest=True, quest_only=quest_only)
                            proof['baseline'] = {k: proof[k] for k in ['revision','xp','actions','kills','ui'] if k in proof}
                            proof['baseline_revision'] = proof['revision']
                            proof.update(revision=proof['item_quest']['final_revision'], xp=state()['quest_journal']['xp'])
                            if not resumed:
                                proof.update(actions=state()['quest_journal']['ui']['applied'], kills=state()['combat']['deaths'], ui=state()['quest_journal']['ui'])
                        if destroy_items:
                            from quest_item_destruction_os import flow as item_flow
                            proof['item_destruction'] = item_flow(desktop, window, state, folder, resumed)
                        proof.update(window=initial['window'], fixture=not production)
                        if fantasy_ui:
                            proof['interface_scale'] = dict(preferred=initial['interface']['preferred'], displayed=initial['interface']['displayed'])
                        (folder/'proof.json').write_text(json.dumps(proof, indent=2)+'\n')
                        children = Path(f'/proc/{launcher.pid}/task/{launcher.pid}/children').read_text().split()
                        servers = [int(pid) for pid in children if b'theia-server' in Path(f'/proc/{pid}/cmdline').read_bytes()]
                        assert len(servers) == 1
                        if interface_exit:
                            assert state().get('interface'), 'Interface exit requires the opt-in UI'
                            if not state()['interface']['open']:
                                desktop.key('F10')
                            point = wait(lambda: next((b['point'] for b in state()['interface']['buttons']
                                if b['action'] == 'exit' and min(b['point']) > 0), None))
                            (folder/'capture.request').write_text('interface-exit.png\n')
                            wait(lambda: capture_ready(folder/'input-os.png'))
                            (folder/'input-os.png').rename(folder/'interface-exit.png')
                            desktop.click(window, point, state()['window'][2], 1)
                            assert launcher.wait(timeout=30) == 0, 'Interface exit did not cleanly stop the owned launcher'
                            proof['interface_exit'] = dict(clicked=True, launcher_exit_code=0)
                        else:
                            os.kill(servers[0], signal.SIGINT)
                            launcher.wait(timeout=30)
                        results[folder.name] = proof
                    except Exception:
                        # Preserve the actual painted layout before owned cleanup.
                        if launcher.poll() is None and (folder/'input-os.json').is_file():
                            (folder/'capture.request').write_text('failure.png\n')
                            try:
                                wait(lambda: capture_ready(folder/'input-os.png'), 5)
                                (folder/'input-os.png').rename(folder/'failure.png')
                            except (TimeoutError, OSError):
                                pass
                        raise
                    finally:
                        if launcher.poll() is None:
                            launcher.send_signal(signal.SIGINT)
                            try:
                                launcher.wait(timeout=30)
                            except subprocess.TimeoutExpired:
                                launcher.kill()
                                launcher.wait()
                        launcher.stdin.close()
                        launcher = None
                shutil.copy2('build/development-session/server.log', folder/'server.log')
                if 'creature_threat' in json.loads(gameplay_manifest.read_text()):
                    server_log = (folder/'server.log').read_text()
                    assert 'development_creature_threat enabled=true' in server_log
                    summaries = [json.loads(line)['creature_threat_summary'] for line in server_log.splitlines() if line.startswith('{"creature_threat_summary":')]
                    assert len(summaries) == 1
                    threat = summaries[0]
                    kills = 2 if documents else ((7 if quest_only else 4) if item_quest else 3)
                    assert resumed or (threat['damage_facts'] >= kills and threat['health_lost'] == kills*100 and threat['clears'] >= kills), threat
                    proof['threat'] = threat
                proof['gpu_diagnostics'] = check_gpu_diagnostics((folder/'launcher.log').read_text(errors='replace'), allow_surface_resize=size == '3440x1440')
                (folder/'proof.json').write_text(json.dumps(proof, indent=2)+'\n')
                for path in folder.glob('*.log'):
                    text = path.read_text(errors='replace')
                    assert identity.password not in text and cluster.password not in text
                    if path.name.startswith('client'):
                        check_gpu_diagnostics(text, allow_surface_resize=size == '3440x1440')
                rows = cluster.query(query)
                assert rows.returncode == 0
                expected = (['16','150','3','6','10'] if repeat_interaction else ['14','150','3','6','8']) if reach_areas else (['13','125','2','4','9'] if repeat_interaction else ['11','125','2','4','7'])
                if documents:expected=['9','75','1','3','4'] if quest_offers else ['8','75','1','2','4']
                if item_quest:expected=['27','250','4','8','14'] if quest_only else ['24','200','4','8','11']
                assert all(row.split('|')[1:] == expected for row in rows.stdout.splitlines()), rows.stdout
                if quest_only:
                    audit=cluster.query("""SELECT n.display_name,
                        count(*) FILTER(WHERE NOT r.generation_eligible),
                        count(*) FILTER(WHERE r.generation_eligible),
                        count(*) FILTER(WHERE r.generation_bag_revision IS NULL OR r.generation_journal_revision IS NULL)
                        FROM theia.character_names n JOIN theia.loot_rolls r USING(realm_id,character_id)
                        WHERE r.roll_key='entry:theia.dev.wolf-pelt-loot' GROUP BY n.display_name ORDER BY n.display_name;""")
                    assert audit.returncode==0 and audit.stdout.strip()
                    assert all(row.split('|')[1:]==['3','4','0'] for row in audit.stdout.splitlines()),audit.stdout
                    (folder/'generation-rights-sql.txt').write_text(audit.stdout)
                if item_rewards:
                    saved=cluster.query(inventory_query)
                    assert saved.returncode==0 and saved.stdout.strip()
                    for row in saved.stdout.splitlines():
                        name,revision,raw=row.split('|')
                        bag=json.loads(bytes.fromhex(raw))
                        assert bag['revision']==int(revision), name
                        assert sum(slot['count'] for slot in bag['slots'] if slot and slot['item']=='theia.dev.training-token')==103, name
                        assert not any(slot and slot['item']=='theia.dev.roadside-orders' for slot in bag['slots']), name
                        if reward_capacity:
                            assert sum(slot['count'] for slot in bag['slots'] if slot and slot['item']=='theia.dev.training-sword')==2, name
                    (folder/'saved-reward-bags.txt').write_text(saved.stdout)
                (folder/'sql.txt').write_text(rows.stdout)
    finally:
        desktop.close()
    assert identity.server is None and cluster.server is None
    assert desktop.wheel_fd is None
    wheel_name=f'Theia owned qualification wheel {os.getpid()}'
    wait(lambda:wheel_name not in subprocess.check_output(['xinput','list','--name-only'],text=True).splitlines(),5)
    assert not any((output/'cluster'/name).exists() for name in ['password','pgpass'])
    result = dict(passed=True, results=results, owned_services_stopped=True, plaintext_password_files_removed=True,
                  scope='Normal launcher + actual local OIDC, owned PostgreSQL, PID-bound synthetic X11 input and Linux native GPU; no physical hardware/Windows claim',
                  wheel_input='temporary Linux uinput wheel axis; XTEST pointer/keyboard; all guarded by owned X11 focus and pointer checks', virtual_wheel_device_removed=True,
                  private_combat_fixture=not production, unchanged_quest_catalogue=True, repeated_preaccept_interaction=repeat_interaction,
                  database_process_restart=True, saved_completion_rejoin=True, reach_area=reach_areas, item_destruction=destroy_items, corpse_loot=corpse_loot, item_quest=item_quest, guaranteed_loot_fixture=item_quest or documents, quest_only=quest_only, documents=documents, fixed_quest_rewards=item_rewards, fixed_reward_capacity_fixture=reward_capacity, bounded_quest_offers=quest_offers)
    (output/'verification.json').write_text(json.dumps(result, indent=2)+'\n')
    return result


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('--binaries', type=Path, required=True)
    parser.add_argument('--size', action='append', dest='sizes')
    parser.add_argument('--fantasy-ui', type=Path, help='Local licensed HUD directory for the normal launcher')
    parser.add_argument('--ui-scale', type=float, help='Preferred UI size, .75 to 1.5; requires --fantasy-ui')
    parser.add_argument('--ui-preferences', type=Path, help='Opt-in local preference file; requires --fantasy-ui')
    parser.add_argument('--interface-exit', action='store_true', help='Click the actual Interface Exit game button after each flow; requires an enabled UI')
    parser.add_argument('--production', action='store_true')
    parser.add_argument('--minimap', action='store_true')
    parser.add_argument('--minimap-traversal', action='store_true', help='After restart/rejoin, walk across chunk and render-origin boundaries using normal keyboard movement')
    parser.add_argument('--repeat-interaction', action='store_true')
    parser.add_argument('--quest-offers',action='store_true',help='Owned 32-offer catalogue: reach the last offer and accept it before completing the note quest')
    parser.add_argument('--reward-capacity',action='store_true',help='Owned extra-sword reward fixture: fill backpack through split UI, refuse turn-in, free space and retry')
    parser.add_argument('--item-rewards',action='store_true',help='Verify configured fixed rewards on offers, turn-in and rejoin with --documents')
    parser.add_argument('--documents',action='store_true',help='Normal note pickup, reader pages, historical credit, possession loss and delivery')
    parser.add_argument('--item-quest',action='store_true',help='Actual looted-item readiness, split/delete reversal, replacement pickup, turn-in consumption and rejoin')
    parser.add_argument('--corpse-loot',action='store_true',help='Normal corpse selection, view-only, single pickup, serial Loot-all and durable partial-loot rejoin')
    parser.add_argument('--destroy-items',action='store_true',help='Check normal split/confirm/cancel/destroy UI and persistence after the quest flow')
    parser.add_argument('--reach-areas', action='store_true', help='Complete the opt-in western overlook using normal movement')
    parser.add_argument('--gameplay-manifest', type=Path, default=Path('content/development-gameplay-classic-journal.json'))
    args = parser.parse_args()
    if args.interface_exit and args.fantasy_ui is None:
        parser.error('--interface-exit requires --fantasy-ui')
    print(json.dumps(run(args.output, args.binaries, args.sizes or ['1920x1200','3440x1440','1280x720'], args.production, args.repeat_interaction, args.gameplay_manifest, args.minimap, args.minimap_traversal, args.reach_areas, args.destroy_items, args.corpse_loot, args.item_quest, args.documents, args.item_rewards, args.reward_capacity, args.quest_offers, interface_exit=args.interface_exit, fantasy_ui=args.fantasy_ui, ui_scale=args.ui_scale, ui_preferences=args.ui_preferences), indent=2))
