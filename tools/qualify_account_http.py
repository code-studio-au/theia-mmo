#!/usr/bin/env python3
"""Owned real HTTP/OIDC/database account API checks; no native game admission."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import signal
import socket
import subprocess
import time
import urllib.error
import urllib.parse
import urllib.request
from private_identity import Identity, _NoRedirect
from private_postgres import Cluster


def run(output, identity_sdk, postgres_sdk):
    output.mkdir(mode=0o700, parents=True, exist_ok=False)
    with (output/'build.log').open('w') as log:
        subprocess.run(['cargo','build','--locked','-p','theia-server','--example','account_service_qualification','-j2'],
                       stdout=log, stderr=subprocess.STDOUT, check=True, timeout=600)
    binary_sha256 = hashlib.sha256(Path('target/debug/examples/account_service_qualification').read_bytes()).hexdigest()
    child = None
    private = []
    with Identity(output/'provider', identity_sdk, users=2) as identity, Cluster(output/'cluster', postgres_sdk) as cluster:
        env = dict(cluster.test_environment(), THEIA_TEST_OIDC_ISSUER=identity.issuer,
                   THEIA_TEST_OIDC_CLIENT=identity.client_id,
                   THEIA_TEST_ACCOUNT_BIND=urllib.parse.urlsplit(identity.redirect).netloc)
        private += [cluster.password]+[password for _,password in identity.users]
        with (output/'service.log').open('w') as log:
            try:
                child = subprocess.Popen(['target/debug/examples/account_service_qualification'], env=env,
                                         stdout=log, stderr=subprocess.STDOUT)
                deadline = time.monotonic()+30
                while time.monotonic() < deadline:
                    assert child.poll() is None, 'Owned HTTP service exited before readiness'
                    lines = (output/'service.log').read_text().splitlines()
                    ready = next((json.loads(line.removeprefix('account_service_ready ')) for line in lines if line.startswith('account_service_ready ')),None)
                    if ready:
                        break
                    time.sleep(.05)
                else:
                    raise TimeoutError('Owned HTTP service readiness timeout')
                base = ready['address']
                opener = urllib.request.build_opener(urllib.request.ProxyHandler({}), _NoRedirect())

                def http(path, *, method='GET', body=None, session=None, headers=None, expected=200):
                    data = json.dumps(body).encode() if body is not None else None
                    values = {'Content-Type':'application/json'}
                    if session:
                        values['Authorization'] = 'Bearer '+session
                    values.update(headers or {})
                    request = urllib.request.Request(base+path, data=data, headers=values, method=method)
                    try:
                        response = opener.open(request, timeout=17)
                    except urllib.error.HTTPError as error:
                        response = error
                    with response:
                        status = response.code
                        raw = response.read(65537)
                        assert len(raw) <= 65536, 'Oversized account response'
                        assert response.headers.get('Cache-Control') == 'no-store', 'Sensitive response was cacheable'
                    assert status == expected, 'Unexpected account HTTP status'
                    if not raw:
                        return None
                    try:
                        return json.loads(raw)
                    except json.JSONDecodeError:
                        return raw.decode()

                def login(user=0, *, bad_nonce=False):
                    start = http('/auth/start', method='POST', body={})
                    private.extend([start['proof'],start['authorization_url']])
                    poll = dict(challenge=start['challenge'],proof=start['proof'])
                    http('/auth/poll', method='POST', body=dict(poll,proof='0'*64), expected=401)
                    http('/auth/poll', method='POST', body=poll, expected=202)
                    url = start['authorization_url']
                    if bad_nonce:
                        parsed = urllib.parse.urlsplit(url)
                        pairs = urllib.parse.parse_qsl(parsed.query)
                        url = urllib.parse.urlunsplit(parsed._replace(query=urllib.parse.urlencode([(key,'synthetic-wrong-nonce' if key=='nonce' else value) for key,value in pairs])))
                    callback = identity.authorize(url,user_index=user)
                    private.append(callback)
                    private.extend(urllib.parse.parse_qs(urllib.parse.urlsplit(callback).query).get('code', []))
                    query = urllib.parse.urlsplit(callback).query
                    http('/callback?'+query, expected=401 if bad_nonce else 200)
                    if bad_nonce:
                        http('/auth/poll', method='POST', body=poll, expected=401)
                        return None
                    http('/callback?'+query, expected=400)
                    result = http('/auth/poll', method='POST', body=poll)
                    http('/auth/poll', method='POST', body=poll, expected=401)
                    private.append(result['session'])
                    return result['session']

                a = login(0);b = login(1)
                assert a != b
                assert http('/characters',session=a)['characters'] == []
                arden = http('/characters',method='POST',body={'name':'Arden'},session=a)
                selene = http('/characters',method='POST',body={'name':'Selene'},session=b)
                assert arden['id'] != selene['id']
                assert [c['id'] for c in http('/characters',session=a)['characters']] == [arden['id']]
                http('/characters/select',method='POST',body={'character_id':arden['id']},session=b,expected=404)
                selected = http('/characters/select',method='POST',body={'character_id':arden['id']},session=a)
                assert selected['id'] == arden['id'] and not selected['game_admission']
                http('/characters',method='POST',body={'name':'ARDEN'},session=b,expected=409)
                for body in ({'name':'-bad'},{'name':'Spoofed','owner':'client-claim'}, {'name':'Spoofed','position':[1,2,3]}):
                    http('/characters',method='POST',body=body,session=b,expected=400)
                http('/characters',method='POST',body={'name':'x'*5000},session=b,expected=413)
                http('/characters/select',method='POST',body={'character_id':'00000000-0000-0000-0000-000000000000'},session=a,expected=400)
                http('/characters',session='0'*64,expected=401)
                http('/auth/start',method='POST',body={},headers={'Host':'wrong.invalid'},expected=400)
                http('/auth/start',method='POST',body={},headers={'Origin':'http://wrong.invalid'},expected=403)
                login(0,bad_nonce=True)
                replacement = login(0)
                http('/characters',session=a,expected=401)
                assert http('/characters',session=replacement)['characters'][0]['id'] == arden['id']
                for i in range(7):
                    http('/characters',method='POST',body={'name':'Adventurer'+chr(ord('A')+i)},session=replacement)
                http('/characters',method='POST',body={'name':'Overflow'},session=replacement,expected=409)
                assert len(http('/characters',session=replacement)['characters']) == 8
                assert len(http('/characters',session=b)['characters']) == 1
                # Cancellation needs the private challenge proof. A verified
                # callback alone must not create/supersede an account session.
                cancelled = http('/auth/start',method='POST',body={})
                private += [cancelled['proof'],cancelled['authorization_url']]
                proof = dict(challenge=cancelled['challenge'],proof=cancelled['proof'])
                http('/auth/cancel',method='POST',body=dict(proof,proof='0'*64),expected=401)
                http('/auth/cancel',method='POST',body=proof,expected=204)
                callback = identity.authorize(cancelled['authorization_url'])
                private.append(callback)
                http('/callback?'+urllib.parse.urlsplit(callback).query,expected=400)
                http('/auth/poll',method='POST',body=proof,expected=401)
                cancelled = http('/auth/start',method='POST',body={})
                private += [cancelled['proof'],cancelled['authorization_url']]
                callback = identity.authorize(cancelled['authorization_url'])
                private.append(callback)
                http('/callback?'+urllib.parse.urlsplit(callback).query)
                assert http('/development/auth-metrics')['active_account_sessions'] == 2
                assert len(http('/characters',session=replacement)['characters']) == 8
                http('/auth/cancel',method='POST',body=dict(challenge=cancelled['challenge'],proof=cancelled['proof']),expected=204)
                assert len(http('/characters',session=replacement)['characters']) == 8
                http('/auth/logout',method='POST',body={},session=replacement,expected=204)
                http('/characters',session=replacement,expected=401)

                # Leave 16 independent challenges pending, then reject overflow.
                for _ in range(16):
                    challenge = http('/auth/start',method='POST',body={})
                    private += [challenge['proof'],challenge['authorization_url']]
                http('/auth/start',method='POST',body={},expected=503)
                assert http('/development/auth-metrics')['pending_logins'] == 16

                address = urllib.parse.urlsplit(base)
                target = (address.hostname,address.port)
                held = []
                try:
                    # Body readers hold the 16 request permits; a seventeenth
                    # complete request must be shed rather than queued.
                    for _ in range(16):
                        raw = socket.create_connection(target,timeout=2)
                        raw.sendall((f'POST /auth/start HTTP/1.1\r\nHost: {address.netloc}\r\nContent-Type: application/json\r\nContent-Length: 2\r\n\r\n'+'{').encode())
                        held.append(raw)
                    time.sleep(.2)
                    http('/development/auth-metrics',expected=503)
                finally:
                    for raw in held:
                        raw.close()
                time.sleep(.2)
                held = []
                try:
                    for _ in range(65):
                        held.append(socket.create_connection(target,timeout=2))
                    time.sleep(.2)
                finally:
                    for raw in held:
                        raw.close()
                time.sleep(.2)
                # Measure the actual header and whole-connection deadlines.
                started = time.monotonic()
                with socket.create_connection(target,timeout=4) as raw:
                    raw.settimeout(4)
                    assert raw.recv(1) == b''
                header_seconds = time.monotonic()-started
                assert 1.5 <= header_seconds < 4
                started = time.monotonic()
                with socket.create_connection(target,timeout=17) as raw:
                    raw.settimeout(17)
                    raw.sendall((f'POST /auth/start HTTP/1.1\r\nHost: {address.netloc}\r\nContent-Type: application/json\r\nContent-Length: 2\r\n\r\n'+'{').encode())
                    assert raw.recv(1) == b''
                body_seconds = time.monotonic()-started
                assert 14 <= body_seconds < 17
                metrics = http('/development/auth-metrics')
                assert metrics['peak_connections'] <= 64 and metrics['refused_connections'] >= 1
                assert metrics['overloaded_requests'] >= 1 and metrics['host_rejections'] >= 1 and metrics['origin_rejections'] >= 1
                assert metrics['verified_logins'] == 4 and metrics['failed_logins'] >= 1
                assert metrics['cancelled_challenges'] == 2
                assert metrics['created_characters'] == 9 and metrics['selected_characters'] == 1
                counts = cluster.query('SELECT (SELECT count(*) FROM theia.accounts),(SELECT count(*) FROM theia.characters);')
                assert counts.returncode == 0 and counts.stdout.strip() == '2|9'

                # Backend death must close an idle service without a new request.
                pids = cluster.query("SELECT json_agg(pid)::text FROM pg_stat_activity WHERE application_name='theia-development-persistence';")
                assert pids.returncode == 0
                owned = json.loads(pids.stdout);assert len(owned) == 1
                killed = cluster.query(f'SELECT pg_terminate_backend({int(owned[0])});')
                assert killed.returncode == 0 and killed.stdout.strip() == 't'
                assert child.wait(timeout=10) != 0, 'Account API survived loss of its database owner'
                text = (output/'service.log').read_text()
                summary = next(json.loads(line.removeprefix('account_service_summary ')) for line in text.splitlines() if line.startswith('account_service_summary '))
                assert summary['failed'] and not summary['game_admission']
            finally:
                if child is not None and child.poll() is None:
                    child.send_signal(signal.SIGINT)
                    try:
                        child.wait(timeout=10)
                    except subprocess.TimeoutExpired:
                        child.kill();child.wait()
        # Restart the database and API on the same endpoint. Process-local
        # sessions must expire while durable owner/character identity survives.
        cluster.restart()
        with (output/'restart-service.log').open('w') as log:
            try:
                child = subprocess.Popen(['target/debug/examples/account_service_qualification'], env=env,
                                         stdout=log, stderr=subprocess.STDOUT)
                deadline = time.monotonic()+30
                while time.monotonic() < deadline:
                    assert child.poll() is None, 'Account API failed to restart'
                    if 'account_service_ready ' in (output/'restart-service.log').read_text():
                        break
                    time.sleep(.05)
                else:
                    raise TimeoutError('Account API restart readiness timeout')
                http('/characters',session=b,expected=401)
                fresh = login(0)
                entries = http('/characters',session=fresh)['characters']
                assert len(entries) == 8 and any(entry['id'] == arden['id'] for entry in entries)
                child.send_signal(signal.SIGINT)
                assert child.wait(timeout=10) == 0, 'Graceful account API shutdown failed'
                text = (output/'restart-service.log').read_text()
                restarted = next(json.loads(line.removeprefix('account_service_summary ')) for line in text.splitlines() if line.startswith('account_service_summary '))
                assert not restarted['failed'] and not restarted['game_admission']
                assert not restarted['worker']['failed']
            finally:
                if child is not None and child.poll() is None:
                    child.send_signal(signal.SIGINT)
                    try:
                        child.wait(timeout=10)
                    except subprocess.TimeoutExpired:
                        child.kill();child.wait()
        for path in (output/'service.log',output/'restart-service.log',output/'provider/server.log'):
            text = path.read_text()
            assert all(secret not in text for secret in private), 'Private login data reached diagnostic logs'
        owners = cluster.query("SELECT count(*) FROM pg_stat_activity WHERE application_name='theia-development-persistence';")
        assert owners.returncode == 0 and owners.stdout.strip() == '0'
    result = dict(passed=True,scope='Actual loopback HTTP + two-user OIDC/PostgreSQL account APIs; no native game admission',
                  actual_accounts=2,actual_characters=9,game_admission=False,server_example_sha256=binary_sha256,
                  foreign_character_denied=True,strict_host_origin_body_name=True,
                  wrong_poll_proof_nonce_callback_replay_rejected=True,revocation_and_supersession=True,
                  proof_bound_cancellation=True,callback_alone_does_not_create_session=True,
                  measured_header_deadline_seconds=header_seconds,measured_body_deadline_seconds=body_seconds,
                  pending_login_limit=16,handler_limit=16,connection_limit=64,metrics=metrics,
                  backend_loss_failed_closed=True,no_remaining_owner_connections=True,
                  actual_database_and_http_restart=True,old_process_session_rejected=True,
                  stable_owned_character_after_restart=True,graceful_shutdown=True,
                  owned_provider_stopped=identity.server is None,owned_postgres_stopped=cluster.server is None,
                  plaintext_fixture_files_removed=not identity.realm_file.exists() and not (output/'cluster/pgpass').exists())
    (output/'verification.json').write_text(json.dumps(result,indent=2)+'\n')
    return result


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output',type=Path)
    parser.add_argument('--identity-sdk',type=Path,default=Path('build/identity-sdk'))
    parser.add_argument('--postgres-sdk',type=Path,default=Path('build/postgres-sdk'))
    args = parser.parse_args()
    print(json.dumps(run(args.output.resolve(),args.identity_sdk.resolve(),args.postgres_sdk.resolve()),indent=2))
