#!/usr/bin/env python3
"""Native secure-client save/reload proof against an owned PostgreSQL cluster."""
import argparse
import json
from pathlib import Path
import subprocess
import os
import re
import time
from qualify_graybox import stop
import uuid
from private_postgres import Cluster
from qualify_graybox import run as graybox

REALM = str(uuid.UUID('74686569-6100-0000-0000-000000000001'))


def rows(cluster):
    result = cluster.query(f"""SELECT coalesce(json_agg(r ORDER BY slot),'[]'::json)::text FROM (
        SELECT b.slot,c.character_id::text AS character,c.owner_id::text AS owner,
          encode(c.world_id,'hex') AS world,ARRAY[c.x,c.y,c.z] AS position,
          c.checkpoint_sequence AS sequence,c.writer_session::text AS writer
        FROM theia.development_bindings b JOIN theia.characters c
          ON b.realm_id=c.realm_id AND b.character_id=c.character_id
        WHERE b.realm_id='{REALM}'::uuid) r;""")
    assert result.returncode == 0, 'Owned checkpoint query failed'
    return json.loads(result.stdout)


def summary(output):
    text = (output/'server.log').read_text()
    stored = next(json.loads(line.removeprefix('persistence_summary ')) for line in text.splitlines() if line.startswith('persistence_summary '))
    assert not stored['failed'] and stored['source']['actual_actor_slots'] == 2, stored
    transport = next(json.loads(line.removeprefix('transport_summary ')) for line in text.splitlines() if line.startswith('transport_summary '))
    assert transport['max_players'] == 2 and transport['simulation']['missed_deadlines'] == 0, transport
    assert stored['worker']['committed_records'] >= 2 and not stored['worker']['failed'], stored
    admissions = [json.loads(line.removeprefix('persistent_admit ')) for line in text.splitlines() if line.startswith('persistent_admit ')]
    assert len(admissions) == 2
    return stored, {a['client_id']: a for a in admissions}


def compare_saved(stored, persisted):
    for actor in stored['positions']:
        record = next(row for row in persisted if row['slot'] == actor['client_id'])
        assert record['position'] == actor['position'], (record, actor)
        assert record['sequence'] == actor['sequence'] > 0, (record, actor)


def outage(output, cluster, env, binaries, players, terrain):
    output.mkdir()
    server = client = None
    logs = []
    try:
        server_log = (output/'server.log').open('w');logs.append(server_log)
        server = subprocess.Popen([str(binaries/'theia-server'), '127.0.0.1:0', str(output/'unused.der'),
                                   '--saved-terrain', str(terrain), '--prediction', '--dev-persistence',
                                   '--renet-tickets', str(output/'tickets')], env=env, stdout=server_log, stderr=subprocess.STDOUT)
        deadline = time.monotonic()+30
        while time.monotonic() < deadline:
            assert server.poll() is None, 'Outage authority failed before readiness'
            text = (output/'server.log').read_text()
            address = re.search(r'listening=(\S+) local_tickets=', text)
            if address: break
            time.sleep(.05)
        else: raise TimeoutError('Outage authority readiness')
        client_env = dict(env, THEIA_PERFORMANCE_UNCAPPED='1', THEIA_GRAYBOX_AUTOPLAY='1',
                          THEIA_GRAYBOX_AUTOPLAY_SPEED='1', THEIA_GRAYBOX_QUALIFY=str(output/'client'),
                          THEIA_GRAYBOX_QUALIFY_SECONDS='15', RUST_LOG='info')
        for key in ['VK_INSTANCE_LAYERS','VK_LOADER_DEBUG','WGPU_VALIDATION','WGPU_DEBUG']:
            client_env.pop(key, None)
        client_log = (output/'client.log').open('w');logs.append(client_log)
        client = subprocess.Popen([str(binaries/'theia-client'), '--saved-terrain', str(terrain), '--prediction',
                                   '--player-assets', str(players), '--renet-connect', address[1],
                                   str(output/'tickets/client-1.token')], env=client_env, stdout=client_log, stderr=subprocess.STDOUT)
        deadline = time.monotonic()+20
        while time.monotonic() < deadline:
            assert server.poll() is None and client.poll() is None
            if 'persistent_admit ' in (output/'server.log').read_text(): break
            time.sleep(.05)
        else: raise TimeoutError('Outage client admission')
        time.sleep(2)
        result = cluster.query("SELECT json_agg(pid)::text FROM pg_stat_activity WHERE application_name='theia-development-persistence';")
        assert result.returncode == 0
        pids = json.loads(result.stdout);assert len(pids) == 1, 'Never terminate an ambiguous backend'
        killed = cluster.query(f'SELECT pg_terminate_backend({int(pids[0])});')
        assert killed.returncode == 0 and killed.stdout.strip() == 't'
        assert server.wait(timeout=10) != 0, 'Persistent authority falsely succeeded after losing its DB owner'
        assert client.wait(timeout=25) == 0, 'Client did not exit its owned qualification cleanly'
        state = json.loads((output/'client/world.json').read_text())
        assert not state['connected'] and state['network_error'], state
        text = (output/'server.log').read_text()
        result = next(json.loads(line.removeprefix('persistence_summary ')) for line in text.splitlines() if line.startswith('persistence_summary '))
        assert result['failed'] and 'checkpoint worker failed' in text, result
        for path in [output/'server.log',output/'client.log']:
            assert cluster.password not in path.read_text()
        report = dict(passed=True, scope='Actual owned PostgreSQL backend loss, one secure native client',
                      server_failed_closed=True, client_observed_disconnect=True, credential_not_logged=True)
        (output/'verification.json').write_text(json.dumps(report,indent=2)+'\n')
        return report
    finally:
        for process in [client,server]:
            if process: stop(process)
        for log in logs: log.close()
        # Tickets were issued only by this owned server in this owned output.
        import shutil
        shutil.rmtree(output/'tickets', ignore_errors=True)


