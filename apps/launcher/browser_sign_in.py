"""Opt-in browser sign-in rehearsal, independent of the game server."""
import base64
from dataclasses import dataclass
import hashlib
from http.client import HTTPException
from http.server import BaseHTTPRequestHandler, HTTPServer
import re
import secrets
import time
from urllib.parse import parse_qs, urlencode, urlsplit
from urllib.request import ProxyHandler, Request, build_opener
import webbrowser

from client_install import InstallError, decode_json, read_json
from hosted_release import NoRedirect


class SignInError(InstallError):
    pass


def provider_url(value):
    if not isinstance(value, str) or len(value) > 2048 or any(ord(c) <= 32 for c in value):
        raise SignInError('Invalid test identity provider URL')
    try:
        url = urlsplit(value)
        if (url.hostname and url.username is None and not url.query and not url.fragment
                and (url.scheme == 'https' or
                     (url.scheme == 'http' and url.hostname == '127.0.0.1' and url.port))):
            return url
    except ValueError:
        pass
    raise SignInError('Use HTTPS or the local identity fixture for test sign-in')


@dataclass(frozen=True)
class SignInConfig:
    issuer: str
    authorization_endpoint: str
    token_endpoint: str
    userinfo_endpoint: str
    client_id: str
    redirect_uri: str

    @classmethod
    def load(cls, path):
        data = read_json(path.expanduser().resolve(strict=True), 16384)
        if not isinstance(data, dict) or data.keys() != cls.__dataclass_fields__.keys():
            raise SignInError('Invalid test sign-in configuration')
        config = cls(**data)
        config.validate()
        return config

    def validate(self):
        if not isinstance(self.client_id, str) or not re.fullmatch(r'[A-Za-z0-9_.-]{1,128}', self.client_id):
            raise SignInError('Invalid public test application ID')
        issuer = provider_url(self.issuer)
        authorization = provider_url(self.authorization_endpoint)
        if authorization.scheme != issuer.scheme or (issuer.scheme == 'http' and issuer.netloc != authorization.netloc):
            raise SignInError('Test sign-in endpoints must use the configured identity provider')
        for value in (self.token_endpoint, self.userinfo_endpoint):
            endpoint = provider_url(value)
            if (endpoint.scheme, endpoint.netloc) != (authorization.scheme, authorization.netloc):
                raise SignInError('Test sign-in endpoints must use the same provider origin')
        if not isinstance(self.redirect_uri, str) or not re.fullmatch(
                r'http://(?:localhost|127\.0\.0\.1):[1-9][0-9]{0,4}/callback', self.redirect_uri):
            raise SignInError('Register a local test sign-in callback first')
        try:
            redirect = urlsplit(self.redirect_uri)
            return redirect.netloc, redirect.port
        except ValueError as error:
            raise SignInError('Invalid local test callback port') from error


class CallbackServer(HTTPServer):
    def get_request(self):
        connection, address = super().get_request()
        connection.settimeout(1)
        return connection, address

    def handle_error(self, *_):
        pass


def callback_handler(state, issuer, host, result):
    class Handler(BaseHTTPRequestHandler):
        def log_message(self, *_):
            # Browser callback URLs contain authorization codes.
            pass

        def do_GET(self):
            try:
                url = urlsplit(self.path)
                params = parse_qs(url.query, keep_blank_values=True, max_num_fields=12)
                valid = (self.headers.get_all('Host') == [host] and len(self.path) <= 8192
                         and not url.netloc and not url.scheme and url.path == '/callback'
                         and len(params.get('state', ())) == 1
                         and secrets.compare_digest(params['state'][0].encode(), state.encode())
                         and ('iss' not in params or params['iss'] == [issuer]))
                if not valid:
                    raise ValueError()
                code = params.get('code', [])
                if len(code) == 1 and 0 < len(code[0]) <= 4096 and 'error' not in params:
                    result.append(code[0])
                elif 'error' in params and not code:
                    result.append(None)
                else:
                    raise ValueError()
            except ValueError:
                status, text = 400, b'Invalid sign-in callback. Return to the launcher.'
            else:
                status, text = 200, b'Return to Theia Online to see the sign-in result.'
            self.send_response(status)
            self.send_header('Content-Type', 'text/plain; charset=utf-8')
            self.send_header('Cache-Control', 'no-store')
            self.send_header('Referrer-Policy', 'no-referrer')
            self.end_headers()
            self.wfile.write(text)
    return Handler


