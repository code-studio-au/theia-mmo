#!/usr/bin/env python3
"""Launcher HTTP/credential handling only; synthetic responses are not OIDC proof."""
import contextlib
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
from pathlib import Path
import tempfile
import threading
import unittest
from account_login import AccountClient, AccountError


@contextlib.contextmanager
def service(respond):
    class Handler(BaseHTTPRequestHandler):
        def log_message(self, *_):
            pass
        def do_POST(self):
            size = int(self.headers.get('Content-Length', '0'))
            body = self.rfile.read(size)
            status, headers, raw = respond(self.path, json.loads(body) if body else None)
            self.send_response(status)
            for name, value in headers.items():
                self.send_header(name, value)
            self.send_header('Content-Length', str(len(raw)))
            self.end_headers()
            self.wfile.write(raw)
    server = ThreadingHTTPServer(('127.0.0.1', 0), Handler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    try:
        yield f'http://127.0.0.1:{server.server_port}'
    finally:
        server.shutdown();server.server_close();thread.join()


class Accounts(unittest.TestCase):
    def test_remote_alias_userinfo_and_non_origin_service_addresses_are_refused(self):
        for value in ['http://localhost:5050','http://203.0.113.1:5050','http://user:secret@127.0.0.1:5050',
                      'http://127.0.0.1:5050/callback','http://127.0.0.1:5050?token=secret','https://127.0.0.1:5050']:
            with self.subTest(value=value), self.assertRaises(AccountError):
                AccountClient(value)

    def test_real_http_redirect_is_not_followed(self):
        seen=[]
        with service(lambda path, body: (seen.append(path) or 200, {}, b'{}')) as target:
            with service(lambda *_: (302, {'Location':target+'/auth/start'}, b'')) as base:
                client=AccountClient(base)
                with self.assertRaises(AccountError) as error:
                    client.request('/auth/start',{})
                self.assertEqual(error.exception.status,302)
        self.assertEqual(seen,[])

    def test_oversized_actual_response_is_bounded(self):
        with service(lambda *_: (200, {}, b'x'*65537)) as base:
            with self.assertRaisesRegex(AccountError,'too large'):
                AccountClient(base).request('/auth/start',{})

    def test_browser_failure_cancels_the_private_challenge(self):
        calls=[]
        flow={'challenge':'a'*32,'proof':'b'*64,'authorization_url':'http://127.0.0.1:5051/authorize?state=example'}
        def reply(path,body):
            calls.append((path,body))
            return (200,{},json.dumps(flow).encode()) if path=='/auth/start' else (204,{},b'')
        with service(reply) as base:
            client=AccountClient(base)
            with self.assertRaisesRegex(AccountError,'browser'):
                client.login(open_browser=lambda _:False)
            self.assertIsNone(client._session)
        self.assertEqual(calls[-1],('/auth/cancel',{'challenge':flow['challenge'],'proof':flow['proof']}))

    def test_invalid_tickets_never_create_a_bearer_file(self):
        character='74686569-6100-0000-0000-000000000123'
        for address,data in [('203.0.113.1:7000',[1]),('127.0.0.1:7000',[True]),('127.0.0.1:7000',[-1]),('127.0.0.1:7000',[256]),('127.0.0.1:7000',[1]*2049)]:
            payload={'id':character,'address':address,'ticket':data,'expires_in_seconds':20}
            with self.subTest(address=address, length=len(data)), tempfile.TemporaryDirectory() as folder:
                with service(lambda *_: (200,{},json.dumps(payload).encode())) as base:
                    client=AccountClient(base);client._session='f'*64
                    self.assertNotIn('f'*64,repr(client))
                    path=Path(folder)/'private.token'
                    with self.assertRaises(AccountError):client.enter(character,path)
                    self.assertFalse(path.exists())


if __name__=='__main__':
    unittest.main()
