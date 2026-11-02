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
        do_GET = do_POST
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


class IncompatibleProfiles(unittest.TestCase):
    def test_known_content_conflict_is_clear_not_retried_and_writes_no_ticket(self):
        calls=[]
        message='Character uses incompatible gameplay content; select a compatible character'
        def respond(path, body):
            calls.append(path)
            return 409, {}, json.dumps({'error':message}).encode()
        with service(respond) as base, tempfile.TemporaryDirectory() as tmp:
            client=AccountClient(base);path=Path(tmp)/'ticket'
            with self.assertRaisesRegex(AccountError,'incompatible gameplay content') as caught:
                client.enter('00000000-0000-0000-0000-000000000001',path)
            self.assertEqual(caught.exception.status,409)
            self.assertFalse(caught.exception.retryable)
            self.assertFalse(path.exists())
            self.assertEqual(calls,['/characters/enter'])


class CharacterMenu(unittest.TestCase):
    old='74686569-6100-0000-0000-000000000101'
    new='74686569-6100-0000-0000-000000000102'

    def test_one_existing_character_offers_creation_and_default_selection(self):
        calls=[];shown=[];prompts=[]
        def respond(path, body):
            calls.append((path,body))
            return 200, {}, json.dumps({'characters':[{'id':self.old,'name':'Legacy'}]}).encode()
        with service(respond) as base:
            selected=AccountClient(base).choose_character(read=lambda prompt:prompts.append(prompt) or '',write=shown.append)
        self.assertEqual(selected,self.old)
        self.assertEqual(calls,[('/characters',None)])
        self.assertIn('N. Create a character',shown)
        self.assertEqual(len(prompts),1)

    def test_create_another_character_on_existing_account(self):
        calls=[];answers=iter(['n','Newhero'])
        def respond(path, body):
            calls.append((path,body))
            data={'id':self.new,'name':'Newhero'} if body else {'characters':[{'id':self.old,'name':'Legacy'}]}
            return 200, {}, json.dumps(data).encode()
        with service(respond) as base:
            selected=AccountClient(base).choose_character(read=lambda _:next(answers),write=lambda _:None)
        self.assertEqual(selected,self.new)
        self.assertEqual(calls,[('/characters',None),('/characters',{'name':'Newhero'})])

    def test_full_account_does_not_offer_or_submit_character_creation(self):
        calls=[];shown=[]
        def respond(path, body):
            calls.append((path,body))
            return 200, {}, json.dumps({'characters':[{'id':self.old,'name':'Legacy'}]*8}).encode()
        with service(respond) as base:
            with self.assertRaisesRegex(AccountError,'limit reached'):
                AccountClient(base).choose_character(read=lambda _:'N',write=shown.append)
        self.assertNotIn('N. Create a character',shown)
        self.assertEqual(calls,[('/characters',None)])

    def test_invalid_menu_rows_are_refused_without_printing_control_characters(self):
        for entry in [None,{'id':self.old,'name':'Unsafe\x1b[31m'},{'id':self.old,'name':False}]:
            with self.subTest(entry=entry), service(lambda *_:(200,{},json.dumps({'characters':[entry]}).encode())) as base:
                shown=[]
                with self.assertRaises(AccountError):
                    AccountClient(base).choose_character(read=lambda _:'1',write=shown.append)
                self.assertEqual(shown,[])

    def test_incompatible_selection_can_return_to_menu_and_create_then_enter(self):
        calls=[];shown=[];answers=iter(['1','n','Newhero'])
        def respond(path,body):
            calls.append((path,body))
            if path=='/characters/enter':
                if body['character_id']==self.old:
                    return 409,{},json.dumps({'error':'Character uses incompatible gameplay content; select a compatible character'}).encode()
                return 200,{},json.dumps({'id':self.new,'address':'127.0.0.1:7000','ticket':[1,2,3],'expires_in_seconds':20}).encode()
            data={'id':self.new,'name':'Newhero'} if body else {'characters':[{'id':self.old,'name':'Legacy'}]}
            return 200,{},json.dumps(data).encode()
        with service(respond) as base, tempfile.TemporaryDirectory() as tmp:
            path=Path(tmp)/'ticket'
            address=AccountClient(base).choose_and_enter(path,read=lambda _:next(answers),write=shown.append)
            self.assertEqual(address,'127.0.0.1:7000');self.assertEqual(path.read_bytes(),bytes([1,2,3]));self.assertEqual(path.stat().st_mode & 0o777,0o600)
        self.assertTrue(any('incompatible gameplay content' in line for line in shown))
        self.assertEqual([body['character_id'] for path,body in calls if path=='/characters/enter'],[self.old,self.new])

    def test_transient_reservation_conflict_still_retries(self):
        calls=[]
        def respond(path,body):
            calls.append(path)
            if len(calls)==1:return 409,{},b'{"error":"Account or character already reserved"}'
            return 200,{},json.dumps({'id':self.old,'address':'127.0.0.1:7000','ticket':[1],'expires_in_seconds':20}).encode()
        with service(respond) as base, tempfile.TemporaryDirectory() as tmp:
            AccountClient(base).enter(self.old,Path(tmp)/'ticket')
        self.assertEqual(calls,['/characters/enter','/characters/enter'])


if __name__=='__main__':
    unittest.main()