def request_json(opener, endpoint, *, form=None, bearer=None):
    headers = {'Accept': 'application/json'}
    if bearer:
        headers['Authorization'] = 'Bearer ' + bearer
    request = Request(endpoint, data=urlencode(form).encode('ascii') if form else None, headers=headers)
    try:
        with opener.open(request, timeout=7) as response:
            raw = response.read(65537)
        if len(raw) > 65536:
            raise ValueError()
        data = decode_json(raw)
        if not isinstance(data, dict):
            raise ValueError()
        return data
    except (OSError, ValueError, HTTPException):
        raise SignInError('The test identity provider could not complete sign-in') from None


def browser_access_token(config, cancelled, *, scope='openid', open_browser=webbrowser.open, timeout=120):
    """Complete browser PKCE and return the access token to the in-memory caller."""
    host, port = config.validate()
    state, verifier = secrets.token_urlsafe(32), secrets.token_urlsafe(48)
    challenge = base64.urlsafe_b64encode(hashlib.sha256(verifier.encode('ascii')).digest()).rstrip(b'=')
    url = config.authorization_endpoint + '?' + urlencode({
        'response_type': 'code',
        'scope': scope,
        'client_id': config.client_id,
        'redirect_uri': config.redirect_uri,
        'state': state,
        'code_challenge': challenge.decode('ascii'),
        'code_challenge_method': 'S256',
        'prompt': 'login',
    })
    result = []
    try:
        server = CallbackServer(('127.0.0.1', port), callback_handler(state, config.issuer, host, result))
    except OSError:
        raise SignInError('The local sign-in port is already in use') from None
    with server:
        server.timeout = 0.2
        try:
            if cancelled():
                raise SignInError('Test sign-in cancelled')
            opened = open_browser(url)
        except (OSError, webbrowser.Error):
            raise SignInError('Could not open the system browser') from None
        if not opened:
            raise SignInError('Could not open the system browser')
        deadline = time.monotonic() + timeout
        while not result and not cancelled() and time.monotonic() < deadline:
            server.handle_request()
    if cancelled():
        raise SignInError('Test sign-in cancelled')
    if not result:
        raise SignInError('Test sign-in timed out')
    if result[0] is None:
        raise SignInError('Test sign-in was declined')
    opener = build_opener(ProxyHandler({}), NoRedirect())
    token = request_json(opener, config.token_endpoint, form={
        'grant_type': 'authorization_code',
        'client_id': config.client_id,
        'redirect_uri': config.redirect_uri,
        'code': result[0],
        'code_verifier': verifier,
    })
    if cancelled():
        raise SignInError('Test sign-in cancelled')
    bearer = token.get('access_token')
    kind = token.get('token_type')
    if (not isinstance(kind, str) or kind.lower() != 'bearer' or not isinstance(bearer, str)
            or not re.fullmatch(r'[A-Za-z0-9._~+/-]{1,16384}={0,2}', bearer)):
        raise SignInError('The identity provider returned an invalid access token')
    return bearer


def rehearse_sign_in(config, cancelled, *, open_browser=webbrowser.open, timeout=120):
    """Return the provider-confirmed subject, discarding credentials and tokens."""
    bearer = browser_access_token(config, cancelled, open_browser=open_browser, timeout=timeout)
    opener = build_opener(ProxyHandler({}), NoRedirect())
    subject = request_json(opener, config.userinfo_endpoint, bearer=bearer).get('sub')
    if cancelled():
        raise SignInError('Test sign-in cancelled')
    if not isinstance(subject, str) or not 0 < len(subject) <= 256 or not subject.isprintable():
        raise SignInError('The identity provider did not confirm a test account')
    return subject
