"""Owned local Keycloak qualification fixture; no production account/service."""
import json
from html.parser import HTMLParser
from http.cookiejar import CookieJar, DefaultCookiePolicy
import os
from pathlib import Path
import secrets
import shutil
import signal
import socket
import subprocess
import time
import urllib.request
import urllib.error
import urllib.parse


class _LoginForm(HTMLParser):
    def __init__(self):
        super().__init__()
        self.action = None

    def handle_starttag(self, tag, attrs):
        attrs = dict(attrs)
        if tag == 'form' and attrs.get('id') == 'kc-form-login':
            self.action = attrs.get('action')


class _NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        return None


class Identity:
    def __init__(self, output, sdk, *, users=1):
        self.output = Path(output).resolve()
        self.sdk = Path(sdk).resolve()
        self.server = None
        self.log = None
        self.password = secrets.token_urlsafe(32)
        if not 1 <= users <= 4:
            raise ValueError('Owned identity fixture supports 1..4 synthetic users')
        self.username = 'theia-synthetic-user'
        self.users = [(self.username, self.password)]+[(f'theia-synthetic-user-{i+1}', secrets.token_urlsafe(32)) for i in range(1, users)]
        self.client_id = 'theia-native-development'
        self.port = None
        self.issuer = None
        self.redirect = None

    def __enter__(self):
        self.output.mkdir(mode=0o700, parents=True, exist_ok=False)
        try:
            with socket.socket() as reserved:
                reserved.bind(('127.0.0.1', 0))
                self.port = reserved.getsockname()[1]
            with socket.socket() as reserved:
                reserved.bind(('127.0.0.1', 0))
                callback_port = reserved.getsockname()[1]
            self.issuer = f'http://127.0.0.1:{self.port}/realms/theia-test'
            self.redirect = f'http://127.0.0.1:{callback_port}/callback'
            runtime = self.output/'runtime'
            shutil.copytree(self.sdk/'keycloak-26.8.1', runtime)
            imports = runtime/'data/import'
            imports.mkdir(parents=True)
            self.realm_file = imports/'theia-test-realm.json'
            realm = dict(realm='theia-test', enabled=True, sslRequired='none',
                         registrationAllowed=False, resetPasswordAllowed=False,
                         accessTokenLifespan=60,
                         clients=[dict(clientId=self.client_id, publicClient=True,
                                       standardFlowEnabled=True, directAccessGrantsEnabled=False,
                                       serviceAccountsEnabled=False, fullScopeAllowed=False,
                                       redirectUris=[self.redirect],
                                       attributes={'pkce.code.challenge.method': 'S256'})],
                         users=[dict(username=username, enabled=True,
                                     firstName='Synthetic', lastName='Fixture',
                                     email=f'synthetic-{i}@example.invalid', emailVerified=True,
                                     credentials=[dict(type='password', value=password, temporary=False)])
                                for i, (username, password) in enumerate(self.users)])
            self.realm_file.write_text(json.dumps(realm))
            self.realm_file.chmod(0o600)
            env = dict(os.environ)
            env['JAVA_HOME'] = str(self.sdk/'usr/lib/jvm/java-25-openjdk-amd64')
            env['JAVA_OPTS_APPEND'] = '-Xms64m -Xmx384m -XX:ActiveProcessorCount=2'
            self.log = (self.output/'server.log').open('w')
            self.server = subprocess.Popen(
                [str(runtime/'bin/kc.sh'), 'start-dev', '--import-realm',
                 '--http-host=127.0.0.1', f'--http-port={self.port}',
                 f'--hostname=http://127.0.0.1:{self.port}', '--log-level=error'],
                env=env, stdout=self.log, stderr=subprocess.STDOUT, start_new_session=True)
            deadline = time.monotonic()+120
            while time.monotonic() < deadline:
                assert self.server.poll() is None, 'Owned identity provider exited; inspect server.log'
                try:
                    with urllib.request.urlopen(self.issuer+'/.well-known/openid-configuration', timeout=1) as response:
                        self.discovery = json.load(response)
                    assert self.discovery['issuer'] == self.issuer
                    self.realm_file.unlink()
                    return self
                except (OSError, TimeoutError):
                    time.sleep(.2)
            raise TimeoutError('Owned identity provider startup timeout')
        except BaseException:
            self.close()
            raise

    def close(self):
        try:
            if self.server is not None:
                try:
                    os.killpg(self.server.pid, signal.SIGTERM)
                except ProcessLookupError:
                    pass
                try:
                    self.server.wait(timeout=10)
                except subprocess.TimeoutExpired:
                    os.killpg(self.server.pid, signal.SIGKILL)
                    self.server.wait()
                self.server = None
        finally:
            if self.log:
                self.log.close()
            if hasattr(self, 'realm_file'):
                self.realm_file.unlink(missing_ok=True)

    def authorize(self, authorization_url, *, user_index=0):
        """Submit only the owned synthetic user's actual provider login page.

        Returns the intercepted callback URL in memory; never opens a desktop
        browser, prints credentials/codes or contacts the callback listener.
        The application under test must validate state and exchange the code.
        """
        expected = urllib.parse.urlsplit(self.issuer)
        target = urllib.parse.urlsplit(authorization_url)
        assert (target.scheme, target.netloc) == (expected.scheme, expected.netloc)
        # Browsers treat numeric loopback as a trustworthy development context.
        # This fixture permits its secure cookies over HTTP only because both
        # requests below are explicitly restricted to the owned loopback origin.
        cookies = CookieJar(DefaultCookiePolicy(secure_protocols=('https', 'wss', 'http')))
        opener = urllib.request.build_opener(
            urllib.request.HTTPCookieProcessor(cookies), _NoRedirect())
        with opener.open(authorization_url, timeout=5) as response:
            html = response.read(1024*1024+1)
        assert len(html) <= 1024*1024, 'Oversized provider login page'
        form = _LoginForm()
        form.feed(html.decode('utf-8'))
        assert form.action, 'Expected actual provider username/password form'
        action = urllib.parse.urlsplit(form.action)
        assert (action.scheme, action.netloc) == (expected.scheme, expected.netloc)
        username, password = self.users[user_index]
        body = urllib.parse.urlencode({'username': username, 'password': password}).encode()
        try:
            with opener.open(urllib.request.Request(form.action, data=body), timeout=5):
                raise AssertionError('Provider did not redirect after synthetic login')
        except urllib.error.HTTPError as error:
            try:
                assert error.code in (302, 303), 'Provider rejected synthetic form login'
                callback = error.headers['Location']
                actual = urllib.parse.urlsplit(callback)
                wanted = urllib.parse.urlsplit(self.redirect)
                assert (actual.scheme, actual.netloc, actual.path) == (wanted.scheme, wanted.netloc, wanted.path)
                return callback
            finally:
                error.close()

    def __exit__(self, *_):
        self.close()