def run(output, sdk, binaries, players, terrain):
    output = output.resolve()
    output.mkdir(mode=0o700, parents=True, exist_ok=False)
    with Cluster(output/'cluster', sdk) as cluster:
        env = dict(os.environ, **cluster.test_environment())
        env.update(THEIA_DATABASE_URL=env['THEIA_TEST_DATABASE_URL'], THEIA_DEV_REALM_ID=REALM)
        first = graybox(output/'before', binaries, players, duration=35, clients=2, tour=False,
                        speed=1, prediction=True, impairment=True, saved_terrain=terrain,
                        persistence=True, spawn='15400,12860', reverse_clients=True, environment=env)
        saved_a, admitted_a = summary(output/'before')
        rows_a = rows(cluster)
        assert len(rows_a) == 8  # Seeded records are not connected players.
        compare_saved(saved_a, rows_a)
        assert saved_a['source']['seeded_records'] == 8 and saved_a['source']['loaded_records'] == 0
        assert admitted_a[2]['actor'] < admitted_a[1]['actor'], admitted_a
        cluster.restart()
        assert rows(cluster) == rows_a, 'Actual PostgreSQL restart changed committed character state'
        second = graybox(output/'after', binaries, players, duration=35, clients=2, tour=False,
                         speed=1, prediction=True, impairment=True, saved_terrain=terrain,
                         persistence=True, spawn='8000,8000', environment=env)
        saved_b, admitted_b = summary(output/'after')
        rows_b = rows(cluster)
        compare_saved(saved_b, rows_b)
        assert saved_b['source']['seeded_records'] == 0 and saved_b['source']['loaded_records'] == 8
        assert admitted_b[1]['actor'] < admitted_b[2]['actor'], admitted_b
        assert saved_a['source']['source_epoch'] != saved_b['source']['source_epoch']
        for slot in [1, 2]:
            old = next(r for r in rows_a if r['slot'] == slot)
            new = next(r for r in rows_b if r['slot'] == slot)
            assert (old['character'], old['owner'], old['world']) == (new['character'], new['owner'], new['world'])
            assert old['writer'] != new['writer'] and new['sequence'] > old['sequence']
            assert admitted_b[slot]['position'] == old['position'], (slot, admitted_b[slot], old)
            # Client's first observed authoritative pose must also be at the saved
            # distant location before the ordinary three-second autoplay delay.
            reports = [json.loads(line)['graybox'] for line in (output/'after'/f'client-{slot}.log').read_text().splitlines() if line.startswith('{"graybox":')]
            assert reports and reports[0]['seconds'] < 3
            assert max(abs(a-b) for a, b in zip(reports[0]['position'], old['position'])) < .05
        old = rows_b[0]
        altered = bytes([bytes.fromhex(old['world'])[0] ^ 1])+bytes.fromhex(old['world'])[1:]
        def set_world(value):
            result = cluster.query(f"UPDATE theia.characters SET world_id=decode('{value}','hex') WHERE realm_id='{REALM}'::uuid AND character_id='{old['character']}'::uuid;")
            assert result.returncode == 0
        try:
            set_world(altered.hex())
            with (output/'wrong-world.log').open('w') as log:
                code = subprocess.run([str(binaries/'theia-server'), '127.0.0.1:0', str(output/'unused.der'),
                                       '--saved-terrain', str(terrain), '--prediction', '--dev-persistence',
                                       '--renet-tickets', str(output/'rejected-tickets')], env=env,
                                      stdout=log, stderr=subprocess.STDOUT, timeout=30).returncode
            text = (output/'wrong-world.log').read_text()
            assert code != 0 and 'world content mismatch' in text and 'listening=' not in text
            assert not (output/'rejected-tickets').exists()
            assert cluster.password not in text
        finally:
            set_world(old['world'])
        fault = outage(output/'outage', cluster, env, binaries, players, terrain)
        result = dict(passed=True, outage=fault, scope='Two native actual encrypted clients; authoritative ordinary pose checkpoints/restart restore; development issuer only',
                      initial_spawn='15400,12860', restart_default_spawn='8000,8000',
                      actual_clients_per_phase=2, seeded_records=8, seeded_records_are_not_players=True,
                      reversed_admission_order=True, stable_persistent_ids=True, fresh_world_epoch=True,
                      server_and_postgres_restarted=True, exact_admission_restore=True,
                      first_client_pose_error_budget_metres=.05, wrong_world_rejected_before_listen=True,
                      before=first, after=second, source_before=saved_a, source_after=saved_b,
                      rows_before=rows_a, rows_after=rows_b)
    result['owned_postgres_stopped'] = cluster.server is None
    result['plaintext_password_files_removed'] = not (output/'cluster'/'pgpass').exists()
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
    result = run(args.output, args.sdk.resolve(), args.binaries.resolve(), args.players.resolve(), args.terrain.resolve())
    print(json.dumps({k:result[k] for k in ['passed','scope','server_and_postgres_restarted','wrong_world_rejected_before_listen','owned_postgres_stopped']}, indent=2))
