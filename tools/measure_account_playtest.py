"""Compare debug/release account playtests without injecting desktop input.

Uses the same exported world, viewport, HUD and stationary spawn. Optional editor
pause/resume isolates its contention while retaining its unsaved in-memory draft.
The existing read-only UI qualification reporter is enabled in both builds; its
cost is included. Validation layers are disabled during performance sampling.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import signal
import statistics
import subprocess
import sys
import time

from edit_local import environment
from private_identity import Identity
from private_postgres import Cluster
from gpu_validation import check_gpu_diagnostics


def wait(check, seconds=90):
    end = time.monotonic() + seconds
    while time.monotonic() < end:
        try:
            value = check()
            if value:
                return value
        except (FileNotFoundError, json.JSONDecodeError):
            pass
        time.sleep(.1)
    raise TimeoutError('Performance fixture readiness timed out')


def cpu(pid):
    result = {}
    for path in Path(f'/proc/{pid}/task').glob('*/stat'):
        try:
            raw = path.read_text()
            fields = raw[raw.rfind(')') + 2:].split()
            result[path.parent.name] = (raw[raw.find('(') + 1:raw.rfind(')')], int(fields[11]) + int(fields[12]))
        except FileNotFoundError:
            pass
    return result


def sample(log, pid, seconds):
    offset = log.stat().st_size
    before = cpu(pid)
    gpu = []
    started = time.monotonic()
    while time.monotonic() - started < seconds:
        for path in Path('/sys/class/drm').glob('card[0-9]/device/gpu_busy_percent'):
            total = path.parent/'mem_info_vram_total'
            if total.exists() and int(total.read_text()) > 8 * 1024**3:
                gpu.append(int(path.read_text()))
        time.sleep(.2)
    elapsed = time.monotonic() - started
    after = cpu(pid)
    frames, mesh = [], []
    with log.open() as file:
        file.seek(offset)
        for line in file:
            try:
                row = json.loads(line).get('camera_performance')
            except json.JSONDecodeError:
                continue
            if row:
                assert row['world_only'], 'Unexpected visual camera refinement'
                frames.extend(row['frame_ms'])
                mesh.extend(row['mesh_ms'])
    assert len(frames) > 10, 'Missing real frame timing samples'
    ticks = os.sysconf('SC_CLK_TCK')
    threads = sorted((dict(name=name, cpu_percent=100 * (value-before.get(tid, (name, value))[1]) / ticks / elapsed)
                      for tid, (name, value) in after.items()), key=lambda r: r['cpu_percent'], reverse=True)
    return dict(frames=len(frames), mean_ms=statistics.mean(frames),
                p95_ms=sorted(frames)[int(.95 * (len(frames)-1))],
                fps=1000/statistics.mean(frames), camera_mesh_ms=statistics.mean(mesh),
                gpu_busy_mean=statistics.mean(gpu) if gpu else None,
                client_cpu_percent=sum(t['cpu_percent'] for t in threads), threads=threads[:8])


def run(args):
    out = args.output.resolve()
    out.mkdir(parents=True, mode=0o700, exist_ok=False)
    package = args.package.resolve()
    editor = args.editor_pid
    if editor:
        assert Path(f'/proc/{editor}/comm').read_text().strip() == 'theia-zone-edit'
    results = []
    with Identity(out/'provider', Path('build/identity-sdk'), token_seconds=1800, interactive=True) as identity, \
            Cluster(out/'cluster', Path('build/postgres-sdk'), isolate_process=True) as cluster:
        cases = [(f'cap-{cap}', args.release_binaries, cap) for cap in args.fps_caps] if args.fps_caps else [('debug', args.debug_binaries, 0), ('release', args.release_binaries, 0)]
        for name, binaries, cap in cases:
            folder = out/name
            folder.mkdir()
            runtime = folder/'owned/runtime'
            env = environment(os.environ, identity, cluster, out, 1800)
            for key in list(env):
                if key.startswith(('VK_', 'WGPU_')):
                    env.pop(key)
            env.update(WGPU_BACKEND='vulkan', WGPU_VALIDATION='0', RUST_LOG='info',
                       THEIA_WORLD_INPUT_OS_QUALIFY=str(folder),
                       THEIA_PERFORMANCE_LOG='1', THEIA_PERFORMANCE_UNCAPPED='1' if cap == 0 and not args.fps_caps else '0')
            log = folder/'launcher.log'
            preferences = folder/'interface.json'
            preferences.write_text(json.dumps(dict(schema=1, ui_scale=1., fps_cap=cap,
                                                   vsync=args.vsync == 'on', view_distance='nearby', debug_stats=True)))
            command = [sys.executable, 'tools/account_population_playtest.py',
                       '--package', str(package), '--players', str(args.players.resolve()),
                       '--binaries', str(binaries.resolve()), '--output', str(folder/'owned'),
                       '--gameplay-manifest', str(package/'world/gameplay/gameplay.manifest.json'),
                       '--fantasy-ui', str(args.fantasy_ui.resolve()), '--window-size', '3440x1440', '--parties', '--ui-preferences', str(preferences)]
            child = None
            paused = False
            try:
                with log.open('w') as stream:
                    child = subprocess.Popen(command, env=env, stdin=subprocess.PIPE, stdout=stream, stderr=subprocess.STDOUT)
                def alive():
                    assert child.poll() is None, f'{name} launcher exited; inspect {log}'
                    return True
                def choices():
                    alive()
                    return json.loads((runtime/'characters.json').read_text())
                selected = wait(choices)
                selection = dict(schema=1, character_id=selected['characters'][0]['id']) if selected['characters'] else dict(schema=1, create_name='PerfTester')
                pending = runtime/'selection.pending'
                pending.write_text(json.dumps(selection)); pending.replace(runtime/'selection.json')
                def state():
                    alive()
                    return json.loads((folder/'input-os.json').read_text())
                def entered():
                    value = state()
                    assert value['loading']['phase'] != 'Failed', value['loading']
                    return value if value['loading']['phase'] == 'Playing' and value['authored_scene']['ready'] else None
                ready = wait(entered, 120)
                assert ready['window'][:2] == [3440., 1400.], ready['window']
                (folder/'capture.request').write_text('Stationary performance baseline\n')
                time.sleep(10)
                pid = ready['pid']
                for phase in (['editor_running'] if args.keep_editor_running else ['editor_running', 'editor_paused']) if editor else ['editor_absent']:
                    if phase == 'editor_paused':
                        os.kill(editor, signal.SIGSTOP); paused = True
                        time.sleep(3)
                    before = state()
                    assert before['interface']['view_distance'] == 'nearby', 'View distance changed before sampling'
                    if args.fps_caps:
                        assert before['interface']['fps_cap'] == cap, 'FPS cap changed before sampling'
                    print(f'Measuring {name}: {phase}', flush=True)
                    result = sample(log, pid, args.seconds)
                    current = state()
                    assert current['interface']['view_distance'] == 'nearby', 'View distance changed during sampling'
                    if args.fps_caps:
                        assert current['interface']['fps_cap'] == cap, 'FPS cap changed during sampling'
                        assert current['interface']['vsync'] == (args.vsync == 'on'), 'VSync changed during sampling'
                        expected_mode = 'Fifo' if args.vsync == 'on' else 'AutoNoVsync'
                        assert current['interface']['present_mode'] == expected_mode, current['interface']
                        ceiling = min(cap or float('inf'), args.display_hz) if args.vsync == 'on' else cap
                        if ceiling:
                            assert .90 * ceiling <= result['fps'] <= 1.03 * ceiling, result
                        else:
                            assert result['fps'] > 120, 'Unlimited did not exceed the tested caps on this fixture'
                    assert current['camera_angles'] == before['camera_angles'], 'Camera moved during sampling'
                    assert current['authored_scene']['ready'] and current['authored_scene']['retractions'] == 0
                    result.update(build=name, fps_cap=cap, vsync=args.vsync, view_distance='nearby', present_mode=current['interface']['present_mode'], display_hz=args.display_hz, phase=phase, adapter=ready['adapter'], window=ready['window'],
                                  scene=ready['authored_scene'], position=current['rendered_body'])
                    results.append(result)
                    (out/'measurements.json').write_text(json.dumps(results, indent=2)+'\n')
                    print(json.dumps({k:result[k] for k in ('build','phase','fps','mean_ms','p95_ms','gpu_busy_mean','client_cpu_percent')}), flush=True)
                check_gpu_diagnostics(log.read_text())
            finally:
                if paused:
                    os.kill(editor, signal.SIGCONT)
                if child is not None and child.poll() is None:
                    child.stdin.close()
                    try:
                        child.wait(timeout=25)
                    except subprocess.TimeoutExpired:
                        child.terminate(); child.wait(timeout=10)
    hashes = {}
    for name, directory in [('debug', args.debug_binaries), ('release', args.release_binaries)]:
        with (directory/'theia-client').open('rb') as file:
            hashes[name] = hashlib.file_digest(file, 'sha256').hexdigest()
    report = dict(results=results, binary_sha256=hashes, package=str(package),
                  scope='Stationary authenticated Millhaven spawn; identical HUD and 10 Hz read-only qualification telemetry; Vulkan, explicit per-case FPS cap; no input injection; no validation layers; editor resumed after each comparison')
    (out/'report.json').write_text(json.dumps(report, indent=2)+'\n')
    return report


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    for option in ['package', 'players', 'fantasy-ui', 'debug-binaries', 'release-binaries', 'output']:
        parser.add_argument('--'+option, type=Path, required=True)
    parser.add_argument('--editor-pid', type=int)
    parser.add_argument('--vsync', choices=['on', 'off'], default='off')
    parser.add_argument('--display-hz', type=float, help='Measured current monitor refresh; required for VSync rate checks')
    parser.add_argument('--fps-caps', nargs='+', type=int, choices=[0, 30, 60, 90, 120, 144, 165, 240])
    parser.add_argument('--keep-editor-running', action='store_true')
    parser.add_argument('--seconds', type=int, default=20)
    args = parser.parse_args()
    if args.vsync == 'on' and (args.display_hz is None or not 1 <= args.display_hz <= 1000):
        parser.error('VSync checks require the current --display-hz')
    if not 5 <= args.seconds <= 60:
        parser.error('Sample duration must be 5..60 seconds')
    signal.signal(signal.SIGTERM, lambda *_: sys.exit(143))
    run(args)
