#!/usr/bin/env python3
"""Native Linux graybox qualification on the caller's display, using real Renet.

Only the game's window is captured. Binaries/assets and credentials remain local.
Run without competing builds or another walkthrough for useful frame-time data.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import signal
import subprocess
import tempfile
import time
from process_metrics import Recorder
from udp_impairment import Relay


def stop(process):
    if process.poll() is None:
        process.send_signal(signal.SIGINT)
        try:
            process.wait(timeout=8)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait()


def check_steady_motion(motion, budget):
    assert motion['steady_samples'] > 100, 'Insufficient steady-input observations'
    ratio = motion['steady_held_frames'] / motion['steady_samples']
    assert ratio <= budget, f'Steady held-frame ratio {ratio:.4%} exceeds {budget:.4%}'


def run(output, binaries, players, *, duration=65, clients=1, tour=True, validation=True, speed=32, continent=False, prediction=False, impairment=False, hold_jump=False, saved_terrain=None, uncapped=True, steady_hold_budget=None, targeting=False, combat_catalog=None, feedback=None, action_ui=False, persistence=False, spawn=None, reverse_clients=False, environment=None):
    if persistence and (tour or continent or speed != 1 or not prediction or saved_terrain is None):
        raise ValueError("Persistence qualification requires saved terrain and normal-speed prediction without a tour")
    if action_ui and not combat_catalog:
        raise ValueError("Action UI requires combat")
    if feedback and not combat_catalog:
        raise ValueError("Feedback requires a combat catalog")
    if combat_catalog is not None:
        if tour or continent or duration < 30:
            raise ValueError('Combat qualification requires no tour, no continent route and at least 30 seconds')
        targeting = True
    if targeting and (not prediction or speed != 1):
        raise ValueError("Targeting qualification requires predicted normal-speed movement")
    if steady_hold_budget is not None and (not 0 <= steady_hold_budget <= 1 or speed != 1 or tour or not prediction):
        raise ValueError('A steady hold budget requires prediction, normal speed, no tour and a ratio in 0..1')
    if hold_jump and not (prediction and tour):
        raise ValueError("Held-jump reset qualification requires prediction and the teleport tour")
    output = output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    env = dict(os.environ)
    env.update(environment or {})
    env.update(THEIA_PERFORMANCE_UNCAPPED='1' if uncapped else '0', THEIA_GRAYBOX_AUTOPLAY='1', THEIA_GRAYBOX_AUTOPLAY_SPEED=str(speed), RUST_LOG='info')
    if action_ui:
        env["THEIA_COMBAT_UI_QUALIFY"] = "1"
    else:
        env.pop("THEIA_COMBAT_UI_QUALIFY", None)
    if combat_catalog is not None:
        env["THEIA_COMBAT_QUALIFY"] = "1"
    else:
        env.pop("THEIA_COMBAT_QUALIFY", None)
    if targeting:
        env["THEIA_TARGETING_QUALIFY"] = "1"
    else:
        env.pop("THEIA_TARGETING_QUALIFY", None)
    if validation:
        env.update(VK_INSTANCE_LAYERS='VK_LAYER_KHRONOS_validation', VK_LOADER_DEBUG='layer', WGPU_VALIDATION='1')
    else:
        for key in ['VK_INSTANCE_LAYERS', 'VK_LOADER_DEBUG', 'WGPU_VALIDATION', 'WGPU_DEBUG']:
            env.pop(key, None)
    if continent:
        env["THEIA_GRAYBOX_CONTINENT"] = "1"
    if hold_jump:
        env["THEIA_GRAYBOX_HOLD_JUMP_RESET"] = "1"
    else:
        env.pop("THEIA_GRAYBOX_HOLD_JUMP_RESET", None)
    combat_options = ['--combat-catalog', str(combat_catalog)] if combat_catalog else []
    terrain_options = ['--saved-terrain', str(saved_terrain)] if saved_terrain else ['--graybox-world']
    processes = []
    logs = []
    relays = [Relay() for _ in range(clients)] if impairment else []
    capture = None
    try:
        if feedback:
            from private_audio import Capture
            capture = Capture(output)
            env.update(capture.start())
            env["THEIA_FEEDBACK_QUALIFY"] = "1"
        with tempfile.TemporaryDirectory(prefix='tickets-', dir=output) as tickets:
            server_log = (output/'server.log').open('w')
            logs.append(server_log)
            server = subprocess.Popen([str(binaries/'theia-server'), '127.0.0.1:0', str(output/'unused.der'),
                *(["--dev-persistence"] if persistence else []), *terrain_options, *combat_options, *(['--targeting'] if targeting else []), *(['--prediction'] if prediction else []), '--dev-speed', str(speed), '--dev-spawn', spawn or ('10,8000' if continent else ('8058,8000' if combat_catalog else '8000,8000')), '--renet-tickets', tickets,
                *(['--dev-tour'] if tour else []), *(['--dev-relays', ','.join(f'{r.address[0]}:{r.address[1]}' for r in relays)] if relays else [])], env=env, stdout=server_log, stderr=subprocess.STDOUT)
            processes.append(server)
            deadline = time.monotonic()+30
            while time.monotonic() < deadline:
                assert server.poll() is None, 'Server failed; inspect server.log'
                match = re.search(r'listening=(\S+) local_tickets=', (output/'server.log').read_text())
                if match:
                    address = match[1]
                    break
                time.sleep(.05)
            else:
                raise TimeoutError('Server did not start')
            for relay in relays:
                host, port = address.rsplit(':', 1)
                relay.start((host, int(port)))
            with Recorder(output/'processes', os.getpid(), interval=.5):
                for index in (reversed(range(clients)) if reverse_clients else range(clients)):
                    client_output = output/f'client-{index+1}'
                    client_env = dict(env, THEIA_GRAYBOX_QUALIFY=str(client_output), THEIA_GRAYBOX_QUALIFY_SECONDS=str(duration))
                    log = (output/f'client-{index+1}.log').open('w')
                    logs.append(log)
                    client_address = f'{relays[index].address[0]}:{relays[index].address[1]}' if relays else address
                    processes.append(subprocess.Popen([str(binaries/'theia-client'), *(['--spell-feedback', str(feedback)] if feedback else []), *terrain_options, *combat_options, *(['--targeting'] if targeting else []), *(['--prediction'] if prediction else []), '--player-assets', str(players),
                        '--renet-connect', client_address, str(Path(tickets)/f'client-{index+1}.token')], env=client_env, stdout=log, stderr=subprocess.STDOUT))
                    if persistence:
                        admitted = time.monotonic()+15
                        while time.monotonic() < admitted:
                            assert server.poll() is None, 'Persistent server failed before admission'
                            rows = [json.loads(line.removeprefix('persistent_admit ')) for line in (output/'server.log').read_text().splitlines() if line.startswith('persistent_admit ')]
                            if any(row['client_id'] == index+1 for row in rows):
                                break
                            time.sleep(.05)
                        else:
                            raise TimeoutError('Persistent client did not reach authenticated admission')
                deadline = time.monotonic()+duration+45
                while time.monotonic() < deadline:
                    failed = [p for p in processes[1:] if p.poll() not in (None, 0)]
                    assert not failed, 'Client failed; inspect native logs'
                    assert server.poll() is None, 'Authority failed'
                    if all(p.poll() == 0 for p in processes[1:]):
                        break
                    if capture:
                        capture.audit([p.pid for p in processes[1:]])
                    time.sleep(.2)
                else:
                    raise TimeoutError('Native qualification did not finish')
            stop(server)
    finally:
        for process in reversed(processes):
            stop(process)
        try:
            if capture:
                capture.close()
        finally:
            for log in logs:
                log.close()
            for relay in relays:
                relay.close()
    audio_report = capture.verify() if capture else None
    for relay in relays:
        assert relay.stats["overflow"] == 0 and relay.stats["high_water"] <= 512, relay.stats
        assert relay.stats["dropped"] > 0 and relay.stats["reordered"] > 0, relay.stats
    results = []
    for index in range(clients):
        text = (output/f'client-{index+1}.log').read_text()
        expected_replays = sum(line.split(' ERROR ', 1)[-1] == 'renetcode::client: Failed to decode packet: sequence already received' for line in text.splitlines()) if impairment else 0
        checked_text = '\n'.join(line for line in text.splitlines() if not (impairment and line.split(' ERROR ', 1)[-1] == 'renetcode::client: Failed to decode packet: sequence already received'))
        for bad in ['VUID-', 'Validation Error', 'panicked at', ' ERROR ', 'Prediction paused:']:
            assert bad not in checked_text, bad
        assert '7900 XTX' in text, 'Expected documented discrete GPU; inspect actual adapter'
        if validation:
            assert 'VK_LAYER_KHRONOS_validation' in text, 'Validation layer not observed'
        reports = [json.loads(line)['graybox'] for line in text.splitlines() if line.startswith('{"graybox":')]
        assert reports and max(r['resident'] for r in reports) <= 81
        result = json.loads((output/f'client-{index+1}'/'world.json').read_text())
        assert result['connected'] and result['network_error'] is None, result
        if prediction:
            assert result['prediction']['predicted_steps'] > 100, result
            assert result['prediction']['corrections'] > 100, result
            assert result['prediction']['max_history'] <= 64, result
            assert result['prediction']['queue_drops'] == 0, result
            stats = result['prediction']
            assert stats['clock_phase_backsteps'] == 0, stats
            assert stats['clock_acquisitions'] == 1, stats
            assert stats['max_clock_slew'] <= .0500001, stats
            assert stats['same_tick_samples'] > 50, stats
            assert stats['same_tick_samples'] + stats['unmatched_tick_samples'] == stats['corrections'], stats
            assert sum(stats['same_tick_error_buckets']) == stats['same_tick_samples'], stats
            assert stats['stable_readiness_samples'] > 50, stats
            assert stats['stable_readiness_samples'] + stats['pending_readiness_samples'] == stats['same_tick_samples'], stats
            assert sum(stats['stable_readiness_error_buckets']) == stats['stable_readiness_samples'], stats
            motion = result['render_motion']
            assert sum(motion['steady_speed_ratio_buckets']) == motion['steady_samples'], motion
            assert motion['samples'] == sum(motion[k] for k in ['input_or_world_transitions','pending_readiness','idle','settling','frame_gaps','large_displacements','steady_samples']), motion
            if speed == 1:
                assert motion['steady_samples'] > 100, motion
                if steady_hold_budget is not None:
                    check_steady_motion(motion, steady_hold_budget)
                if not tour:
                    assert motion['large_displacements'] == 0, motion
        if saved_terrain:
            assert result['saved_terrain']['tile_reads'] > 81, result
            assert result['saved_terrain']['tile_bytes'] == result['saved_terrain']['tile_reads'] * 4380, result
        assert result['mesh_assets'] <= result['resident']+16, result
        assert result['gpu_mesh_assets'] <= result['resident']+16, result
        assert all(r['mesh_assets'] <= r['resident']+16 for r in reports[5:]), 'Unbounded CPU mesh assets'
        assert max(r['gpu_mesh_assets'] for r in reports) <= 2*81+16, 'Unbounded GPU staging'
        assert result['loaded']-result['unloaded'] == result['resident'], result
        assert result['max_ground_error'] < .003 and result['probes'] > 100, result
        assert all(value > 0 for value in result['pose_changes']), result
        assert result['moving_frames'] > 100, result
        assert result['held_frames']/result['moving_frames'] < .5, 'Local presentation still holds between most authority updates'
        assert any(r['visible'] < r['resident'] for r in reports if r['resident'] > 20), 'No observed frustum culling'
        assert result['loaded'] > result['max_resident'] and result['unloaded'] > 0, result
        if speed >= 8:
            assert result['loaded'] > 100 and result['unloaded'] > 50, result
        result['expected_netcode_replay_rejections'] = expected_replays
        if continent:
            assert result['position'][0] > 15900, 'Did not traverse the continent'
        steady = [r for r in reports if r['seconds'] >= 8]
        frames = sum(r['frame_count'] for r in steady)
        mean = sum(r['mean_ms']*r['frame_count'] for r in steady)/frames
        result.update(mean_frame_ms=mean, fps=1000/mean, worst_one_second_p95_ms=max(r['p95_ms'] for r in steady), frames=frames)
        results.append(result)
        assert (output/f'client-{index+1}'/'scene.png').is_file()
        if targeting:
            target = json.loads((output/f'client-{index+1}'/'targeting.json').read_text())
            assert target['phase'] == 6, target
            assert target['samples'] > 100 and target['selected_samples'] > 10 and target['cleared_samples'] > 10, target
            assert target['max_position_error_m'] < .003 and target['physics_body_samples'] == 0, target
            assert target['max_visuals'] <= 16 and len(target['observed_revisions']) >= 2, target
            for image in ['selected.png', 'cleared.png', 'reselected.png']:
                assert (output/f'client-{index+1}'/image).is_file(), image
            result['targeting'] = target
            if action_ui:
                ui = target['ui_metrics']
                assert ui['button_layout_samples'] > 100, ui
                assert min(ui['health_samples'], ui['cast_samples'], ui['cooldown_samples']) > 50, ui
                assert ui['invalid_layouts'] == 0 and ui['max_fill_error_px'] <= 1.5, ui
                assert target['pointer_inputs'] >= 2, target
                assert target['combat']['widget_requests'] == target['pointer_inputs'], target
                assert target['combat']['keyboard_requests'] >= 2, target
            if feedback:
                fx = target['feedback']
                assert fx['critical_visuals'] > 0 and fx['critical_voices'] > 0, fx
                assert fx['critical_spatial_initialized'] > 0 and fx['critical_visible_samples'] > 0, fx
                assert fx['critical_queue_overflow'] == fx['critical_visual_overflow'] == fx['critical_voice_overflow'] == 0, fx
                assert fx['max_visuals'] <= 64 and fx['max_voices'] <= 32, fx
                assert fx['max_position_error_metres'] < .003, fx
                assert target['feedback_rebased'] and fx['rebased_visual_samples'] > 0 and fx['rebased_voice_samples'] > 0, fx
                assert (output/f'client-{index+1}'/'feedback.png').is_file()
                if env.get('THEIA_FEEDBACK_STRESS') == '1':
                    assert fx['injected_cosmetics'] > 256 and fx['cosmetic_queue_drops'] > 0, fx
                    assert fx['cosmetic_voice_drops'] > 0, fx
                    if env.get('THEIA_FEEDBACK_COSMETICS') != '0':
                        assert fx['critical_visuals_under_cosmetics'] > 0 and fx['critical_voices_under_cosmetics'] > 0, fx
            if combat_catalog:
                combat = target['combat']
                assert combat['event_gaps'] == 0 and combat['input_backpressure'] == 0, combat
                assert combat['damage'] >= 5 and combat['deaths'] >= 1 and combat['respawns'] >= 1, combat
                assert combat['moving_cast_authority_samples'] > 10, combat
                assert combat['selected_health_frames'] > 100 and combat['defeated_frames'] > 10, combat
                assert combat['max_health'] == 100 and combat['last_respawn_entity'] > combat['last_respawn_previous'] > 0, combat
                for image in ['damaged.png', 'defeated.png', 'respawned.png']:
                    assert (output/f'client-{index+1}'/image).is_file(), image
    server_text = (output/'server.log').read_text()
    if saved_terrain:
        summary = next(json.loads(line)['terrain_summary'] for line in server_text.splitlines() if line.startswith('{"terrain_summary":'))
        assert summary['saved_terrain']['tile_reads'] >= 9, summary
        assert summary['saved_terrain']['tile_bytes'] == summary['saved_terrain']['tile_reads'] * 4380, summary
    if targeting:
        summary = next(json.loads(line.removeprefix('transport_summary ')) for line in server_text.splitlines() if line.startswith('transport_summary '))
        if combat_catalog:
            assert summary['target_selections_accepted'] >= 3*clients and summary['target_selections_rejected'] == 0, summary
            assert summary['cast_requests_accepted'] >= 5 and summary['combat_backpressure_disconnects'] == 0, summary
            assert summary['combat_events_sent'] >= 10*clients, summary
        else:
            assert summary['target_selections_accepted'] == 3*clients and summary['target_selections_rejected'] == 0, summary
        assert summary['max_event_queue_bytes'] <= 8192, summary
        if tour:
            assert any(c['targeting']['empty_after_selection'] > 10 and c['targeting']['returned_samples'] > 10 for c in results), 'No client retired and revisited its target'
    assert server_text.count('player_join ') == clients
    if hold_jump:
        assert any(c["held_jump_reset_samples"] > 100 for c in results), results
        assert all(c["held_jump_reset_failures"] == 0 for c in results), results
    if tour:
        assert server_text.count('terrain_teleport ') == 5, server_text
        assert any(c['visited'] >= 8 and c['patches'] >= 8 for c in results), 'No client completed the teleport/revisit route'
    result = dict(passed=True, persistence=persistence, initial_spawn=spawn, reverse_clients=reverse_clients, action_ui=action_ui, audio=audio_report, feedback_config_sha256=hashlib.sha256(feedback.read_bytes()).hexdigest() if feedback else None, combat_catalog_sha256=hashlib.sha256(combat_catalog.read_bytes()).hexdigest() if combat_catalog else None, targeting=targeting, uncapped=uncapped, steady_hold_budget=steady_hold_budget, saved_terrain_manifest_sha256=hashlib.sha256((saved_terrain/"terrain.manifest.json").read_bytes()).hexdigest() if saved_terrain else None, prediction=prediction, network_impairment=impairment, held_jump_reset=hold_jump, relay_stats=[r.stats for r in relays], clients=results, validation=validation, tour=tour, dev_speed=speed,
        continent=continent, resolution=[1280,720], display='caller native desktop',
        client_sha256=hashlib.sha256((binaries/'theia-client').read_bytes()).hexdigest(),
        server_sha256=hashlib.sha256((binaries/'theia-server').read_bytes()).hexdigest(),
        character_sha256=hashlib.sha256((players/'character.glb').read_bytes()).hexdigest(),
        scope='Linux native real secure Renet, standard Bevy/Avian; bounded terrain traversal, not MMO population capacity')
    (output/'verification.json').write_text(json.dumps(result, indent=2)+'\n')
    return result


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('--binaries', type=Path, default=Path('target/release'))
    parser.add_argument('--players', type=Path, default=Path('build/synty-character-canonical-stage'))
    parser.add_argument('--duration', type=int, default=65)
    parser.add_argument('--clients', type=int, choices=[1,2], default=1)
    parser.add_argument('--hold-jump-teleport', action='store_true')
    parser.add_argument('--impairment', action='store_true', help='40ms + 0..60ms jitter, 10%% loss, duplication/reordering over bounded loopback relays')
    parser.add_argument('--steady-hold-budget', type=float, help='Maximum held-frame ratio for normal-speed predicted movement without a teleport tour')
    parser.add_argument('--paced', action='store_true', help='Retain ordinary VSync/focus pacing instead of the uncapped qualification mode')
    parser.add_argument('--saved-terrain', type=Path)
    parser.add_argument('--prediction', action='store_true')
    parser.add_argument('--targeting', action='store_true')
    parser.add_argument('--persistence', action='store_true')
    parser.add_argument('--spawn', help='Trusted developer spawn x,z')
    parser.add_argument('--reverse-clients', action='store_true')
    parser.add_argument('--action-ui', action='store_true', help='Alternate real Bevy picking/widget activation with keyboard input in combat fixture')
    parser.add_argument('--spell-feedback', type=Path)
    parser.add_argument('--combat-catalog', type=Path, help='Native ability/death/respawn fixture; requires --prediction --speed 1 --no-tour')
    parser.add_argument('--continent', action='store_true', help='Continuous full-continent accelerated traversal')
    parser.add_argument('--no-tour', action='store_true')
    parser.add_argument('--no-validation', action='store_true')
    parser.add_argument('--speed', type=int, choices=[1,8,32], default=32)
    args = parser.parse_args()
    print(json.dumps(run(args.output, args.binaries.resolve(), args.players.resolve(), duration=args.duration,
        clients=args.clients, tour=not args.no_tour, validation=not args.no_validation, speed=args.speed, continent=args.continent, prediction=args.prediction, impairment=args.impairment, hold_jump=args.hold_jump_teleport, saved_terrain=args.saved_terrain.resolve() if args.saved_terrain else None, uncapped=not args.paced, steady_hold_budget=args.steady_hold_budget, targeting=args.targeting, combat_catalog=args.combat_catalog.resolve() if args.combat_catalog else None, feedback=args.spell_feedback.resolve() if args.spell_feedback else None, action_ui=args.action_ui, persistence=args.persistence, spawn=args.spawn, reverse_clients=args.reverse_clients), indent=2))
