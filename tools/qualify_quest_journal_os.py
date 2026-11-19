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
from qualify_world_input_os import Desktop, wait, owned_window, capture_ready

FIRST = 'theia.dev.quest-introduction'
SECOND = 'theia.dev.quest-follow-up'
WOLVES = SECOND + '.objective.wolves'


def flow(desktop, window, state, folder, resumed=False, repeat_interaction=False, minimap=False, reach_areas=False):
    scale = state()['window'][2]
    base_head = 13 if repeat_interaction else 11
    expected_head = base_head + (3 if reach_areas else 0)
    expected_xp = 150 if reach_areas else 125

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
        if state()['dialogue_open']:
            desktop.key('Escape')
            wait(lambda: not state()['dialogue_open'] and not state().get('dialogue_pending', False) and not any(state().get('dialogue_ui', {}).get(key, False) for key in ['open', 'pending', 'close_when_open']))
        if state()['inventory_open']:
            desktop.key('Escape')
            wait(lambda: not state()['inventory_open'])

    def speak():
        if not state()['dialogue_open']:
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
        wanted = 'theia.dev.interaction.'+name
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

    def approach(point):
        for _ in range(45):
            target = point()
            face(target)
            distance = math.hypot(target[0]-feet()[0], target[2]-feet()[2])
            if distance < 2.1:
                return
            desktop.key_state('w', True)
            try:
                time.sleep(min(.35, max(.02, (distance-1.9)/5)))
            finally:
                desktop.key_state('w', False)
            time.sleep(.15)
        raise AssertionError('Normal movement did not reach the contact')

    wait(lambda: journal() and journal()['tracker'] and journal()['tracker']['glyphs'] > 0)
    if resumed:
        assert all(quest(q)['status'] == 'Completed' for q in [FIRST, SECOND])
        assert journal()['revision'] == expected_head and journal()['xp'] == expected_xp
        if reach_areas:
            assert quest('theia.dev.quest-overlook')['status'] == 'Completed'
        assert journal()['ui']['submitted'] == 0
        speak()
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
    action(SECOND, 'Active')
    capture('follow-up-accepted.png')
    close_panels()

    if not state()['sword_equipped']:
        desktop.key('i')
        wait(lambda: state()['inventory_open'])
        desktop.click(window, wait(lambda: state()['sword_slot_screen']), scale, 1)
        desktop.click(window, wait(lambda: state()['equip_screen']), scale, 1)
        wait(lambda: state()['sword_equipped'])
        close_panels()
    initial_deaths = state()['combat']['deaths']
    for count in range(1, 4):
        wait(lambda: state().get('enemy_health') is not None and state()['enemy_health'] > 0, 15)
        approach(lambda: state()['enemy_global'])
        clicks = state()['controls']['enemy_clicks']
        for dx, dy in [(20, -25), (-20, -25), (0, -30), (0, 0)]:
            point = wait(lambda: state()['enemy_screen'])
            desktop.click(window, [point[0]+dx, point[1]+dy], scale, 3)
            if state()['controls']['enemy_clicks'] > clicks:
                break
        assert state()['controls']['enemy_clicks'] > clicks
        deadline = time.monotonic()+60
        while quest(SECOND)['counts'][WOLVES] < count:
            assert time.monotonic() < deadline, 'Normal wolf fight timed out'
            assert state()['combat']['player_deaths'] == 0, 'Quest fixture player died'
            desktop.key('2')
            time.sleep(.25)
        wait(lambda: journal()['xp'] == 25+25*count)
    assert state()['combat']['deaths'] == initial_deaths+3
    capture('three-kills.png')
    approach(lambda: [8004., 0., 8000.])
    speak()
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
                               close_panels, speak, action, capture)
        assert journal()['revision'] == expected_head
        assert journal()['ui']['submitted'] == 6 and journal()['ui']['applied'] == 6
        assert journal()['ui']['rejected'] == 0
    return dict(passed=True, resumed=False, revision=expected_head, xp=expected_xp,
                actions=6 if reach_areas else 4, reach_area=area_proof,
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
        time.sleep(.3)
    finally:
        desktop.key_state('a', False)
    turned = wait(lambda: state() if abs((state()['classic_facing']-old+32768)%65536-32768) > 1000 else None)
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


def run(output, binaries, sizes, production=False, repeat_interaction=False, gameplay_manifest=Path("content/development-gameplay-classic-journal.json"), minimap=False, minimap_traversal=False, reach_areas=False, destroy_items=False):
    if minimap_traversal and not minimap:
        raise ValueError("Map traversal requires --minimap")
    if reach_areas and (minimap or json.loads(gameplay_manifest.read_text())['schema'] != 16):
        raise ValueError('Area flow requires schema 16 and its separate marker assertions')
    output = output.resolve()
    binaries = binaries.resolve()
    output.mkdir(mode=0o700, parents=True, exist_ok=False)
    bundle = output/'content'
    shutil.copytree('content/gameplay', bundle/'gameplay')
    shutil.copytree('content/quests', bundle/'quests')
    shutil.copy2(gameplay_manifest, bundle)
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
    try:
        with Identity(output/'provider', Path('build/identity-sdk'), token_seconds=300) as identity, Cluster(output/'cluster', Path('build/postgres-sdk')) as cluster:
            for index, size in enumerate([*sizes, sizes[0]]):
                resumed = index == len(sizes)
                if resumed:
                    before = cluster.query(query)
                    assert before.returncode == 0, before.stderr
                    (output/'before-restart.txt').write_text(before.stdout)
                    cluster.restart()
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
                    launcher = subprocess.Popen(command, env=env, stdin=subprocess.PIPE, stdout=log, stderr=subprocess.STDOUT, text=True)
                    try:
                        launcher.stdin.write('1\n' if resumed else (('' if index == 0 else 'n\n')+'JournalArden'+chr(65+index)+'\n'))
                        launcher.stdin.flush()
                        def state():
                            assert launcher.poll() is None, 'Owned launcher exited early'
                            return wait(lambda: json.loads((folder/'input-os.json').read_text()), 2)
                        initial = wait(lambda: state() if state()['connected'] and state()['quest_journal'] and state()['guide_screen'] else None, 60)
                        assert initial['window'][:2] == [int(v) for v in size.split('x')]
                        status = Path(f'/proc/{initial["pid"]}/status').read_text()
                        assert int(re.search(r'^PPid:\s+(\d+)', status, re.M)[1]) == launcher.pid
                        window = wait(lambda: owned_window(initial['pid']))
                        desktop.focus(window)
                        time.sleep(1)
                        proof = flow(desktop, window, state, folder, resumed, repeat_interaction, minimap, reach_areas)
                        if minimap:
                            proof['minimap'] = check_minimap(desktop, window, state, folder)
                            if minimap_traversal and resumed:
                                proof['minimap_walk'] = check_minimap_walk(desktop, window, state, folder)
                        if destroy_items:
                            from quest_item_destruction_os import flow as item_flow
                            proof['item_destruction'] = item_flow(desktop, window, state, folder, resumed)
                        proof.update(window=initial['window'], fixture=not production)
                        (folder/'proof.json').write_text(json.dumps(proof, indent=2)+'\n')
                        children = Path(f'/proc/{launcher.pid}/task/{launcher.pid}/children').read_text().split()
                        servers = [int(pid) for pid in children if b'theia-server' in Path(f'/proc/{pid}/cmdline').read_bytes()]
                        assert len(servers) == 1
                        os.kill(servers[0], signal.SIGINT)
                        launcher.wait(timeout=30)
                        results[folder.name] = proof
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
                    assert resumed or (threat['damage_facts'] >= 3 and threat['health_lost'] == 300 and threat['clears'] >= 3), threat
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
                assert all(row.split('|')[1:] == expected for row in rows.stdout.splitlines()), rows.stdout
                (folder/'sql.txt').write_text(rows.stdout)
    finally:
        desktop.close()
    assert identity.server is None and cluster.server is None
    assert not any((output/'cluster'/name).exists() for name in ['password','pgpass'])
    result = dict(passed=True, results=results, owned_services_stopped=True, plaintext_password_files_removed=True,
                  scope='Normal launcher + actual local OIDC, owned PostgreSQL, PID-bound synthetic X11 input and Linux native GPU; no physical hardware/Windows claim',
                  private_combat_fixture=not production, unchanged_quest_catalogue=True, repeated_preaccept_interaction=repeat_interaction,
                  database_process_restart=True, saved_completion_rejoin=True, reach_area=reach_areas, item_destruction=destroy_items)
    (output/'verification.json').write_text(json.dumps(result, indent=2)+'\n')
    return result


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('--binaries', type=Path, required=True)
    parser.add_argument('--size', action='append', dest='sizes')
    parser.add_argument('--production', action='store_true')
    parser.add_argument('--minimap', action='store_true')
    parser.add_argument('--minimap-traversal', action='store_true', help='After restart/rejoin, walk across chunk and render-origin boundaries using normal keyboard movement')
    parser.add_argument('--repeat-interaction', action='store_true')
    parser.add_argument('--destroy-items',action='store_true',help='Check normal split/confirm/cancel/destroy UI and persistence after the quest flow')
    parser.add_argument('--reach-areas', action='store_true', help='Complete the opt-in western overlook using normal movement')
    parser.add_argument('--gameplay-manifest', type=Path, default=Path('content/development-gameplay-classic-journal.json'))
    args = parser.parse_args()
    print(json.dumps(run(args.output, args.binaries, args.sizes or ['1920x1200','3440x1440','1280x720'], args.production, args.repeat_interaction, args.gameplay_manifest, args.minimap, args.minimap_traversal, args.reach_areas, args.destroy_items), indent=2))
