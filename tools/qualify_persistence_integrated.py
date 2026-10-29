#!/usr/bin/env python3
"""Actual persistence/combat/UI/feedback and launcher cleanup in an owned database."""
import argparse
import json
from pathlib import Path
from private_postgres import Cluster
from qualify_graybox import run as graybox
from verify_launcher import run as launcher


def run(output, sdk, binaries, players, terrain):
    output.mkdir(mode=0o700, parents=True, exist_ok=False)
    with Cluster(output/'cluster', sdk) as cluster:
        env = dict(cluster.env, THEIA_DATABASE_URL=cluster.test_environment()['THEIA_TEST_DATABASE_URL'],
                   THEIA_DEV_REALM_ID='74686569-6100-0000-0000-000000000002',
                   THEIA_FEEDBACK_STRESS='1')
        native = graybox(output/'combat', binaries, players, duration=35, clients=2,
                         tour=False, validation=True, speed=1, prediction=True,
                         impairment=True, saved_terrain=terrain, targeting=True,
                         combat_catalog=Path('content/gameplay/development-abilities.json').resolve(),
                         feedback=Path('content/gameplay/development-spell-feedback.json').resolve(),
                         action_ui=True, persistence=True, environment=env)
        launcher(output/'launcher-persistent', binaries, terrain, players,
                 use_default_world=True, feedback=True, persistence=True, environment=env)
        owners = cluster.query("SELECT count(*) FROM pg_stat_activity WHERE application_name='theia-development-persistence';")
        assert owners.returncode == 0 and owners.stdout.strip() == '0'
    result = dict(passed=True, scope='Persistent secure two-client mobile combat/UI/critical feedback plus actual launcher cleanup',
                  native=native, owned_cluster_stopped_on_exit=cluster.server is None,
                  no_database_owner_connections_after_launcher=True,
                  plaintext_password_files_removed=not (output/'cluster/pgpass').exists())
    assert result['owned_cluster_stopped_on_exit'] and result['plaintext_password_files_removed']
    (output/'verification.json').write_text(json.dumps(result, indent=2)+'\n')
    return result


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('--sdk', type=Path, default=Path('build/postgres-sdk'))
    parser.add_argument('--binaries', type=Path, default=Path('target/release'))
    parser.add_argument('--players', type=Path, default=Path('build/synty-character-canonical-stage'))
    parser.add_argument('--terrain', type=Path, default=Path('build/graybox-terrain-v1'))
    args = parser.parse_args()
    result = run(args.output.resolve(), args.sdk.resolve(), args.binaries.resolve(), args.players.resolve(), args.terrain.resolve())
    print(json.dumps({k:v for k,v in result.items() if k != 'native'}, indent=2))
