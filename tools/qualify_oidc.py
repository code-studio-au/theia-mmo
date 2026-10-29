#!/usr/bin/env python3
"""Rust OIDC verification against an owned actual Keycloak authorization flow."""
import argparse
import base64
import hashlib
import json
import os
from pathlib import Path
import selectors
import subprocess
import urllib.parse
from private_identity import Identity


def query_change(url, key, value):
    parsed = urllib.parse.urlsplit(url)
    query = urllib.parse.parse_qsl(parsed.query)
    return urllib.parse.urlunsplit(parsed._replace(query=urllib.parse.urlencode(
        [(k, value if k == key else v) for k, v in query])))


def run(output, sdk):
    # Compile before starting the owned provider. Build output contains no tokens.
    output.mkdir(mode=0o700, parents=True, exist_ok=False)
    with (output/'build.log').open('w') as log:
        subprocess.run(['cargo', 'build', '--locked', '-p', 'theia-auth', '--example', 'oidc_qualification', '-j2'],
                       stdout=log, stderr=subprocess.STDOUT, check=True, timeout=300)
    child = None
    with Identity(output/'provider', sdk) as identity:
        env = dict(os.environ, THEIA_TEST_OIDC_ISSUER=identity.issuer,
                   THEIA_TEST_OIDC_CLIENT=identity.client_id, THEIA_TEST_OIDC_REDIRECT=identity.redirect)
        with (output/'rust-errors.log').open('w') as errors:
            try:
                child = subprocess.Popen(['target/debug/examples/oidc_qualification'], env=env,
                                         stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                         stderr=errors, bufsize=0)
                cases = []
                first_callback = None
                pending = b''
                with selectors.DefaultSelector() as selector:
                    selector.register(child.stdout, selectors.EVENT_READ)
                    while True:
                        while b'\n' not in pending:
                            assert selector.select(timeout=20), 'Rust identity boundary timed out'
                            chunk = os.read(child.stdout.fileno(), 16384)
                            assert chunk, 'Rust identity boundary exited before qualification summary'
                            pending += chunk
                            assert len(pending) <= 32768, 'Oversized private qualification message'
                        line, pending = pending.split(b'\n', 1)
                        message = json.loads(line)
                        if 'authorization_url' in message:
                            case = message['case']
                            if case == 'replay_callback':
                                callback = first_callback
                            else:
                                url = message['authorization_url']
                                if case == 'wrong_nonce':
                                    url = query_change(url, 'nonce', 'synthetic-wrong-nonce')
                                if case == 'wrong_pkce':
                                    challenge = base64.urlsafe_b64encode(hashlib.sha256(b'synthetic-wrong-verifier').digest()).decode().rstrip('=')
                                    url = query_change(url, 'code_challenge', challenge)
                                callback = identity.authorize(url)
                                if case == 'success':
                                    first_callback = callback
                                if case == 'wrong_state':
                                    callback = query_change(callback, 'state', 'synthetic-wrong-state')
                            child.stdin.write((json.dumps(callback)+'\n').encode())
                            child.stdin.flush()
                        elif 'case' in message:
                            assert message['passed']
                            cases.append(message['case'])
                        else:
                            assert message['passed'] and message['same_verified_subject_across_logins']
                            assert not message['game_admission']
                            break
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
            assert identity.password not in (output/'rust-errors.log').read_text()
        result = dict(passed=True, scope='Renderer-free Rust OIDC verification with actual local identity provider; no game session/admission yet',
                      actual_authorization_page=True, verified_signature_issuer_audience_nonce_expiry=True,
                      cases=cases, same_verified_subject_across_logins=True, game_admission=False)
    result['owned_provider_stopped'] = identity.server is None
    result['plaintext_import_removed'] = not identity.realm_file.exists()
    (output/'verification.json').write_text(json.dumps(result, indent=2)+'\n')
    return result


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('--sdk', type=Path, default=Path('build/identity-sdk'))
    args = parser.parse_args()
    print(json.dumps(run(args.output.resolve(), args.sdk.resolve()), indent=2))
