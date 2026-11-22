#!/usr/bin/env python3
"""Native Millhaven checks using encrypted Renet and ordinary Classic commands.

No desktop input injection. Requires prepared terrain, matched optimized binaries
and the caller's native display. Captures only the owned game viewport.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile
import time

from gpu_validation import check_gpu_diagnostics
from launch_environment import client_environment
from player_assets import default_player_assets
from qualify_graybox import stop

CASES = {
    'map-turns': ([2048, 1810], 30, [dict(duration_ms=1000, turn=-1),
        dict(duration_ms=3500), dict(duration_ms=500, turn=-1),
        dict(duration_ms=4500, forward=1000), dict(duration_ms=500, turn=-1),
        dict(duration_ms=4500), dict(duration_ms=500, turn=-1),
        dict(duration_ms=4500), dict(duration_ms=500, turn=-1), dict(duration_ms=9000)]),
    'village': ([2040, 1810], 38, [dict(duration_ms=24000, strafe=1000),
                                dict(duration_ms=1000, jump=True), dict(duration_ms=6000)]),
    'east-edge': ([4080, 1600], 20, [dict(duration_ms=12000, strafe=1000)]),
    'north-west': ([64, 3008], 20, [dict(duration_ms=8000, strafe=1000), dict(duration_ms=3000, walk=True, forward=1000)]),
    'south-east': ([4000, 64], 20, [dict(duration_ms=8000, strafe=-1000)]),
    'bandit-terrace': ([2560, 2540], 20, [dict(duration_ms=1000, jump=True), dict(duration_ms=10000)]),
}


def digest(path):
    result = hashlib.sha256()
    with path.open('rb') as stream:
        while block := stream.read(1024*1024):
            result.update(block)
    return result.hexdigest()


def check_minimap_coverage(result):
    # Independently count the 4 m pixel centres that fall inside this package.
    # A dark edge is valid only outside the zone, never for a missing live tile.
    x, _, z = result['minimap']['centre']
    expected = sum(0 <= x - 126 + col*4 < 4096 and 0 <= z - 126 + row*4 < 3072
                   for row in range(64) for col in range(64))
    assert result['minimap']['known_pixels'] == expected, (result['minimap'], expected)
    return expected


def run(output, terrain, binaries, selected, window_size='1280x720', zone_map=False):
    output, terrain, binaries = (p.resolve() for p in (output, terrain, binaries))
    manifest = json.loads((terrain/'terrain.manifest.json').read_text())
    assert manifest['schema'] == 2 and manifest['samples_per_axis'] == 129
    assert manifest['chunks_xz'] == [32, 24] and manifest['chunk_origin'] == [0, 0]
    output.mkdir(parents=True, exist_ok=False)
    hashes = {name: digest(binaries/name)
              for name in ('theia-client', 'theia-server')}
    base = client_environment(os.environ)
    base.update(WGPU_BACKEND='vulkan', VK_INSTANCE_LAYERS='VK_LAYER_KHRONOS_validation', VK_LOADER_DEBUG='layer',
                WGPU_VALIDATION='1', RUST_LOG='info', THEIA_PERFORMANCE_UNCAPPED='0')
    base.pop('THEIA_GRAYBOX_AUTOPLAY', None)
    sdk = Path('build/native-sdk/usr/lib/x86_64-linux-gnu').resolve()
    if sdk.is_dir():
        base['LD_LIBRARY_PATH'] = str(sdk) + (os.pathsep + base['LD_LIBRARY_PATH'] if base.get('LD_LIBRARY_PATH') else '')
    records = []
    for name in selected:
        spawn, duration, steps = CASES[name]
        if zone_map:
            duration = max(duration, 28)
        folder = output/name
        folder.mkdir()
        script = folder/'controller.json'
        script.write_text(json.dumps(dict(schema=1, name='Millhaven '+name, steps=steps))+'\n')
        env = dict(base, THEIA_GRAYBOX_QUALIFY=str(folder/'capture'),
                   THEIA_GRAYBOX_QUALIFY_SECONDS=str(duration), THEIA_WINDOW_SIZE=window_size)
        if zone_map:
            env['THEIA_ZONE_MAP_QUALIFY'] = str(folder/'map')
        processes = []
        with tempfile.TemporaryDirectory(prefix='tickets-', dir=folder) as tickets:
            with (folder/'server.log').open('w') as server_log, (folder/'client.log').open('w') as client_log:
                try:
                    common = ['--saved-terrain', str(terrain), '--prediction', '--classic']
                    server = subprocess.Popen([str(binaries/'theia-server'), '127.0.0.1:0', str(folder/'unused.der'),
                        *common, '--dev-spawn', ','.join(map(str, spawn)), '--renet-tickets', tickets],
                        env=env, stdout=server_log, stderr=subprocess.STDOUT)
                    processes.append(server)
                    deadline = time.monotonic()+30
                    while True:
                        assert server.poll() is None, f'Server exited: {folder}'
                        match = re.search(r'listening=(\S+) local_tickets=', (folder/'server.log').read_text())
                        if match:
                            break
                        if time.monotonic()>deadline:
                            raise TimeoutError('Terrain authority startup')
                        time.sleep(.05)
                    client = subprocess.Popen([str(binaries/'theia-client'), *common,
                        '--player-assets', str(default_player_assets().resolve()),
                        '--test-controller', str(script), '--renet-connect', match[1], str(Path(tickets)/'client-1.token')],
                        env=env, stdout=client_log, stderr=subprocess.STDOUT)
                    processes.append(client)
                    assert client.wait(timeout=duration+60) == 0, f'Client exited: {folder}'
                    assert server.poll() is None
                finally:
                    for process in reversed(processes):
                        stop(process)
        text = (folder/'client.log').read_text()
        diagnostics = check_gpu_diagnostics(text, allow_surface_resize=False)
        assert 'VK_LAYER_KHRONOS_validation' in text
        gpu = next(line for line in text.splitlines() if 'AdapterInfo' in line)
        rows = [json.loads(line)['graybox'] for line in text.splitlines() if line.startswith('{"graybox":')]
        result = json.loads((folder/'capture/world.json').read_text())
        assert result['connected'] and result['network_error'] is None
        assert result['prediction']['predicted_steps'] > 100 and result['prediction']['corrections'] > 100
        assert result['prediction']['queue_drops'] == 0 and result['prediction']['max_history'] <= 64
        assert result['probes'] > 100 and result['max_ground_error'] < .003
        check_minimap_coverage(result)
        assert result['prediction']['max_same_tick_error_metres'] < .003
        assert 0 < result['resident'] <= 81 and result['max_resident'] <= 81
        assert result['mesh_assets'] <= result['resident'] + 16
        assert result['gpu_mesh_assets'] <= result['resident'] + 16
        assert result['saved_terrain']['tile_bytes'] == result['saved_terrain']['tile_reads'] * (24+129*129*4)
        assert (folder/'capture/scene.png').is_file()
        if zone_map:
            width, height = map(int, window_size.split('x'))
            for action in ['opened', 'caves', 'closed-m', 'reopened', 'closed-escape', 'button-opened', 'village']:
                snapshot = json.loads((folder/'map'/f'{action}.json').read_text())
                if not snapshot['open']:
                    continue
                fx, fz = snapshot['world_forward']
                expected_forward = [-fx, -fz]  # +Z north, -X east in Y-up
                for field in ['map_forward', 'minimap_forward']:
                    assert sum((a-b)**2 for a, b in zip(snapshot[field], expected_forward)) < .0001, snapshot
                if name == 'map-turns':
                    cardinal = {'opened': [0, -1], 'caves': [1, 0], 'reopened': [0, 1],
                                'button-opened': [-1, 0], 'village': [0, -1]}[action]
                    assert sum((a-b)**2 for a, b in zip(snapshot['map_forward'], cardinal)) < .001, snapshot
                assert (folder/'map'/f'{action}.png').is_file()
                x, y, w, h = snapshot['panel']
                assert x >= 0 and y >= 0 and x+w <= width+1 and y+h <= height+1, snapshot
                x, y, w, h = snapshot['surface']
                labels = snapshot['labels']
                assert len(labels) == 4
                for index, (lx, ly, lw, lh) in enumerate(labels):
                    assert x <= lx and y <= ly and lx+lw <= x+w+1 and ly+lh <= y+h+1, snapshot
                    for rx, ry, rw, rh in labels[index+1:]:
                        assert lx+lw <= rx or rx+rw <= lx or ly+lh <= ry or ry+rh <= ly, snapshot
                px, py, pw, ph = snapshot['player']
                u, v = snapshot['player_uv']
                assert abs(px+pw/2-(x+u*w)) < 2 and abs(py+ph/2-(y+v*h)) < 2, snapshot
            first = json.loads((folder/'map/opened.json').read_text())
            last = json.loads((folder/'map/village.json').read_text())
            if name == 'village':
                assert last['player_uv'][0] < first['player_uv'][0] - .02
            if name == 'map-turns':
                east = json.loads((folder/'map/caves.json').read_text())
                assert east['player_uv'][0] > first['player_uv'][0] + .001
        p = result['position']
        assert 0 < p[0] < 4096 and 0 < p[2] < 3072
        if name == 'village':
            assert p[0] > 2176 and result['visited'] >= 3, p
            assert result['loaded'] > result['max_resident'] and result['unloaded'] > 0
        if name == 'east-edge':
            assert 4095 < p[0] < 4095.7, p
        if name == 'north-west':
            assert p[0] > spawn[0] + 40 and p[1] > 256, p
        steady = [r for r in rows if r['seconds'] >= 8]
        count = sum(r['frame_count'] for r in steady)
        mean = sum(r['mean_ms']*r['frame_count'] for r in steady)/count
        records.append(dict(name=name, spawn_xz=spawn, final_position=p, gpu=gpu, gpu_diagnostics=diagnostics,
                            mean_frame_ms=mean, fps=1000/mean, result=result))
        (output/'qualification.json').write_text(json.dumps(dict(passed=len(records)==len(selected),
            binary_sha256=hashes, cases=records, scope='Linux native Classic/Renet terrain walkthrough samples; no Windows, hydrology or full route coverage claim'),indent=2)+'\n')
        print(json.dumps(dict(case=name, passed=True, fps=1000/mean, position=p)), flush=True)
    assert all(digest(binaries/name) == value for name, value in hashes.items())


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--terrain', type=Path, default=Path('build/millhaven-v3-1m'))
    parser.add_argument('--binaries', type=Path, default=Path('target/release'))
    parser.add_argument('--case', choices=CASES, action='append')
    parser.add_argument('--window-size', default='1280x720')
    parser.add_argument('--zone-map', action='store_true')
    args = parser.parse_args()
    run(args.output, args.terrain, args.binaries, args.case or list(CASES), args.window_size, args.zone_map)
