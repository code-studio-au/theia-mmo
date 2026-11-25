"""Cognito browser sign-in and the authenticated native character menu."""
import ipaddress
import os
from pathlib import Path
import re
import ssl
import tempfile
import threading
import time
import uuid

from account_login import AccountClient, AccountError
from browser_sign_in import SignInConfig, browser_access_token
import cognito_native
import client_install
from game_frontend import run_hosted
from launcher_update import VERSION
from realm_gateway import GatewayUnavailable, RealmGateway


class HostedRealm:
    def __init__(self, path):
        data = client_install.read_json(path, 16384)
        fields = {'identity', 'address', 'certificate', 'client_version', 'game_arguments'}
        if (not isinstance(data, dict) or not fields <= set(data)
                or set(data) - fields - {'gateway', 'native_sign_in'}
                or type(data.get('native_sign_in', False)) is not bool):
            raise client_install.InstallError('Invalid development realm configuration')
        self.native_sign_in = data.get('native_sign_in', False) and cognito_native.available()
        if not isinstance(data['identity'], dict) or data['identity'].keys() != SignInConfig.__dataclass_fields__.keys():
            raise client_install.InstallError('Invalid Cognito sign-in configuration')
        self.identity = SignInConfig(**data['identity'])
        self.identity.validate()
        prefix = r'https://[a-z0-9-]+\.auth\.ap-southeast-2\.amazoncognito\.com'
        domain, _, _ = self.identity.authorization_endpoint.rpartition('/oauth2/authorize')
        if (not re.fullmatch(r'https://cognito-idp\.ap-southeast-2\.amazonaws\.com/ap-southeast-2_[A-Za-z0-9]+', self.identity.issuer)
                or not re.fullmatch(prefix, domain)
                or self.identity.authorization_endpoint != domain + '/oauth2/authorize'
                or self.identity.token_endpoint != domain + '/oauth2/token'
                or self.identity.userinfo_endpoint != domain + '/oauth2/userInfo'):
            raise client_install.InstallError('Development realm requires Sydney Cognito HTTPS endpoints')
        address = data['address']
        try:
            ip, _, port = address.rpartition(':')
            if not ipaddress.IPv4Address(ip).is_global or port != '5000':
                raise ValueError()
        except (AttributeError, ValueError):
            raise client_install.InstallError('Invalid development game endpoint') from None
        self.address = address
        self.origin = 'https://' + ip
        certificate = data['certificate']
        if not isinstance(certificate, str) or Path(certificate).name != certificate:
            raise client_install.InstallError('Use the realm certificate supplied with this configuration')
        self.tls = ssl.SSLContext(ssl.PROTOCOL_TLS_CLIENT)
        self.tls.minimum_version = ssl.TLSVersion.TLSv1_2
        self.tls.load_verify_locations(cafile=path.parent / certificate)
        self.client_version = client_install.version(data['client_version'])
        self.gateway = None
        if 'gateway' in data:
            config = data['gateway']
            if (not isinstance(config, dict) or set(config) != {'origin', 'protocol', 'content'}
                    or any(not isinstance(config[key], str) or not re.fullmatch(r'[A-Za-z0-9._-]{1,128}', config[key])
                           for key in ('protocol', 'content'))):
                raise client_install.InstallError('Invalid realm compatibility configuration')
            self.gateway = RealmGateway(config['origin'])
            self.build = dict(launcher=VERSION, client=self.client_version,
                              protocol=config['protocol'], content=config['content'])
        self.game_arguments = data['game_arguments']
        if (not isinstance(self.game_arguments, list) or len(self.game_arguments) > 128
                or any(not isinstance(arg, str) or not 0 < len(arg) <= 4096 or '\0' in arg
                       for arg in self.game_arguments)):
            raise client_install.InstallError('Invalid development game configuration')

    def account(self, cancelled=lambda: False):
        return AccountClient(self.origin, cancelled=cancelled, tls=self.tls, game_address=self.address)

    def ready(self):
        return self.account().ready()

    def launch(self, root, window_arguments, cancelled, progress=lambda _message: None, *, authenticate=None):
        if client_install.active(root)['current'] != self.client_version:
            raise client_install.InstallError(f'Install client {self.client_version} to play this realm')
        account = self.account(cancelled)
        if not self.gateway and not account.ready():
            raise client_install.InstallError('The Sydney game server is unavailable')
        directory = token = reservation = None
        try:
            if self.gateway:
                installed, manifest, total = client_install.installed(root, self.client_version)
                client_install.verify(installed, manifest, total, cancelled=cancelled)
            token = (authenticate(cancelled) if authenticate is not None else browser_access_token(
                self.identity, cancelled, scope='openid theia-development/play', timeout=240,
            ))
            if cancelled():
                raise KeyboardInterrupt
            progress('Signed in. Connecting to the Sydney realm…')
            if self.gateway:
                reservation = str(uuid.uuid4())
                self.gateway.wait_ready(token, reservation, self.build, cancelled, progress, account.ready)
            # The host observes persisted grants on its next heartbeat.
            for attempt in range(8):
                try:
                    account.use_access_token(token, reservation)
                    break
                except AccountError as error:
                    if not self.gateway or error.status != 503 or attempt == 7 or cancelled():
                        raise
                    time.sleep(1)
            directory = tempfile.TemporaryDirectory(prefix='theia-hosted-game-')
            session = Path(directory.name)
            ticket = session / 'admission.token'
            args = [*self.game_arguments, *window_arguments,
                    '--game-front-end', str(session), '--renet-connect', '127.0.0.1:1', str(ticket)]
            process = client_install.launch(root, args, required_version=self.client_version,
                    env={**os.environ, 'THEIA_HOSTED_GAME_ADDRESS': self.address})
        except (AccountError, KeyboardInterrupt, OSError, client_install.InstallError) as error:
            if directory is not None:
                directory.cleanup()
            account.close()
            if reservation is not None:
                self.gateway.release(token, reservation)
            raise client_install.InstallError(str(error) or 'Sign-in cancelled') from None

        ended = threading.Event()

        def renew_join():
            while not ended.wait(15) and process.poll() is None:
                try:
                    result = self.gateway.request('POST', 'reservation', token, {'reservation': reservation})
                except GatewayUnavailable:
                    continue
                except client_install.InstallError:
                    return
                if result is None:
                    return  # Consumed by admission or expired.

        def maintain_session():
            renew = None
            try:
                if reservation is not None:
                    renew = threading.Thread(target=renew_join, name='theia-hosted-reservation')
                    renew.start()
                if not run_hosted(account, session, ticket, lambda: process.poll() is not None):
                    if process.poll() is None:
                        process.terminate()
            except (AccountError, OSError, ValueError) as error:
                process.theia_hosted_error = str(error)
                if process.poll() is None:
                    process.terminate()
            except KeyboardInterrupt:
                if process.poll() is None:
                    process.terminate()
            finally:
                process.wait()
                ended.set()
                if renew is not None:
                    renew.join()
                    self.gateway.release(token, reservation)
                account.close()
                directory.cleanup()

        worker = threading.Thread(target=maintain_session, name='theia-hosted-account')
        worker.start()
        return process
