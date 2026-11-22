"""Account front-end handshake tests; no credentials or windows."""
import json
import socket
import struct
from pathlib import Path
import tempfile
import threading
import time
import unittest
from unittest.mock import Mock
from account_login import AccountError
DEFAULT = dict(schema=1, body=0, face=0, hair=1, facial_hair=0, skin=0, hair_colour=0, outfit=0)
from game_frontend import run, publish


class FrontEnd(unittest.TestCase):
    def test_retry_create_owned_selection_and_one_admission(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            char = dict(id='12345678-1234-4234-8234-123456789abc', name='Arden', appearance=DEFAULT)
            account = Mock()
            account.login.side_effect = [AccountError('Try again'), None]
            account.characters.side_effect = [[], [char]]
            account.enter.return_value = '127.0.0.1:12345'
            stopped = threading.Event()
            errors = []
            def worker():
                try:
                    run(account, root, root/'ticket', stopped.is_set)
                except KeyboardInterrupt:
                    pass
                except Exception as error:
                    errors.append(error)
            thread = threading.Thread(target=worker)
            thread.start()
            def wait(serial, message=None):
                end = time.monotonic()+3
                while time.monotonic()<end:
                    try:
                        state=json.loads((root/'frontend-state.json').read_text())
                        if state['serial']==serial and not state['busy'] and (message is None or message in state['message']):
                            return state
                    except (FileNotFoundError,json.JSONDecodeError):
                        pass
                    time.sleep(.01)
                self.fail('Timed out waiting for native bridge')
            def command(serial, action, **fields):
                if action=='credentials':
                    endpoint=json.loads((root/'frontend-credentials.json').read_text())
                    data=dict(serial=serial,action=action,username='fixture',password='test-only',token=endpoint['token'])
                    raw=json.dumps(data).encode()
                    host,port=endpoint['address'].split(':')
                    with socket.create_connection((host,int(port))) as connection:
                        connection.sendall(struct.pack('!I',len(raw))+raw)
                        self.assertEqual(connection.recv(1),bytes([1]))
                else:
                    publish(root,'frontend-command.json',dict(serial=serial,action=action,**fields))
            try:
                wait(0)
                command(1,'credentials'); self.assertEqual(wait(1,'Try again')['phase'],'login')
                command(2,'credentials'); self.assertEqual(wait(2)['phase'],'characters')
                command(3,'enter',id='someone-else'); wait(3,'this account')
                account.enter.assert_not_called()
                command(4,'create',name='Arden',appearance=DEFAULT); wait(4,'saved')
                account.create_character.assert_called_once_with('Arden',DEFAULT)
                command(5,'enter',id=char['id']); self.assertEqual(wait(5)['phase'],'entering')
                thread.join(2)
                self.assertFalse(thread.is_alive())
                self.assertEqual(json.loads((root/'frontend-admission.json').read_text()),{'address':'127.0.0.1:12345'})
                account.enter.assert_called_once_with(char['id'],root/'ticket')
                self.assertFalse(errors)
            finally:
                stopped.set(); thread.join(2)

    def test_closed_game_cancels_without_account_action(self):
        with tempfile.TemporaryDirectory() as tmp:
            account=Mock()
            with self.assertRaises(KeyboardInterrupt):
                run(account,Path(tmp),Path(tmp)/'ticket',lambda:True)
            account.login.assert_not_called()
            account.enter.assert_not_called()

if __name__=='__main__':unittest.main()
