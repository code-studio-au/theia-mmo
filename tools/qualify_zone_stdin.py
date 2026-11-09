#!/usr/bin/env python3
"""Qualify bounded stdin authoring commands without desktop connections."""
import argparse
import copy
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
LIMIT = 1024 * 1024


def run(output, binary):
    output = output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    binary = binary.resolve()
    runtime = output / 'no-desktop-runtime'
    runtime.mkdir(mode=0o700)
    env = dict(os.environ, DISPLAY='', WAYLAND_DISPLAY='stdin-disabled', XDG_RUNTIME_DIR=str(runtime))
    env.pop('WAYLAND_SOCKET', None)
    cases = []

    def execute(label, args, data=None, code=0, launcher=False, cwd=ROOT):
        prefix = [sys.executable, str(ROOT / 'tools/edit_zone.py'), '--no-build'] if launcher else [str(binary)]
        result = subprocess.run([*prefix, *map(str, args)], input=data, cwd=cwd, env=env, capture_output=True, timeout=45)
        (output / f'{label}.stdout').write_bytes(result.stdout)
        (output / f'{label}.stderr').write_bytes(result.stderr)
        assert result.returncode == code, (label, result.returncode, result.stderr)
        cases.append(label)
        return json.loads(result.stdout) if result.stdout else None

    def lines(*commands):
        return ''.join(json.dumps(command, ensure_ascii=False) + '\n' for command in commands).encode()

    def rename(revision, name='Piped zone'):
        return dict(command='apply', expected_revision=revision, edits=[dict(op='rename_zone', name=name)])

    seed = output / 'seed.json'
    execute('seed', ['--headless', '--commands', ROOT / 'content/zones/selection.commands.jsonl', '--output', seed])
    base = json.loads(seed.read_text())
    for label, origin in [('local', [.125, .25, -.5]), ('far', [128000000000.125, 20.25, -128000000000.5])]:
        zone = copy.deepcopy(base)
        def shifted(position):
            return [a + b for a, b in zip(position, origin)]
        zone['origin'] = shifted(zone['origin'])
        zone['spawn'] = [origin[0], origin[1] + .0625, origin[2] - 64]
        for obj in zone['objects']:
            obj['pose']['position'] = shifted(obj['pose']['position'])
        for entry in zone['plants'] + zone['markers']:
            entry['position'] = shifted(entry['position'])
        for guide in zone['paths']:
            guide['points'] = [shifted(point) for point in guide['points']]
        source = output / f'{label}.source.json'
        source.write_text(json.dumps(zone))
        original = source.read_bytes()
        move = dict(command='apply', expected_revision=0, edits=[dict(op='translate', id=1, offset=[.125, 0, -.5])])
        sequence = lines(move, dict(command='undo', expected_revision=1), dict(command='redo', expected_revision=2))
        command_file = output / f'{label}.commands.jsonl'
        command_file.write_bytes(sequence)
        piped, filed = output / f'{label}.piped.json', output / f'{label}.filed.json'
        report = execute(label + '-pipe', ['--headless', '--zone', source, '--commands', '-', '--output', piped], sequence)
        file_report = execute(label + '-file', ['--headless', '--zone', source, '--commands', command_file, '--output', filed])
        assert report == file_report and report['revision'] == 3 and report['history'] == [1, 0]
        expected = copy.deepcopy(zone)
        expected['objects'][0]['pose']['position'] = [a + b for a, b in zip(expected['objects'][0]['pose']['position'], [.125, 0, -.5])]
        assert json.loads(piped.read_text()) == json.loads(filed.read_text()) == expected
        checked = execute(label + '-preflight', ['--zone', source, '--commands', '-', '--check-playtest'], lines(move))
        assert checked['revision'] == 1 and checked['playtest']['ready']
        assert checked['playtest']['requested_spawn'] == zone['spawn']
        checked = execute(label + '-launcher', ['--zone', source, '--commands', '-', '--check-playtest'], lines(move), launcher=True)
        assert checked['revision'] == 1 and checked['playtest']['ready']
        assert source.read_bytes() == original

    source = output / 'local.source.json'
    original = source.read_bytes()
    target = output / 'retained.json'
    target.write_bytes(b'preserve existing destination exactly')
    sentinel = target.read_bytes()
    command = rename(0, 'Élan 山谷')
    named = output / 'named.json'
    report = execute('unicode-no-final-newline', ['--headless', '--zone', source, '--commands', '-', '--output', named], lines(command).rstrip(b'\n'))
    assert report['revision'] == 1 and json.loads(named.read_text())['name'] == 'Élan 山谷'
    empty = output / 'empty.json'
    report = execute('empty-eof', ['--headless', '--zone', source, '--commands', '-', '--output', empty], b'')
    assert report['revision'] == 0 and report['history'] == [0, 0] and json.loads(empty.read_text()) == json.loads(source.read_text())
    failures = {
        'late-json': lines(rename(0)) + b'{bad json\n',
        'late-utf8': lines(rename(0)) + b'\xff\n',
        'late-stale': lines(rename(0), rename(0)),
        'unknown-id': lines(dict(command='apply', expected_revision=0, edits=[dict(op='translate', id=9999, offset=[1, 0, 0])])),
        'edit-budget': lines(dict(command='apply', expected_revision=0, edits=[dict(op='rename_zone', name='Too many')] * 129)),
        'line-budget': b''.join(lines(rename(i)) for i in range(1025)),
    }
    for label, payload in failures.items():
        execute(label, ['--headless', '--zone', source, '--commands', '-', '--output', target], payload, code=1)
        assert target.read_bytes() == sentinel and source.read_bytes() == original
    report = execute('exact-line-budget', ['--headless', '--zone', source, '--commands', '-'], b''.join(lines(rename(i)) for i in range(1024)))
    assert report['revision'] == 1024 and report['history'] == [64, 0]
    payload = lines(rename(0)).rstrip(b'\n')
    payload += b' ' * (LIMIT - 1 - len(payload)) + b'\n'
    report = execute('exact-byte-budget', ['--headless', '--zone', source, '--commands', '-'], payload)
    assert report['revision'] == 1
    execute('excess-byte-budget', ['--headless', '--zone', source, '--commands', '-', '--output', target], payload + b' ', code=1)
    assert target.read_bytes() == sentinel and source.read_bytes() == original
    crlf = lines(rename(0)).rstrip(b'\n')
    crlf += b' ' * (LIMIT - 2 - len(crlf)) + b'\r\n'
    report = execute('exact-byte-budget-crlf', ['--headless', '--zone', source, '--commands', '-'], crlf)
    assert report['revision'] == 1
    # BufRead.lines strips CRLF. The raw byte bound must still count both bytes
    # and reject a capped prefix, rather than silently dropping later commands.
    execute('excess-byte-budget-crlf', ['--headless', '--zone', source, '--commands', '-', '--output', target], crlf[:-2] + b' \r\n', code=1)
    execute('truncated-prefix', ['--headless', '--zone', source, '--commands', '-', '--output', target], crlf + lines(rename(1)), code=1)
    assert target.read_bytes() == sentinel and source.read_bytes() == original
    # Only an exact '-' means stdin. A relative './-' still addresses a literal file.
    (output / '-').write_bytes(lines(rename(0, 'Literal dash file')))
    literal = output / 'literal.json'
    report = execute('literal-dash-path', ['--headless', '--zone', source, '--commands', './-', '--output', literal], cwd=output)
    assert report['revision'] == 1 and json.loads(literal.read_text())['name'] == 'Literal dash file'
    result = dict(cases=cases, editor_sha256=hashlib.sha256(binary.read_bytes()).hexdigest(), scope='Linux CPU/headless; X11/Wayland connections disabled; no desktop input, native UI, Windows or multiplayer qualification')
    (output / 'qualification.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps(result, indent=2))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--binary', type=Path, default=Path('target/debug/theia-zone-editor'))
    args = parser.parse_args()
    run(args.output, args.binary)
