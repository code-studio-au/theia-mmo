"""Rehearse launcher sign-in with two temporary accounts and no game server."""
import argparse
from dataclasses import asdict
from pathlib import Path
import subprocess
import sys
import tempfile
import threading
from urllib.error import HTTPError
from urllib.request import ProxyHandler, build_opener

from client_install import write_json
from private_identity import Identity

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'apps/launcher'))
from browser_sign_in import SignInConfig, SignInError, rehearse_sign_in


def configuration(identity):
    discovery = identity.discovery
    return SignInConfig(
        issuer=identity.issuer,
        authorization_endpoint=discovery['authorization_endpoint'],
        token_endpoint=discovery['token_endpoint'],
        userinfo_endpoint=discovery['userinfo_endpoint'],
        client_id=identity.client_id,
        redirect_uri=identity.redirect,
    )


def qualify(identity, config):
    subjects = []
    for user in range(len(identity.users)):
        jobs = []
        received = []

        def browser(url):
            callback = identity.authorize(url, user_index=user)

            def send_callback():
                opener = build_opener(ProxyHandler({}))
                try:
                    with opener.open(config.redirect_uri + '?state=wrong&code=ignored', timeout=5):
                        received.append(200)
                except HTTPError as error:
                    received.append(error.code)
                    error.close()
                with opener.open(callback, timeout=5) as response:
                    received.append(response.status)

            thread = threading.Thread(target=send_callback)
            jobs.append(thread)
            thread.start()
            return True

        try:
            subjects.append(rehearse_sign_in(config, lambda: False, open_browser=browser, timeout=12))
        finally:
            for job in jobs:
                job.join(timeout=12)
        if received != [400, 200]:
            raise RuntimeError('Expected invalid callback rejection followed by a successful callback')
    if len(set(subjects)) != 2:
        raise RuntimeError('Expected two distinct provider-confirmed test accounts')
    try:
        rehearse_sign_in(config, lambda: True, open_browser=lambda _: False)
    except SignInError as error:
        if str(error) != 'Test sign-in cancelled':
            raise
    else:
        raise RuntimeError('Expected cancellation')
    print('Two distinct test accounts signed in. Invalid callbacks and cancellation were verified.')


def preview(identity, config, directory):
    profile = directory / 'sign-in.json'
    write_json(profile, asdict(config))
    credentials = directory / 'temporary-test-accounts.txt'
    with open(credentials, 'x', encoding='utf-8', opener=private_file) as stream:
        for name, password in identity.users:
            stream.write(f'{name}\t{password}\n')
    print(f'Account → Test Sign-in. Temporary account details are in {credentials}', flush=True)
    subprocess.run(['/usr/bin/python3', str(ROOT / 'apps/launcher/launcher.py'),
                    '--settings-file', str(directory / 'launcher.json'),
                    '--install-root', str(directory / 'game'),
                    '--test-login-config', str(profile)], check=True)


def private_file(path, flags):
    import os
    return os.open(path, flags, 0o600)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--identity-sdk', type=Path, default=ROOT / 'build/identity-sdk')
    parser.add_argument('--desktop', action='store_true',
                        help='Open the launcher with a private local provider and temporary test accounts')
    args = parser.parse_args()
    with tempfile.TemporaryDirectory(prefix='theia-launcher-sign-in-') as path:
        directory = Path(path)
        with Identity(directory / 'provider', args.identity_sdk, users=2) as identity:
            config = configuration(identity)
            if args.desktop:
                preview(identity, config, directory)
            else:
                qualify(identity, config)


if __name__ == '__main__':
    main()
