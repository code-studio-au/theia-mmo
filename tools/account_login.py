"""Bounded native-launcher account flow. Credentials stay in memory/private files."""
import ipaddress
import json
import os
from pathlib import Path
import time
import urllib.error
import urllib.parse
import urllib.request
import uuid
import webbrowser


class AccountError(RuntimeError):
    def __init__(self, message, status=None):
        super().__init__(message)
        self.status = status


class _NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, *_args, **_kwargs):
        return None


def _endpoint(value, *, browser=False):
    try:
        parsed = urllib.parse.urlsplit(value)
        if len(value) > 8192 or parsed.username is not None or parsed.password is not None or parsed.fragment:
            raise ValueError()
        if browser and parsed.scheme == 'https' and parsed.hostname:
            return parsed
        if parsed.scheme != 'http' or not ipaddress.ip_address(parsed.hostname).is_loopback or not parsed.port:
            raise ValueError()
        if not browser and (parsed.path not in ('', '/') or parsed.query):
            raise ValueError()
        return parsed
    except (ValueError, TypeError):
        raise AccountError('Invalid account service or browser endpoint') from None


class AccountClient:
    def __init__(self, base, alive=lambda: True):
        _endpoint(base)
        self._base = base.rstrip('/')
        self._alive = alive
        self._session = None
        self._opener = urllib.request.build_opener(urllib.request.ProxyHandler({}), _NoRedirect())

    def __repr__(self):
        return 'AccountClient([redacted])'

    def request(self, path, body=None):
        if path not in {'/auth/start', '/auth/poll', '/auth/cancel', '/auth/logout', '/characters', '/characters/enter'}:
            raise AccountError('Unknown account operation')
        if not self._alive():
            raise AccountError('The game server stopped')
        data = json.dumps(body).encode() if body is not None else None
        if data is not None and len(data) > 4096:
            raise AccountError('Account request is too large')
        headers = {'Content-Type': 'application/json'}
        if self._session is not None:
            headers['Authorization'] = 'Bearer ' + self._session
        request = urllib.request.Request(self._base + path, data=data, headers=headers)
        try:
            try:
                response = self._opener.open(request, timeout=7)
            except urllib.error.HTTPError as error:
                response = error
            with response:
                status = response.code
                raw = response.read(65537)
            if status not in (200, 202, 204):
                raise AccountError('Account operation was refused; sign in again or wait for your previous character to leave', status)
            if len(raw) > 65536:
                raise AccountError('Account response is too large')
            return (json.loads(raw) if raw else None), status
        except AccountError:
            raise
        except (OSError, ValueError, urllib.error.URLError):
            raise AccountError('Could not complete the account operation') from None

    def login(self, open_browser=webbrowser.open):
        collected = False
        flow = None
        try:
            flow, _ = self.request('/auth/start', {})
            if not isinstance(flow, dict) or not all(isinstance(flow.get(key), str) for key in ['challenge', 'proof', 'authorization_url']):
                raise AccountError('Invalid login challenge')
            if len(flow['challenge']) != 32 or len(flow['proof']) != 64:
                raise AccountError('Invalid login challenge')
            _endpoint(flow['authorization_url'], browser=True)
            deadline = time.monotonic() + 120
            try:
                opened = open_browser(flow['authorization_url'])
            except (OSError, webbrowser.Error):
                raise AccountError('Could not open the system browser') from None
            if not opened:
                raise AccountError('Could not open the system browser')
            poll = {key: flow[key] for key in ['challenge', 'proof']}
            while time.monotonic() < deadline:
                result, status = self.request('/auth/poll', poll)
                if status == 200:
                    token = result.get('session') if isinstance(result, dict) else None
                    if not isinstance(token, str) or len(token) != 64 or any(c not in '0123456789abcdef' for c in token):
                        raise AccountError('Invalid account session')
                    self._session = token
                    collected = True
                    return
                time.sleep(.5)
            raise AccountError('Login timed out; please try again')
        finally:
            if not collected and isinstance(flow, dict) and all(key in flow for key in ['challenge', 'proof']):
                try:
                    self.request('/auth/cancel', {key: flow[key] for key in ['challenge', 'proof']})
                except AccountError:
                    pass

    def choose_character(self, read=input, write=print):
        result, _ = self.request('/characters')
        if not isinstance(result, dict) or not isinstance(result.get('characters'), list) or len(result['characters']) > 8:
            raise AccountError('Invalid character list')
        characters = result['characters']
        if not characters:
            name = read('Name your first character (3–20 letters): ').strip()
            character, _ = self.request('/characters', {'name': name})
        elif len(characters) == 1:
            character = characters[0]
        else:
            for index, character in enumerate(characters, 1):
                write(f"{index}. {character.get('name', '')}")
            try:
                index = int(read('Choose a character number: ')) - 1
                if not 0 <= index < len(characters):
                    raise ValueError()
                character = characters[index]
            except ValueError:
                raise AccountError('Choose one of the listed character numbers') from None
        try:
            identifier = str(uuid.UUID(character['id']))
            if identifier == str(uuid.UUID(int=0)):
                raise ValueError()
        except (KeyError, TypeError, ValueError):
            raise AccountError('Invalid selected character') from None
        return identifier

    def enter(self, character, path):
        deadline = time.monotonic() + 10
        while True:
            try:
                result, _ = self.request('/characters/enter', {'character_id': character})
                break
            except AccountError as error:
                if error.status != 409 or time.monotonic() >= deadline:
                    raise
                time.sleep(.2)
        try:
            address = result['address']
            parsed = urllib.parse.urlsplit('http://' + address)
            if not ipaddress.ip_address(parsed.hostname).is_loopback or not parsed.port or parsed.path or parsed.query or parsed.fragment or parsed.username is not None or parsed.password is not None:
                raise ValueError()
            data = result['ticket']
            if result['id'] != character or not isinstance(data, list) or not 1 <= len(data) <= 2048 or any(type(value) is not int or not 0 <= value <= 255 for value in data):
                raise ValueError()
            if result['expires_in_seconds'] != 20:
                raise ValueError()
        except (KeyError, TypeError, ValueError):
            raise AccountError('Invalid game admission ticket') from None
        fd = os.open(Path(path), os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
        with os.fdopen(fd, 'wb') as file:
            file.write(bytes(data))
        return address

    def close(self):
        if self._session is not None:
            try:
                self.request('/auth/logout', {})
            except AccountError:
                pass
            self._session = None
