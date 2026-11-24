#!/usr/bin/env python3
"""Normal schema-21 server CLI, real OIDC/SQL and HTTP ticket preflight, no game window.

The ticket is deliberately not consumed by a game client. This establishes
configuration and preflight only, not encrypted gameplay, item rewards or UI.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import time
import urllib.error
import urllib.parse
import urllib.request
from gameplay_bundle import snapshot
from private_identity import Identity, _NoRedirect
from private_postgres import Cluster

ROLE_FLAGS = {
    'items': '--item-catalog', 'abilities': '--combat-catalog',
    'stats': '--character-stats', 'statuses': '--status-catalog',
    'creature': '--creature-profile', 'dialogue': '--dialogue-catalog',
    'vendor': '--vendor-catalog', 'rewards': '--reward-catalog',
    'progression': '--progression-catalog', 'quest': '--quest-catalog',
    'creature_retaliation': '--creature-retaliation', 'player_respawn': '--player-respawn',
    'melee_weapons': '--melee-weapons', 'physical_definitions': '--physical-definitions',
    'physical_stats': '--physical-stats', 'white_melee': '--white-melee',
    'ability_starting': '--ability-starting', 'ability_damage': '--ability-damage',
    'combat_membership': '--combat-membership', 'creature_credit': '--creature-credit',
    'creature_reset': '--creature-reset', 'creature_threat': '--creature-threat',
    'quest_areas': '--quest-areas', 'documents': '--document-catalog',
    'quest_item_rewards': '--quest-item-rewards',
}


def run(args):
    assert sys.platform == "linux", "Owned server process qualification is Linux-only"
    output = args.output.resolve()
    output.mkdir(mode=0o700, parents=True, exist_ok=False)
    child = None
    with snapshot(args.manifest, args.validator, output/'snapshot') as (paths, report):
        assert report['manifest_schema'] == 21 and set(paths) == set(ROLE_FLAGS)
        sealed_hashes = {role: hashlib.sha256(path.read_bytes()).hexdigest() for role, path in paths.items()}
        # Run the actual normal client parser and identity calculation, returning
        # before Network::from_args, App construction, winit or GPU startup.
        client_env = {k: v for k, v in os.environ.items()
                      if k.upper() not in ('DISPLAY', 'WAYLAND_DISPLAY', 'XDG_RUNTIME_DIR')
                      and not k.upper().startswith(('THEIA_', 'PG'))}
        client_env['LD_LIBRARY_PATH'] = str(Path('build/native-sdk/usr/lib/x86_64-linux-gnu').resolve())+':'+client_env.get('LD_LIBRARY_PATH', '')
        client_command = [str(args.client.resolve()), '--configuration-report',
                          '--saved-terrain', str(args.terrain.resolve()), '--navigation', str(args.navigation.resolve()),
                          '--prediction', '--classic', '--quest-journal', '--quest-items', '--quest-loot',
                          '--creature-loot', '--public-creature-attacks', '--public-avatar-life', '--guide-position', '8004,8000']
        for role, flag in ROLE_FLAGS.items():
            client_command += [flag, str(paths[role])]
        inspected = subprocess.run(client_command, env=client_env, capture_output=True, text=True, timeout=30)
        (output/'client-config-errors.log').write_text(inspected.stderr)
        assert inspected.returncode == 0, 'Client configuration preflight failed; inspect owned error log'
        client_report = json.loads(inspected.stdout)
        assert client_report['schema'] == 1 and not any(client_report[k] for k in ('renderer_started', 'network_started', 'saved_state_admitted'))
        assert client_report['fixed_reward_catalog'] == json.loads(paths['quest_item_rewards'].read_text())['id']
        (output/'client-config.json').write_text(inspected.stdout)
        with Identity(output/'provider', args.identity_sdk) as identity, Cluster(output/'cluster', args.postgres_sdk) as cluster:
            env = {k: v for k, v in cluster.env.items()
                   if k.upper() not in ('DISPLAY', 'WAYLAND_DISPLAY', 'XDG_RUNTIME_DIR')
                   and not k.upper().startswith('THEIA_')}
            env.update(THEIA_DATABASE_URL=cluster.test_environment()['THEIA_TEST_DATABASE_URL'],
                       THEIA_OIDC_ISSUER=identity.issuer, THEIA_OIDC_CLIENT_ID=identity.client_id,
                       THEIA_ACCOUNT_BIND=urllib.parse.urlsplit(identity.redirect).netloc)
            env['LD_LIBRARY_PATH'] = str(Path('build/native-sdk/usr/lib/x86_64-linux-gnu').resolve())+':'+env.get('LD_LIBRARY_PATH', '')
            command = [str(args.server.resolve()), '127.0.0.1:0', str(output/'unused-cert.der'),
                       '--saved-terrain', str(args.terrain.resolve()), '--navigation', str(args.navigation.resolve()),
                       '--prediction', '--classic', '--quest-journal', '--quest-items', '--quest-loot',
                       '--creature-loot', '--public-creature-attacks', '--public-avatar-life',
                       '--guide-position', '8004,8000', '--dev-spawn', '8000,8000', '--tick-hz', '30', '--account-login']
            for role, flag in ROLE_FLAGS.items():
                command += [flag, str(paths[role])]
            secrets = [cluster.password, identity.password]
            with (output/'server.log').open('w') as log:
                try:
                    child = subprocess.Popen(command, env=env, stdout=log, stderr=subprocess.STDOUT)
                    deadline = time.monotonic()+40
                    while time.monotonic() < deadline:
                        assert child.poll() is None, 'Normal server exited before readiness; inspect owned server.log'
                        ready = next((json.loads(line.removeprefix('account_game_ready '))
                                      for line in (output/'server.log').read_text().splitlines()
                                      if line.startswith('account_game_ready ')), None)
                        if ready:
                            break
                        time.sleep(.05)
                    else:
                        raise TimeoutError('Normal headless server configuration timed out')
                    assert ready['world'] == client_report['world'], 'Normal client/server content identities disagree'
                    base = ready['account_address']
                    assert urllib.parse.urlsplit(base).netloc == urllib.parse.urlsplit(identity.redirect).netloc
                    opener = urllib.request.build_opener(urllib.request.ProxyHandler({}), _NoRedirect())

                    def http(path, body=None, session=None, expected=200):
                        headers = {'Content-Type': 'application/json'}
                        if session:
                            headers['Authorization'] = 'Bearer '+session
                        request = urllib.request.Request(base+path, data=json.dumps(body).encode() if body is not None else None,
                                                         headers=headers, method='POST' if body is not None else 'GET')
                        try:
                            response = opener.open(request, timeout=17)
                        except urllib.error.HTTPError as error:
                            response = error
                        with response:
                            raw = response.read(65537)
                            assert response.code == expected and len(raw) <= 65536, 'Unexpected owned HTTP response'
                            return json.loads(raw) if raw.startswith(b'{') else None

                    flow = http('/auth/start', {})
                    secrets += [flow['proof'], flow['authorization_url']]
                    callback = identity.authorize(flow['authorization_url'])
                    secrets.append(callback)
                    secrets.extend(urllib.parse.parse_qs(urllib.parse.urlsplit(callback).query).get("code", []))
                    http('/callback?'+urllib.parse.urlsplit(callback).query)
                    login = http('/auth/poll', {'challenge': flow['challenge'], 'proof': flow['proof']})
                    session = login['session']
                    secrets.append(session)
                    character = http('/characters', {'name': 'Rewardconfig'}, session)
                    http('/characters/enter', {'character_id': character['id']}, expected=401)
                    ticket = http('/characters/enter', {'character_id': character['id']}, session)
                    assert ticket['id'] == character['id'] and ticket['expires_in_seconds'] == 20
                    assert isinstance(ticket['ticket'], list) and len(ticket['ticket']) > 512
                    secrets += [json.dumps(ticket['ticket']), json.dumps(ticket['ticket'], separators=(',', ':'))]
                    http('/characters/enter', {'character_id': character['id']}, session, expected=409)
                    counts = cluster.query('SELECT (SELECT count(*) FROM theia.accounts),(SELECT count(*) FROM theia.characters),(SELECT count(*) FROM theia.backpacks),(SELECT count(*) FROM theia.progression),(SELECT count(*) FROM theia.quest_journal_heads),(SELECT count(*) FROM theia.quest_journal_actions);')
                    assert counts.returncode == 0 and counts.stdout.strip() == '1|1|0|0|0|0'
                    assert sealed_hashes == {role: hashlib.sha256(path.read_bytes()).hexdigest() for role, path in paths.items()}
                    child.send_signal(signal.SIGINT)
                    assert child.wait(timeout=15) == 0, 'Normal server did not shut down cleanly'
                    text = (output/'server.log').read_text()
                    summary = next(json.loads(line.removeprefix('account_persistence_summary '))
                                   for line in text.splitlines() if line.startswith('account_persistence_summary '))
                    assert not summary['failed'] and not summary['worker']['failed']
                    assert summary['worker']['gameplay_profile_checks'] == 1
                    assert all(summary['worker'][key] == 0 for key in ('inventory_ensures', 'progression_ensures', 'journal_ensures', 'journal_commands', 'inventory_committed'))
                    assert not any(secret in text for secret in secrets)
                    assert not (output/'unused-cert.der').exists()
                finally:
                    if child is not None and child.poll() is None:
                        child.send_signal(signal.SIGINT)
                        try:
                            child.wait(timeout=15)
                        except subprocess.TimeoutExpired:
                            child.kill()
                            child.wait()
        result = dict(passed=True, scope='normal server configuration and HTTP ticket preflight only',
                      manifest_schema=21, exact_roles=len(paths), bundle_sha256=report['validation_bundle_sha256'],
                      real_oidc_and_postgres=True, normal_cli_and_reward_owner_configuration=True,
                      actual_client_configuration_only=True, client_server_world_identities_match=True,
                      client_sha256=hashlib.sha256(args.client.read_bytes()).hexdigest(),
                      unauthorized_ticket_refused=True, issued_reservation_blocks_duplicate=True,
                      private_ticket_not_written=True, gameplay_ticket_consumed=False,
                      economic_state_created=False, item_reward_or_native_gameplay_qualified=False,
                      server_sha256=hashlib.sha256(args.server.read_bytes()).hexdigest(),
                      owned_services_stopped=identity.server is None and cluster.server is None)
    assert not list((output/'snapshot').glob('gameplay-*'))
    result['sealed_snapshot_removed'] = True
    (output/'verification.json').write_text(json.dumps(result, indent=2)+'\n')
    return result


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('--client', type=Path, default=Path('target/debug/theia-client'))
    parser.add_argument('--server', type=Path, default=Path('target/debug/theia-server'))
    parser.add_argument('--validator', type=Path, default=Path('target/debug/examples/validate_gameplay'))
    parser.add_argument('--manifest', type=Path, default=Path('content/development-gameplay-classic-item-rewards-fixture.json'))
    parser.add_argument('--terrain', type=Path, default=Path('build/graybox-terrain-v1'))
    parser.add_argument('--navigation', type=Path, default=Path('build/navigation-terrain-v1'))
    parser.add_argument('--identity-sdk', type=Path, default=Path('build/identity-sdk'))
    parser.add_argument('--postgres-sdk', type=Path, default=Path('build/postgres-sdk'))
    print(json.dumps(run(parser.parse_args()), indent=2))
