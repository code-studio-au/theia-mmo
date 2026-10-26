#!/usr/bin/env python3
"""Native Linux check: corrupt local heights close the client, never generate fallback."""
import argparse
import json
import sys
from pathlib import Path
import shutil
import signal
import subprocess
import tempfile
import time


def stop(process):
    if process.poll() is None:
        process.send_signal(signal.SIGINT)
        try:
            process.wait(timeout=10)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait()


def run(output, terrain, binaries, players):
    output.mkdir(parents=True, exist_ok=False)
    processes = []
    try:
        with tempfile.TemporaryDirectory(prefix='fault-', dir=output) as temporary:
            temporary = Path(temporary)
            broken = temporary / 'terrain'
            broken.mkdir()
            shutil.copyfile(terrain/'terrain.manifest.json', broken/'terrain.manifest.json')
            for x in range(61, 64):
                for z in range(61, 64):
                    name = f'tile_{x}_{z}.height'
                    shutil.copyfile(terrain/name, broken/name)
            tile = broken/'tile_62_62.height'
            data = bytearray(tile.read_bytes())
            data[24] ^= 1
            tile.write_bytes(data)
            with (output/'server.log').open('w') as server_log, (output/'client.log').open('w') as client_log:
                server = subprocess.Popen([str(binaries/'theia-server'), '127.0.0.1:0', str(temporary/'unused.der'),
                    '--saved-terrain', str(terrain), '--prediction', '--renet-tickets', str(temporary/'tickets')], stdout=server_log, stderr=subprocess.STDOUT)
                processes.append(server)
                deadline = time.monotonic()+20
                while time.monotonic() < deadline:
                    assert server.poll() is None, 'Valid saved authority failed'
                    text = (output/'server.log').read_text()
                    address = next((line.split()[0].removeprefix('listening=') for line in text.splitlines() if line.startswith('listening=')), None)
                    if address:
                        break
                    time.sleep(.05)
                else:
                    raise TimeoutError('Authority startup')
                client = subprocess.Popen([str(binaries/'theia-client'), '--saved-terrain', str(broken), '--prediction',
                    '--player-assets', str(players), '--renet-connect', address, str(temporary/'tickets/client-1.token')], stdout=client_log, stderr=subprocess.STDOUT)
                processes.append(client)
                code = client.wait(timeout=30)
                assert code != 0, 'Corrupt terrain must not run via fallback'
                text = (output/'client.log').read_text()
                assert 'Terrain load failed:' in text and 'hash mismatch: 62,62' in text, text
                assert 'panicked at' not in text and 'VUID-' not in text, text
                assert server.poll() is None, 'Client content failure killed authority'
                stop(server)
            result = {'passed': True, 'platform': sys.platform, 'scope': 'native corrupt-tile client shutdown, valid authority survives', 'client_exit': code}
            (output/'verification.json').write_text(json.dumps(result, indent=2)+'\n')
            print(json.dumps(result, indent=2))
    finally:
        for process in reversed(processes):
            stop(process)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('--terrain', type=Path, default=Path('build/graybox-terrain-v1'))
    parser.add_argument('--binaries', type=Path, default=Path('target/release'))
    parser.add_argument('--players', type=Path, default=Path('build/synty-character-canonical-stage'))
    args = parser.parse_args()
    run(args.output.resolve(), args.terrain.resolve(), args.binaries.resolve(), args.players.resolve())
