"""Owned paused-route handoff through the ordinary focused F7 key."""
import argparse
import json
import math
import os
from pathlib import Path
import signal
import subprocess
import sys
from qualify_world_input_os import Desktop, owned_window, wait


def run(args):
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    source = args.route.resolve()
    document = json.loads(source.read_text())
    document['navigation'] = str((source.parent / document['navigation']).resolve())
    document['start_paused'] = True
    route = out / 'paused-route.json'
    route.write_text(json.dumps(document, indent=2) + '\n')
    native = out / 'native'
    command = [sys.executable, str(Path(__file__).with_name('qualify_population_visual.py')),
               '--world', str(args.world.resolve()), '--population', str(args.population.resolve()),
               '--route', str(route), '--output', str(native), '--seconds', '40',
               '--binaries', str(args.binaries.resolve())]
    desktop = None
    with (out / 'qualification.log').open('w') as log:
        process = subprocess.Popen(command, stdout=log, stderr=subprocess.STDOUT, start_new_session=True)
        try:
            def reports():
                assert process.poll() is None, 'Owned qualifier exited; inspect qualification.log'
                path = native / 'client.log'
                values = []
                if path.exists():
                    for line in path.read_text().splitlines():
                        if line.startswith('{"test_controller":'):
                            try:
                                values.append(json.loads(line)['test_controller'])
                            except json.JSONDecodeError:
                                pass  # The writer can still be completing this line.
                return [value for value in values if value['feet'] is not None]
            observed = wait(lambda: reports() if len(reports()) >= 4 else None, 60)
            assert all(not r['active'] and not r['finished'] and
                       r['travel']['elapsed_seconds'] == 0 for r in observed), observed
            assert all(math.dist(r['feet'], observed[0]['feet']) < .02 for r in observed), observed
            children = Path(f'/proc/{process.pid}/task/{process.pid}/children').read_text().split()
            clients = [int(pid) for pid in children
                       if Path(f'/proc/{pid}/comm').read_text().strip() == 'theia-client']
            assert len(clients) == 1, clients
            window = wait(lambda: owned_window(clients[0]))
            desktop = Desktop()
            desktop.focus(window)
            desktop.key('F7')
            result = wait(lambda: next((r for r in reversed(reports())
                                       if r['finished'] and not r['active']), None), 30)
            assert result['travel']['failure'] is None, result
            assert process.wait(timeout=60) == 0, 'Native route qualification failed'
            proof = json.loads((native / 'qualification.json').read_text())
            assert proof['passed']
            (out / 'qualification.json').write_text(json.dumps(dict(
                passed=True, paused_observations=observed, activated_with='Owned focused F7 key',
                final=proof['travel'], scope='Paused preparation, ordinary key activation, arrival and manual release'), indent=2) + '\n')
        finally:
            if desktop:
                desktop.close()
            if process.poll() is None:
                process.send_signal(signal.SIGINT)
                try:
                    process.wait(timeout=30)
                except subprocess.TimeoutExpired:
                    os.killpg(process.pid, signal.SIGKILL)
                    process.wait(timeout=5)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('world', 'population', 'route', 'output'):
        parser.add_argument('--' + name, type=Path, required=True)
    parser.add_argument('--binaries', type=Path, default=Path('target/debug'))
    os.environ.setdefault('DISPLAY', ':0')
    run(parser.parse_args())
