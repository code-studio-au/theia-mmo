#!/usr/bin/env python3
"""Owned public-World/PostgreSQL solo-credit qualification; no native/OIDC claim."""
import argparse
import json
from pathlib import Path
import subprocess
from private_postgres import Cluster


def run(output, sdk, binary, terrain, navigation, population=None, threat=False, tick_hz=30):
    if threat and population is None:
        raise ValueError('Threat receipts require an explicit pursuing population package')
    if tick_hz not in (20, 30, 60):
        raise ValueError('Unsupported simulation rate')
    output = output.resolve()
    output.mkdir(mode=0o700, parents=True, exist_ok=False)
    with Cluster(output/'cluster', sdk.resolve()) as cluster:
        durability = cluster.query('SHOW fsync; SHOW synchronous_commit; SHOW full_page_writes;')
        assert durability.returncode == 0 and durability.stdout.split() == ['on', 'on', 'on']
        env = cluster.test_environment()
        for key in ('DISPLAY', 'WAYLAND_DISPLAY', 'THEIA_POPULATION_PACKAGE', 'THEIA_CREDIT_THREAT', 'THEIA_CREDIT_TICK_HZ'):
            env.pop(key, None)
        env['THEIA_CREDIT_TICK_HZ'] = str(tick_hz)
        if threat:
            env['THEIA_CREDIT_THREAT'] = '1'
        if population is not None:
            env['THEIA_POPULATION_PACKAGE'] = str(population.resolve())
        else:
            env.update(THEIA_TEST_SAVED_TERRAIN=str(terrain.resolve()),
                       THEIA_TEST_NAVIGATION=str(navigation.resolve()))
        with (output/'world-postgres.log').open('w') as log:
            subprocess.run([str(binary.resolve())], env=env, stdout=log,
                           stderr=subprocess.STDOUT, timeout=90, check=True)
        records = [json.loads(line) for line in (output/'world-postgres.log').read_text().splitlines()
                   if line.startswith('{')]
        result = next(record for record in records if record.get('actual_world_competing_hitter'))
        assert result['passed']
        assert result['population_first_tag'] == (population is not None)
        assert result['population_actors'] == (5 if population is not None else 1)
        assert result['defeated_lives'] == 1
        assert result['damage_threat'] == threat and result['tick_hz'] == tick_hz
        if threat:
            assert all(result['threat_proof'].values())
            assert result['threat_metrics']['transfers'] > 0
            assert result['threat_metrics']['health_lost'] == 100
        # Both the qualifier process and PostgreSQL process have ended before
        # these persisted recipient/receipt rows are read again.
        cluster.restart()
        query = cluster.query('''
            SELECT n.display_name,p.experience,q.kill_count,
              (SELECT count(*) FROM theia.reward_claims r WHERE r.realm_id=n.realm_id AND r.character_id=n.character_id)
            FROM theia.character_names n
            JOIN theia.progression p USING(realm_id,character_id)
            JOIN theia.quest_lifecycles q USING(realm_id,character_id)
            ORDER BY n.display_name;
            SELECT count(*) FROM theia.experience_receipts;
            SELECT count(*) FROM theia.quest_lifecycle_kills;
            SELECT count(*) FROM theia.reward_claims WHERE consumed_revision IS NOT NULL;
        ''')
        assert query.returncode == 0, query.stderr
        assert query.stdout.splitlines() == ['Creditkiller|0|0|0', 'Creditowner|25|1|1', '1', '1', '0'], query.stdout
        (output/'restart-rows.txt').write_text(query.stdout)
        result.update(postgres_process_restart_verified=True, fsync=True,
                      synchronous_commit=True, full_page_writes=True)
    assert not any((output/'cluster'/name).exists() for name in ['password', 'pgpass'])
    result['plaintext_password_files_removed'] = True
    (output/'verification.json').write_text(json.dumps(result, indent=2)+'\n')
    return result


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('--sdk', type=Path, default=Path('build/postgres-sdk'))
    parser.add_argument('--binary', type=Path, default=Path('target/debug/examples/creature_credit_durable_qualification'))
    parser.add_argument('--population', type=Path, help='Checked five-actor population package instead of the saved-terrain patrol fixture')
    parser.add_argument('--threat', action='store_true', help='Require pursuing population victim transfer with unchanged first-tag durable recipients')
    parser.add_argument('--tick-hz', type=int, choices=(20,30,60), default=30)
    parser.add_argument('--terrain', type=Path, default=Path('build/graybox-terrain-v1'))
    parser.add_argument('--navigation', type=Path, default=Path('build/navigation-terrain-v1'))
    args = parser.parse_args()
    print(json.dumps(run(args.output, args.sdk, args.binary, args.terrain, args.navigation, args.population, args.threat, args.tick_hz), indent=2))
