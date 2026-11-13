#!/usr/bin/env python3
"""Headless population/SQL receipt qualification in an owned disposable cluster."""
import argparse
import json
from pathlib import Path
import subprocess
from private_postgres import Cluster


def run(output, sdk, binary, package):
    output = output.resolve()
    output.mkdir(mode=0o700, parents=True, exist_ok=False)
    with Cluster(output / 'cluster', sdk.resolve()) as cluster:
        durability = cluster.query('SHOW fsync; SHOW synchronous_commit; SHOW full_page_writes;')
        assert durability.returncode == 0 and durability.stdout.split() == ['on', 'on', 'on']
        env = cluster.test_environment()
        for key in ('DISPLAY', 'WAYLAND_DISPLAY'):
            env.pop(key, None)
        env['THEIA_POPULATION_PACKAGE'] = str(package.resolve())
        with (output / 'world-postgres.log').open('w') as log:
            subprocess.run([str(binary.resolve())], env=env, stdout=log,
                           stderr=subprocess.STDOUT, timeout=120, check=True)
        records = [json.loads(line) for line in (output / 'world-postgres.log').read_text().splitlines()
                   if line.startswith('{')]
        result = next(record for record in records if record.get('population_receipts'))
        assert result['passed'] and result['distinct_lives'] == 10
        assert result['shared_player_life'] and result['player_health'] == 100
        cluster.restart()
        query = cluster.query('''
            SELECT n.display_name,p.experience,q.revision
            FROM theia.character_names n
            JOIN theia.progression p USING(realm_id,character_id)
            JOIN theia.quest_journal_heads q USING(realm_id,character_id);
            SELECT count(*) FROM theia.experience_receipts;
            SELECT count(*) FROM theia.quest_journal_events;
        ''')
        assert query.returncode == 0, query.stderr
        assert query.stdout.splitlines() == ['Populationowner|250|11', '10', '10'], query.stdout
        (output / 'restart-rows.txt').write_text(query.stdout)
        result.update(postgres_process_restart_verified=True, fsync=True,
                      synchronous_commit=True, full_page_writes=True)
    assert not any((output / 'cluster' / name).exists() for name in ('password', 'pgpass'))
    result['plaintext_password_files_removed'] = True
    (output / 'verification.json').write_text(json.dumps(result, indent=2) + '\n')
    return result


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('--sdk', type=Path, default=Path('build/postgres-sdk'))
    parser.add_argument('--binary', type=Path,
                        default=Path('target/debug/examples/population_durable_qualification'))
    parser.add_argument('--package', type=Path, required=True,
                        help='Existing checked five-actor passive population export')
    args = parser.parse_args()
    print(json.dumps(run(args.output, args.sdk, args.binary, args.package), indent=2))
