#!/usr/bin/env python3
"""Actual owned Keycloak registration/login and server-side policy refusals."""
import argparse
import base64
import hashlib
from http.server import BaseHTTPRequestHandler, HTTPServer
import json
import os
from pathlib import Path
import secrets
import threading
import urllib.parse
from unittest.mock import patch
from private_identity import Identity
from native_login import authorize
from account_login import AccountError


def run(out):
    root=Path(__file__).resolve().parents[1]
    with Identity(out,root/'build/identity-sdk',registration=True) as identity:
        callbacks=[]
        class Handler(BaseHTTPRequestHandler):
            def do_GET(self):
                assert 'code=' in self.path
                callbacks.append(True)
                self.send_response(200);self.end_headers()
            def log_message(self,*_):pass
        callback=urllib.parse.urlsplit(identity.redirect)
        server=HTTPServer(('127.0.0.1',callback.port),Handler)
        thread=threading.Thread(target=server.serve_forever,daemon=True);thread.start()
        def attempt(reg=False,password='TestAccount123!',name='new-adventurer',email='new-adventurer@example.invalid'):
            proof=secrets.token_urlsafe(32)
            challenge=base64.urlsafe_b64encode(hashlib.sha256(proof.encode()).digest()).rstrip(b'=').decode()
            url=identity.issuer+'/protocol/openid-connect/auth?'+urllib.parse.urlencode(dict(client_id=identity.client_id,redirect_uri=identity.redirect,response_type='code',scope='openid',code_challenge=challenge,code_challenge_method='S256',state=secrets.token_hex(16)))
            return authorize(url,'http://'+callback.netloc,name,password,**({'email':email} if reg else {}))
        try:
            with patch.dict(os.environ,THEIA_OIDC_ISSUER=identity.issuer,THEIA_NATIVE_REGISTRATION='1'):
                assert attempt(True)
                assert attempt()
                failures=[]
                for label,fields in [('duplicate',{}),('short',dict(password='Short1')),('no-letter',dict(password='123456789')),('no-number',dict(password='lettersOnly')),('space',dict(password='Has space1')),('unicode-space',dict(password='Has\u00a0space1')),('control',dict(password='Has\u0080control1')),('bad-name',dict(name='bad name')),('bad-email',dict(email='invalid'))]:
                    if label!='duplicate':fields.setdefault('name','test-'+label);fields.setdefault('email',label+'@example.invalid')
                    # Bypass the adapter's preflight to prove the provider's own policy.
                    with patch('native_login.registration_error',return_value=None):
                        try:attempt(True,**fields)
                        except AccountError:failures.append(label)
                        else:raise AssertionError('Provider accepted '+label)
                assert len(callbacks)==2
                result=dict(provider='Keycloak 26.8.1',registered=True,login=True,provider_refusals=failures,callback_count=len(callbacks),scope='Provider code issuance; full code/PKCE exchange is qualified separately by native account flow')
                (out/'qualification.json').write_text(json.dumps(result,indent=2))
                print(json.dumps(result))
        finally:server.shutdown();thread.join();server.server_close()

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--output',type=Path,required=True);run(p.parse_args().output.resolve())
