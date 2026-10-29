#!/usr/bin/env python3
"""Actual local IdP fixture protocol smoke; not application token verification/login."""
import argparse
import base64
import hashlib
import json
from pathlib import Path
import secrets
import urllib.error
import urllib.parse
import urllib.request
from private_identity import Identity, _NoRedirect


def run(output, sdk):
    with Identity(output, sdk) as identity:
        verifier = secrets.token_urlsafe(48)
        challenge = base64.urlsafe_b64encode(hashlib.sha256(verifier.encode()).digest()).decode().rstrip('=')
        state = secrets.token_urlsafe(32)
        nonce = secrets.token_urlsafe(32)
        authorization = identity.discovery['authorization_endpoint']+'?'+urllib.parse.urlencode(
            dict(response_type='code', client_id=identity.client_id, redirect_uri=identity.redirect,
                 scope='openid', state=state, nonce=nonce, code_challenge=challenge,
                 code_challenge_method='S256'))
        callback = urllib.parse.parse_qs(urllib.parse.urlsplit(identity.authorize(authorization)).query)
        assert callback.get('state') == [state] and len(callback.get('code', [])) == 1
        code = callback['code'][0]
        opener = urllib.request.build_opener(_NoRedirect())

        def exchange(code_verifier):
            body = urllib.parse.urlencode(dict(grant_type='authorization_code',
                                               client_id=identity.client_id,
                                               redirect_uri=identity.redirect,
                                               code=code, code_verifier=code_verifier)).encode()
            request = urllib.request.Request(identity.discovery['token_endpoint'], data=body)
            return opener.open(request, timeout=5)

        with exchange(verifier) as response:
            tokens = json.load(response)
        assert isinstance(tokens.get('id_token'), str) and tokens['id_token']
        assert isinstance(tokens.get('access_token'), str) and tokens['access_token']
        # Presence is fixture smoke only. The application must independently
        # verify signature/issuer/audience/nonce/expiry before trusting identity.
        try:
            with exchange(verifier):
                raise AssertionError('Identity provider accepted a reused authorization code')
        except urllib.error.HTTPError as error:
            try:
                assert error.code == 400
                assert json.load(error)['error'] == 'invalid_grant'
            finally:
                error.close()
        result = dict(passed=True, scope='Owned identity-provider fixture only; no application token verification or game session',
                      actual_authorization_page=True, authorization_code_pkce=True,
                      matching_callback_state=True, code_reuse_rejected=True,
                      application_signature_verification=False, game_admission=False)
    result['owned_process_stopped'] = identity.server is None
    result['plaintext_realm_file_removed'] = not identity.realm_file.exists()
    (output/'verification.json').write_text(json.dumps(result, indent=2)+'\n')
    return result


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('--sdk', type=Path, default=Path('build/identity-sdk'))
    args = parser.parse_args()
    print(json.dumps(run(args.output, args.sdk), indent=2))
