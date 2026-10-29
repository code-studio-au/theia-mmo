#!/usr/bin/env python3
"""Real OIDC -> committed PostgreSQL owner -> bounded session, including DB restart."""
import argparse
import json
import os
from pathlib import Path
import selectors
import subprocess
from private_identity import Identity
from private_postgres import Cluster


def run(output, identity_sdk, postgres_sdk):
    output.mkdir(mode=0o700, parents=True, exist_ok=False)
    with (output/'build.log').open('w') as log:
        subprocess.run(['cargo','build','--locked','-p','theia-server','--example','account_qualification','-j2'],
                       stdout=log, stderr=subprocess.STDOUT, check=True, timeout=600)
    child = None
    with Identity(output/'provider', identity_sdk) as identity, Cluster(output/'cluster', postgres_sdk) as cluster:
        previous = {}
        for phase in ('before', 'after'):
            if phase == 'after':
                cluster.restart()
            env = dict(cluster.test_environment(), THEIA_TEST_OIDC_ISSUER=identity.issuer,
                       THEIA_TEST_OIDC_CLIENT=identity.client_id, THEIA_TEST_OIDC_REDIRECT=identity.redirect)
            with (output/(phase+'-errors.log')).open('w') as errors:
                try:
                    child = subprocess.Popen(['target/debug/examples/account_qualification'], env=env,
                                             stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=errors, bufsize=0)
                    pending = b''
                    with selectors.DefaultSelector() as selector:
                        selector.register(child.stdout, selectors.EVENT_READ)
                        while True:
                            while b'\n' not in pending:
                                assert selector.select(timeout=20), 'Owned account qualifier timed out'
                                chunk = os.read(child.stdout.fileno(), 16384)
                                assert chunk, 'Account qualifier exited before summary'
                                pending += chunk
                                assert len(pending) <= 32768, 'Oversized private account message'
                            line, pending = pending.split(b'\n', 1)
                            message = json.loads(line)
                            if message.get('request') == 'prior_session':
                                response = previous
                            elif message.get('request') == 'authorization':
                                response = identity.authorize(message['authorization_url'])
                            else:
                                assert message['passed'] and message['revocation_checked'] and message['supersession_checked']
                                assert message['committed_owner_resolutions'] == 1 and not message['game_admission']
                                if previous:
                                    assert previous['owner'] == message['owner'], 'Verified owner changed across database restart'
                                if previous:
                                    assert previous['character'] == message['selected_character'], 'Owned character changed across database restart'
                                previous = dict(owner=message['owner'], credential=message['credential'], character=message['selected_character'])
                                break
                            child.stdin.write((json.dumps(response)+'\n').encode());child.stdin.flush()
                    assert child.wait(timeout=10) == 0
                finally:
                    if child is not None:
                        if child.poll() is None:
                            child.terminate()
                            try:
                                child.wait(timeout=5)
                            except subprocess.TimeoutExpired:
                                child.kill();child.wait()
                        child.stdin.close();child.stdout.close()
            text = (output/(phase+'-errors.log')).read_text()
            assert all(secret not in text for secret in (cluster.password, identity.password, previous['credential']))
            owners = cluster.query("SELECT count(*) FROM pg_stat_activity WHERE application_name='theia-development-persistence';")
            assert owners.returncode == 0 and owners.stdout.strip() == '0'
        count = cluster.query('SELECT (SELECT count(*) FROM theia.accounts), (SELECT count(*) FROM theia.characters);')
        assert count.returncode == 0 and count.stdout.strip() == '1|1'
    result = dict(passed=True, scope='Actual OIDC verification, committed owner/session and owned character selection; no native game admission',
                  real_provider_and_database=True, stable_owner_across_database_and_process_restart=True,
                  old_process_session_rejected=True, superseded_and_revoked_sessions_rejected=True,
                  committed_owner_acknowledgement=True, game_admission=False,
                  actual_accounts=1, actual_characters=1,
                  session_owner_character_selection=True, stable_character_across_database_restart=True,
                  no_remaining_database_owner_connections=True,
                  owned_provider_stopped=identity.server is None, owned_postgres_stopped=cluster.server is None,
                  plaintext_fixture_files_removed=not identity.realm_file.exists() and not (output/'cluster/pgpass').exists())
    (output/'verification.json').write_text(json.dumps(result, indent=2)+'\n')
    return result


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('--identity-sdk', type=Path, default=Path('build/identity-sdk'))
    parser.add_argument('--postgres-sdk', type=Path, default=Path('build/postgres-sdk'))
    args = parser.parse_args()
    print(json.dumps(run(args.output.resolve(), args.identity_sdk.resolve(), args.postgres_sdk.resolve()), indent=2))
