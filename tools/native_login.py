"""Bounded in-memory credential handoff and local Keycloak form adapter.

The development provider's authorization-code/PKCE callback remains authoritative.
This adapter deliberately supports only a configured loopback Keycloak provider;
other provider configurations are refused without launching a browser. No password grant,
credential file, credential logging, or browser-cookie reuse is introduced.
"""
from html.parser import HTMLParser
from http.cookiejar import CookieJar, DefaultCookiePolicy
import hmac
import ipaddress
import json
import os
import secrets
import socket
import struct
import urllib.error
import urllib.parse
import urllib.request
from account_login import AccountError, _NoRedirect


class Credentials:
    def __init__(self, directory):
        self.directory = directory
        self.socket = socket.socket()
        self.socket.bind(('127.0.0.1', 0))
        self.socket.listen(4)
        self.socket.settimeout(.01)
        self.token = secrets.token_hex(32)
        from game_frontend import publish
        publish(directory, 'frontend-credentials.json', dict(address=f'127.0.0.1:{self.socket.getsockname()[1]}', token=self.token))

    def close(self):
        self.socket.close()
        (self.directory/'frontend-credentials.json').unlink(missing_ok=True)

    def receive(self):
        try:
            connection, _ = self.socket.accept()
        except socket.timeout:
            return None
        with connection:
            connection.settimeout(.3)
            def read(size):
                result = bytearray()
                while len(result) < size:
                    data = connection.recv(size-len(result))
                    if not data:
                        raise ValueError()
                    result.extend(data)
                return result
            try:
                size, = struct.unpack('!I', read(4))
                if not 1 <= size <= 8192:
                    return None
                value = json.loads(read(size))
                if not isinstance(value, dict) or not isinstance(value.get('token'), str) or not hmac.compare_digest(value.pop('token'), self.token):
                    return None
                if set(value) != {'serial', 'action', 'username', 'password'} or value['action'] != 'credentials':
                    return None
                if not all(isinstance(value[k], str) and 0 < len(value[k]) <= limit for k, limit in [('username', 256), ('password', 1024)]):
                    return None
                connection.sendall(bytes([1]))
                return value
            except (OSError, ValueError, TypeError):
                return None


class LoginForm(HTMLParser):
    def __init__(self):
        super().__init__()
        self.actions = []
    def handle_starttag(self, tag, attrs):
        values = dict(attrs)
        if tag == 'form' and values.get('id') == 'kc-form-login':
            self.actions.append(values.get('action'))


def authorize(url, base, username, password):
    issuer = urllib.parse.urlsplit(os.environ.get('THEIA_OIDC_ISSUER', ''))
    target = urllib.parse.urlsplit(url)
    try:
        local = issuer.scheme == 'http' and ipaddress.ip_address(issuer.hostname).is_loopback and issuer.port
    except (ValueError, TypeError):
        local = False
    if not local or issuer.query or issuer.fragment or issuer.username or not issuer.path.startswith('/realms/'):
        raise AccountError('In-game sign-in is not configured for this account provider.')
    origin = (issuer.scheme, issuer.netloc)
    if (target.scheme, target.netloc, target.path) != (*origin, issuer.path+'/protocol/openid-connect/auth'):
        raise AccountError('The sign-in provider is unavailable.')
    jar = CookieJar(DefaultCookiePolicy(secure_protocols=('http', 'https', 'wss')))
    opener = urllib.request.build_opener(urllib.request.ProxyHandler({}), urllib.request.HTTPCookieProcessor(jar), _NoRedirect())
    try:
        with opener.open(url, timeout=7) as response:
            html = response.read(1024*1024+1)
        if len(html) > 1024*1024:
            raise AccountError('The sign-in provider is unavailable.')
        form = LoginForm(); form.feed(html.decode('utf-8'))
        if len(form.actions) != 1 or not form.actions[0]:
            raise AccountError('This account requires verification that is not supported in this preview.')
        action = urllib.parse.urlsplit(form.actions[0])
        if (action.scheme, action.netloc, action.path) != (*origin, issuer.path+'/login-actions/authenticate') or action.username or action.fragment:
            raise AccountError('The sign-in provider is unavailable.')
        request = urllib.request.Request(form.actions[0], data=urllib.parse.urlencode(dict(username=username, password=password)).encode())
        try:
            with opener.open(request, timeout=7):
                raise AccountError('Sign-in failed. Check your username and password.')
        except urllib.error.HTTPError as error:
            with error:
                if error.code not in (302, 303):
                    raise AccountError('Sign-in failed. Check your username and password.') from None
                callback = error.headers.get('Location', '')
        actual = urllib.parse.urlsplit(callback)
        expected = urllib.parse.urlsplit(base+'/callback')
        if (actual.scheme, actual.netloc, actual.path) != (expected.scheme, expected.netloc, expected.path) or actual.fragment or actual.username:
            raise AccountError('The sign-in provider returned an invalid response.')
        with opener.open(callback, timeout=7) as response:
            if response.code != 200:
                raise AccountError('Sign-in verification failed.')
        return True
    except AccountError:
        raise
    except (OSError, ValueError, UnicodeError):
        raise AccountError('Could not complete sign-in. Please try again.') from None
