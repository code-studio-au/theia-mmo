"""Credential transport and provider boundaries, without real account secrets."""
import json
import os
from pathlib import Path
import socket
import struct
import tempfile
import threading
import unittest
from unittest.mock import patch
from account_login import AccountError
from native_login import Credentials, authorize, registration_error

class NativeLogin(unittest.TestCase):
    def test_capability_bounds_and_no_password_file(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp); service=Credentials(root)
            try:
                endpoint=json.loads((root/'frontend-credentials.json').read_text())
                self.assertEqual((root/'frontend-credentials.json').stat().st_mode&0o777,0o600)
                def send(value, expected):
                    raw=json.dumps(value).encode()
                    with socket.create_connection(service.socket.getsockname()) as stream:
                        stream.sendall(struct.pack('!I',len(raw))+raw)
                        result=service.receive()
                        self.assertEqual(result,expected)
                        if result:self.assertEqual(stream.recv(1),bytes([1]))
                data=dict(serial=1,action='credentials',username='fixture',password='test-only-unique-password')
                send(dict(data,token='wrong'),None)
                send(dict(data,token=endpoint['token']),data)
                send(dict(data,token=endpoint['token'],password='x'*1025),None)
                registration=dict(serial=2,action='register',username='new-account',password='TestAccount123!',email='new@example.invalid')
                send(dict(registration,token=endpoint['token']),registration)
                send(dict(registration,token=endpoint['token'],email='x'*255),None)
                send(dict(registration,token=endpoint['token'],confirmation='TestAccount123!'),None)
                for path in root.iterdir():self.assertNotIn(data['password'],path.read_text())
            finally:service.close()
            self.assertFalse((root/'frontend-credentials.json').exists())

    def test_provider_must_be_configured_loopback_and_exact_auth_path(self):
        for issuer,url in [('', 'http://127.0.0.1:8000/auth'),
                           ('https://example.com/realms/theia','https://example.com/realms/theia/protocol/openid-connect/auth'),
                           ('http://127.0.0.1:8000/realms/theia','http://127.0.0.1:8001/realms/theia/protocol/openid-connect/auth'),
                           ('http://127.0.0.1:8000/realms/theia','http://127.0.0.1:8000/evil')]:
            with self.subTest(issuer=issuer,url=url),patch.dict(os.environ,THEIA_OIDC_ISSUER=issuer),patch('urllib.request.build_opener') as opener:
                with self.assertRaises(AccountError):authorize(url,'http://127.0.0.1:9000','fixture','test-only')
                opener.assert_not_called()

    def test_hostile_form_action_never_receives_credentials(self):
        issuer='http://127.0.0.1:8000/realms/theia'
        for action in ['http://127.0.0.1:8001/steal','http://127.0.0.1:8000/steal','https://example.com/steal']:
            with self.subTest(action=action),patch.dict(os.environ,THEIA_OIDC_ISSUER=issuer),patch('urllib.request.build_opener') as build:
                opener=build.return_value
                opener.open.return_value.__enter__.return_value.read.return_value=f'<form id="kc-form-login" action="{action}">'.encode()
                with self.assertRaises(AccountError):authorize(issuer+'/protocol/openid-connect/auth','http://127.0.0.1:9000','fixture','test-only')
                self.assertEqual(opener.open.call_count,1)

    def test_registration_validation_and_disabled_provider(self):
        self.assertIsNone(registration_error('new-account', 'a@example.invalid', 'TestAccount123!'))
        for name, email, password in [('ab','a@example.invalid','TestAccount123!'),
                ('a b','a@example.invalid','TestAccount123!'), ('valid','a@.invalid','TestAccount123!'),
                ('valid','a@example.invalid','lettersOnly'), ('valid','a@example.invalid','123456789'),
                ('valid','a@example.invalid','Has space1'), ('valid','a@example.invalid','x'*129+'1')]:
            self.assertIsNotNone(registration_error(name,email,password))
        with patch.dict(os.environ, THEIA_NATIVE_REGISTRATION='0'), patch('urllib.request.build_opener') as opener:
            with self.assertRaises(AccountError):authorize('unused','unused','valid','TestAccount123!',email='a@example.invalid')
            opener.assert_not_called()

    def test_hostile_registration_form_never_receives_credentials(self):
        issuer='http://127.0.0.1:8000/realms/theia'
        with patch.dict(os.environ,THEIA_OIDC_ISSUER=issuer,THEIA_NATIVE_REGISTRATION='1'),patch('urllib.request.build_opener') as build:
            opener=build.return_value
            opener.open.return_value.__enter__.return_value.read.return_value=b'<form id="kc-register-form" action="http://127.0.0.1:8001/steal">'
            with self.assertRaises(AccountError):authorize(issuer+'/protocol/openid-connect/auth','http://127.0.0.1:9000','valid','TestAccount123!',email='a@example.invalid')
            self.assertEqual(opener.open.call_count,1)

if __name__=='__main__':unittest.main()
