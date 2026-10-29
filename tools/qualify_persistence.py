#!/usr/bin/env python3
"""Store-only real PostgreSQL qualification in an owned isolated loopback cluster."""
import argparse
import json
from pathlib import Path
import subprocess
from private_postgres import Cluster


def run(output, sdk):
    output = output.resolve()
    output.mkdir(mode=0o700, parents=True, exist_ok=False)
    commands = []
    with Cluster(output/'cluster', sdk) as cluster:
        env = cluster.test_environment()
        flags = cluster.query('SHOW fsync; SHOW synchronous_commit; SHOW full_page_writes;')
        assert flags.returncode == 0 and flags.stdout.split() == ['on', 'on', 'on']
        wrong = cluster.query('SELECT 1;', wrong_password=True)
        assert wrong.returncode != 0 and 'password authentication failed' in wrong.stderr

        def cargo(name, args):
            with (output/(name+'.log')).open('w') as log:
                subprocess.run(['cargo', *args], env=env, stdout=log, stderr=subprocess.STDOUT,
                               timeout=300, check=True)
            commands.append(name)

        cargo('unit', ['test', '--locked', '-p', 'theia-persistence', '--lib', '-j', '2'])
        cargo('postgres-contract', ['test', '--locked', '-p', 'theia-persistence', '--test', 'postgres_contract',
                                    '-j', '2', '--', '--ignored'])
        cargo('worker-contract', ['test', '--locked', '-p', 'theia-persistence', '--test', 'worker_contract',
                                  '-j', '2', '--', '--ignored'])
        cargo('characters-contract', ['test', '--locked', '-p', 'theia-persistence', '--test', 'characters_contract',
                                      '-j', '2', '--', '--ignored'])
        cargo('write', ['run', '--locked', '-p', 'theia-persistence', '--example', 'store_qualification', '--', 'write'])
        cargo('character-write', ['run', '--locked', '-p', 'theia-persistence', '--example', 'character_qualification', '--', 'write'])
        cluster.restart()
        cargo('reload', ['run', '--locked', '-p', 'theia-persistence', '--example', 'store_qualification', '--', 'read'])
        cargo('character-reload', ['run', '--locked', '-p', 'theia-persistence', '--example', 'character_qualification', '--', 'read'])
        def character_probe(name):
            return next(json.loads(line.removeprefix('character_probe ')) for line in (output/(name+'.log')).read_text().splitlines() if line.startswith('character_probe '))
        assert character_probe('character-write') == character_probe('character-reload'), 'Character identity/state changed across actual database restart'

        version = subprocess.check_output([str(cluster.bin/'postgres'), '--version'], env=cluster.env, text=True).strip()
    assert cluster.server is None
    assert not (output/'cluster'/'password').exists() and not (output/'cluster'/'pgpass').exists()
    result = dict(passed=True, scope='PostgreSQL store/account/character boundaries; native game admission is qualified separately',
                  postgres=version, commands=commands, authenticated_loopback=True,
                  wrong_password_rejected=True, durability_flags=['fsync=on','synchronous_commit=on','full_page_writes=on'],
                  real_clean_restart_reload=True, named_character_identity_and_pose_survive_restart=True,
                  bounded_character_create_list_select=True, owned_cluster_stopped=True,
                  plaintext_password_files_removed=True)
    (output/'verification.json').write_text(json.dumps(result, indent=2)+'\n')
    return result


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('--sdk', type=Path, default=Path('build/postgres-sdk'))
    args = parser.parse_args()
    print(json.dumps(run(args.output, args.sdk.resolve()), indent=2))
